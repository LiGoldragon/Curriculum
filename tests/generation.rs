use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};

use nota::NotaSource;
use skills::{
    Error,
    schema::assembly::{
        GenerationMode, GenerationRequest, ManifestPath, RoleDepths, RoleDescriptions,
        RolePermissions, SourceRoot, VisualizationRequest, WorkspaceRoot,
    },
    trunk_guard::{TrunkDescendantGuard, TrunkDivergence},
};
use tempfile::TempDir;

fn flat_frontmatter(packet: &str) -> BTreeMap<String, String> {
    let block = packet
        .strip_prefix("---\n")
        .and_then(|packet| packet.split_once("\n---\n"))
        .map(|(frontmatter, _)| frontmatter)
        .expect("packet has frontmatter");
    block
        .lines()
        .map(|line| {
            let (key, value) = line.split_once(':').expect("flat frontmatter field");
            let value = value.trim();
            let value = value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .or_else(|| {
                    value
                        .strip_prefix('"')
                        .and_then(|value| value.strip_suffix('"'))
                })
                .unwrap_or(value);
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

#[test]
fn generation_writes_derived_skill_surfaces_with_manifest_frontmatter() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\nname: stale\n---\n\n# Skill — example\n\n## Rule\n\nKeep the prose.\n",
    );

    let report = fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");

    let generated_paths: Vec<&str> = report
        .payload()
        .payload()
        .iter()
        .map(|file| file.output_path.as_ref())
        .collect();
    assert!(generated_paths.contains(&".agents/skills/example/SKILL.md"));
    assert!(generated_paths.contains(&".claude/skills/example/SKILL.md"));
    assert!(!generated_paths.contains(&"skills/skills.nota"));

    let generated = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert_eq!(
        generated,
        "---\nname: example\ndescription: 'Example skill.'\n---\n\n# example\n\n## Rule\n\nKeep the prose.\n"
    );
    assert_eq!(
        generated,
        fixture.read_workspace_file(".claude/skills/example/SKILL.md")
    );
    assert!(!fixture.workspace.path().join("skills/skills.nota").exists());
}

#[test]
fn generation_allows_fenced_frontmatter_examples_inside_modules() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "# Skill — example\n\n## Rule\n\n```markdown\n---\nname: example\n---\n```\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("fenced frontmatter example is ordinary markdown");

    let generated = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert!(generated.starts_with("---\nname: example\ndescription: 'Example skill.'\n---\n\n"));
    assert!(generated.contains("```markdown\n---\nname: example\n---\n```"));
}

#[test]
fn generation_rejects_second_unfenced_frontmatter_delimiter_in_skill() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "# Skill — example\n\n## Rule\n\n---\n\nKeep the prose.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("only the leading frontmatter delimiter pair is allowed");

    assert!(
        matches!(error, Error::NestedFrontmatter { .. }),
        "{error:?}"
    );
}

#[test]
fn generation_does_not_rebase_link_syntax_inside_code_spans() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "# Skill — example\n\n## Rule\n\nUse `[text](url)` only as a literal example.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("code span link syntax is preserved");

    let generated = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert!(generated.contains("`[text](url)`"));
}

#[test]
fn generation_allows_zero_or_one_title_and_rejects_multiple_titles() {
    let zero_title = Fixture::new();
    zero_title.write_default_manifest();
    zero_title.write_source_file("skills/example.md", "No title.\n");
    zero_title
        .generate(GenerationMode::Write)
        .expect("zero titles generate");
    assert_eq!(
        zero_title.read_workspace_file(".agents/skills/example/SKILL.md"),
        "---\nname: example\ndescription: 'Example skill.'\n---\n\nNo title.\n"
    );

    let one_title = Fixture::new();
    one_title.write_default_manifest();
    one_title.write_source_file("skills/example.md", "# Skill — example\n\nOne title.\n");
    one_title
        .generate(GenerationMode::Write)
        .expect("one title generates");
    assert!(
        one_title
            .read_workspace_file(".agents/skills/example/SKILL.md")
            .contains("# example\n\nOne title.\n")
    );

    let multiple_titles = Fixture::new();
    multiple_titles.write_default_manifest();
    multiple_titles.write_source_file("skills/example.md", "# First\n\n# Second\n");
    let error = multiple_titles
        .generate(GenerationMode::Write)
        .expect_err("multiple titles fail");
    assert!(
        matches!(error, Error::InvalidTitleCount { count: 2, .. }),
        "{error:?}"
    );
}

#[test]
fn generation_rejects_nested_legacy_module_source_paths() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file("skills/example.md", "# example\n");
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example modules/example/full.md [] RuntimeSkill)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("nested legacy source paths are rejected");

    assert!(
        matches!(
            error,
            Error::InvalidModuleSourcePath {
                ref module_identifier,
                ref expected,
                ref actual,
            } if module_identifier == "example"
                && expected == "skills/example.md"
                && actual == "modules/example/full.md"
        ),
        "{error:?}"
    );
}

#[test]
fn skill_module_compositions_reject_inactive_and_duplicate_skill_entries() {
    let inactive = Fixture::new();
    inactive.write_default_manifest();
    inactive.write_source_file("skills/example.md", "# Skill — example\n\nExample.\n");
    inactive.write_source_file(
        "manifests/skill-module-compositions.nota",
        "[(missing [])]\n",
    );
    let error = inactive
        .generate(GenerationMode::Write)
        .expect_err("inactive skill composition rejects generation");
    assert!(matches!(error, Error::StaleSkillModuleComposition { .. }));

    let duplicate = Fixture::new();
    duplicate.write_default_manifest();
    duplicate.write_source_file("skills/example.md", "# Skill — example\n\nExample.\n");
    duplicate.write_source_file(
        "manifests/skill-module-compositions.nota",
        "[(example []) (example [])]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate skill composition rejects generation");
    assert!(matches!(
        error,
        Error::DuplicateSkillModuleComposition { .. }
    ));
}

