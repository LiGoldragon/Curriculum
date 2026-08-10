{
  description = "skills — generated skill surface assembler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-build,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromPkgs pkgs;
        inherit (rust) craneLib toolchain;

        skillSourceFilter =
          path: type:
          type == "directory"
          || pkgs.lib.hasSuffix ".md" path
          || pkgs.lib.hasSuffix ".dotos" path
          || pkgs.lib.hasSuffix ".rs" path;

        cleanSource = rust.cleanSource {
          root = ./.;
          extraFilters = [ skillSourceFilter ];
        };

        src = cleanSource;

        cargoVendorDirectory = craneLib.vendorCargoDeps {
          inherit src;
          cargoLock = ./Cargo.lock;
        };

        commonArguments = {
          inherit src cargoVendorDirectory;
          cargoLock = ./Cargo.lock;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
        skillsPackage = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });

        generatorApp =
          name: requestFile: requiresConsumerWorkspace:
          let
            workspaceSetup =
              if requiresConsumerWorkspace then
                ''
                  if [ -z "''${SKILLS_WORKSPACE_ROOT:-}" ]; then
                    echo "SKILLS_WORKSPACE_ROOT must name an explicit consumer workspace" >&2
                    exit 2
                  fi
                  if [ -f "$SKILLS_WORKSPACE_ROOT/Cargo.toml" ] \
                    && [ -f "$SKILLS_WORKSPACE_ROOT/skills-generate.dotos" ] \
                    && [ -f "$SKILLS_WORKSPACE_ROOT/manifests/active-outputs.dotos" ]; then
                    echo "SKILLS_WORKSPACE_ROOT must not be the skills source checkout" >&2
                    exit 2
                  fi
                ''
              else
                ''
                  export SKILLS_WORKSPACE_ROOT="''${SKILLS_WORKSPACE_ROOT:-$PWD}"
                '';
            script = pkgs.writeShellApplication {
              inherit name;
              runtimeInputs = [ skillsPackage ];
              # The single-argument rule (standard-component-architecture.md:
              # "Give every executable exactly one argument: a DOTOS payload
              # carrying its fully typed configuration") applies to this
              # wrapper too. It never treats its one optional argument as a
              # bare workspace-root path: omitted, the fixed request file
              # below runs against the explicitly supplied consumer workspace
              # for generation and checking; visualization is read-only and
              # may inspect the current source checkout;
              # given, the argument is forwarded
              # verbatim as the `skills` binary's one DOTOS argument (inline
              # literal or a path to a `.dotos` file), so a stray flag like
              # `--write` fails DOTOS decoding loudly instead of silently
              # becoming a directory name.
              text = ''
                if [ "$#" -gt 1 ]; then
                  echo "usage: ${name} [dotos-payload]" >&2
                  exit 2
                fi
                ${workspaceSetup}
                export SKILLS_SOURCE_ROOT=${cleanSource}
                if [ "$#" -eq 1 ]; then
                  exec skills "$1"
                fi
                exec skills ${cleanSource}/${requestFile}
              '';
            };
          in
          {
            type = "app";
            program = "${script}/bin/${name}";
            meta.description =
              if requiresConsumerWorkspace then
                "Run ${name} against an explicit consumer workspace"
              else
                "Inspect generated outputs without writing a workspace";
          };
      in
      rec {
        packages = {
          skills = skillsPackage;
          default = skillsPackage;
        };

        apps = rec {
          skills = {
            type = "app";
            program = "${skillsPackage}/bin/skills";
            meta.description = "Run the skills generator CLI";
          };
          generate-skills = generatorApp "generate-skills" "skills-generate.dotos" true;
          check-skills = generatorApp "check-skills" "skills-check.dotos" true;
          visualize-skills = generatorApp "visualize-skills" "skills-visualize.dotos" false;
          default = skills;
        };

        checks = rec {
          skills = skillsPackage;
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (
            commonArguments
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
            }
          );
          no-hard-coded-generation-roots = pkgs.runCommand "skills-no-hard-coded-generation-roots" { } ''
            grep -F '$SKILLS_SOURCE_ROOT' ${cleanSource}/skills-check.dotos >/dev/null
            grep -F '$SKILLS_WORKSPACE_ROOT' ${cleanSource}/skills-check.dotos >/dev/null
            grep -F '$SKILLS_SOURCE_ROOT' ${cleanSource}/skills-generate.dotos >/dev/null
            grep -F '$SKILLS_WORKSPACE_ROOT' ${cleanSource}/skills-generate.dotos >/dev/null
            grep -F '$SKILLS_SOURCE_ROOT' ${cleanSource}/skills-visualize.dotos >/dev/null
            grep -F '$SKILLS_WORKSPACE_ROOT' ${cleanSource}/skills-visualize.dotos >/dev/null
            if grep -n -E '/(home|git)/' ${cleanSource}/skills-check.dotos ${cleanSource}/skills-generate.dotos ${cleanSource}/skills-visualize.dotos; then
              echo "generator requests must not hard-code source or workspace roots" >&2
              exit 1
            fi
            touch "$out"
          '';
          visualize-request-is-non-writing = pkgs.runCommand "skills-visualize-request-is-non-writing" { } ''
            grep -F 'Visualize.' ${cleanSource}/skills-visualize.dotos >/dev/null
            if grep -E 'Generate\.| Write}' ${cleanSource}/skills-visualize.dotos; then
              echo "visualization request must not write generated output" >&2
              exit 1
            fi
            touch "$out"
          '';
          check-request-is-non-writing = pkgs.runCommand "skills-check-request-is-non-writing" { } ''
            grep -F ' Check}' ${cleanSource}/skills-check.dotos >/dev/null
            if grep -F ' Write}' ${cleanSource}/skills-check.dotos; then
              echo "check request must not use Write mode" >&2
              exit 1
            fi
            touch "$out"
          '';
          generation-requests-use-active-manifest =
            pkgs.runCommand "skills-generation-requests-use-active-manifest" { }
              ''
                grep -F 'manifests/active-outputs.dotos' ${cleanSource}/skills-check.dotos >/dev/null
                grep -F 'manifests/active-outputs.dotos' ${cleanSource}/skills-generate.dotos >/dev/null
                grep -F 'manifests/active-outputs.dotos' ${cleanSource}/skills-visualize.dotos >/dev/null
                if find ${cleanSource}/manifests -mindepth 2 -type f -name '*.dotos' | grep .; then
                  echo "generation must be driven by the single active output manifest, not per-output manifests" >&2
                  exit 1
                fi
                touch "$out"
              '';
          flat-active-source-layout = pkgs.runCommand "skills-flat-active-source-layout" { } ''
            index=${cleanSource}/manifests/module-dependencies.dotos
            manifest=${cleanSource}/manifests/active-outputs.dotos
            test ! -e ${cleanSource}/modules
            test ! -e ${cleanSource}/skills/archive
            test ! -e ${cleanSource}/manifests/skills-roster.dotos
            if grep -E 'modules/|/full\.md|architecture-editor|skills/archive' "$index" "$manifest"; then
              echo "active source manifests must use flat current paths and names" >&2
              exit 1
            fi
            while read -r source; do
              test -f "${cleanSource}/$source"
            done < <(sed -n 's/^[[:space:]]*(\([^[:space:]]*\)[[:space:]]\+\([^[:space:]]*\.md\)[[:space:]].*/\2/p' "$index")
            test ! -e ${cleanSource}/roles
            if find ${cleanSource}/skills -mindepth 2 -type f -name '*.md' | grep .; then
              echo "active source files must not use nested directories" >&2
              exit 1
            fi
            touch "$out"
          '';
          canonical-source-has-no-runtime-trees-or-retired-skills = pkgs.runCommand "skills-canonical-source-has-no-runtime-trees-or-retired-skills" { } ''
            for tree in .agents .claude .codex .pi; do
              test ! -e "${cleanSource}/$tree"
            done
            for retired in engine-analysis working pi-extension-updates; do
              test ! -e "${cleanSource}/skills/$retired.md"
              if grep -R -n -E "engine-analysis|pi-extension-updates|working\\.md|Skill\\.\\{working|\\{working " "${cleanSource}/manifests" "${cleanSource}/skills"; then
                echo "retired skill remains in canonical manifests or sources: $retired" >&2
                exit 1
              fi
            done
            touch "$out"
          '';
          source-checkout-workspace-is-rejected = pkgs.runCommand "skills-source-checkout-workspace-is-rejected" { } ''
            workspace=$TMPDIR/source-checkout
            mkdir -p "$workspace"
            cp -R ${cleanSource}/. "$workspace/"
            export SKILLS_SOURCE_ROOT=${cleanSource}
            export SKILLS_WORKSPACE_ROOT="$workspace"
            if ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.dotos >"$TMPDIR/stdout" 2>"$TMPDIR/stderr"; then
              echo "generation unexpectedly wrote into a source checkout" >&2
              exit 1
            fi
            grep -F "choose a distinct consumer workspace" "$TMPDIR/stderr" >/dev/null
            test ! -e "$workspace/.agents"
            test ! -e "$workspace/.claude"
            test ! -e "$workspace/.codex"
            test ! -e "$workspace/.pi"
            touch "$out"
          '';
          generator-interface-requires-explicit-consumer = pkgs.runCommand "skills-generator-interface-requires-explicit-consumer" { } ''
            if ${apps.generate-skills.program} >"$TMPDIR/default-stdout" 2>"$TMPDIR/default-stderr"; then
              echo "the default generator invocation must not choose a workspace" >&2
              exit 1
            fi
            grep -F "SKILLS_WORKSPACE_ROOT must name an explicit consumer workspace" "$TMPDIR/default-stderr" >/dev/null
            workspace=$TMPDIR/consumer
            mkdir -p "$workspace"
            export SKILLS_WORKSPACE_ROOT="$workspace"
            ${apps.generate-skills.program} >/dev/null
            test -f "$workspace/.agents/skills/psyche/SKILL.md"
            test -f "$workspace/.claude/skills/psyche/SKILL.md"
            test -f "$workspace/.codex/agents/read-trivial.toml"
            test -f "$workspace/.pi/agents/write-critical.md"
            touch "$out"
          '';
          visualize-source-checkout-is-read-only = pkgs.runCommand "skills-visualize-source-checkout-is-read-only" { } ''
            workspace=$TMPDIR/source-checkout
            mkdir -p "$workspace"
            cp -R ${cleanSource}/. "$workspace/"
            (cd "$workspace" && SKILLS_WORKSPACE_ROOT= ${apps.visualize-skills.program} >/dev/null)
            for tree in .agents .claude .codex .pi; do
              test ! -e "$workspace/$tree"
            done
            touch "$out"
          '';
          orphaned-output-cleanup =
            pkgs.runCommand "skills-orphaned-output-cleanup" { }
              ''
                workspace=$TMPDIR/workspace
                mkdir -p "$workspace/.agents/skills/human-interaction" "$workspace/.claude/skills/human-interaction"
                printf 'stale\n' > "$workspace/.agents/skills/human-interaction/SKILL.md"
                printf 'stale\n' > "$workspace/.claude/skills/human-interaction/SKILL.md"
                export SKILLS_SOURCE_ROOT=${cleanSource}
                export SKILLS_WORKSPACE_ROOT="$workspace"
                ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.dotos >/dev/null
                test ! -e "$workspace/.agents/skills/human-interaction/SKILL.md"
                test ! -e "$workspace/.claude/skills/human-interaction/SKILL.md"
                touch "$out"
              '';
          role-cross-product-manifests = pkgs.runCommand "skills-role-cross-product-manifests" { } ''
            permissions=${cleanSource}/manifests/role-permissions.dotos
            depths=${cleanSource}/manifests/role-depths.dotos
            descriptions=${cleanSource}/manifests/role-descriptions.dotos
            catalog=${cleanSource}/manifests/model-catalog.dotos
            for retired in \
              manifests/role-model-assignments.dotos \
              manifests/role-model-profiles.dotos \
              manifests/role-optional-skills.dotos \
              manifests/nested-role-relations.dotos; do
              test ! -e "${cleanSource}/$retired"
            done
            grep -F '{read (|Do not edit files, commit, or push. Fetching, cloning, and tool queries are fine.|) Restricted}' "$permissions" >/dev/null
            grep -F '{write (||) Unrestricted}' "$permissions" >/dev/null
            grep -F '{claude-haiku-4-5 Claude []}' "$catalog" >/dev/null
            grep -F '{gpt-5.4-mini ChatGpt [Low Medium High Xhigh]}' "$catalog" >/dev/null
            grep -F '{trivial {claude-haiku-4-5 None} {gpt-5.4-mini Some.Medium}}' "$depths" >/dev/null
            critical_row=$(grep -F '{critical {(|claude-opus-4-6[1m]|) Some.High}' "$depths")
            test -n "$critical_row"
            critical_model=$(printf '%s' "$critical_row" | sed -E 's/.*\{([A-Za-z0-9.-]+) Some\.[A-Za-z]+\}\}$/\1/')
            critical_effort=$(printf '%s' "$critical_row" | sed -E 's/.*\{[A-Za-z0-9.-]+ Some\.([A-Za-z]+)\}\}$/\1/')
            test -n "$critical_model"
            test -n "$critical_effort"
            grep -F "{$critical_model ChatGpt" "$catalog" >/dev/null
            grep -F "{$critical_model ChatGpt" "$catalog" | grep -F "$critical_effort" >/dev/null
            test "$(grep -c '^  {' "$descriptions")" -eq 8
            if grep -F '(Role (' ${cleanSource}/manifests/active-outputs.dotos; then
              echo "roles are generated from the permission-by-depth cross product, not listed as active outputs" >&2
              exit 1
            fi
            workspace=$TMPDIR/workspace
            export SKILLS_SOURCE_ROOT=${cleanSource}
            export SKILLS_WORKSPACE_ROOT="$workspace"
            ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.dotos >/dev/null
            for permission in read write; do
              for depth in trivial ordinary demanding critical; do
                test -f "$workspace/.claude/agents/$permission-$depth.md"
                test -f "$workspace/.codex/agents/$permission-$depth.toml"
                test -f "$workspace/.pi/agents/$permission-$depth.md"
              done
            done
            for depth in trivial ordinary demanding critical; do
              grep -F 'disallowedTools' "$workspace/.claude/agents/read-$depth.md" >/dev/null
              grep -F 'Edit, Write, NotebookEdit' "$workspace/.claude/agents/read-$depth.md" >/dev/null
              grep -F 'disallowed_tools' "$workspace/.pi/agents/read-$depth.md" >/dev/null
              grep -F 'edit, write' "$workspace/.pi/agents/read-$depth.md" >/dev/null
              ! grep -F 'disallowedTools' "$workspace/.claude/agents/write-$depth.md"
              ! grep -F 'disallowed_tools' "$workspace/.pi/agents/write-$depth.md"
            done
            ! grep -F 'effort:' "$workspace/.claude/agents/read-trivial.md"
            grep -Fx 'model: claude-haiku-4-5' "$workspace/.claude/agents/read-trivial.md" >/dev/null
            touch "$out"
          '';
          default = test;
        };

        devShells.default = pkgs.mkShell {
          name = "skills";
          packages = [
            pkgs.jujutsu
            toolchain
          ];
        };
      }
    );
}
