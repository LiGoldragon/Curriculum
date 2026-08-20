use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};

use dotos::DotosSource;
use skills::{
    Error,
    schema::assembly::{
        GenerationMode, GenerationRequest, ManifestPath, Operation, RoleDepths, RoleDescriptions,
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
fn current_dotos_assembly_contract_decodes_the_generator_request() {
    let operation: Operation = DotosSource::new(include_str!("../skills-generate.dotos"))
        .parse()
        .expect("current generator request decodes through the handwritten contract");

    assert!(matches!(operation, Operation::Generate(_)));
}

#[test]
fn generation_rejects_the_source_checkout_as_the_workspace() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# example\n\nKeep the source checkout safe.\n",
    );

    let error = GenerationRequest {
        source_root: SourceRoot::new(fixture.source.path().to_string_lossy().into_owned()),
        workspace_root: WorkspaceRoot::new(fixture.source.path().to_string_lossy().into_owned()),
        manifest_path: ManifestPath::new("manifests/active-outputs.dotos"),
        generation_mode: GenerationMode::Write,
    }
    .generate()
    .expect_err("source checkout must never receive generated runtime output");

    assert!(matches!(error, Error::WorkspaceIsSourceCheckout { .. }));
    assert!(!fixture.source.path().join(".agents").exists());
}

#[test]
fn visualization_allows_the_source_checkout_without_writing_runtime_outputs() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# example\n\nKeep visualization read-only.\n",
    );

    let source_root = fixture.source.path().to_string_lossy().into_owned();
    let report = VisualizationRequest {
        source_root: SourceRoot::new(source_root.clone()),
        workspace_root: WorkspaceRoot::new(source_root),
        manifest_path: ManifestPath::new("manifests/active-outputs.dotos"),
    }
    .visualize()
    .expect("visualization reads a source checkout");

    assert!(
        report
            .generated_output_visualizations
            .payload()
            .iter()
            .any(|output| output.output_path.as_ref() == ".agents/skills/example/SKILL.md")
    );
    for tree in [".agents", ".claude", ".codex", ".pi"] {
        assert!(
            !fixture.source.path().join(tree).exists(),
            "visualization must not create {tree}"
        );
    }
}

#[test]
fn generation_writes_derived_skill_surfaces_with_manifest_frontmatter() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\nname: stale\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\nKeep the prose.\n",
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
    assert!(!generated_paths.contains(&"skills/skills.dotos"));

    let generated = fixture.read_workspace_file(".agents/skills/example/SKILL.md");
    assert_eq!(
        generated,
        "---\nname: example\ndescription: 'Example skill.'\n---\n\n# example\n\n## Rule\n\nKeep the prose.\n"
    );
    assert_eq!(
        generated,
        fixture.read_workspace_file(".claude/skills/example/SKILL.md")
    );
    assert!(
        !fixture
            .workspace
            .path()
            .join("skills/skills.dotos")
            .exists()
    );
}

#[test]
fn generation_allows_fenced_frontmatter_examples_inside_modules() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\n```markdown\n---\nname: example\n---\n```\n",
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
        "---\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\n---\n\nKeep the prose.\n",
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
        "---\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\nUse `[text](url)` only as a literal example.\n",
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
    zero_title.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\nNo title.\n",
    );
    zero_title
        .generate(GenerationMode::Write)
        .expect("zero titles generate");
    assert_eq!(
        zero_title.read_workspace_file(".agents/skills/example/SKILL.md"),
        "---\nname: example\ndescription: 'Example skill.'\n---\n\nNo title.\n"
    );

    let one_title = Fixture::new();
    one_title.write_default_manifest();
    one_title.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill — example\n\nOne title.\n",
    );
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
    multiple_titles.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# First\n\n# Second\n",
    );
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
        "manifests/module-dependencies.dotos",
        "[{example modules/example/full.md RuntimeSkill}]
",
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
        "manifests/skill-module-compositions.dotos",
        "[{missing []}]
",
    );
    let error = inactive
        .generate(GenerationMode::Write)
        .expect_err("inactive skill composition rejects generation");
    assert!(matches!(error, Error::StaleSkillModuleComposition { .. }));

    let duplicate = Fixture::new();
    duplicate.write_default_manifest();
    duplicate.write_source_file("skills/example.md", "# Skill — example\n\nExample.\n");
    duplicate.write_source_file(
        "manifests/skill-module-compositions.dotos",
        "[{example []} {example []}]
",
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
        "---\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Repeat\n\nFirst.\n\n## Repeat\n\nSecond.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("duplicate headings fail");

    assert!(matches!(error, Error::DuplicateHeading { .. }), "{error:?}");
}