#[test]
fn generation_fails_on_duplicate_headings() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "# Skill — example\n\n## Repeat\n\nFirst.\n\n## Repeat\n\nSecond.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("duplicate headings fail");

    assert!(matches!(error, Error::DuplicateHeading { .. }), "{error:?}");
}

#[test]
fn repository_visibility_doctrine_defaults_public_without_weakening_privacy() {
    let lifecycle = include_str!("../skills/repository-lifecycle.md");
    assert!(lifecycle.contains("Do not publish private material"));
    assert!(lifecycle.contains("public visibility as default"));
}

#[test]
fn management_is_subagent_scoped_and_has_no_psyche_interaction_doctrine() {
    let management = include_str!("../skills/management.md");
    for required in [
        "Reserve your context for managing subagents.",
        "Delegate all task work.",
        "Read and write beads, reports, design documents, and the design log.",
    ] {
        assert!(
            management.contains(required),
            "management retains `{required}`"
        );
    }
    for excluded in [
        "Align with the psyche’s vision.",
        "Ask the psyche *until the vision is clear.*",
        "Never wait for subagents; they report asynchronously.",
        "Never block on subagents.",
        "Use no tools except subagent coordination.",
        "Do other work while agents run.",
        "Return a synthesis to the caller.",
        "Poll until they finish.",
        "Never pretend to know what you don't know; admit you don't know.",
    ] {
        assert!(
            !management.contains(excluded),
            "management excludes `{excluded}`"
        );
    }
}

#[test]
fn harness_api_fields_do_not_leak_into_general_management_doctrine() {
    let fields = ["turnBudget", "toolBudget", "timeoutMs", "maxRuntimeMs"];
    for (name, source) in [
        ("management", include_str!("../skills/management.md")),
        (
            "manager-boundary",
            include_str!("../skills/manager-boundary.md"),
        ),
        (
            "manager-intent-classification",
            include_str!("../skills/manager-intent-classification.md"),
        ),
        (
            "manager-safeguards",
            include_str!("../skills/manager-safeguards.md"),
        ),
        (
            "manager-dispatch",
            include_str!("../skills/manager-dispatch.md"),
        ),
        (
            "manager-liveness",
            include_str!("../skills/manager-liveness.md"),
        ),
        (
            "manager-decisions",
            include_str!("../skills/manager-decisions.md"),
        ),
        (
            "manager-communication",
            include_str!("../skills/manager-communication.md"),
        ),
        (
            "manager-synthesis",
            include_str!("../skills/manager-synthesis.md"),
        ),
    ] {
        for field in fields {
            assert!(
                !source.contains(field),
                "general {name} doctrine leaks harness API field {field}"
            );
        }
    }

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("harness-placement profile generates");
    for path in [
        ".agents/skills/management/SKILL.md",
        ".claude/skills/management/SKILL.md",
    ] {
        let output = fixture.read_workspace_file(path).replace("\\n", "\n");
        for field in fields {
            assert!(
                !output.contains(field),
                "general generated output {path} leaks harness API field {field}"
            );
        }
    }
    for path in [
        ".pi/agents/write-critical.md",
        ".claude/agents/write-critical.md",
        ".codex/agents/write-critical.toml",
    ] {
        assert!(
            !fixture
                .read_workspace_file(path)
                .contains("Keep shared guidance independent of harness APIs."),
            "{path} excludes retired skill-editor harness operations"
        );
    }
}

#[test]
fn pi_extension_update_protocol_uses_declarative_source_ownership() {
    let protocol = include_str!("../skills/pi-extension-updates.md");
    for required in [
        "Reconcile each local extension change with upstream evidence.",
        "Change the source and declarative package owner, not installed output.",
        "Push a producer before updating its consumer pin.",
        "Verify the activated revision.",
    ] {
        assert!(
            protocol.contains(required),
            "missing Pi extension rule: {required}"
        );
    }
}

#[test]
fn psyche_interraction_claude_briefness_is_typed_and_target_scoped() {
    const CENTRAL: &str = "## Central\nBe very brief unless writing a context handover.\nAlign with the psyche’s vision.\nAsk the psyche *until the vision is clear.*\n";
    const CLAUDE_BRIEFNESS: &str = "Use the fewest words that preserve the answer.\nDo not repeat context the psyche already knows.\n";
    assert_eq!(include_str!("../skills/psyche-interraction.md"), CENTRAL);
    assert_eq!(
        include_str!("../skills/psyche-interraction-claude-briefness.md"),
        CLAUDE_BRIEFNESS
    );
    for interaction_source in [
        include_str!("../skills/management.md"),
        include_str!("../skills/psyche-interraction.md"),
        include_str!("../skills/psyche-interraction-continuation.md"),
    ] {
        assert!(!interaction_source.contains("## Basic tenets"));
        assert!(!interaction_source.contains("## Delegation"));
    }

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("psyche interaction profile generates");
    let agents = fixture.read_workspace_file(".agents/skills/psyche-interraction/SKILL.md");
    let claude = fixture.read_workspace_file(".claude/skills/psyche-interraction/SKILL.md");
    assert!(!agents.contains(CLAUDE_BRIEFNESS));
    assert!(claude.contains(CLAUDE_BRIEFNESS));
    for output in [&agents, &claude] {
        assert!(
            !output.contains("Never pretend to know what you don't know; admit you don't know."),
            "psyche interaction keeps tenets separate"
        );
        assert!(output.contains("Preserve approved meaning until the psyche changes it."));
        assert!(!output.contains("Keep the last approved wording as the current draft."));
        assert!(!output.contains("Apply clarifications by changing only the clause they address."));
    }
    assert!(
        claude.find(CENTRAL).expect("central emitted")
            < claude.find(CLAUDE_BRIEFNESS).expect("briefness emitted")
    );
    assert!(
        claude.find(CLAUDE_BRIEFNESS).expect("briefness emitted")
            < claude
                .find("## Conversation")
                .expect("conversation emitted")
    );
    assert_eq!(claude.matches(CLAUDE_BRIEFNESS).count(), 1);
    assert!(!agents.contains("Use the fewest words that preserve the answer."));
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/claude-psyche-interraction.md")
            .exists()
    );
}

