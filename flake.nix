{
  description = "Curriculum — generated skill surface assembler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/2d1e72b652ee13fd1297641ce735e06416d22827"; # lunation 2026-08-12
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
