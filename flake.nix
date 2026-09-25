# SPDX-License-Identifier: GPL-3.0-or-later
# Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
# https://Construct.SpacecraftSoftware.org/
{
  description = "Spacecraft Software Construct — agent skill catalogue";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      # ───────────────────────────────────────────────────────────────────
      # Skill auto-detection
      # ───────────────────────────────────────────────────────────────────
      # A "cross-platform" skill is any top-level directory that contains a
      # SKILL.md and is not in the excluded list. A "Grok" skill is any
      # subdirectory of grok-skills/ that contains a SKILL.md.
      excludedDirs = [ "grok-skills" "android-skills" "orca-skills" "perplexity-skills" "Excluded" ".claude" ".git" "construct-cli" ];

      hasSkillMd = parent: name:
        builtins.pathExists (parent + "/${name}/SKILL.md");

      skillNamesIn = parent:
        let
          entries = builtins.readDir parent;
          dirs = nixpkgs.lib.filterAttrs (n: t: t == "directory") entries;
        in
          builtins.filter
            (n: !(builtins.elem n excludedDirs) && hasSkillMd parent n)
            (builtins.attrNames dirs);

      crossPlatformSkills = skillNamesIn self;
      grokSkills =
        if builtins.pathExists (self + "/grok-skills") then
          skillNamesIn (self + "/grok-skills")
        else
          [];
      # Vendored Google Android skills — same open-standard SKILL.md format as
      # the cross-platform skills, so they can share the canonical install tree.
      androidSkills =
        if builtins.pathExists (self + "/android-skills") then
          skillNamesIn (self + "/android-skills")
        else
          [];
      # Vendored Orca skills — same open-standard SKILL.md format, but OPT-IN
      # (`enableOrca`), not merged by default.
      #
      # They were unconditional until it turned out that installing them from
      # the Nix store is what BREAKS Orca's own updater. Orca scans the agent
      # skill directories and, in `observeSkillPackage`, throws
      # `skill-package-link` on any file with `nlink !== 1`; the catch turns
      # that into status `unrecognized`, which is the "The copy here doesn't
      # match the official version" row in Settings → Update skills. Store
      # files are hardlinked by store optimisation (ours sat at nlink 5–7), and
      # they are mode 444 besides, so `classifyHomeSkillTopology` would mark
      # them `read-only` even if the byte check passed. No pin of the vendored
      # revision can clear it — the copies were byte-identical to the official
      # rev and still flagged.
      #
      # So the default is now: Orca ships these skills, Orca installs them
      # (`orca skills install`, which is `npx skills add` underneath) into a
      # real writable directory, and Orca updates them. Turn `enableOrca` on
      # for a host with no Orca app, where the vendored copies are the only
      # ones. See `orca-skills/CREDITS.md` for the provenance procedure.
      orcaSkills =
        if builtins.pathExists (self + "/orca-skills") then
          skillNamesIn (self + "/orca-skills")
        else
          [];

      # ───────────────────────────────────────────────────────────────────
      # System support
      # ───────────────────────────────────────────────────────────────────
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      # Per-skill derivation — copies one skill directory into the store.
      mkSkillPackage = pkgs: source: name:
        pkgs.runCommandLocal "skill-${name}" { } ''
          mkdir -p $out
          cp -r ${source}/${name}/. $out/
        '';

      # Combined derivation — one flat skill tree from any number of sources.
      # Each source is { source; names; }; leaves are copied in list order, so
      # a name appearing twice would be silently overwritten rather than
      # merged. Every caller below therefore relies on leaf names being
      # disjoint across sources (see mkSkills).
      mkMerged = pkgs: outName: sources:
        pkgs.runCommandLocal outName { } (''
          mkdir -p $out
        '' + nixpkgs.lib.concatMapStringsSep "\n" ({ source, names }:
          nixpkgs.lib.concatMapStringsSep "\n" (n: ''
            mkdir -p $out/${n}
            cp -r ${source}/${n}/. $out/${n}/
          '') names) sources);

      # Combined derivation — one skill tree from one source.
      mkCombined = pkgs: source: skillList: outName:
        mkMerged pkgs outName [ { inherit source; names = skillList; } ];

      # The base tree every non-Grok consumer starts from: the cross-platform
      # skills, plus the vendored Orca ones when the caller asks for them. Leaf
      # names don't collide — cross-platform skills are all spacecraft-* /
      # gnu-* / microsoft-* / steelbore-*, and the three Orca leaves are
      # distinct from those — so a flat merge is safe. `orca-skills/CREDITS.md`
      # records that the generic Orca leaf names (`computer-use`,
      # `orchestration`) are reserved and must not be claimed by a future
      # Spacecraft skill, whether or not this tree carries them.
      baseSources = orca: [
        { source = self; names = crossPlatformSkills; }
      ] ++ nixpkgs.lib.optional (orca && orcaSkills != [])
        { source = self + "/orca-skills"; names = orcaSkills; };

      # THE skill tree builder. Every consumer goes through this — the
      # `packages` outputs below and the Home-Manager module alike.
      #
      # That shared route is the point, not a tidiness exercise. A consumer
      # that instantiates nixpkgs differently from this flake (a `follows`, an
      # overlay, `useGlobalPkgs`) gets a DIFFERENT store path for a
      # byte-identical tree. So a consumer wanting to compare "what the flake
      # pins" against "what is installed" must pass one derivation from here to
      # both sides; comparing `packages.skills` against a separately-built
      # module tree compares two nixpkgs, and reports drift forever.
      mkSkills = { pkgs, android ? false, grok ? false, orca ? false }:
        if grok then
          mkCombined pkgs (self + "/grok-skills") grokSkills "construct-grok-skills"
        else if android && androidSkills != [] then
          mkMerged pkgs "construct-skills-with-android"
            (baseSources orca ++ [
              { source = self + "/android-skills"; names = androidSkills; }
            ])
        else
          mkMerged pkgs "construct-skills" (baseSources orca);
    in {

      # ───────────────────────────────────────────────────────────────────
      # packages.${system}.${skill-name}
      # ───────────────────────────────────────────────────────────────────
      # One derivation per cross-platform skill, plus one per Grok skill
      # (prefixed `grok-` to avoid collision in the flat attrset).
      packages = forAllSystems (pkgs:
        (builtins.listToAttrs (map (n: {
          name = n;
          value = mkSkillPackage pkgs self n;
        }) crossPlatformSkills))
        //
        (builtins.listToAttrs (map (n: {
          name = "grok-${n}";
          value = mkSkillPackage pkgs (self + "/grok-skills") n;
        }) grokSkills))
        //
        (builtins.listToAttrs (map (n: {
          name = "android-${n}";
          value = mkSkillPackage pkgs (self + "/android-skills") n;
        }) androidSkills))
        //
        (builtins.listToAttrs (map (n: {
          name = "orca-${n}";
          value = mkSkillPackage pkgs (self + "/orca-skills") n;
        }) orcaSkills))
        // {
          # The whole trees, as buildable outputs. `skills` is what a consumer
          # points a mutable pointer at (see `mutablePointer` below): building
          # it is a `cp -r` of a few megabytes, so re-pointing costs seconds
          # rather than a system generation.
          skills = mkSkills { inherit pkgs; };
          skills-with-android = mkSkills {
            inherit pkgs;
            android = true;
          };
        }
        // nixpkgs.lib.optionalAttrs (orcaSkills != [ ]) {
          # The vendored Orca leaves merged in, for a host with no Orca app to
          # install and update them itself. See `orcaSkills` above for why that
          # is the exception rather than the default.
          skills-with-orca = mkSkills {
            inherit pkgs;
            orca = true;
          };
        }
        // nixpkgs.lib.optionalAttrs (grokSkills != [ ]) {
          skills-grok = mkSkills {
            inherit pkgs;
            grok = true;
          };
        }
        // {
          # First executable in the catalogue: the `construct` skills CLI.
          # Its source lives in construct-cli/ (excluded from skill detection
          # above). Built from the in-tree Cargo.lock for reproducibility.
          construct = pkgs.rustPlatform.buildRustPackage {
            pname = "construct";
            version = "0.1.0";
            src = self + "/construct-cli";
            cargoLock.lockFile = self + "/construct-cli/Cargo.lock";
            # The ship-loop tests shell out to `git`; make it available to the
            # check phase (the binary itself invokes the user's system git/nix).
            nativeCheckInputs = [ pkgs.git ];
            meta = {
              description = "Spacecraft Software Construct skills package manager";
              homepage = "https://Construct.SpacecraftSoftware.org/";
              license = pkgs.lib.licenses.gpl3Plus;
              mainProgram = "construct";
            };
          };
        }
      );

      # ───────────────────────────────────────────────────────────────────
      # homeManagerModules.default
      # ───────────────────────────────────────────────────────────────────
      # Wires up the canonical ~/.agents/skills/ location, populates each
      # agent's own skills directory the way `agentPaths` says to (a directory
      # symlink to the hub, a real directory of per-skill links, or nothing at
      # all for an agent that reads the hub itself), and (when enableGrok is
      # on) installs Grok skills to ~/.grok/skills/.
      homeManagerModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.spacecraft.construct;

          combinedGrok =
            if grokSkills == [] then null
            else mkSkills { inherit pkgs; grok = true; };

          # Absolute path of the pointer directory, and of the two links in it.
          stateDir = "$HOME/${cfg.mutablePointer.stateDir}";

          # What "a symlink this module made" means, as `case` patterns over
          # `readlink` output. Bound once and used by every place that decides
          # whether a symlink may be replaced, so the answer cannot differ by
          # mode. Anything that matches neither is someone else's — a Vercel
          # `skills add` relative link (`../../.agents/skills/<x>`), a user's
          # override into a checkout — and is reported and left as found.
          #
          #   ownDirLink  — a whole-directory link at an agent path: to the hub,
          #                 or straight into the store (an older home.file
          #                 install).
          #   ownLeafLink — a per-skill link on a skill's name: into the store,
          #                 or through the pointer — how an earlier generation
          #                 drew it before a tree bump or a pointer toggle. The
          #                 renderer adds its own current `"$src"/*` in front.
          ownDirLink = ''"$HOME/.agents/skills" | "${builtins.storeDir}"/*'';
          ownLeafLink = ''"${builtins.storeDir}"/* | "${stateDir}/current"/*'';

          # THE per-skill link renderer, shared by every tree this module
          # materialises. One implementation on purpose: the ~/.agents and
          # ~/.grok trees differ only in their source and destination, and two
          # copies of a loop that decides what to clobber and what to prune is
          # how one of them quietly grows a different answer.
          perSkillLinkScript = { src, canonical }: ''
            src="${src}"
            canonical="${canonical}"

            # A directory symlink from a generation before this option was on
            # has to go before the directory can be made.
            if [ -L "$canonical" ]; then
              $DRY_RUN_CMD rm -f "$canonical"
            fi
            $DRY_RUN_CMD mkdir -p "$canonical"

            for d in "$src"/*/; do
              [ -d "$d" ] || continue
              n="$(basename "$d")"
              t="$canonical/$n"
              # A REAL directory on that name belongs to whoever put it there.
              # Report it and move on — clobbering another installer's skill is
              # how this module would become the thing that breaks an updater
              # from the other side.
              if [ -e "$t" ] && [ ! -L "$t" ]; then
                echo "construct: $t is a real directory owned by another installer — left as is" >&2
                continue
              fi
              # A symlink on that name is re-pointed only when this module
              # made it: into `$src`, or — from an earlier generation — into
              # the store or through the pointer. Any other link is someone
              # else's (a Vercel `skills add` relative link, a user's
              # override); re-pointing it would be the clobbering the docs
              # promise never happens.
              if [ -L "$t" ]; then
                case "$(readlink "$t")" in
                  "$src"/* | ${ownLeafLink}) ;;
                  *)
                    echo "construct: $t is a symlink to $(readlink "$t"), which this module did not make — left as is" >&2
                    continue
                    ;;
                esac
              fi
              $DRY_RUN_CMD ln -sfn "$src/$n" "$t"
            done

            # Prune only what this module made: a link INTO the tree whose
            # skill the tree no longer carries. A link pointing anywhere else
            # is someone else's and is left alone.
            for l in "$canonical"/*; do
              [ -L "$l" ] || continue
              case "$(readlink "$l")" in
                "$src"/*)
                  [ -d "$src/$(basename "$l")" ] || $DRY_RUN_CMD rm -f "$l"
                  ;;
              esac
            done
          '';

          # Where every per-skill link points: through the pointer under
          # mutablePointer, straight at the store otherwise. Bound once so the
          # hub render and a per-skill agent directory can never disagree.
          hubSrc =
            if cfg.mutablePointer.enable
            then "${stateDir}/current"
            else "${cfg.package}";

          # The agent directories populated when the consumer sets nothing.
          # Bare strings, which coerce to `mode = "dir-symlink"` — the
          # historical behaviour — so an existing consumer sees no change.
          # Add more (`.copilot/skills`, `.cursor/skills`, …), and pick a mode
          # per agent, via `agentPaths`.
          defaultAgentPaths = [
            ".agent/skills"
            ".claude/skills"
            ".ai/skills"
            ".gemini/skills"
            ".codex/skills"
          ];

          # A home-relative path with any trailing `/` removed. The slash is
          # not cosmetic: `[ -L "$target/" ]` is FALSE for a symlink (the
          # trailing slash makes the kernel resolve it first), so an
          # un-normalised `.kiro/skills/` sitting on a hub link would not be
          # recognised as a link at all, and per-skill would render THROUGH
          # it — into the shared hub, the very leak the mode exists to stop.
          normalisePath = p:
            let m = builtins.match "(.*[^/])/+" p;
            in if m == null then p else builtins.head m;

          # One `agentPaths` entry. A bare string is the pre-mode spelling and
          # is coerced, so `agentPaths = [ ".claude/skills" ]` keeps meaning
          # exactly what it always did.
          agentPathType = lib.types.coercedTo lib.types.str (p: { path = p; })
            (lib.types.submodule {
              options = {
                path = lib.mkOption {
                  type = lib.types.str;
                  apply = normalisePath;
                  description = ''
                    Home-relative path of the agent's skills directory,
                    e.g. `.claude/skills`. A trailing `/` is stripped; an
                    absolute path, an empty one, or a path listed twice
                    fails evaluation.
                  '';
                };
                mode = lib.mkOption {
                  type = lib.types.enum [ "dir-symlink" "per-skill" "none" ];
                  default = "dir-symlink";
                  description = ''
                    How this module populates the directory. See
                    `agentPaths` for how to choose.
                  '';
                };
              };
            });

          # One shell block per `agentPaths` entry, dispatched on its mode at
          # evaluation time so the rendered activation script reads as a list
          # of decisions rather than a loop over a table. `path` is the
          # home-relative directory; `target` is its shell spelling.
          agentPathScript = { path, mode, ... }:
            let
              target = ''"$HOME"/${lib.escapeShellArg path}'';
            in {
              # The Vercel `skills` CLI's "universal" agents: nothing to
              # render. The only write is undoing this module's own earlier
              # dir-symlink at the path — a symlink to the hub is, by
              # construction, one this module made.
              none = ''
                # ${path}: mode = "none"
                target=${target}
                if [ -L "$target" ] && [ "$(readlink "$target")" = "$HOME/.agents/skills" ]; then
                  $DRY_RUN_CMD rm -f "$target"
                fi
              '';

              # The hub's renderer, aimed at the agent's own directory: a REAL
              # directory of per-skill links. Whatever the agent writes there
              # (Claude Code's `synced/`, Codex's `.system/`) stays private to
              # it and survives every activation, because the renderer never
              # replaces a real entry or a symlink it did not make, and prunes
              # only links into `$src` — a Vercel-style relative link
              # (`../../.agents/skills/<x>`), a real directory another
              # installer owns, or the agent's own dot-directory is untouched.
              per-skill = ''
                # ${path}: mode = "per-skill"
                target=${target}
                ours=1
                if [ -e "$target" ] && [ ! -d "$target" ] && [ ! -L "$target" ]; then
                  # A regular file (or a fifo, a socket …) on the path. The
                  # renderer's `mkdir -p` fails on it, and under the
                  # activation's `set -e` that one failure takes the whole
                  # script down — every entry after this one is left
                  # unrendered while `nixos-rebuild` still reports success.
                  # Say so and skip the entry instead.
                  echo "construct: $target is not a directory — left as is" >&2
                  ours=
                elif [ -L "$target" ]; then
                  case "$(readlink "$target")" in
                    ${ownDirLink})
                      # A directory symlink this module made in an earlier
                      # generation (to the hub, or straight into the store).
                      # It has to go before a directory can take its place.
                      $DRY_RUN_CMD rm -f "$target"
                      ;;
                    *)
                      # Someone else's link. Rendering into whatever it points
                      # at would be writing into a tree this module does not
                      # own — say so and leave the whole entry alone.
                      echo "construct: $target is a symlink to $(readlink "$target"), which this module did not make — left as is" >&2
                      ours=
                      ;;
                  esac
                fi
                if [ -n "$ours" ]; then
              '' + perSkillLinkScript { src = hubSrc; canonical = "$target"; } + ''
                fi
              '';

              # A single directory symlink to the hub. Kept for compatibility;
              # anything the agent writes under its own skills directory then
              # lands in the shared hub, which is why per-skill exists.
              dir-symlink = ''
                # ${path}: mode = "dir-symlink"
                target=${target}
                if [ -e "$target" ] && [ ! -L "$target" ]; then
                  # A REAL directory (or file) here belongs to the agent or
                  # the user — this mode never creates one, and removing it
                  # was this module's one data-loss path.
                  echo "construct: $target is not a symlink — left as is (mode = \"per-skill\" populates a real directory)" >&2
                elif [ -L "$target" ]; then
                  # The same classification per-skill applies to the path: a
                  # link this module made (to the hub, or straight into the
                  # store from an earlier generation) is re-pointed; anyone
                  # else's is reported and left exactly as found.
                  case "$(readlink "$target")" in
                    "$HOME/.agents/skills")
                      # Already right — no write, so a dry run stays quiet.
                      ;;
                    ${ownDirLink})
                      # Ours, from an earlier generation: re-point it.
                      $DRY_RUN_CMD ln -sfn "$HOME/.agents/skills" "$target"
                      ;;
                    *)
                      echo "construct: $target is a symlink to $(readlink "$target"), which this module did not make — left as is" >&2
                      ;;
                  esac
                else
                  $DRY_RUN_CMD mkdir -p "$(dirname "$target")"
                  $DRY_RUN_CMD ln -sfn "$HOME/.agents/skills" "$target"
                fi
              '';
            }.${mode};
        in {
          options.spacecraft.construct = {
            enable = lib.mkEnableOption
              "Spacecraft Software Construct cross-platform agent skills";

            enableGrok = lib.mkEnableOption
              "Spacecraft Software Construct Grok-specific agent skills";

            enableAndroid = lib.mkEnableOption
              "vendored Google Android skills (merged into ~/.agents/skills/)";

            enableOrca = lib.mkEnableOption ''
              the vendored Orca skills (`computer-use`, `orca-cli`,
              `orchestration`) in the canonical tree.

              Leave this OFF on any host that runs the Orca app. Orca installs
              and updates its own copies, and it refuses to touch a copy it
              cannot verify: its scanner throws `skill-package-link` on a file
              with `nlink != 1`, which every Nix-store file eventually is once
              store optimisation hardlinks it, and reports the skill as
              `Unrecognized` in Settings → Update skills. Matching the vendored
              bytes to the official revision does not help — a byte-identical
              copy is flagged just the same, and the store's 444 modes make the
              path `read-only` for the updater regardless.

              Turn it on where nothing else provides these skills (no Orca app
              installed, an air-gapped host, a container image)
            '';

            package = lib.mkOption {
              type = lib.types.package;
              default = mkSkills {
                inherit pkgs;
                android = cfg.enableAndroid;
                orca = cfg.enableOrca;
              };
              defaultText = lib.literalExpression
                "this flake's own combined skill tree (Android merged in when enableAndroid)";
              description = ''
                The skill tree installed as the canonical `~/.agents/skills`.

                Override it to hand in a tree built by `construct.lib.mkSkills`
                with YOUR nixpkgs, so that the derivation installed here and the
                one you expose as a flake output are literally the same store
                path. Without that, the two differ whenever your nixpkgs differs
                from this flake's — and any pinned-vs-live comparison built on
                store paths reports drift that is not there.

                Setting this supersedes `enableAndroid` and `enableOrca` for
                the installed tree; those flags then only select this option's
                default.
              '';
            };

            mutablePointer = {
              enable = lib.mkEnableOption ''
                installing `~/.agents/skills` as a symlink to a mutable pointer
                rather than straight into the Nix store.

                The tree stays a derivation; only the POINTER becomes mutable
                state — the same shape `nix profile` and Home Manager themselves
                use. `<stateDir>/current` is the pointer, and it aims at one of
                two links beside it: `pinned`, which Home Manager renders (and
                which GC-roots the tree via the generation), or `built`, which a
                user-level `construct skill sync --build` produces with `nix
                build --out-link` (GC-rooted by its own indirect root). Moving
                between them takes seconds — no rebuild and no `sudo`.

                `built` exists because `nix build --out-link` REFUSES to replace
                a link whose current target is outside the store, and `current`
                points at `pinned` after every rebuild. Building onto its own
                link sidesteps that, and lets `current` be swapped by an atomic
                rename so it is never momentarily absent.

                `flake.lock` stays authoritative: every activation re-points
                `current` at `pinned`, so a rebuild always re-asserts the lock
                and the pointer only runs ahead BETWEEN rebuilds. Consumers that
                assert lock-derived byte-identity elsewhere (a vendored copy for
                a cloud agent, say) should still compare against `pinned`
              '';

              stateDir = lib.mkOption {
                type = lib.types.str;
                default = ".local/state/construct";
                description = ''
                  Home-RELATIVE directory holding `pinned`, `built` and `current`.

                  Deliberately not under `~/.agents/`: harnesses that read
                  `~/.agents/` directly would discover a second complete copy of
                  every skill there and offer each one twice.
                '';
              };
            };

            perSkillLinks.enable = lib.mkEnableOption ''
              rendering `~/.agents/skills` — and `~/.grok/skills` when
              `enableGrok` is on — as REAL directories holding one symlink per
              skill, instead of a single directory symlink into the store (or
              into the mutable pointer).

              The skills themselves are unchanged — each entry still resolves
              into the same tree. What changes is that the directory has room
              for entries this module does not own, which is the whole point:
              an installer that manages its own skills (Orca's `orca skills
              install`, i.e. `npx skills add`) needs somewhere writable to put
              them, and a directory symlink into the store gives it nowhere.

              A real directory already sitting on a skill's name is left alone
              and reported, never replaced — and so is a symlink that points
              anywhere but into this module's own tree, the store or the
              pointer. Pruning is limited to symlinks that point INTO the
              tree, so a foreign skill survives every rebuild.

              The Grok tree needs this for the same reason and gets the same
              renderer: while `~/.grok/skills` is a whole-directory store link,
              `npx skills add` cannot create a leaf under it at all, and fails
              with EROFS — or ENOENT if it runs in the window during a rebuild
              where the old generation has been collected and the new link is
              not yet in place.

              Caveat under `mutablePointer`: the links point through
              `<stateDir>/current`, so swapping the pointer between rebuilds
              still changes what every existing link resolves to, but a skill
              the swapped-in tree ADDS is not linked until the next activation
            '';

            agentPaths = lib.mkOption {
              type = lib.types.listOf agentPathType;
              default = defaultAgentPaths;
              example = lib.literalExpression ''
                [
                  { path = ".claude/skills"; mode = "per-skill"; }
                  { path = ".gemini/config/skills"; mode = "per-skill"; }
                  { path = ".codex/skills"; mode = "none"; }
                  ".kiro/skills"
                ]
              '';
              description = ''
                Per-agent skills directories, home-relative (`.claude/skills`),
                each with a `mode` saying how — or whether — this module
                populates it. A bare string is accepted and means
                `mode = "dir-symlink"`, so an existing list keeps working
                unchanged.

                Most agents read the canonical `~/.agents/skills` themselves
                (Codex, Gemini CLI, Goose, Kimi, OpenCode, Kilo, Mimo, Cursor,
                Grok, Copilot, Orca) and need nothing here. That is the split
                the Vercel `skills` CLI draws too: one canonical tree, and
                links only into the directories of agents that cannot read it
                — Claude Code (`.claude/skills`), Kiro CLI (`.kiro/skills`),
                Qwen Code (`.qwen/skills`), Antigravity
                (`.gemini/config/skills`).

                - `"none"` — for an agent that reads `~/.agents/skills` itself.
                  The path is touched only to remove a symlink to the hub that
                  an earlier generation of this module left there; anything
                  else is left exactly as found.
                - `"per-skill"` — for an agent that only reads its own
                  directory. The path becomes a REAL directory holding one
                  symlink per Construct skill, rendered by the same code as
                  the hub. The renderer makes, re-points and prunes only its
                  own links — those into its source tree, the Nix store or
                  the pointer. A real entry on a skill's name (Claude Code's
                  `synced/`, a directory another installer owns) and a
                  symlink pointing anywhere else (a Vercel `skills add`
                  relative link, a user's override) are each reported and
                  left as found. The path itself gets the same treatment: a
                  symlink there is replaced only when it points at the hub
                  or into the Nix store; any other symlink, or a regular
                  file, is reported and the entry skipped — the activation
                  carries on with the next entry.
                - `"dir-symlink"` — the default, kept for compatibility and
                  discouraged: the path becomes one directory symlink to the
                  hub, so anything the agent writes under its own skills
                  directory lands in the shared hub, where every other agent
                  sees it. A real directory or file already at the path is
                  reported and left alone, never removed, and a symlink is
                  re-pointed only when this module made it (to the hub or
                  into the Nix store) — any other symlink is reported and
                  left as found.

                A trailing `/` on a path is stripped. An absolute path, an
                empty one, or the same path listed twice fails evaluation
                with a message naming the entry.
              '';
            };
          };

          config = lib.mkMerge [
            # Store-link install (the default). Home Manager owns
            # ~/.agents/skills outright and every change needs a rebuild.
            # Skipped under perSkillLinks, where the activation below renders
            # the directory instead — the two cannot both own that path.
            (lib.mkIf (cfg.enable
              && !cfg.mutablePointer.enable
              && !cfg.perSkillLinks.enable) {
              home.file.".agents/skills".source = cfg.package;
            })

            # Pointer install. HM owns <stateDir>/pinned — which is what keeps
            # the tree GC-rooted through the generation — and the activation
            # below owns ~/.agents/skills, pointing it at <stateDir>/current.
            (lib.mkIf (cfg.enable && cfg.mutablePointer.enable) {
              home.file."${cfg.mutablePointer.stateDir}/pinned".source = cfg.package;

              # entryAfter [ "linkGeneration" ] is load-bearing: this seeds
              # `current` from `pinned`, and `linkGeneration` is what creates
              # `pinned`. A bare writeBoundary constraint leaves the two
              # unordered and hm.dag settles such ties by NAME, which is not a
              # guarantee — it is a coincidence that happens to hold today.
              home.activation."spacecraft-construct-skill-pointer" =
                lib.hm.dag.entryAfter [ "linkGeneration" ] (''
                  $DRY_RUN_CMD mkdir -p "${stateDir}"
                ''
                # The move-aside guard belongs to the directory-symlink layout
                # ONLY. Under perSkillLinks the hub IS a real directory — that
                # is the option's whole purpose — so the guard's test was true
                # on every activation and it moved the entire hub aside each
                # time (one `skills.pre-pointer.<ts>` per rebuild: 42 of them,
                # 140 MB, on the host where this was diagnosed). The per-skill
                # entry then recreated the hub holding only this module's
                # links, so every foreign entry — Orca's three, claude.ai's
                # `synced/`, Codex's `.system/` — vanished at each rebuild
                # until its owner reinstalled it. The per-skill renderer
                # already handles the one case the guard existed for (a
                # directory symlink left by an earlier generation); a real
                # directory at the hub is its normal state there, not a fossil.
                + lib.optionalString (!cfg.perSkillLinks.enable) ''

                  # A REAL directory here predates the pointer (or Construct
                  # itself). `ln -sfn` will not replace one, it fails and takes
                  # activation down with it — so move it aside rather than
                  # letting a fossil break every rebuild.
                  if [ -d "$HOME/.agents/skills" ] && [ ! -L "$HOME/.agents/skills" ]; then
                    $DRY_RUN_CMD mv "$HOME/.agents/skills" \
                      "$HOME/.agents/skills.pre-pointer.$(date -u +%Y%m%dT%H%M%SZ)"
                  fi
                ''
                + ''

                  # Point at `pinned` — NEVER at pinned's store target. Via
                  # `pinned` the tree is rooted by this generation for free, and
                  # "am I tracking the flake?" stays a pointer comparison rather
                  # than a hash comparison.
                  #
                  # Done UNCONDITIONALLY, on every activation. Seeding only when
                  # absent looks kinder — it would preserve a pointer moved by
                  # `skill sync --build` — but it is a trap: a later rebuild
                  # bumps `pinned` to a newer tree while `current` stays on the
                  # old one, so a rebuild would leave the machine running STALE
                  # skills with nothing to indicate it. Resetting here makes the
                  # rule simple and the lock authoritative: `sync --build` moves
                  # the pointer forward between rebuilds, and a rebuild
                  # re-asserts whatever flake.lock pins. Nothing is lost, since
                  # `sync --build` moves the lock in the same breath.
                  if [ -d "${stateDir}/current" ] && [ ! -L "${stateDir}/current" ]; then
                    $DRY_RUN_CMD rm -rf "${stateDir}/current"
                  fi
                  $DRY_RUN_CMD ln -sfn "${stateDir}/pinned" "${stateDir}/current"
                ''
                # Under perSkillLinks the entry below renders ~/.agents/skills
                # as a real directory, so the pointer stops at <stateDir> and
                # this link is not made.
                + lib.optionalString (!cfg.perSkillLinks.enable) ''

                  $DRY_RUN_CMD ln -sfn "${stateDir}/current" "$HOME/.agents/skills"
                '');
            })

            # Per-skill links. ~/.agents/skills becomes a real directory whose
            # entries point into the tree one skill at a time, leaving the
            # names this module does not carry free for another installer to
            # own — which is what Orca's updater needs, since it refuses any
            # copy it cannot verify and a store copy is hardlinked (nlink != 1)
            # and read-only (444). See `enableOrca` for the full mechanism.
            (lib.mkIf (cfg.enable && cfg.perSkillLinks.enable) {
              home.activation."spacecraft-construct-per-skill-links" =
                lib.hm.dag.entryAfter
                  ([ "linkGeneration" ]
                    ++ lib.optional cfg.mutablePointer.enable
                      "spacecraft-construct-skill-pointer")
                  (perSkillLinkScript {
                    src = hubSrc;
                    canonical = "$HOME/.agents/skills";
                  });
            })

            # The same treatment for the Grok tree. It needs it for the same
            # reason and had never had it: ~/.grok/skills was a whole-directory
            # store link, so `npx skills add` could not create a leaf there at
            # all. The install fails with EROFS (the store is mounted read-only)
            # — or ENOENT if it lands in the window during a rebuild where the
            # old generation has been collected and the new link is not yet in
            # place, which reads like a missing directory and is not one.
            #
            # No pointer here: the Grok tree has no `mutablePointer` equivalent,
            # so the links go straight at the store path.
            (lib.mkIf (cfg.enableGrok && combinedGrok != null && cfg.perSkillLinks.enable) {
              home.activation."spacecraft-construct-per-skill-links-grok" =
                lib.hm.dag.entryAfter [ "linkGeneration" ]
                  (perSkillLinkScript {
                    src = "${combinedGrok}";
                    canonical = "$HOME/.grok/skills";
                  });
            })

            (lib.mkIf cfg.enable {
              # Every path is checked AFTER `normalisePath`, so `.kiro/skills`
              # and `.kiro/skills/` count as the same entry. An absolute path
              # would render as `"$HOME"/'/home/…'`; an empty one as `$HOME`
              # itself — and per-skill would then draw the whole hub into it.
              assertions =
                let
                  paths = map (e: e.path) cfg.agentPaths;
                  quoted = lib.concatMapStringsSep ", " (p: "\"${p}\"");
                  notRelative = builtins.filter
                    (p: p == "" || lib.hasPrefix "/" p) paths;
                  duplicates = lib.unique
                    (builtins.filter (p: lib.count (q: q == p) paths > 1) paths);
                in [
                  {
                    assertion = notRelative == [ ];
                    message = ''
                      spacecraft.construct.agentPaths: every `path` must be
                      non-empty and home-relative (`.claude/skills`, not
                      `/home/you/.claude/skills`); offending: ${quoted notRelative}
                    '';
                  }
                  {
                    assertion = duplicates == [ ];
                    message = ''
                      spacecraft.construct.agentPaths: each `path` may be
                      listed once (a trailing `/` is stripped before the
                      comparison); listed more than once: ${quoted duplicates}
                    '';
                  }
                ];

              # Each agent's own skills directory, populated per its
              # `agentPaths` mode. Done via activation so a dir-symlink can
              # point at the home-relative ~/.agents/skills rather than a
              # Nix-store path (which would require a rebuild on every commit
              # for the symlink target alone), and so a per-skill directory is
              # rendered in place without HM claiming the whole path.
              #
              # entryAfter [ "linkGeneration" ], not [ "writeBoundary" ]: the
              # dir-symlink mode links at ~/.agents/skills, which
              # `linkGeneration` is what creates. Under a bare writeBoundary
              # constraint the two are unordered, and hm.dag breaks such ties
              # ALPHABETICALLY — this entry ran last only because "s" sorts
              # after "l". A sibling module's writeBoundary entry named below
              # "linkGeneration" (`engramDataDir`, in the consuming config, is
              # exactly that) demonstrates the tie going the other way. State
              # the real dependency rather than relying on the name.
              #
              # Under mutablePointer this must additionally follow the pointer
              # entry, which creates `<stateDir>/current` — what every
              # per-skill link resolves through — and under perSkillLinks the
              # hub render, so the hub is complete before any agent directory
              # is drawn from the same source. The entry NAME is stable on
              # purpose: a consumer's own migration step orders itself after it.
              home.activation."spacecraft-construct-agent-symlinks" =
                lib.hm.dag.entryAfter
                  ([ "linkGeneration" ]
                    ++ lib.optional cfg.mutablePointer.enable
                      "spacecraft-construct-skill-pointer"
                    ++ lib.optional cfg.perSkillLinks.enable
                      "spacecraft-construct-per-skill-links")
                  (lib.concatMapStringsSep "\n" agentPathScript cfg.agentPaths);
            })

            (lib.mkIf (cfg.enableGrok && combinedGrok != null
                       && !cfg.perSkillLinks.enable) {
              # Grok exception — its bundle format is flat, so it gets its
              # own install path and is NOT symlinked from ~/.agents/skills.
              # Skipped under perSkillLinks, where the activation above renders
              # the directory instead — the two cannot both own that path.
              home.file.".grok/skills".source = combinedGrok;
            })
          ];
        };

      # ───────────────────────────────────────────────────────────────────
      # Convenience: list of detected skill names (useful for `nix eval`).
      # ───────────────────────────────────────────────────────────────────
      lib = {
        inherit crossPlatformSkills grokSkills androidSkills orcaSkills;

        # Build a skill tree with the CALLER's nixpkgs. Pass the result to both
        # your own flake output and `spacecraft.construct.package` so the two
        # are one store path — see that option's description for why comparing
        # separately-built trees does not work.
        inherit mkSkills;
      };
    };
}