#[test]
fn host_reboot_requires_specific_psyche_approval() {
    for source in [
        include_str!("../skills/manager-safeguards.md"),
        include_str!("../skills/operating-system-operations.md"),
    ] {
        assert!(source.contains("Require explicit psyche approval"));
        assert!(source.contains("reboot"));
    }
}

#[test]
fn general_instructions_compose_once_and_keep_authority_gates() {
    let general = include_str!("../skills/general-instructions.md");
    assert!(general.contains("Use plain established language."));
    assert!(general.contains("Do not introduce limits on agent execution."));
    assert!(general.contains("explicit psyche approval"));
    assert!(!general.contains("Clarify, gate, dispatch"));
    assert!(
        include_str!("../manifests/universal-role-modules.nota")
            .contains("[general-instructions tenets]")
    );
}

#[test]
fn generation_strips_source_maintenance_notes_from_runtime_surfaces() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "# Skill - example\n\n## Rule\n\nGenerated.\n\n## Source Maintenance Notes\n\nMaintainer-only synchronization steps.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("source maintenance notes stay source-only");

    let agents_skill = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert!(agents_skill.contains("# example"));
    assert!(agents_skill.contains("Generated."));
    assert!(!agents_skill.contains("Skill - example"));
    assert!(!agents_skill.contains("Source Maintenance Notes"));
    assert!(!agents_skill.contains("Maintainer-only synchronization steps"));
}

#[test]
fn target_module_insertions_apply_only_to_matching_generated_surfaces() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (management management Meta Mechanism [Management skill] [AgentsSkill ClaudeSkill]))]\n",
    );
    fixture.write_role_cross_product_sources();
    fixture.write_universal_role_modules(
        "[management]\n",
        "[(management skills/management.md [] RuntimeSkill) (claude-management skills/claude-management.md [] RuntimeSkill)]\n",
    );
    fixture.write_source_file(
        "manifests/target-module-insertions.nota",
        "[(management ClaudeSkill [claude-management]) (management ClaudeAgent [claude-management])]\n",
    );
    fixture.write_source_file(
        "skills/management.md",
        "# Skill - management\n\n## Shared Rule\n\nShared management.\n",
    );
    fixture.write_source_file(
        "skills/claude-management.md",
        "# Module - Target reply surface\n\n## Clarification UI\n\nTarget overlay.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("target insertions generate");

    let agents_skill = fixture.read_workspace_file(".agents/skills/management/SKILL.md");
    assert!(agents_skill.contains("Shared management."));
    assert!(!agents_skill.contains("Target overlay."));

    let claude_skill = fixture.read_workspace_file(".claude/skills/management/SKILL.md");
    assert!(claude_skill.contains("Shared management."));
    assert!(claude_skill.contains("Target overlay."));

    let claude_role = fixture.read_workspace_file(".claude/agents/write-deep.md");
    assert!(claude_role.contains("Shared management."));
    assert!(claude_role.contains("Target overlay."));

    let codex_role = fixture.read_workspace_file(".codex/agents/write-deep.toml");
    assert!(codex_role.contains("Shared management."));
    assert!(!codex_role.contains("Target overlay."));

    let pi_role = fixture.read_workspace_file(".pi/agents/write-deep.md");
    assert!(pi_role.contains("Shared management."));
    assert!(!pi_role.contains("Target overlay."));
}

#[test]
fn generation_rejects_direct_module_dependency_cycle() {
    let fixture = Fixture::new();
    fixture.write_role_cross_product_sources();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example skills/example.md [example] RuntimeSkill)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("direct dependency cycle fails generation");

    assert!(
        matches!(
            error,
            Error::ModuleDependencyCycle {
                ref module_identifiers
            } if module_identifiers
                .iter()
                .map(String::as_str)
                .eq(["example", "example"])
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("example -> example"));
}

