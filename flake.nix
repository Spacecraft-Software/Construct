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
      # Vendored Orca skills — same open-standard SKILL.md format, merged into
      # the canonical tree unconditionally (unlike the opt-in Android set): the
      # `orca` CLI looks its skills up by exact leaf name, so they have to be
      # present wherever an agent reads skills from, not behind a toggle.
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
      # skills plus the vendored Orca ones. Leaf names don't collide —
      # cross-platform skills are all spacecraft-* / gnu-* / microsoft-* /
      # steelbore-*, and the three Orca leaves are distinct from those — so a
      # flat merge is safe. `orca-skills/CREDITS.md` records that the generic
      # Orca leaf names (`computer-use`, `orchestration`) are reserved and must
      # not be claimed by a future Spacecraft skill.
      baseSources = [
        { source = self; names = crossPlatformSkills; }
      ] ++ nixpkgs.lib.optional (orcaSkills != [])
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
      mkSkills = { pkgs, android ? false, grok ? false }:
        if grok then
          mkCombined pkgs (self + "/grok-skills") grokSkills "construct-grok-skills"
        else if android && androidSkills != [] then
          mkMerged pkgs "construct-skills-with-android"
            (baseSources ++ [
              { source = self + "/android-skills"; names = androidSkills; }
            ])
        else
          mkMerged pkgs "construct-skills" baseSources;
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
      # Wires up the canonical ~/.agents/skills/ location, symlinks every
      # known agent harness's skill path to it, and (when enableGrok is on)
      # installs Grok skills to ~/.grok/skills/.
      homeManagerModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.spacecraft.construct;

          combinedGrok =
            if grokSkills == [] then null
            else mkSkills { inherit pkgs; grok = true; };

          # Absolute path of the pointer directory, and of the two links in it.
          stateDir = "$HOME/${cfg.mutablePointer.stateDir}";

          # Per-harness paths that should symlink to ~/.agents/skills.
          # Extensible — add more (`.copilot/skills`, `.cursor/skills`, …)
          # by passing them in `agentPaths`.
          defaultAgentPaths = [
            ".agent/skills"
            ".claude/skills"
            ".ai/skills"
            ".gemini/skills"
            ".codex/skills"
          ];
        in {
          options.spacecraft.construct = {
            enable = lib.mkEnableOption
              "Spacecraft Software Construct cross-platform agent skills";

            enableGrok = lib.mkEnableOption
              "Spacecraft Software Construct Grok-specific agent skills";

            enableAndroid = lib.mkEnableOption
              "vendored Google Android skills (merged into ~/.agents/skills/)";

            package = lib.mkOption {
              type = lib.types.package;
              default = mkSkills {
                inherit pkgs;
                android = cfg.enableAndroid;
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

                Setting this supersedes `enableAndroid` for the installed tree;
                `enableAndroid` then only selects this option's default.
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

            agentPaths = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              default = defaultAgentPaths;
              description = ''
                Home-relative paths to symlink to ~/.agents/skills/.
                Each entry becomes a directory symlink so any agent harness
                that scans one of these locations sees the same skill set.
              '';
            };
          };

          config = lib.mkMerge [
            # Store-link install (the default). Home Manager owns
            # ~/.agents/skills outright and every change needs a rebuild.
            (lib.mkIf (cfg.enable && !cfg.mutablePointer.enable) {
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
                lib.hm.dag.entryAfter [ "linkGeneration" ] ''
                  $DRY_RUN_CMD mkdir -p "${stateDir}"

                  # A REAL directory here predates the pointer (or Construct
                  # itself). `ln -sfn` will not replace one, it fails and takes
                  # activation down with it — so move it aside rather than
                  # letting a fossil break every rebuild.
                  if [ -d "$HOME/.agents/skills" ] && [ ! -L "$HOME/.agents/skills" ]; then
                    $DRY_RUN_CMD mv "$HOME/.agents/skills" \
                      "$HOME/.agents/skills.pre-pointer.$(date -u +%Y%m%dT%H%M%SZ)"
                  fi

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

                  $DRY_RUN_CMD ln -sfn "${stateDir}/current" "$HOME/.agents/skills"
                '';
            })

            (lib.mkIf cfg.enable {
              # Per-harness directory symlinks. Done via activation so the
              # symlink can point at the home-relative ~/.agents/skills
              # rather than a Nix-store path (which would require rebuild
              # on every commit for the symlink target alone).
              #
              # entryAfter [ "linkGeneration" ], not [ "writeBoundary" ]: this
              # loop links at ~/.agents/skills, which `linkGeneration` is what
              # creates. Under a bare writeBoundary constraint the two are
              # unordered, and hm.dag breaks such ties ALPHABETICALLY — this
              # entry ran last only because "s" sorts after "l". A sibling
              # module's writeBoundary entry named below "linkGeneration"
              # (`engramDataDir`, in the consuming config, is exactly that)
              # demonstrates the tie going the other way. State the real
              # dependency rather than relying on the name.
              #
              # Under mutablePointer this must additionally follow the pointer
              # entry, which is what creates ~/.agents/skills at all.
              home.activation."spacecraft-construct-agent-symlinks" =
                lib.hm.dag.entryAfter
                  ([ "linkGeneration" ]
                    ++ lib.optional cfg.mutablePointer.enable
                      "spacecraft-construct-skill-pointer") ''
                  for p in ${lib.escapeShellArgs cfg.agentPaths}; do
                    target="$HOME/$p"
                    # Remove anything that isn't already the right symlink.
                    if [ -L "$target" ] || [ -e "$target" ]; then
                      current="$(readlink "$target" 2>/dev/null || true)"
                      if [ "$current" = "$HOME/.agents/skills" ]; then
                        continue
                      fi
                      $DRY_RUN_CMD rm -rf "$target"
                    fi
                    $DRY_RUN_CMD mkdir -p "$(dirname "$target")"
                    $DRY_RUN_CMD ln -s "$HOME/.agents/skills" "$target"
                  done
                '';
            })

            (lib.mkIf (cfg.enableGrok && combinedGrok != null) {
              # Grok exception — its bundle format is flat, so it gets its
              # own install path and is NOT symlinked from ~/.agents/skills.
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
