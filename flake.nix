{
  description = "skills — generated skill surface assembler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nota-source = {
      url = "github:LiGoldragon/nota/ce7c564de0a0518eaa1938d55dccc460a67cadb4";
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
      nota-source,
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
          || pkgs.lib.hasSuffix ".nota" path
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
            cp -R ${nota-source} "$out/vendor-sources/nota"
            cp -R ${schema-source} "$out/vendor-sources/schema"
            cp -R ${schema-rust-source} "$out/vendor-sources/schema-rust"
            cp -R ${signal-frame-source} "$out/vendor-sources/signal-frame"
            cp -R ${triad-runtime-source} "$out/vendor-sources/triad-runtime"
            cp -R ${kameo-source} "$out/vendor-sources/kameo"
            chmod -R u+w "$out/vendor-sources"
            cat >> "$out/Cargo.toml" <<'EOF'

          [patch."https://github.com/LiGoldragon/nota.git"]
          nota = { path = "vendor-sources/nota" }
          nota-derive = { path = "vendor-sources/nota/derive" }

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
              "nota",
              "nota-derive",
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
              text = ''
                if [ "$#" -gt 1 ]; then
                  echo "usage: ${name} [workspace-root]" >&2
                  exit 2
                fi
                workspace_root="''${1:-$PWD}"
                export SKILLS_SOURCE_ROOT=${cleanSource}
                export SKILLS_WORKSPACE_ROOT="$workspace_root"
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
          generate-skills = generatorApp "generate-skills" "skills-generate.nota";
          check-skills = generatorApp "check-skills" "skills-check.nota";
          visualize-skills = generatorApp "visualize-skills" "skills-visualize.nota";
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
            grep -F '$SKILLS_SOURCE_ROOT' ${cleanSource}/skills-check.nota >/dev/null
            grep -F '$SKILLS_WORKSPACE_ROOT' ${cleanSource}/skills-check.nota >/dev/null
            grep -F '$SKILLS_SOURCE_ROOT' ${cleanSource}/skills-generate.nota >/dev/null
            grep -F '$SKILLS_WORKSPACE_ROOT' ${cleanSource}/skills-generate.nota >/dev/null
            if grep -n -E '/(home|git)/' ${cleanSource}/skills-check.nota ${cleanSource}/skills-generate.nota; then
              echo "generation requests must not hard-code source or workspace roots" >&2
              exit 1
            fi
            touch "$out"
          '';
          check-request-is-non-writing = pkgs.runCommand "skills-check-request-is-non-writing" { } ''
            grep -F ' Check))' ${cleanSource}/skills-check.nota >/dev/null
            if grep -F ' Write))' ${cleanSource}/skills-check.nota; then
              echo "check request must not use Write mode" >&2
              exit 1
            fi
            touch "$out"
          '';
          generation-requests-use-active-manifest =
            pkgs.runCommand "skills-generation-requests-use-active-manifest" { }
              ''
                grep -F 'manifests/active-outputs.nota' ${cleanSource}/skills-check.nota >/dev/null
                grep -F 'manifests/active-outputs.nota' ${cleanSource}/skills-generate.nota >/dev/null
                if find ${cleanSource}/manifests -mindepth 2 -type f -name '*.nota' | grep .; then
                  echo "generation must be driven by the single active output manifest, not per-output manifests" >&2
                  exit 1
                fi
                touch "$out"
              '';
          flat-active-source-layout = pkgs.runCommand "skills-flat-active-source-layout" { } ''
            index=${cleanSource}/manifests/module-dependencies.nota
            manifest=${cleanSource}/manifests/active-outputs.nota
            test ! -e ${cleanSource}/modules
            test ! -e ${cleanSource}/skills/archive
            test ! -e ${cleanSource}/manifests/skills-roster.nota
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
            grep -F '(Skill (documentation-placement documentation-placement ' "$manifest" >/dev/null
            touch "$out"
          '';
          interaction-skill-composition-guardrails = pkgs.runCommand "skills-interaction-skill-composition-guardrails" { } ''
            management=${cleanSource}/skills/management.md
            psyche=${cleanSource}/skills/psyche-interraction.md
            briefness=${cleanSource}/skills/psyche-interraction-claude-briefness.md
            continuation=${cleanSource}/skills/psyche-interraction-continuation.md
            tenets=${cleanSource}/skills/tenets.md
            handover=${cleanSource}/skills/context-handover.md
            index=${cleanSource}/manifests/module-dependencies.nota
            compositions=${cleanSource}/manifests/skill-module-compositions.nota
            insertions=${cleanSource}/manifests/target-module-insertions.nota
            universal=${cleanSource}/manifests/universal-role-modules.nota
            manifest=${cleanSource}/manifests/active-outputs.nota
            test -f "$management"
            test -f "$psyche"
            test -f "$briefness"
            test -f "$continuation"
            test -f "$tenets"
            test -f "$handover"
            expected_management=$TMPDIR/management
            printf '%s\n' \
              'Reserve your context for managing subagents.' \
              'Delegate all task work.' \
              'Read and write beads, reports, design documents, and the design log.' \
              "" \
              '{% if codex %}' \
              'Keep a wait active while agents run; you are resumed only through it.' \
              'Pass the longest timeout the wait accepts. Omitting it defaults to thirty seconds.' \
              'A psyche message interrupts the wait immediately, so a long wait costs no responsiveness.' \
              'Say nothing when a wait returns with nothing finished.' \
              '{% endif %}' \
              > "$expected_management"
            cmp "$management" "$expected_management"
            grep -F '(Skill (management management Meta Mechanism [|Use when coordinating delegated work for a caller.|] [AgentsSkill ClaudeSkill]))' "$manifest" >/dev/null
            grep -F '(Skill (psyche-interraction psyche-interraction Meta Mechanism [|Use when interacting directly with the psyche.|] [AgentsSkill ClaudeSkill]))' "$manifest" >/dev/null
            grep -F '(Skill (tenets tenets Meta Mechanism [|Use in every agent task.|] [AgentsSkill ClaudeSkill]))' "$manifest" >/dev/null
            grep -F '(management skills/management.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(psyche-interraction skills/psyche-interraction.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(psyche-interraction-claude-briefness skills/psyche-interraction-claude-briefness.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(psyche-interraction-continuation skills/psyche-interraction-continuation.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(tenets skills/tenets.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(psyche-interraction [psyche-interraction-continuation])' "$compositions" >/dev/null
            grep -F '(psyche-interraction ClaudeSkill [psyche-interraction-claude-briefness])' "$insertions" >/dev/null
            ! grep -F '(psyche-interraction AgentsSkill ' "$insertions"
            ! grep -F '(psyche-interraction CodexAgent ' "$insertions"
            grep -Fx '[general-instructions tenets]' "$universal" >/dev/null
            for retired in roles/manager.md skills/manager.md manifests/manager-packet-composition.nota; do
              test ! -e "${cleanSource}/$retired"
            done
            for retired_module in manager-boundary manager-intent-classification manager-safeguards manager-dispatch manager-liveness manager-decisions manager-communication manager-synthesis psyche-facing-commitments; do
              test -f "${cleanSource}/skills/$retired_module.md"
              ! grep -F "($retired_module " "$index"
            done
            workspace=$TMPDIR/workspace
            export SKILLS_SOURCE_ROOT=${cleanSource}
            export SKILLS_WORKSPACE_ROOT="$workspace"
            ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
            ${skillsPackage}/bin/skills ${cleanSource}/skills-check.nota >/dev/null
            for retired in \
              "$workspace/.agents/skills/manager/SKILL.md" \
              "$workspace/.claude/skills/manager/SKILL.md" \
              "$workspace/.claude/agents/manager.md" \
              "$workspace/.codex/agents/manager.toml" \
              "$workspace/.pi/agents/manager.md"; do
              test ! -e "$retired"
            done
            for packet in "$workspace"/.claude/agents/*.md "$workspace"/.codex/agents/*.toml "$workspace"/.pi/agents/*.md; do
              test "$(grep -oF "Never pretend to know what you don't know; admit you don't know." "$packet" | wc -l)" -eq 1
            done
            for output in "$workspace/.agents/skills/management/SKILL.md" "$workspace/.claude/skills/management/SKILL.md"; do
              grep -F 'Reserve your context for managing subagents.' "$output" >/dev/null
              grep -F 'Delegate all task work.' "$output" >/dev/null
              grep -F 'Read and write beads, reports, design documents, and the design log.' "$output" >/dev/null
              ! grep -F 'Never block on subagents.' "$output"
              ! grep -F 'Use no tools except subagent coordination.' "$output"
              ! grep -F 'Do other work while agents run.' "$output"
              ! grep -F 'Poll until they finish.' "$output"
              ! grep -F "Never pretend to know what you don't know; admit you don't know." "$output"
            done
            codex_wait_lines=(
              'Keep a wait active while agents run; you are resumed only through it.'
              'Pass the longest timeout the wait accepts. Omitting it defaults to thirty seconds.'
              'A psyche message interrupts the wait immediately, so a long wait costs no responsiveness.'
              'Say nothing when a wait returns with nothing finished.'
            )
            for wait_line in "''${codex_wait_lines[@]}"; do
              grep -F "$wait_line" "$workspace/.agents/skills/management/SKILL.md" >/dev/null
              ! grep -F "$wait_line" "$workspace/.claude/skills/management/SKILL.md"
              for packet in "$workspace"/.claude/agents/*.md "$workspace"/.pi/agents/*.md "$workspace"/.codex/agents/*.toml; do
                ! grep -F "$wait_line" "$packet"
              done
            done
            ! grep -F 'Use the fewest words that preserve the answer.' "$workspace/.agents/skills/psyche-interraction/SKILL.md"
            grep -F 'Use the fewest words that preserve the answer.' "$workspace/.claude/skills/psyche-interraction/SKILL.md" >/dev/null
            for output in "$workspace/.agents/skills/psyche-interraction/SKILL.md" "$workspace/.claude/skills/psyche-interraction/SKILL.md"; do
              grep -F 'Preserve approved meaning until the psyche changes it.' "$output" >/dev/null
              ! grep -F 'Keep the last approved wording as the current draft.' "$output"
              ! grep -F 'Apply clarifications by changing only the clause they address.' "$output"
              ! grep -F "Never pretend to know what you don't know; admit you don't know." "$output"
            done
            touch "$out"
          '';
          human-interaction-removed-from-active-and-generated =
            pkgs.runCommand "skills-human-interaction-removed-from-active-and-generated" { }
              ''
                manifest=${cleanSource}/manifests/active-outputs.nota
                index=${cleanSource}/manifests/module-dependencies.nota
                if grep -R -F 'human-interaction' "$manifest" "$index" ${cleanSource}/skills; then
                  echo "human-interaction must be deleted from source manifests and active sources" >&2
                  exit 1
                fi
                workspace=$TMPDIR/workspace
                mkdir -p "$workspace/.agents/skills/human-interaction" "$workspace/.claude/skills/human-interaction"
                printf 'stale\n' > "$workspace/.agents/skills/human-interaction/SKILL.md"
                printf 'stale\n' > "$workspace/.claude/skills/human-interaction/SKILL.md"
                export SKILLS_SOURCE_ROOT=${cleanSource}
                export SKILLS_WORKSPACE_ROOT="$workspace"
                ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
                test ! -e "$workspace/.agents/skills/human-interaction/SKILL.md"
                test ! -e "$workspace/.claude/skills/human-interaction/SKILL.md"
                touch "$out"
              '';
          skill-designing-source-of-truth-guardrails =
            pkgs.runCommand "skills-skill-designing-source-of-truth-guardrails" { }
              ''
                test ! -e ${cleanSource}/skills/skill-editor.md
                test ! -e ${cleanSource}/roles
                grep -F '(Skill (skill-designing skill-designing Workflow Mechanism [|Use when designing a skill.|]' ${cleanSource}/manifests/active-outputs.nota >/dev/null
                grep -Fx 'Write skills with brutal minimalism.' ${cleanSource}/skills/skill-designing.md >/dev/null
                grep -Fx '## Cut these' ${cleanSource}/skills/skill-designing.md >/dev/null
                grep -Fx '## Keep these' ${cleanSource}/skills/skill-designing.md >/dev/null
                if grep -R -F 'skill-editor' ${cleanSource}/manifests ${cleanSource}/skills; then
                  echo "skill-editor must not remain in active source" >&2
                  exit 1
                fi
                retired_lines=$TMPDIR/retired-skill-editor-role-lines
                printf '%s\n' \
                  'Keep only unusual guidance that changes agent behavior.' \
                  'Keep distinct instructions separate.' \
                  'Shorten skills by deleting weak guidance, not by compressing it.' \
                  'Make a skill only when the same guidance is needed across repositories.' \
                  'Reject operational guidance and repository-specific facts.' \
                  'Remove anything repeated, unverified, outdated, or already done without the skill.' \
                  'Use headings only when they aid navigation; never repeat the skill name.' \
                  > "$retired_lines"
                while IFS= read -r line; do
                  if grep -R -F -- "$line" ${cleanSource}/skills ${cleanSource}/manifests; then
                    echo "retired skill-editor role guidance remains: $line" >&2
                    exit 1
                  fi
                done < "$retired_lines"
                workspace=$TMPDIR/workspace
                export SKILLS_SOURCE_ROOT=${cleanSource}
                export SKILLS_WORKSPACE_ROOT="$workspace"
                ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
                test ! -e "$workspace/.agents/skills/skill-editor/SKILL.md"
                test ! -e "$workspace/.claude/skills/skill-editor/SKILL.md"
                test ! -e "$workspace/.pi/agents/skill-editor.md"
                test ! -e "$workspace/.claude/agents/skill-editor.md"
                test ! -e "$workspace/.codex/agents/skill-editor.toml"
                touch "$out"
              '';
          role-cross-product-manifests = pkgs.runCommand "skills-role-cross-product-manifests" { } ''
            permissions=${cleanSource}/manifests/role-permissions.nota
            depths=${cleanSource}/manifests/role-depths.nota
            descriptions=${cleanSource}/manifests/role-descriptions.nota
            catalog=${cleanSource}/manifests/model-catalog.nota
            for retired in \
              manifests/role-model-assignments.nota \
              manifests/role-model-profiles.nota \
              manifests/role-optional-skills.nota \
              manifests/nested-role-relations.nota; do
              test ! -e "${cleanSource}/$retired"
            done
            grep -F '(read [|Do not edit files, commit, or push. Fetching, cloning, and tool queries are fine.|] Restricted)' "$permissions" >/dev/null
            grep -F '(write [] Unrestricted)' "$permissions" >/dev/null
            grep -F '(claude-haiku-4-5 Claude [])' "$catalog" >/dev/null
            grep -F '(gpt-5.4-mini ChatGpt [Low Medium High Xhigh])' "$catalog" >/dev/null
            grep -F '(trivial (claude-haiku-4-5 None) (gpt-5.4-mini (Some Medium)))' "$depths" >/dev/null
            critical_row=$(grep -F '(critical (claude-opus-5 (Some High))' "$depths")
            test -n "$critical_row"
            critical_model=$(printf '%s' "$critical_row" | sed -E 's/.*\)\) \(([A-Za-z0-9.-]+) \(Some ([A-Za-z]+)\)\)\)$/\1/')
            critical_effort=$(printf '%s' "$critical_row" | sed -E 's/.*\)\) \(([A-Za-z0-9.-]+) \(Some ([A-Za-z]+)\)\)\)$/\2/')
            test -n "$critical_model"
            test -n "$critical_effort"
            grep -F "($critical_model ChatGpt" "$catalog" >/dev/null
            grep -F "($critical_model ChatGpt" "$catalog" | grep -F "$critical_effort" >/dev/null
            test "$(grep -c '^  (' "$descriptions")" -eq 8
            if grep -F '(Role (' ${cleanSource}/manifests/active-outputs.nota; then
              echo "roles are generated from the permission-by-depth cross product, not listed as active outputs" >&2
              exit 1
            fi
            workspace=$TMPDIR/workspace
            export SKILLS_SOURCE_ROOT=${cleanSource}
            export SKILLS_WORKSPACE_ROOT="$workspace"
            ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
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
          active-appellations = pkgs.runCommand "skills-active-appellations" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            index=${cleanSource}/manifests/module-dependencies.nota
            for required in version-control work-tracking management psyche-interraction psyche-vision tenets skill-designing documentation-placement; do
              grep -F "$required" "$manifest" >/dev/null || {
                echo "$required must be present in active output manifest" >&2
                exit 1
              }
              grep -F "$required" "$index" >/dev/null || true
            done
            grep -F '(Skill (management management ' "$manifest" >/dev/null
            ! grep -F '(Skill (manager ' "$manifest"
            if grep -F '(Skill (orchestration ' "$manifest"; then
              echo "orchestration must not be an active skill output" >&2
              exit 1
            fi
            for retired in component-triad beauty 'Skill (jj ' 'Skill (beads ' human-interaction 'Skill (design-quality ' 'Skill (privacy ' 'Skill (secrets ' 'Skill (prose ' 'Skill (reporting ' 'Skill (mermaid ' 'Skill (repo-intent ' 'Skill (spirit-query ' 'Skill (spirit-cli ' 'Skill (push-not-pull ' 'Skill (micro-components ' skill-editor; do
              if grep -F "$retired" "$manifest"; then
                echo "$retired must not be an active output appellation" >&2
                exit 1
              fi
            done
            for retired_title in 'Repo Operator' 'Weave Operator' 'Intent Maintainer'; do
              if grep -R -F "$retired_title" ${cleanSource}/skills; then
                echo "$retired_title must not appear as active current-destination prose" >&2
                exit 1
              fi
            done
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