#[test]
fn generation_rejects_transitive_module_dependency_cycle() {
    let fixture = Fixture::new();
    fixture.write_role_cross_product_sources();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example first Craft Topic [Example skill.] [AgentsSkill]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(first skills/first.md [second] RuntimeSkill) (second skills/second.md [third] RuntimeSkill) (third skills/third.md [second] RuntimeSkill)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("transitive dependency cycle fails generation");

    assert!(
        matches!(
            error,
            Error::ModuleDependencyCycle {
                ref module_identifiers
            } if module_identifiers
                .iter()
                .map(String::as_str)
                .eq(["second", "third", "second"])
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("second -> third -> second"));
}

#[test]
fn generation_rejects_role_composition_module_as_skill_output() {
    let fixture = Fixture::new();
    fixture.write_role_cross_product_sources();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (edit-coordination-core edit-coordination-core Workflow Mechanism [Internal role component.] [AgentsSkill]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(edit-coordination-core skills/edit-coordination-core.md [] RoleComposition)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("role composition modules do not emit as skills");

    assert!(
        matches!(
            error,
            Error::InvalidModuleKind {
                ref module_identifier,
                ref expected,
                ref actual,
            } if module_identifier == "edit-coordination-core"
                && expected == "RuntimeSkill"
                && actual == "RoleComposition"
        ),
        "{error:?}"
    );
    assert!(
        !fixture
            .workspace
            .path()
            .join(".agents/skills/edit-coordination-core/SKILL.md")
            .exists()
    );
}

#[test]
fn check_mode_reports_stale_output_with_guidance() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "# Skill — example\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_workspace_file(".agents/skills/example/SKILL.md", "old\n");
    fixture.write_workspace_file(".claude/skills/example/SKILL.md", "old\n");
    fixture.write_workspace_file("skills/skills.nota", "old\n");

    let error = fixture
        .generate(GenerationMode::Check)
        .expect_err("stale output fails check mode");

    assert!(matches!(error, Error::StaleOutput { .. }), "{error:?}");
    assert!(!error.to_string().contains("skills.nota"));
    assert!(error.to_string().contains("generate-skills"));
    assert!(error.to_string().contains("check-skills"));
}

#[test]
fn generation_rejects_skill_with_oversized_serialized_block() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        &format!("# Skill — example\n\n## Rule\n\n{}\n", "x".repeat(33_000)),
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("oversized serialized skill block fails generation");

    assert!(
        matches!(
            error,
            Error::GeneratedSkillBlockTooLarge {
                ref skill_name,
                ref location,
                byte_count,
                limit,
            } if skill_name == "example"
                && location == ".agents/skills/example/SKILL.md"
                && byte_count > limit
                && limit == 32 * 1024
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("generated skill `example`"));
    assert!(error.to_string().contains("exceeding the 32768 byte limit"));
    assert!(
        !fixture
            .workspace
            .path()
            .join(".agents/skills/example/SKILL.md")
            .exists()
    );
}

#[test]
fn retired_skill_index_is_rejected_in_check_mode_and_pruned_in_write_mode() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current generated outputs write to fixture workspace");
    fixture.write_workspace_file("skills/skills.nota", "old retired index\n");

    let error = fixture
        .generate_from_repo(GenerationMode::Check)
        .expect_err("retired skill index fails deployment check");
    assert!(
        matches!(error, Error::StaleGeneratedOutput { ref path } if path.ends_with("skills/skills.nota")),
        "{error:?}"
    );

    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("write mode prunes retired skill index");
    assert!(
        !fixture.workspace.path().join("skills/skills.nota").exists(),
        "retired skill index is removed"
    );
    fixture
        .generate_from_repo(GenerationMode::Check)
        .expect("pruned outputs satisfy deployment check");
}

#[test]
fn write_mode_prunes_generated_skill_directories_before_writing() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "# Skill — example\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_workspace_file(".agents/skills/old/SKILL.md", "stale\n");
    fixture.write_workspace_file(".claude/skills/old/SKILL.md", "stale\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("write mode prunes stale generated skill dirs");

    assert!(
        !fixture
            .workspace
            .path()
            .join(".agents/skills/old/SKILL.md")
            .exists()
    );
    assert!(
        !fixture
            .workspace
            .path()
            .join(".claude/skills/old/SKILL.md")
            .exists()
    );
    assert!(
        fixture
            .workspace
            .path()
            .join(".agents/skills/example/SKILL.md")
            .exists()
    );
}

#[test]
fn trunk_guard_passes_source_without_jujutsu_working_copy() {
    let source = TempDir::new().expect("source tempdir");

    TrunkDescendantGuard::new(source.path())
        .verify()
        .expect("an immutable source with no Jujutsu working copy is inherently safe");
}

#[test]
fn trunk_divergence_permits_regeneration_when_no_trunk_commits_are_unreached() {
    let divergence = TrunkDivergence::from_revset_output("\n  \n");

    assert!(
        !divergence.requires_refusal(),
        "a descendant working copy leaves no trunk commit unreached"
    );
}

#[test]
fn trunk_divergence_refuses_regeneration_when_trunk_has_unreached_commits() {
    let divergence = TrunkDivergence::from_revset_output("oxxluyzymxmv\nrlkyomtvabcd\n");

    assert!(
        divergence.requires_refusal(),
        "a sibling or behind working copy leaves trunk commits unreached and must refuse"
    );
}

#[test]
fn role_cross_product_writes_one_packet_per_permission_depth_and_surface() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample rule.\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("cross-product roles generate");

    for role in ["read-shallow", "read-deep", "write-shallow", "write-deep"] {
        for path in [
            format!(".claude/agents/{role}.md"),
            format!(".codex/agents/{role}.toml"),
            format!(".pi/agents/{role}.md"),
        ] {
            let packet = fixture.read_workspace_file(&path);
            assert!(packet.contains(role), "{path} names its role");
        }
    }
    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.nota");
    for role in ["read-shallow", "read-deep", "write-shallow", "write-deep"] {
        assert!(inventory.contains(&format!(".claude/agents/{role}.md")));
        assert!(inventory.contains(&format!(".codex/agents/{role}.toml")));
        assert!(inventory.contains(&format!(".pi/agents/{role}.md")));
    }
    assert!(!inventory.contains("worker"));
}

