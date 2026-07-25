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
            if find ${cleanSource}/skills ${cleanSource}/roles -mindepth 2 -type f -name '*.md' | grep .; then
              echo "active source files must not use nested directories" >&2
              exit 1
            fi
            grep -F '(Skill (documentation-placement documentation-placement ' "$manifest" >/dev/null
            touch "$out"
          '';
          management-source-is-target-scoped = pkgs.runCommand "skills-management-source-is-target-scoped" { } ''
            management=${cleanSource}/skills/management.md
            overlay=${cleanSource}/skills/claude-manager-non-fable.md
            index=${cleanSource}/manifests/module-dependencies.nota
            insertions=${cleanSource}/manifests/target-module-insertions.nota
            test -f "$management"
            test -f "$overlay"
            grep -F '(management skills/management.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(claude-manager-non-fable skills/claude-manager-non-fable.md [] RuntimeSkill)' "$index" >/dev/null
            grep -F '(management ClaudeSkill [claude-manager-non-fable])' "$insertions" >/dev/null
            ! grep -F '(management ClaudeAgent [claude-manager-non-fable])' "$insertions"
            touch "$out"
          '';
          human-interaction-removed-from-active-and-generated =
            pkgs.runCommand "skills-human-interaction-removed-from-active-and-generated" { }
              ''
                manifest=${cleanSource}/manifests/active-outputs.nota
                index=${cleanSource}/manifests/module-dependencies.nota
                if grep -R -F 'human-interaction' "$manifest" "$index" ${cleanSource}/skills ${cleanSource}/roles; then
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
                expected_skill=$TMPDIR/skill-designing.md
                printf '%s\n' \
                  'Write skills with brutal minimalism.' \
                  'Descriptions say when the skill applies.' \
                  'State unusual, impactful instructions once and directly.' \
                  'Flag anything noisy, unclear, unsafe, or misplaced. Explain what each proposed change preserves, changes, or removes.' \
                  > "$expected_skill"
                expected_role=$TMPDIR/role-skill-maintainer.md
                printf '%s\n' \
                  'Before changing a skill, show the psyche the exact diff and get approval. A proposal is not approval.' \
                  'Change only the approved diff.' \
                  'Generate, verify, and report.' \
                  > "$expected_role"
                cmp "$expected_skill" ${cleanSource}/skills/skill-designing.md
                cmp "$expected_role" ${cleanSource}/roles/skill-maintainer.md
                test ! -e ${cleanSource}/skills/skill-editor.md
                test ! -e ${cleanSource}/roles/skill-editor.md
                grep -F '(Skill (skill-designing skill-designing Workflow Mechanism [|Use when designing a skill.|]' ${cleanSource}/manifests/active-outputs.nota >/dev/null
                grep -F '(Role (skill-maintainer role-skill-maintainer [skill-designing] [|Use when applying an approved skill change.|]' ${cleanSource}/manifests/active-outputs.nota >/dev/null
                grep -F '(skill-maintainer [])' ${cleanSource}/manifests/role-optional-skills.nota >/dev/null
                if grep -R -F 'skill-editor' ${cleanSource}/manifests ${cleanSource}/skills ${cleanSource}/roles; then
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
                  if grep -R -F -- "$line" ${cleanSource}/skills ${cleanSource}/roles ${cleanSource}/manifests; then
                    echo "retired skill-editor role guidance remains: $line" >&2
                    exit 1
                  fi
                done < "$retired_lines"
                workspace=$TMPDIR/workspace
                export SKILLS_SOURCE_ROOT=${cleanSource}
                export SKILLS_WORKSPACE_ROOT="$workspace"
                ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
                for packet in "$workspace/.pi/agents/skill-maintainer.md" "$workspace/.claude/agents/skill-maintainer.md"; do
                  while IFS= read -r line; do
                    test "$(grep -Fxc "$line" "$packet")" -eq 1
                  done < "$expected_skill"
                  ! grep -F '## Allowed child-role roster' "$packet"
                done
                test ! -e "$workspace/.agents/skills/skill-editor/SKILL.md"
                test ! -e "$workspace/.claude/skills/skill-editor/SKILL.md"
                test ! -e "$workspace/.pi/agents/skill-editor.md"
                test ! -e "$workspace/.claude/agents/skill-editor.md"
                test ! -e "$workspace/.codex/agents/skill-editor.toml"
                touch "$out"
              '';
          manager-packet-wipe-guardrails = pkgs.runCommand "skills-manager-packet-wipe-guardrails" { } ''
            manager=${cleanSource}/skills/manager.md
            role=${cleanSource}/roles/manager.md
            management=${cleanSource}/skills/management.md
            index=${cleanSource}/manifests/module-dependencies.nota
            manifest=${cleanSource}/manifests/active-outputs.nota
            composition=${cleanSource}/manifests/manager-packet-composition.nota
            optional=${cleanSource}/manifests/role-optional-skills.nota
            assignments=${cleanSource}/manifests/role-model-assignments.nota
            expected_skill=$TMPDIR/manager-skill
            printf '%s\n' \
              'Delegate assigned work to child workers.' \
              'Poll until they finish.' \
              'Keep observations, hypotheses, and unknowns distinct.' \
              'Return unresolved authority, safety, privacy, or scope to the parent.' \
              'Return a concise synthesis to the parent.' \
              > "$expected_skill"
            printf '%s\n' 'Manage delegated work for the parent.' > "$TMPDIR/manager-role"
            cmp "$manager" "$expected_skill"
            cmp "$role" "$TMPDIR/manager-role"
            grep -F '(Skill (manager manager Meta Mechanism [|Use when managing delegated workers for a parent agent.|] [AgentsSkill ClaudeSkill]))' "$manifest" >/dev/null
            grep -F '(Role (manager role-manager [manager] [|Routes intent and work.|] [ClaudeAgent CodexAgent PiAgent]))' "$manifest" >/dev/null
            grep -Fx 'Minimal' "$composition" >/dev/null
            grep -F '(manager skills/manager.md [] RuntimeSkill)' "$index" >/dev/null
            grep -Fx '  (manager [])' "$optional" >/dev/null
            grep -F '(Direct (manager (gpt-5.6-sol High) (claude-opus-4-8 High)))' "$assignments" >/dev/null
            grep -F 'Management may only coordinate subagents and directly read relevant skill or role files; it never performs other tool actions or writes files.' "$management" >/dev/null
            grep -F 'Never wait for subagents; they report asynchronously.' "$management" >/dev/null
            for retired_module in manager-boundary manager-intent-classification manager-safeguards manager-dispatch manager-liveness manager-decisions manager-communication manager-synthesis psyche-facing-commitments; do
              test -f "${cleanSource}/skills/$retired_module.md"
              ! grep -F "($retired_module " "$index"
            done
            workspace=$TMPDIR/workspace
            export SKILLS_SOURCE_ROOT=${cleanSource}
            export SKILLS_WORKSPACE_ROOT="$workspace"
            ${skillsPackage}/bin/skills ${cleanSource}/skills-generate.nota >/dev/null
            ${skillsPackage}/bin/skills ${cleanSource}/skills-check.nota >/dev/null
            for skill in "$workspace/.agents/skills/manager/SKILL.md" "$workspace/.claude/skills/manager/SKILL.md"; do
              grep -F "description: 'Use when managing delegated workers for a parent agent.'" "$skill" >/dev/null
              while IFS= read -r line; do
                test "$(grep -oF -- "$line" "$skill" | wc -l)" -eq 1
              done < "$expected_skill"
            done
            for packet in "$workspace/.claude/agents/manager.md" "$workspace/.codex/agents/manager.toml" "$workspace/.pi/agents/manager.md"; do
              test "$(grep -oF 'Manage delegated work for the parent.' "$packet" | wc -l)" -eq 1
              while IFS= read -r line; do
                test "$(grep -oF -- "$line" "$packet" | wc -l)" -eq 1
              done < "$expected_skill"
              for forbidden in psyche 'general instructions' 'skill loading' 'Manager dispatch roster' 'Allowed child-role roster' 'optional skills' 'Management may only coordinate subagents' 'Delegate investigation and operations.' 'Keep requested rules, mechanisms, and architecture as matter.' 'Require explicit psyche approval before a host reboot.' 'Use expected type and position to interpret every value.'; do
                ! grep -F -- "$forbidden" "$packet"
              done
            done
            grep -F 'model: claude-opus-4-8' "$workspace/.claude/agents/manager.md" >/dev/null
            grep -F 'model = "gpt-5.6-sol"' "$workspace/.codex/agents/manager.toml" >/dev/null
            grep -F "model: 'openai-codex/gpt-5.6-sol'" "$workspace/.pi/agents/manager.md" >/dev/null
            grep -F 'projectRoleIdentity: manager' "$workspace/.pi/agents/manager.md" >/dev/null
            grep -F 'projectRoleDispatchKind: manager' "$workspace/.pi/agents/manager.md" >/dev/null
            ! grep -F 'allowedChildRoleNames:' "$workspace/.pi/agents/manager.md"
            ! grep -F 'skills:' "$workspace/.pi/agents/manager.md"
            touch "$out"
          '';
          slim-role-composition = pkgs.runCommand "skills-slim-role-composition" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            if grep -F '(Role (' "$manifest" | grep -E '\[[^]]*(spirit-query|nota-design)[^]]*\]'; then
              echo "roles must not preload broad Spirit or NOTA runtime skills" >&2
              exit 1
            fi
            grep -F '(Role (intent-recorder role-intent-recorder [spirit-submission]' "$manifest" >/dev/null
            grep -F '[general-instructions]' ${cleanSource}/manifests/universal-role-modules.nota >/dev/null
            grep -F '(Role (manager role-manager [manager]' "$manifest" >/dev/null
            touch "$out"
          '';
          role-profile-manifests = pkgs.runCommand "skills-role-profile-manifests" { } ''
            model_catalog=${cleanSource}/manifests/model-catalog.nota
            profile_catalog=${cleanSource}/manifests/role-model-profiles.nota
            role_assignments=${cleanSource}/manifests/role-model-assignments.nota
            grep -F '(ChatGpt (gpt-5.6-sol openai-codex [(Medium 50) (High 60)]))' "$model_catalog" >/dev/null
            grep -F '(ChatGpt (gpt-5.6-terra openai-codex [(Medium 20) (High 30) (Xhigh 40)]))' "$model_catalog" >/dev/null
            grep -F '(Claude (fable-5 [(Medium 50) (High 60)]))' "$model_catalog" >/dev/null
            grep -F '(Claude (claude-opus-4-8 [(High 30) (Xhigh 40)]))' "$model_catalog" >/dev/null
            grep -F '(Claude (claude-sonnet-5 [(Medium 10)]))' "$model_catalog" >/dev/null
            grep -F '(Direct (manager (gpt-5.6-sol High) (claude-opus-4-8 High)))' "$role_assignments" >/dev/null
            grep -F '(Direct (generalist (gpt-5.6-terra Xhigh) (claude-opus-4-8 High)))' "$role_assignments" >/dev/null
            grep -F '(Direct (intent-translator (gpt-5.6-terra Xhigh) (claude-opus-4-8 Xhigh)))' "$role_assignments" >/dev/null
            grep -F '(Direct (operating-system-implementer (gpt-5.6-terra Xhigh) (claude-opus-4-8 High)))' "$role_assignments" >/dev/null
            grep -F '(Direct (skill-maintainer (gpt-5.6-terra Xhigh) (claude-opus-4-8 Xhigh)))' "$role_assignments" >/dev/null
            grep -F '(Direct (intent-curator (gpt-5.6-terra Xhigh) (claude-opus-4-8 Xhigh)))' "$role_assignments" >/dev/null
            grep -F '(Direct (intent-recorder (gpt-5.6-luna Medium) (claude-sonnet-5 Medium)))' "$role_assignments" >/dev/null
            grep -F '(Direct (scout (gpt-5.6-luna Medium) (claude-sonnet-5 Medium)))' "$role_assignments" >/dev/null
            grep -F '(Direct (repository-closeout (gpt-5.6-luna Medium) (claude-sonnet-5 Medium)))' "$role_assignments" >/dev/null
            grep -F '(minimalFastEconomical (gpt-5.6-luna Medium) (claude-sonnet-5 Medium))' "$profile_catalog" >/dev/null
            grep -F '(Profile (trivial-task minimalFastEconomical))' "$role_assignments" >/dev/null
            ! grep -F 'gpt-5.6-luna' ${cleanSource}/roles/trivial-task.md
            ! grep -F 'claude-sonnet-5' ${cleanSource}/roles/trivial-task.md
            if grep -R -F 'claude-sonnet-4-6' ${cleanSource}/manifests; then
              echo "Claude Sonnet roles must not regress to Sonnet 4.6" >&2
              exit 1
            fi
            grep -F '(manager [])' ${cleanSource}/manifests/role-optional-skills.nota >/dev/null
            touch "$out"
          '';
          active-appellations = pkgs.runCommand "skills-active-appellations" { } ''
            manifest=${cleanSource}/manifests/active-outputs.nota
            index=${cleanSource}/manifests/module-dependencies.nota
            for required in component-architecture design-quality version-control work-tracking management manager skill-designing generalist intent-recorder intent-curator repository-closeout tracker-weaver skill-maintainer trivial-task; do
              grep -F "$required" "$manifest" >/dev/null || {
                echo "$required must be present in active output manifest" >&2
                exit 1
              }
              grep -F "$required" "$index" >/dev/null || true
            done
            grep -F '(Skill (management management ' "$manifest" >/dev/null
            grep -F '(Skill (manager manager ' "$manifest" >/dev/null
            if grep -F '(Skill (orchestration ' "$manifest"; then
              echo "orchestration must not be an active skill output" >&2
              exit 1
            fi
            for retired in component-triad beauty 'Skill (jj ' 'Skill (beads ' human-interaction 'Role (orchestrator ' intent-maintainer repo-operator weave-operator skill-editor role-skill-editor; do
              if grep -F "$retired" "$manifest"; then
                echo "$retired must not be an active output appellation" >&2
                exit 1
              fi
            done
            for retired_title in 'Repo Operator' 'Weave Operator' 'Intent Maintainer'; do
              if grep -R -F "$retired_title" ${cleanSource}/roles ${cleanSource}/skills; then
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