#[test]
fn psyche_interraction_has_required_structure() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("psyche interaction profile generates");
    let agents = fixture.read_workspace_file(".agents/skills/psyche-interraction/SKILL.md");
    let claude = fixture.read_workspace_file(".claude/skills/psyche-interraction/SKILL.md");
    assert_eq!(agents, claude);
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/psyche-interraction-continuation.md")
            .exists()
    );
}

#[test]
fn general_instructions_is_registered_and_tenets_is_not_auto_injected() {
    assert!(
        include_str!("../manifests/universal-role-modules.dotos")
            .contains("[general-instructions]")
    );
    assert!(
        !include_str!("../manifests/universal-role-modules.dotos").contains("tenets"),
        "tenets is a loadable skill and must not be auto-injected into roles"
    );
}

#[test]
fn generation_strips_source_maintenance_notes_from_runtime_surfaces() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\n## Rule\n\nGenerated.\n\n## Source Maintenance Notes\n\nMaintainer-only synchronization steps.\n",
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
        "manifests/active-outputs.dotos",
        "[Skill.{management management Meta Mechanism [AgentsSkill ClaudeSkill]}]
",
    );
    fixture.write_role_cross_product_sources();
    fixture.write_universal_role_modules(
        "[management]\n",
        "[{management skills/management.md RuntimeSkill} {claude-management skills/claude-management.md RuntimeSkill}]
",
    );
    fixture.write_source_file(
        "manifests/target-module-insertions.dotos",
        "[{management ClaudeSkill [claude-management]} {management ClaudeAgent [claude-management]}]\n",
    );
    fixture.write_source_file(
        "skills/management.md",
        "---\ndescription: Management skill.\n---\n\n# Skill - management\n\n## Shared Rule\n\nShared management.\n",
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
        "manifests/active-outputs.dotos",
        "[Skill.{example example Craft Topic [AgentsSkill]}]
",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.dotos",
        "[{example skills/example.md RuntimeSkill}]
",
    );
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example.\ndependencies: [example]\n---\n\nExample.\n",
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
fn generation_requires_dependencies_in_source_frontmatter() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_file(
        fixture.source.path(),
        "skills/example.md",
        "---\ndescription: Example.\n---\n\nExample.\n",
    );

    assert!(matches!(
        fixture
            .generate(GenerationMode::Write)
            .expect_err("dependency declaration is required"),
        Error::MissingSkillDependencies { .. }
    ));
}

#[test]
fn generation_rejects_transitive_module_dependency_cycle() {
    let fixture = Fixture::new();
    fixture.write_role_cross_product_sources();
    fixture.write_source_file(
        "manifests/active-outputs.dotos",
        "[Skill.{example first Craft Topic [AgentsSkill]}]
",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.dotos",
        "[{first skills/first.md RuntimeSkill} {second skills/second.md RuntimeSkill} {third skills/third.md RuntimeSkill}]\n",
    );
    fixture.write_source_file(
        "skills/first.md",
        "---\ndescription: First.\ndependencies: [second]\n---\n\nFirst.\n",
    );
    fixture.write_source_file(
        "skills/second.md",
        "---\ndescription: Second.\ndependencies: [third]\n---\n\nSecond.\n",
    );
    fixture.write_source_file(
        "skills/third.md",
        "---\ndescription: Third.\ndependencies: [second]\n---\n\nThird.\n",
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
        "manifests/active-outputs.dotos",
        "[Skill.{edit-coordination-core edit-coordination-core Workflow Mechanism [AgentsSkill]}]
",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.dotos",
        "[{edit-coordination-core skills/edit-coordination-core.md RoleComposition}]\n",
    );
    fixture.write_source_file("skills/edit-coordination-core.md", "Role-only content.\n");

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
        "---\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_workspace_file(".agents/skills/example/SKILL.md", "old\n");
    fixture.write_workspace_file(".claude/skills/example/SKILL.md", "old\n");
    fixture.write_workspace_file("skills/skills.dotos", "old\n");

    let error = fixture
        .generate(GenerationMode::Check)
        .expect_err("stale output fails check mode");

    assert!(matches!(error, Error::StaleOutput { .. }), "{error:?}");
    assert!(!error.to_string().contains("skills.dotos"));
    assert!(error.to_string().contains("generate-skills"));
    assert!(error.to_string().contains("check-skills"));
}