#[test]
fn permission_body_precedes_the_shared_body_only_for_restricted_permissions() {
    const SHARED_FIRST: &str =
        "The brief is your authority. Decide what it settles; return what it does not.";
    const SHARED_SECOND: &str = "Finish everything that does not depend on what you return.";
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample rule.\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("role bodies generate");

    let read_packet = fixture.read_workspace_file(".claude/agents/read-deep.md");
    let write_packet = fixture.read_workspace_file(".claude/agents/write-deep.md");
    assert!(read_packet.contains("Read body."));
    assert!(!write_packet.contains("Read body."));
    for packet in [&read_packet, &write_packet] {
        assert!(packet.contains(SHARED_FIRST));
        assert!(packet.contains(SHARED_SECOND));
        assert!(packet.find(SHARED_FIRST) < packet.find(SHARED_SECOND));
    }
    assert!(read_packet.find("Read body.") < read_packet.find(SHARED_FIRST));
}

#[test]
fn restricted_permissions_block_editing_tools_on_the_surfaces_that_carry_them() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample rule.\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("tool restrictions generate");

    let claude_read = flat_frontmatter(&fixture.read_workspace_file(".claude/agents/read-deep.md"));
    assert_eq!(
        claude_read.get("disallowedTools").map(String::as_str),
        Some("Edit, Write, NotebookEdit")
    );
    let pi_read = flat_frontmatter(&fixture.read_workspace_file(".pi/agents/read-deep.md"));
    assert_eq!(
        pi_read.get("disallowed_tools").map(String::as_str),
        Some("edit, write")
    );
    let claude_write =
        flat_frontmatter(&fixture.read_workspace_file(".claude/agents/write-deep.md"));
    assert!(!claude_write.contains_key("disallowedTools"));
    let pi_write = flat_frontmatter(&fixture.read_workspace_file(".pi/agents/write-deep.md"));
    assert!(!pi_write.contains_key("disallowed_tools"));
    let codex_read = fixture.read_workspace_file(".codex/agents/read-deep.toml");
    assert!(!codex_read.contains("disallowed"));
}

#[test]
fn depth_rows_resolve_models_by_provider_and_omit_effort_for_effortless_models() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample rule.\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("depth rows resolve");

    let shallow_claude =
        flat_frontmatter(&fixture.read_workspace_file(".claude/agents/write-shallow.md"));
    assert_eq!(
        shallow_claude.get("model").map(String::as_str),
        Some("claude-flat")
    );
    assert!(!shallow_claude.contains_key("effort"));

    let deep_claude =
        flat_frontmatter(&fixture.read_workspace_file(".claude/agents/write-deep.md"));
    assert_eq!(
        deep_claude.get("model").map(String::as_str),
        Some("claude-test")
    );
    assert_eq!(deep_claude.get("effort").map(String::as_str), Some("high"));

    let pi_shallow = flat_frontmatter(&fixture.read_workspace_file(".pi/agents/write-shallow.md"));
    assert_eq!(
        pi_shallow.get("model").map(String::as_str),
        Some("openai-codex/gpt-test")
    );
    assert_eq!(pi_shallow.get("thinking").map(String::as_str), Some("low"));
    assert_eq!(
        pi_shallow.get("projectRoleIdentity").map(String::as_str),
        Some("write-shallow")
    );
    assert_eq!(
        pi_shallow
            .get("projectRoleDispatchKind")
            .map(String::as_str),
        Some("leaf")
    );

    let codex_shallow = fixture.read_workspace_file(".codex/agents/write-shallow.toml");
    assert!(codex_shallow.contains("model = \"gpt-test\""));
    assert!(codex_shallow.contains("model_reasoning_effort = \"low\""));
}

#[test]
fn depth_rows_reject_unknown_models_provider_mismatch_and_effort_inconsistency() {
    let unknown = Fixture::new();
    unknown.write_default_manifest();
    unknown.write_source_file(
        "manifests/role-depths.nota",
        "[(shallow (claude-flat None) (gpt-test (Some Low))) (deep (claude-missing (Some High)) (gpt-test (Some High)))]\n",
    );
    assert!(matches!(
        unknown
            .generate(GenerationMode::Write)
            .expect_err("unknown model fails"),
        Error::UnsupportedRoleModel { .. }
    ));

    let mismatched = Fixture::new();
    mismatched.write_default_manifest();
    mismatched.write_source_file(
        "manifests/role-depths.nota",
        "[(shallow (claude-flat None) (gpt-test (Some Low))) (deep (gpt-test (Some High)) (gpt-test (Some High)))]\n",
    );
    assert!(matches!(
        mismatched
            .generate(GenerationMode::Write)
            .expect_err("provider mismatch fails"),
        Error::RoleModelProviderMismatch { .. }
    ));

    let unsupported_effort = Fixture::new();
    unsupported_effort.write_default_manifest();
    unsupported_effort.write_source_file(
        "manifests/model-catalog.nota",
        "[(claude-flat Claude []) (claude-test Claude [Medium]) (gpt-test ChatGpt [Low Medium High Xhigh])]\n",
    );
    assert!(matches!(
        unsupported_effort
            .generate(GenerationMode::Write)
            .expect_err("unsupported effort fails"),
        Error::UnsupportedRoleModelEffort { .. }
    ));

    let effortless_with_effort = Fixture::new();
    effortless_with_effort.write_default_manifest();
    effortless_with_effort.write_source_file(
        "manifests/role-depths.nota",
        "[(shallow (claude-flat (Some Low)) (gpt-test (Some Low))) (deep (claude-test (Some High)) (gpt-test (Some High)))]\n",
    );
    assert!(matches!(
        effortless_with_effort
            .generate(GenerationMode::Write)
            .expect_err("effortless model with effort fails"),
        Error::EffortlessRoleModelCarriesEffort { .. }
    ));

    let missing_effort = Fixture::new();
    missing_effort.write_default_manifest();
    missing_effort.write_source_file(
        "manifests/role-depths.nota",
        "[(shallow (claude-flat None) (gpt-test None)) (deep (claude-test (Some High)) (gpt-test (Some High)))]\n",
    );
    assert!(matches!(
        missing_effort
            .generate(GenerationMode::Write)
            .expect_err("missing effort fails"),
        Error::MissingRoleModelEffort { .. }
    ));
}

