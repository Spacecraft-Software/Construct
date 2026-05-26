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
      excludedDirs = [ "grok-skills" "Excluded" ".claude" ".git" ];

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

      # Combined derivation — used internally by the HM module, NOT exposed
      # as a flake `packages` output.
      mkCombined = pkgs: source: skillList: outName:
        pkgs.runCommandLocal outName { } (''
          mkdir -p $out
        '' + nixpkgs.lib.concatMapStringsSep "\n" (n: ''
          mkdir -p $out/${n}
          cp -r ${source}/${n}/. $out/${n}/
        '') skillList);
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

          combinedCrossPlatform =
            mkCombined pkgs self crossPlatformSkills "construct-skills";

          combinedGrok =
            if grokSkills == [] then null
            else mkCombined pkgs (self + "/grok-skills") grokSkills
                   "construct-grok-skills";

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
            (lib.mkIf cfg.enable {
              # Canonical install — agents resolve all paths through here.
              home.file.".agents/skills".source = combinedCrossPlatform;

              # Per-harness directory symlinks. Done via activation so the
              # symlink can point at the home-relative ~/.agents/skills
              # rather than a Nix-store path (which would require rebuild
              # on every commit for the symlink target alone).
              home.activation."spacecraft-construct-agent-symlinks" =
                lib.hm.dag.entryAfter [ "writeBoundary" ] ''
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
        inherit crossPlatformSkills grokSkills;
      };
    };
}