#[test]
fn generation_rejects_skill_with_oversized_serialized_block() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        &format!(
            "---\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\n{}\n",
            "x".repeat(33_000)
        ),
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
    fixture.write_workspace_file("skills/skills.dotos", "old retired index\n");

    let error = fixture
        .generate_from_repo(GenerationMode::Check)
        .expect_err("retired skill index fails deployment check");
    assert!(
        matches!(error, Error::StaleGeneratedOutput { ref path } if path.ends_with("skills/skills.dotos")),
        "{error:?}"
    );

    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("write mode prunes retired skill index");
    assert!(
        !fixture
            .workspace
            .path()
            .join("skills/skills.dotos")
            .exists(),
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
        "---\ndescription: Example skill.\n---\n\n# Skill — example\n\n## Rule\n\nGenerated.\n",
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
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\nExample rule.\n",
    );

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
    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.dotos");
    for role in ["read-shallow", "read-deep", "write-shallow", "write-deep"] {
        assert!(inventory.contains(&format!(".claude/agents/{role}.md")));
        assert!(inventory.contains(&format!(".codex/agents/{role}.toml")));
        assert!(inventory.contains(&format!(".pi/agents/{role}.md")));
    }
    assert!(!inventory.contains("worker"));
}

#[test]
fn permission_body_precedes_the_shared_body_only_for_restricted_permissions() {
    const SHARED_BODY: &str =
        "The brief is your authority. Decide what it settles; return what it does not.";
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\nExample rule.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("role bodies generate");

    let read_packet = fixture.read_workspace_file(".claude/agents/read-deep.md");
    let write_packet = fixture.read_workspace_file(".claude/agents/write-deep.md");
    assert!(read_packet.contains("Read body."));
    assert!(!write_packet.contains("Read body."));
    for packet in [&read_packet, &write_packet] {
        assert!(packet.contains(SHARED_BODY));
    }
    assert!(read_packet.find("Read body.") < read_packet.find(SHARED_BODY));
}

#[test]
fn restricted_permissions_block_editing_tools_on_the_surfaces_that_carry_them() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\nExample rule.\n",
    );

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
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\nExample rule.\n",
    );

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
        "manifests/role-depths.dotos",
        "[{shallow {claude-flat None} {gpt-test Some.Low}} {deep {claude-missing Some.High} {gpt-test Some.High}}]
",
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
        "manifests/role-depths.dotos",
        "[{shallow {claude-flat None} {gpt-test Some.Low}} {deep {gpt-test Some.High} {gpt-test Some.High}}]
",
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
        "manifests/model-catalog.dotos",
        "[{claude-flat Claude []} {claude-test Claude [Medium]} {gpt-test ChatGpt [Low Medium High Xhigh]}]
",
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
        "manifests/role-depths.dotos",
        "[{shallow {claude-flat Some.Low} {gpt-test Some.Low}} {deep {claude-test Some.High} {gpt-test Some.High}}]
",
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
        "manifests/role-depths.dotos",
        "[{shallow {claude-flat None} {gpt-test None}} {deep {claude-test Some.High} {gpt-test Some.High}}]
",
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
        "manifests/role-descriptions.dotos",
        "[{read shallow (|Read shallow.|)} {read deep (|Read deep.|)} {write shallow (|Write shallow.|)}]
",
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
        "manifests/role-descriptions.dotos",
        "[{read shallow (|Read shallow.|)} {read shallow (|Read again.|)} {read deep (|Read deep.|)} {write shallow (|Write shallow.|)} {write deep (|Write deep.|)}]
",
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
        "manifests/role-descriptions.dotos",
        "[{read shallow (|Read shallow.|)} {read deep (|Read deep.|)} {write shallow (|Write shallow.|)} {write deep (|Write deep.|)} {write retired (|Retired cell.|)}]
",
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
        "manifests/role-permissions.dotos",
        "[{read (|Read body.|) Restricted} {read (|Read body.|) Restricted}]
",
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
        "manifests/role-depths.dotos",
        "[{shallow {claude-flat None} {gpt-test Some.Low}} {shallow {claude-test Some.High} {gpt-test Some.High}}]
",
    );
    assert!(matches!(
        duplicate_depth
            .generate(GenerationMode::Write)
            .expect_err("duplicate depth fails"),
        Error::DuplicateRoleDepth { .. }
    ));

    let empty_permissions = Fixture::new();
    empty_permissions.write_default_manifest();
    empty_permissions.write_source_file("manifests/role-permissions.dotos", "[]\n");
    assert!(matches!(
        empty_permissions
            .generate(GenerationMode::Write)
            .expect_err("empty permission axis fails"),
        Error::MissingRolePermissions
    ));

    let empty_depths = Fixture::new();
    empty_depths.write_default_manifest();
    empty_depths.write_source_file("manifests/role-depths.dotos", "[]\n");
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
        "manifests/active-outputs.dotos",
        "[Skill.{example example Craft Topic [AgentsSkill]}]