#[test]
fn role_descriptions_reject_missing_duplicate_and_stale_cells() {
    let missing = Fixture::new();
    missing.write_default_manifest();
    missing.write_source_file(
        "manifests/role-descriptions.nota",
        "[(read shallow [Read shallow.]) (read deep [Read deep.]) (write shallow [Write shallow.])]\n",
    );
    assert!(matches!(
        missing
            .generate(GenerationMode::Write)
            .expect_err("missing cell fails"),
        Error::MissingRoleDescription { .. }
    ));

    let duplicated = Fixture::new();
    duplicated.write_default_manifest();
    duplicated.write_source_file(
        "manifests/role-descriptions.nota",
        "[(read shallow [Read shallow.]) (read shallow [Read again.]) (read deep [Read deep.]) (write shallow [Write shallow.]) (write deep [Write deep.])]\n",
    );
    let duplicate_error = duplicated
        .generate(GenerationMode::Write)
        .expect_err("duplicate cell fails");
    assert!(
        matches!(duplicate_error, Error::DuplicateRoleDescription { .. }),
        "{duplicate_error:?}"
    );

    let stale = Fixture::new();
    stale.write_default_manifest();
    stale.write_source_file(
        "manifests/role-descriptions.nota",
        "[(read shallow [Read shallow.]) (read deep [Read deep.]) (write shallow [Write shallow.]) (write deep [Write deep.]) (write retired [Retired cell.])]\n",
    );
    assert!(matches!(
        stale
            .generate(GenerationMode::Write)
            .expect_err("stale cell fails"),
        Error::StaleRoleDescription { .. }
    ));
}

#[test]
fn role_permissions_and_depths_reject_duplicates_and_empty_axes() {
    let duplicate_permission = Fixture::new();
    duplicate_permission.write_default_manifest();
    duplicate_permission.write_source_file(
        "manifests/role-permissions.nota",
        "[(read [Read body.] Restricted) (read [Read body.] Restricted)]\n",
    );
    assert!(matches!(
        duplicate_permission
            .generate(GenerationMode::Write)
            .expect_err("duplicate permission fails"),
        Error::DuplicateRolePermission { .. }
    ));

    let duplicate_depth = Fixture::new();
    duplicate_depth.write_default_manifest();
    duplicate_depth.write_source_file(
        "manifests/role-depths.nota",
        "[(shallow (claude-flat None) (gpt-test (Some Low))) (shallow (claude-test (Some High)) (gpt-test (Some High)))]\n",
    );
    assert!(matches!(
        duplicate_depth
            .generate(GenerationMode::Write)
            .expect_err("duplicate depth fails"),
        Error::DuplicateRoleDepth { .. }
    ));

    let empty_permissions = Fixture::new();
    empty_permissions.write_default_manifest();
    empty_permissions.write_source_file("manifests/role-permissions.nota", "[]\n");
    assert!(matches!(
        empty_permissions
            .generate(GenerationMode::Write)
            .expect_err("empty permission axis fails"),
        Error::MissingRolePermissions
    ));

    let empty_depths = Fixture::new();
    empty_depths.write_default_manifest();
    empty_depths.write_source_file("manifests/role-depths.nota", "[]\n");
    assert!(matches!(
        empty_depths
            .generate(GenerationMode::Write)
            .expect_err("empty depth axis fails"),
        Error::MissingRoleDepths
    ));
}

#[test]
fn universal_role_modules_expand_into_every_generated_role_packet() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill]))]\n",
    );
    fixture.write_role_cross_product_sources();
    fixture.write_universal_role_modules(
        "[shared feature]\n",
        "[(example skills/example.md [] RuntimeSkill) (shared skills/shared.md [] RoleComposition) (feature skills/feature.md [shared] RoleComposition)]\n",
    );
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample rule.\n");
    fixture.write_source_file("skills/shared.md", "# Module - shared\n\nShared rule.\n");
    fixture.write_source_file("skills/feature.md", "# Module - feature\n\nFeature rule.\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("universal modules expand");

    for role in ["read-shallow", "read-deep", "write-shallow", "write-deep"] {
        let packet = fixture.read_workspace_file(&format!(".claude/agents/{role}.md"));
        assert!(
            packet.contains("Shared rule."),
            "{role} keeps shared module"
        );
        assert!(
            packet.contains("Feature rule."),
            "{role} keeps feature module"
        );
        assert!(
            packet.find("Shared rule.") < packet.find("Feature rule."),
            "{role} keeps dependency order"
        );
    }
}

