{
  description = "skills — generated skill surface assembler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dotos-source = {
      url = "github:LiGoldragon/dotos/1facca44fbcb37633f71fcf6f73bd693fbe56a5e";
      flake = false;
    };
    schema-source = {
      url = "github:LiGoldragon/schema/f351f90d3b8898205cf3057f3c253a5e451180a9";
      flake = false;
    };
    schema-rust-source = {
      url = "github:LiGoldragon/schema-rust";
      flake = false;
    };
    signal-frame-source = {
      url = "github:LiGoldragon/signal-frame/bb86bef67e478ff52690a4dcceec8f22d2b005ad";
      flake = false;
    };
    triad-runtime-source = {
      url = "github:LiGoldragon/triad-runtime/0031b5519572f4571bf3895f78221de9404d4810";
      flake = false;
    };
    kameo-source = {
      url = "github:LiGoldragon/kameo/main";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-build,
      dotos-source,
      schema-source,
      schema-rust-source,
      signal-frame-source,
      triad-runtime-source,
      kameo-source,
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
          || pkgs.lib.hasSuffix ".schema" path;

        cleanSource = rust.cleanSource {
          root = ./.;
          extraFilters = [ skillSourceFilter ];
        };

        src = pkgs.runCommand "skills-source-with-flake-input-patches" { } ''
            mkdir -p "$out"
            cp -R ${cleanSource}/. "$out"/
            chmod -R u+w "$out"
            mkdir -p "$out/vendor-sources"
            cp -R ${dotos-source} "$out/vendor-sources/dotos"
            cp -R ${schema-source} "$out/vendor-sources/schema"
            cp -R ${schema-rust-source} "$out/vendor-sources/schema-rust"
            cp -R ${signal-frame-source} "$out/vendor-sources/signal-frame"
            cp -R ${triad-runtime-source} "$out/vendor-sources/triad-runtime"
            cp -R ${kameo-source} "$out/vendor-sources/kameo"
            chmod -R u+w "$out/vendor-sources"
            cat >> "$out/Cargo.toml" <<'EOF'

          [patch."https://github.com/LiGoldragon/dotos.git"]
          dotos = { path = "vendor-sources/dotos" }
          dotos-derive = { path = "vendor-sources/dotos/derive" }

          [patch."https://github.com/LiGoldragon/schema.git"]
          schema = { path = "vendor-sources/schema" }
          schema-cc = { path = "vendor-sources/schema/schema-cc" }

          [patch."https://github.com/LiGoldragon/schema-rust.git"]
          schema-rust = { path = "vendor-sources/schema-rust" }

          [patch."https://github.com/LiGoldragon/signal-frame.git"]
          signal-frame = { path = "vendor-sources/signal-frame" }
          signal-frame-macros = { path = "vendor-sources/signal-frame/macros" }

          [patch."https://github.com/LiGoldragon/triad-runtime.git"]
          triad-runtime = { path = "vendor-sources/triad-runtime" }

          [patch."https://github.com/LiGoldragon/kameo.git"]
          kameo = { path = "vendor-sources/kameo" }
          kameo_macros = { path = "vendor-sources/kameo/macros" }
          EOF
        '';

        patchedCargoLock = pkgs.runCommand "skills-patched-Cargo.lock" { } ''
          ${pkgs.python3}/bin/python3 - ${./Cargo.lock} "$out" <<'PYEOF'
          import re
          import sys

          path_dependency_names = {
              "kameo",
              "kameo_macros",
              "dotos",
              "dotos-derive",
              "schema",
              "schema-cc",
              "schema-rust",
              "signal-frame",
              "signal-frame-macros",
              "triad-runtime",
          }
          source_text = open(sys.argv[1]).read()
          blocks = source_text.split("[[package]]")
          header, entries = blocks[0], blocks[1:]

          def field(entry, name):
              found = re.search(r'^%s = "([^"]*)"' % name, entry, re.M)
              return found.group(1) if found else ""

          stripped = []
          for entry in entries:
              if field(entry, "name") in path_dependency_names:
                  entry = "\n".join(
                      line for line in entry.split("\n")
                      if not line.startswith('source = "git+https://github.com/LiGoldragon/')
                  )
              stripped.append(entry)

          open(sys.argv[2], "w").write(header + "".join("[[package]]" + entry for entry in stripped))
          PYEOF
        '';

        cargoVendorDirectory = craneLib.vendorCargoDeps {
          inherit src;
          cargoLock = patchedCargoLock;
        };

        commonArguments = {
          inherit src cargoVendorDirectory;
          cargoLock = patchedCargoLock;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
        skillsPackage = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });

        generatorApp =
          name: requestFile:
          let
            script = pkgs.writeShellApplication {
              inherit name;
              runtimeInputs = [ skillsPackage ];
              # The single-argument rule (standard-component-architecture.md:
              # "Give every executable exactly one argument: a DOTOS payload
              # carrying its fully typed configuration") applies to this
              # wrapper too. It never treats its one optional argument as a
              # bare workspace-root path: omitted, the fixed request file
              # below runs against $PWD; given, the argument is forwarded
              # verbatim as the `skills` binary's one DOTOS argument (inline
              # literal or a path to a `.dotos` file), so a stray flag like
              # `--write` fails DOTOS decoding loudly instead of silently
              # becoming a directory name.
              text = ''
                if [ "$#" -gt 1 ]; then
                  echo "usage: ${name} [dotos-payload]" >&2
                  exit 2
                fi
                export SKILLS_SOURCE_ROOT=${cleanSource}
                export SKILLS_WORKSPACE_ROOT="$PWD"
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
            meta.description = "Run ${name} against an explicit workspace root";
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
          generate-skills = generatorApp "generate-skills" "skills-generate.dotos";
          check-skills = generatorApp "check-skills" "skills-check.dotos";
          visualize-skills = generatorApp "visualize-skills" "skills-visualize.dotos";
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
            if grep -n -E '/(home|git)/' ${cleanSource}/skills-check.dotos ${cleanSource}/skills-generate.dotos; then
              echo "generation requests must not hard-code source or workspace roots" >&2
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
            grep -F '(read [|Do not edit files, commit, or push. Fetching, cloning, and tool queries are fine.|] Restricted)' "$permissions" >/dev/null
            grep -F '(write [] Unrestricted)' "$permissions" >/dev/null
            grep -F '(claude-haiku-4-5 Claude [])' "$catalog" >/dev/null
            grep -F '(gpt-5.4-mini ChatGpt [Low Medium High Xhigh])' "$catalog" >/dev/null
            grep -F '(trivial (claude-haiku-4-5 None) (gpt-5.4-mini (Some Medium)))' "$depths" >/dev/null
            critical_row=$(grep -F '(critical ([|claude-opus-4-6[1m]|] (Some High))' "$depths")
            test -n "$critical_row"
            critical_model=$(printf '%s' "$critical_row" | sed -E 's/.*\)\) \(([A-Za-z0-9.-]+) \(Some ([A-Za-z]+)\)\)\)$/\1/')
            critical_effort=$(printf '%s' "$critical_row" | sed -E 's/.*\)\) \(([A-Za-z0-9.-]+) \(Some ([A-Za-z]+)\)\)\)$/\2/')
            test -n "$critical_model"
            test -n "$critical_effort"
            grep -F "($critical_model ChatGpt" "$catalog" >/dev/null
            grep -F "($critical_model ChatGpt" "$catalog" | grep -F "$critical_effort" >/dev/null
            test "$(grep -c '^  (' "$descriptions")" -eq 8
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