",
    );
    fixture.write_role_cross_product_sources();
    fixture.write_universal_role_modules(
        "[shared feature]\n",
        "[{example skills/example.md RuntimeSkill} {shared skills/shared.md RoleComposition} {feature skills/feature.md RoleComposition}]
",
    );
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\nExample rule.\n",
    );
    fixture.write_source_file("skills/shared.md", "# Module - shared\n\nShared rule.\n");
    fixture.write_source_file(
        "skills/feature.md",
        "---\ndependencies: [shared]\n---\n\n# Module - feature\n\nFeature rule.\n",
    );

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
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\nExample rule.\n",
    );

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
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\n# Skill - example\n\nExample rule.\n",
    );
    fixture.write_workspace_file(".claude/agents/retired.md", "stale\n");
    fixture.write_workspace_file(
        "skills/generated-role-outputs.dotos",
        "[(|.claude/agents/retired.md|)]\n",
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
        DotosSource::new(include_str!("../manifests/role-permissions.dotos"))
            .parse()
            .expect("role permissions parse");
    let depths: RoleDepths = DotosSource::new(include_str!("../manifests/role-depths.dotos"))
        .parse()
        .expect("role depths parse");
    let descriptions: RoleDescriptions =
        DotosSource::new(include_str!("../manifests/role-descriptions.dotos"))
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
        Some("claude-opus-4-6[1m]")
    );
    assert_eq!(critical.get("effort").map(String::as_str), Some("high"));
}

#[test]
fn target_conditionals_render_per_harness_surface() {
    let fixture = Fixture::new();
    fixture.write_default_manifest();
    fixture.write_universal_role_modules(
        "[example]\n",
        "[{example skills/example.md RuntimeSkill}]
",
    );
    fixture.write_source_file(
        "skills/example.md",
        "---\ndescription: Example skill.\n---\n\nShared line.\n\n{% if claude %}\nClaude line.\n{% endif %}\n{% if codex %}\nCodex line.\n{% endif %}\n{% if pi %}\nPi line.\n{% endif %}\n",
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
        "---\ndescription: Example skill.\n---\n\nFirst line.\nSecond line.\n\n{% if codex %}\nCodex line.\n{% endif %}\n",
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
        fixture.write_source_file(
            "skills/example.md",
            &format!("---\ndescription: Example skill.\n---\n\n{source}"),
        );

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
        fixture.write_source_file(
            "skills/example.md",
            &format!("---\ndescription: Example skill.\n---\n\n{source}"),
        );

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
            "manifests/active-outputs.dotos",
            "[Skill.{example example Craft Topic [AgentsSkill ClaudeSkill]}]
",
        );
        self.write_source_file(
            "manifests/module-dependencies.dotos",
            "[{example skills/example.md RuntimeSkill}]
",
        );
        self.write_role_cross_product_sources();
    }

    /// Every generation reads the permission, depth, description, and catalog
    /// manifests, so even a skill-only fixture needs a valid cross product.
    fn write_role_cross_product_sources(&self) {
        self.write_source_file(
            "manifests/model-catalog.dotos",
            "[{claude-flat Claude []} {claude-test Claude [Low Medium High Xhigh]} {gpt-test ChatGpt [Low Medium High Xhigh]}]
",
        );
        self.write_source_file(
            "manifests/role-permissions.dotos",
            "[{read (|Read body.|) Restricted} {write (||) Unrestricted}]
",
        );
        self.write_source_file(
            "manifests/role-depths.dotos",
            "[{shallow {claude-flat None} {gpt-test Some.Low}} {deep {claude-test Some.High} {gpt-test Some.High}}]
",
        );
        self.write_source_file(
            "manifests/role-descriptions.dotos",
            "[{read shallow (|Read shallow.|)} {read deep (|Read deep.|)} {write shallow (|Write shallow.|)} {write deep (|Write deep.|)}]
",
        );
    }

    fn write_universal_role_modules(&self, modules: &str, dependencies: &str) {
        self.write_source_file("manifests/universal-role-modules.dotos", modules);
        self.write_source_file("manifests/module-dependencies.dotos", dependencies);
    }

    fn write_source_file(&self, path: &str, text: &str) {
        let text = if path.starts_with("skills/") && !text.contains("dependencies:") {
            if let Some((frontmatter, body)) = text.split_once("---\n\n") {
                format!("{frontmatter}dependencies: []\n---\n\n{body}")
            } else {
                format!("---\ndependencies: []\n---\n\n{text}")
            }
        } else {
            text.to_owned()
        };
        self.write_file(self.source.path(), path, &text);
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
            manifest_path: ManifestPath::new("manifests/active-outputs.dotos"),
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
            manifest_path: ManifestPath::new("manifests/active-outputs.dotos"),
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
            manifest_path: ManifestPath::new("manifests/active-outputs.dotos"),
            generation_mode,
        }
        .generate()
    }
}