#[test]
fn visualization_reports_every_generated_role_packet_composition() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample rule.\n");

    let report = fixture.visualize().expect("visualization succeeds");

    let role_names = report
        .role_visualizations
        .payload()
        .iter()
        .map(|visualization| visualization.output_identifier.as_ref().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        role_names,
        vec!["read-deep", "read-shallow", "write-deep", "write-shallow"]
    );
    for visualization in report.role_visualizations.payload() {
        assert_eq!(visualization.role_packet_compositions.payload().len(), 3);
    }
    let generated_paths = report
        .generated_output_visualizations
        .payload()
        .iter()
        .map(|output| output.output_path.as_ref().to_owned())
        .collect::<BTreeSet<_>>();
    assert!(generated_paths.contains(".claude/agents/read-deep.md"));
    assert!(generated_paths.contains(".codex/agents/write-shallow.toml"));
}

#[test]
fn write_mode_removes_role_outputs_the_inventory_no_longer_claims() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample rule.\n");
    fixture.write_workspace_file(".claude/agents/retired.md", "stale\n");
    fixture.write_workspace_file(
        "skills/generated-role-outputs.nota",
        "[.claude/agents/retired.md]\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("stale role output is pruned");

    assert!(
        !fixture
            .workspace
            .path()
            .join(".claude/agents/retired.md")
            .exists()
    );
    assert!(
        fixture
            .workspace
            .path()
            .join(".claude/agents/read-deep.md")
            .exists()
    );
}

#[test]
fn repository_manifests_generate_the_eight_permission_by_depth_roles() {
    let permissions: RolePermissions =
        NotaSource::new(include_str!("../manifests/role-permissions.nota"))
            .parse()
            .expect("role permissions parse");
    let depths: RoleDepths = NotaSource::new(include_str!("../manifests/role-depths.nota"))
        .parse()
        .expect("role depths parse");
    let descriptions: RoleDescriptions =
        NotaSource::new(include_str!("../manifests/role-descriptions.nota"))
            .parse()
            .expect("role descriptions parse");
    assert_eq!(permissions.payload().len(), 2);
    assert_eq!(depths.payload().len(), 4);
    assert_eq!(descriptions.payload().len(), 8);

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("repository manifests generate");
    for role in [
        "read-trivial",
        "read-ordinary",
        "read-demanding",
        "read-critical",
        "write-trivial",
        "write-ordinary",
        "write-demanding",
        "write-critical",
    ] {
        for path in [
            format!(".claude/agents/{role}.md"),
            format!(".codex/agents/{role}.toml"),
            format!(".pi/agents/{role}.md"),
        ] {
            assert!(
                fixture.workspace.path().join(&path).exists(),
                "{path} generates"
            );
        }
    }
    let trivial = flat_frontmatter(&fixture.read_workspace_file(".claude/agents/read-trivial.md"));
    assert_eq!(
        trivial.get("model").map(String::as_str),
        Some("claude-haiku-4-5")
    );
    assert!(!trivial.contains_key("effort"));
    let critical =
        flat_frontmatter(&fixture.read_workspace_file(".claude/agents/write-critical.md"));
    assert_eq!(
        critical.get("model").map(String::as_str),
        Some("claude-opus-5")
    );
    assert_eq!(critical.get("effort").map(String::as_str), Some("high"));
}

#[test]
fn target_conditionals_render_per_harness_surface() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_universal_role_modules(
        "[example]\n",
        "[(example skills/example.md [] RuntimeSkill)]\n",
    );
    fixture.write_source_file(
        "skills/example.md",
        "Shared line.\n\n{% if claude %}\nClaude line.\n{% endif %}\n{% if codex %}\nCodex line.\n{% endif %}\n{% if pi %}\nPi line.\n{% endif %}\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");

    for (path, present, absent) in [
        (
            ".claude/skills/example/SKILL.md",
            "Claude line.",
            ["Codex line.", "Pi line."],
        ),
        (
            ".agents/skills/example/SKILL.md",
            "Codex line.",
            ["Claude line.", "Pi line."],
        ),
        (
            ".claude/agents/read-deep.md",
            "Claude line.",
            ["Codex line.", "Pi line."],
        ),
        (
            ".codex/agents/read-deep.toml",
            "Codex line.",
            ["Claude line.", "Pi line."],
        ),
        (
            ".pi/agents/read-deep.md",
            "Pi line.",
            ["Claude line.", "Codex line."],
        ),
    ] {
        let generated = fixture.read_workspace_file(path);
        assert!(
            generated.contains(present),
            "{path} carries `{present}`:\n{generated}"
        );
        for excluded in absent {
            assert!(
                !generated.contains(excluded),
                "{path} excludes `{excluded}`:\n{generated}"
            );
        }
        assert_eq!(
            generated.matches("Shared line.").count(),
            1,
            "{path} carries the unconditional line exactly once"
        );
    }
}

#[test]
fn a_false_target_block_leaves_no_blank_line_behind() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "First line.\nSecond line.\n\n{% if codex %}\nCodex line.\n{% endif %}\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");

    assert_eq!(
        fixture.read_workspace_file(".claude/skills/example/SKILL.md"),
        "---\nname: example\ndescription: 'Example skill.'\n---\n\nFirst line.\nSecond line.\n"
    );
    assert_eq!(
        fixture.read_workspace_file(".agents/skills/example/SKILL.md"),
        "---\nname: example\ndescription: 'Example skill.'\n---\n\nFirst line.\nSecond line.\n\nCodex line.\n"
    );
}

#[test]
fn a_misspelled_target_fails_generation_and_names_the_known_targets() {
    for source in [
        "Shared line.\n\n{% if kodex %}\nCodex line.\n{% endif %}\n",
        "Shared line.\n\n{% if not kodex %}\nCodex line.\n{% endif %}\n",
    ] {
        let fixture = Fixture::new();
        fixture.write_default_manifest();
        fixture.write_source_file("skills/example.md", source);

        let error = fixture
            .generate(GenerationMode::Write)
            .expect_err("a misspelled target fails generation");
        let message = error.to_string();
        assert!(
            message.contains("skills/example.md"),
            "error names the source file: {message}"
        );
        assert!(
            message.contains("claude, codex, pi"),
            "error names the known targets: {message}"
        );
    }
}

#[test]
fn the_conditional_grammar_stays_closed() {
    for source in [
        "Shared line.\n\n{% for target in targets %}\nLooped.\n{% endfor %}\n",
        "Shared line.\n\n{{ codex }}\n",
        "Shared line.\n\n{% include \"other.md\" %}\n",
        "Shared line.\n\n{% raw %}\n{% if codex %}\n{% endraw %}\n",
        "Shared line.\n\nA brace { in prose.\n",
        "Shared line.\n\n{ % if codex % }\nNear miss.\n{ % endif % }\n",
        "Shared line.\n\n{% if codex %} inline text {% endif %}\n",
    ] {
        let fixture = Fixture::new();
        fixture.write_default_manifest();
        fixture.write_source_file("skills/example.md", source);

        let error = fixture
            .generate(GenerationMode::Write)
            .expect_err("an out-of-grammar construct fails generation");
        assert!(
            error.to_string().contains("skills/example.md"),
            "error names the source file for source `{source}`: {error}"
        );
    }
}

#[test]
fn no_generated_repository_output_contains_a_brace() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("repository generation succeeds");

    let mut checked = 0usize;
    for directory in [".agents", ".claude", ".codex", ".pi"] {
        for entry in walkdir(&fixture.workspace.path().join(directory)) {
            let generated = fs::read_to_string(&entry).expect("read generated output");
            assert!(
                !generated.contains(['{', '}']),
                "generated output {} contains a brace",
                entry.display()
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 68,
        "checked every generated output, found {checked}"
    );
}

fn walkdir(root: &Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(walkdir(&path));
        } else {
            found.push(path);
        }
    }
    found
}

struct Fixture {
    source: TempDir,
    workspace: TempDir,
}

impl Fixture {
    fn new() -> Self {
        Self {
            source: TempDir::new().expect("source tempdir"),
            workspace: TempDir::new().expect("workspace tempdir"),
        }
    }

    fn write_default_manifest(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill ClaudeSkill]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(example skills/example.md [] RuntimeSkill)]\n",
        );
        self.write_role_cross_product_sources();
    }

    /// Every generation reads the permission, depth, description, and catalog
    /// manifests, so even a skill-only fixture needs a valid cross product.
    fn write_role_cross_product_sources(&self) {
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(claude-flat Claude []) (claude-test Claude [Low Medium High Xhigh]) (gpt-test ChatGpt [Low Medium High Xhigh])]\n",
        );
        self.write_source_file(
            "manifests/role-permissions.nota",
            "[(read [Read body.] Restricted) (write [] Unrestricted)]\n",
        );
        self.write_source_file(
            "manifests/role-depths.nota",
            "[(shallow (claude-flat None) (gpt-test (Some Low))) (deep (claude-test (Some High)) (gpt-test (Some High)))]\n",
        );
        self.write_source_file(
            "manifests/role-descriptions.nota",
            "[(read shallow [Read shallow.]) (read deep [Read deep.]) (write shallow [Write shallow.]) (write deep [Write deep.])]\n",
        );
    }

    fn write_universal_role_modules(&self, modules: &str, dependencies: &str) {
        self.write_source_file("manifests/universal-role-modules.nota", modules);
        self.write_source_file("manifests/module-dependencies.nota", dependencies);
    }

    fn write_source_file(&self, path: &str, text: &str) {
        self.write_file(self.source.path(), path, text);
    }

    fn write_workspace_file(&self, path: &str, text: &str) {
        self.write_file(self.workspace.path(), path, text);
    }

    fn read_workspace_file(&self, path: &str) -> String {
        fs::read_to_string(self.workspace.path().join(path)).expect("read workspace file")
    }

    fn write_file(&self, root: &Path, path: &str, text: &str) {
        let full_path = root.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(full_path, text).expect("write fixture file");
    }

    fn generate(
        &self,
        generation_mode: GenerationMode,
    ) -> Result<skills::schema::assembly::GenerationReport, Error> {
        GenerationRequest {
            source_root: SourceRoot::new(self.source.path().to_string_lossy().into_owned()),
            workspace_root: WorkspaceRoot::new(
                self.workspace.path().to_string_lossy().into_owned(),
            ),
            manifest_path: ManifestPath::new("manifests/active-outputs.nota"),
            generation_mode,
        }
        .generate()
    }

    fn visualize(&self) -> Result<skills::schema::assembly::VisualizationReport, Error> {
        VisualizationRequest {
            source_root: SourceRoot::new(self.source.path().to_string_lossy().into_owned()),
            workspace_root: WorkspaceRoot::new(
                self.workspace.path().to_string_lossy().into_owned(),
            ),
            manifest_path: ManifestPath::new("manifests/active-outputs.nota"),
        }
        .visualize()
    }

    fn generate_from_repo(
        &self,
        generation_mode: GenerationMode,
    ) -> Result<skills::schema::assembly::GenerationReport, Error> {
        GenerationRequest {
            source_root: SourceRoot::new(env!("CARGO_MANIFEST_DIR")),
            workspace_root: WorkspaceRoot::new(
                self.workspace.path().to_string_lossy().into_owned(),
            ),
            manifest_path: ManifestPath::new("manifests/active-outputs.nota"),
            generation_mode,
        }
        .generate()
    }
}
