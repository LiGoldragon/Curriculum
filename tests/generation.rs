use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};

use nota::NotaSource;
use skills::{
    Error,
    schema::assembly::{
        ActiveOutputs, EffortLevel, GenerationMode, GenerationRequest, ManifestPath, ModelCatalog,
        ModuleDependencies, ModuleKind, NamedRoleModelProfiles, NestedRoleRelations,
        RoleGenerationKind, RoleModelAssignments, RoleOptionalSkills, RoleTargetSurface,
        SkillModuleCompositions, SourceRoot, TargetModuleInsertions, UniversalRoleModules,
        VisualizationRequest, WorkspaceRoot,
    },
    trunk_guard::{TrunkDescendantGuard, TrunkDivergence},
};
use tempfile::TempDir;

#[derive(Debug, Eq, PartialEq)]
struct ParsedProjectRoleContract {
    project_role_identity: String,
    project_role_dispatch_kind: String,
    allowed_child_role_names: Vec<String>,
}

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

fn project_role_contract(packet: &str, runtime_role_name: &str) -> ParsedProjectRoleContract {
    let frontmatter = flat_frontmatter(packet);
    let identity = frontmatter
        .get("projectRoleIdentity")
        .expect("projectRoleIdentity exists");
    assert_eq!(identity, runtime_role_name);
    let dispatch_kind = frontmatter
        .get("projectRoleDispatchKind")
        .expect("projectRoleDispatchKind exists");
    assert!(matches!(dispatch_kind.as_str(), "nested" | "leaf"));
    let allowed_child_role_names = frontmatter
        .get("allowedChildRoleNames")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if dispatch_kind == "nested" {
        assert!(frontmatter.contains_key("allowedChildRoleNames"));
    } else {
        assert!(!frontmatter.contains_key("allowedChildRoleNames"));
    }
    for incompatible_key in [
        "delegation-role-classification",
        "allowed-child-role-identifiers",
    ] {
        assert!(!frontmatter.contains_key(incompatible_key));
    }
    ParsedProjectRoleContract {
        project_role_identity: identity.clone(),
        project_role_dispatch_kind: dispatch_kind.clone(),
        allowed_child_role_names,
    }
}

fn frontmatter_block(packet: &str) -> &str {
    let end = packet.find("\n---\n").expect("frontmatter closes") + "\n---\n".len();
    &packet[..end]
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
fn active_manifest_and_module_index_cover_current_skills_and_roles() {
    let active_outputs = NotaSource::new(include_str!("../manifests/active-outputs.nota"))
        .parse::<ActiveOutputs>()
        .expect("active output manifest parses");
    let module_dependencies =
        NotaSource::new(include_str!("../manifests/module-dependencies.nota"))
            .parse::<ModuleDependencies>()
            .expect("module dependency index parses");
    let target_module_insertions =
        NotaSource::new(include_str!("../manifests/target-module-insertions.nota"))
            .parse::<TargetModuleInsertions>()
            .expect("target module insertion index parses");
    let universal_role_modules =
        NotaSource::new(include_str!("../manifests/universal-role-modules.nota"))
            .parse::<UniversalRoleModules>()
            .expect("universal role module manifest parses");
    let skill_module_compositions =
        NotaSource::new(include_str!("../manifests/skill-module-compositions.nota"))
            .parse::<SkillModuleCompositions>()
            .expect("skill module composition manifest parses");
    let model_catalog = NotaSource::new(include_str!("../manifests/model-catalog.nota"))
        .parse::<ModelCatalog>()
        .expect("model catalog parses");
    let role_model_assignments =
        NotaSource::new(include_str!("../manifests/role-model-assignments.nota"))
            .parse::<RoleModelAssignments>()
            .expect("role model assignments parse");
    let named_role_model_profiles =
        NotaSource::new(include_str!("../manifests/role-model-profiles.nota"))
            .parse::<NamedRoleModelProfiles>()
            .expect("named role-model profiles parse");
    let role_optional_skills =
        NotaSource::new(include_str!("../manifests/role-optional-skills.nota"))
            .parse::<RoleOptionalSkills>()
            .expect("role optional skills parse");
    let nested_role_relations =
        NotaSource::new(include_str!("../manifests/nested-role-relations.nota"))
            .parse::<NestedRoleRelations>()
            .expect("nested role relations parse");

    // These hardcoded generation expectations intentionally catch membership drift.
    // Update them when module membership, role includes, or universal role modules change.
    let skill_count = active_outputs
        .payload()
        .iter()
        .filter(|output| matches!(output, skills::schema::assembly::ActiveOutput::Skill(_)))
        .count();
    let role_count = active_outputs
        .payload()
        .iter()
        .filter(|output| matches!(output, skills::schema::assembly::ActiveOutput::Role(_)))
        .count();

    assert_eq!(skill_count, 65);
    assert_eq!(role_count, 14);
    assert_eq!(model_catalog.payload().len(), 6);
    assert_eq!(named_role_model_profiles.payload().len(), 1);
    assert_eq!(nested_role_relations.payload().len(), 2);
    assert_eq!(role_model_assignments.payload().len(), role_count);
    assert_eq!(role_optional_skills.payload().len(), role_count);

    let model_catalog_source = include_str!("../manifests/model-catalog.nota");
    let role_model_assignments_source = include_str!("../manifests/role-model-assignments.nota");
    assert!(model_catalog_source.contains("(Claude (claude-sonnet-5 [(Medium 10)]))"));
    assert!(
        model_catalog_source
            .contains("(ChatGpt (gpt-5.6-sol openai-codex [(Medium 50) (High 60)]))")
    );
    assert!(
        model_catalog_source
            .contains("(ChatGpt (gpt-5.6-terra openai-codex [(Medium 20) (High 30) (Xhigh 40)]))")
    );
    assert!(model_catalog_source.contains("(Claude (fable-5 [(Medium 50) (High 60)]))"));
    assert!(model_catalog_source.contains("(Claude (claude-opus-4-8 [(High 30) (Xhigh 40)]))"));
    for sonnet_role in ["intent-recorder", "scout", "repository-closeout"] {
        assert!(
            role_model_assignments_source.contains(&format!(
                "(Direct ({sonnet_role} (gpt-5.6-luna Medium) (claude-sonnet-5 Medium)))"
            )),
            "{sonnet_role} uses Claude Sonnet 5"
        );
    }
    assert!(!model_catalog_source.contains("claude-sonnet-4-6"));
    assert!(!role_model_assignments_source.contains("claude-sonnet-4-6"));

    let active_skill_identifiers: BTreeSet<&str> = active_outputs
        .payload()
        .iter()
        .filter_map(|output| match output {
            skills::schema::assembly::ActiveOutput::Skill(skill) => {
                Some(skill.output_identifier.as_ref())
            }
            skills::schema::assembly::ActiveOutput::Role(_) => None,
        })
        .collect();
    for required_skill in [
        "component-architecture",
        "design-quality",
        "version-control",
        "work-tracking",
        "repository-publication",
        "pi-extension-updates",
        "nota-shape-checklist",
        "management",
        "psyche-interraction",
        "tenets",
        "documentation-placement",
        "skill-designing",
    ] {
        assert!(
            active_skill_identifiers.contains(required_skill),
            "{required_skill} active skill uses approved appellation"
        );
    }
    for deprecated_skill in [
        "component-triad",
        "beauty",
        "jj",
        "beads",
        "human-interaction",
        "context-maintenance",
        "orchestration",
        "kameo",
        "architecture-editor",
        "skill-editor",
    ] {
        assert!(
            !active_skill_identifiers.contains(deprecated_skill),
            "{deprecated_skill} active skill appellation stays retired or removed"
        );
    }

    let dependency_modules: BTreeSet<&str> = module_dependencies
        .payload()
        .iter()
        .map(|dependency| dependency.module_identifier.as_ref())
        .collect();
    let module_kinds: BTreeMap<&str, ModuleKind> = module_dependencies
        .payload()
        .iter()
        .map(|dependency| {
            (
                dependency.module_identifier.as_ref(),
                dependency.module_kind,
            )
        })
        .collect();
    for dependency in module_dependencies.payload() {
        let identifier = dependency.module_identifier.as_ref();
        let expected_path = if dependency.module_kind == ModuleKind::RoleSource {
            format!(
                "roles/{}.md",
                identifier.strip_prefix("role-").unwrap_or(identifier)
            )
        } else {
            format!("skills/{identifier}.md")
        };
        assert_eq!(dependency.module_path.as_ref(), expected_path);
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(dependency.module_path.as_ref())
                .is_file(),
            "{expected_path} is an active flat source"
        );
    }
    let mut reachable_modules: BTreeSet<&str> = universal_role_modules
        .payload()
        .iter()
        .map(|module| module.as_ref())
        .collect();
    for output in active_outputs.payload() {
        match output {
            skills::schema::assembly::ActiveOutput::Skill(skill) => {
                reachable_modules.insert(skill.module_identifier.as_ref());
            }
            skills::schema::assembly::ActiveOutput::Role(role) => {
                reachable_modules.insert(role.module_identifier.as_ref());
                reachable_modules.extend(
                    role.included_modules
                        .payload()
                        .iter()
                        .map(|module| module.as_ref()),
                );
            }
        }
    }
    for composition in skill_module_compositions.payload() {
        reachable_modules.extend(
            composition
                .included_modules
                .payload()
                .iter()
                .map(|module| module.as_ref()),
        );
    }
    loop {
        let previous_count = reachable_modules.len();
        for dependency in module_dependencies.payload() {
            if reachable_modules.contains(dependency.module_identifier.as_ref()) {
                reachable_modules.extend(
                    dependency
                        .dependency_modules
                        .payload()
                        .iter()
                        .map(|module| module.as_ref()),
                );
            }
        }
        for insertion in target_module_insertions.payload() {
            if reachable_modules.contains(insertion.module_identifier.as_ref()) {
                reachable_modules.extend(
                    insertion
                        .included_modules
                        .payload()
                        .iter()
                        .map(|module| module.as_ref()),
                );
            }
        }
        if reachable_modules.len() == previous_count {
            break;
        }
    }
    assert_eq!(
        dependency_modules, reachable_modules,
        "module index retains only active sources reachable from outputs, skill and role composition, and target insertions"
    );
    let role_composition_modules = [
        "general-instructions",
        "codex-skill-loading",
        "edit-coordination-core",
        "editing-closeout",
        "code-implementation-core",
        "rust-core",
        "nix-core",
        "intent-core",
        "repo-scaffold-core",
        "repo-operation-core",
        "architectural-truth-tests",
        "rust-discipline",
        "bead-weaver",
        "spirit-submission",
    ];
    for module_identifier in role_composition_modules {
        assert_eq!(
            module_kinds.get(module_identifier),
            Some(&ModuleKind::RoleComposition),
            "{module_identifier} is generator-only role composition"
        );
    }
    assert_eq!(
        module_kinds.get("spirit-query"),
        Some(&ModuleKind::RuntimeSkill),
        "spirit-query remains a first-class read-only skill"
    );
    assert_eq!(
        module_kinds.get("psyche-interraction-claude-briefness"),
        Some(&ModuleKind::RuntimeSkill),
        "Claude-only psyche interaction overlay can emit to the Claude skill surface"
    );
    assert!(
        !dependency_modules.contains("human-interaction"),
        "human-interaction is deleted from the dependency index"
    );
    let spirit_query_dependency = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "spirit-query")
        .expect("spirit-query dependency indexed");
    assert_eq!(
        spirit_query_dependency
            .dependency_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        Vec::<&str>::new()
    );
    let management_dependency = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "management")
        .expect("management dependency indexed");
    assert_eq!(
        management_dependency
            .dependency_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["tenets"]
    );
    let psyche_interraction_dependency = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "psyche-interraction")
        .expect("psyche-interraction dependency indexed");
    assert_eq!(
        psyche_interraction_dependency
            .dependency_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["tenets"]
    );
    assert_eq!(skill_module_compositions.payload().len(), 1);
    let psyche_interraction_composition = skill_module_compositions
        .payload()
        .first()
        .expect("psyche interaction composition exists");
    assert_eq!(
        psyche_interraction_composition.output_identifier.as_ref(),
        "psyche-interraction"
    );
    assert_eq!(
        psyche_interraction_composition
            .included_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["psyche-interraction-continuation"]
    );
    for nota_module in ["nota-design", "nota-schema-design", "nota-literacy"] {
        let dependency = module_dependencies
            .payload()
            .iter()
            .find(|dependency| dependency.module_identifier.as_ref() == nota_module)
            .unwrap_or_else(|| panic!("{nota_module} dependency indexed"));
        assert!(
            dependency
                .dependency_modules
                .payload()
                .iter()
                .any(|module| module.as_ref() == "nota-shape-checklist"),
            "{nota_module} includes nota-shape-checklist"
        );
    }
    assert!(
        !management_dependency
            .dependency_modules
            .payload()
            .iter()
            .any(|module| module.as_ref() == "context-handover"),
        "context-handover remains separate/manual-load only"
    );
    assert_eq!(
        target_module_insertions
            .payload()
            .iter()
            .map(|insertion| (
                insertion.module_identifier.as_ref(),
                insertion.output_surface,
                insertion
                    .included_modules
                    .payload()
                    .iter()
                    .map(|module| module.as_ref())
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>(),
        [
            (
                "general-instructions",
                skills::schema::assembly::OutputSurface::CodexAgent,
                vec!["codex-skill-loading"]
            ),
            (
                "psyche-interraction",
                skills::schema::assembly::OutputSurface::ClaudeSkill,
                vec!["psyche-interraction-claude-briefness"]
            ),
        ]
    );
    assert_eq!(
        universal_role_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["general-instructions", "tenets"]
    );

    let active_roles: BTreeMap<&str, _> = active_outputs
        .payload()
        .iter()
        .filter_map(|output| match output {
            skills::schema::assembly::ActiveOutput::Role(role) => {
                Some((role.output_identifier.as_ref(), role))
            }
            skills::schema::assembly::ActiveOutput::Skill(_) => None,
        })
        .collect();
    let expected_roles: &[(&str, &str, &[&str])] = &[
        (
            "generalist",
            "role-generalist",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
                "non-ideal-registry",
            ],
        ),
        (
            "intent-recorder",
            "role-intent-recorder",
            &["spirit-submission"],
        ),
        (
            "intent-translator",
            "role-intent-translator",
            &["edit-coordination-core", "bead-weaver"],
        ),
        ("scout", "role-scout", &["edit-coordination-core"]),
        (
            "repo-scaffolder",
            "role-repo-scaffolder",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "repo-scaffold-core",
                "code-implementation-core",
                "non-ideal-registry",
            ],
        ),
        (
            "general-code-implementer",
            "role-general-code-implementer",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
                "non-ideal-registry",
            ],
        ),
        (
            "operating-system-implementer",
            "role-operating-system-implementer",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "code-implementation-core",
                "nix-core",
                "operating-system-operations",
                "nixos-vm-testing",
                "non-ideal-registry",
            ],
        ),
        (
            "rust-auditor",
            "role-rust-auditor",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "rust-core",
                "architectural-truth-tests",
                "non-ideal-registry",
            ],
        ),
        (
            "nix-auditor",
            "role-nix-auditor",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "nix-core",
                "nixos-vm-testing",
                "non-ideal-registry",
            ],
        ),
        (
            "skill-maintainer",
            "role-skill-maintainer",
            &["skill-designing"],
        ),
        ("trivial-task", "role-trivial-task", &[]),
        (
            "intent-curator",
            "role-intent-curator",
            &["edit-coordination-core", "editing-closeout", "intent-core"],
        ),
        (
            "repository-closeout",
            "role-repository-closeout",
            &[
                "edit-coordination-core",
                "editing-closeout",
                "repo-operation-core",
            ],
        ),
        (
            "tracker-weaver",
            "role-tracker-weaver",
            &["edit-coordination-core", "editing-closeout", "bead-weaver"],
        ),
    ];

    assert_eq!(active_roles.len(), expected_roles.len());
    for deprecated_role in ["intent-maintainer", "repo-operator", "weave-operator"] {
        assert!(
            !active_roles.contains_key(deprecated_role),
            "{deprecated_role} active role appellation stays retired"
        );
    }
    for (output_identifier, module_identifier, included_modules) in expected_roles {
        let role = active_roles
            .get(output_identifier)
            .unwrap_or_else(|| panic!("{output_identifier} role output modeled"));
        assert_eq!(role.module_identifier.as_ref(), *module_identifier);
        assert_eq!(
            role.included_modules
                .payload()
                .iter()
                .map(|module| module.as_ref())
                .collect::<Vec<_>>(),
            *included_modules
        );
        let expected_surfaces: &[RoleTargetSurface] = &[
            RoleTargetSurface::ClaudeAgent,
            RoleTargetSurface::CodexAgent,
            RoleTargetSurface::PiAgent,
        ];
        assert_eq!(role.role_target_surfaces.payload(), expected_surfaces);
        assert!(dependency_modules.contains(module_identifier));
        assert_eq!(
            module_kinds.get(module_identifier),
            Some(&ModuleKind::RoleSource),
            "{module_identifier} is a role source module"
        );
        for included_module in *included_modules {
            assert!(dependency_modules.contains(included_module));
        }
    }
    let indexed_role_sources: BTreeSet<&str> = module_dependencies
        .payload()
        .iter()
        .filter(|dependency| dependency.module_kind == ModuleKind::RoleSource)
        .map(|dependency| dependency.module_identifier.as_ref())
        .collect();
    let active_role_sources: BTreeSet<&str> = active_roles
        .values()
        .map(|role| role.module_identifier.as_ref())
        .collect();
    assert_eq!(
        indexed_role_sources, active_role_sources,
        "only active role sources remain indexed"
    );
}

#[test]
fn human_interaction_and_context_maintenance_are_removed_while_handover_and_deep_remain() {
    const HANDOVER: &str = "Write the handover in the response.\n\
## Psyche vision\n\
Psyche vision is the psyche's aims, values, priorities, and desired outcome for the work.\n\
Preserve every non-repetitive, load-bearing psyche statement in recognizable language and full resolution.\n\
## References\n\
Include only the references needed to resume the thread.\n";

    let manifest_text = include_str!("../manifests/active-outputs.nota");
    let index_text = include_str!("../manifests/module-dependencies.nota");

    assert!(!manifest_text.contains("human-interaction"));
    assert!(!index_text.contains("human-interaction"));
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/human-interaction.md")
            .exists(),
        "human-interaction source module is deleted, not archived"
    );
    assert!(!manifest_text.contains("(Skill (context-maintenance "));
    assert!(!index_text.contains("(context-maintenance "));
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/context-maintenance.md")
            .exists(),
        "context-maintenance source module is deleted, not archived"
    );

    let module_dependencies = NotaSource::new(index_text)
        .parse::<ModuleDependencies>()
        .expect("module dependency index parses");
    let management = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "management")
        .expect("management dependency indexed");
    assert_eq!(
        management
            .dependency_modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["tenets"]
    );
    assert!(manifest_text.contains("(Skill (context-handover context-handover Meta Mechanism"));
    assert!(
        !management
            .dependency_modules
            .payload()
            .iter()
            .any(|module| module.as_ref() == "context-handover")
    );
    let context_maintenance_deep = module_dependencies
        .payload()
        .iter()
        .find(|dependency| dependency.module_identifier.as_ref() == "context-maintenance-deep")
        .expect("context-maintenance-deep dependency indexed");
    assert!(
        context_maintenance_deep
            .dependency_modules
            .payload()
            .is_empty(),
        "context-maintenance-deep does not depend on deleted context-maintenance"
    );
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/context-maintenance-deep.md")
            .exists(),
        "context-maintenance-deep source remains"
    );
    assert_eq!(
        include_str!("../skills/context-handover.md"),
        HANDOVER,
        "context-handover source is the approved exact handover guidance"
    );

    let fixture = Fixture::new();
    fixture.write_workspace_file(".agents/skills/context-maintenance/SKILL.md", "stale\n");
    fixture.write_workspace_file(".claude/skills/context-maintenance/SKILL.md", "stale\n");
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current source prunes removed context-maintenance outputs");
    for stale_path in [
        ".agents/skills/context-maintenance/SKILL.md",
        ".claude/skills/context-maintenance/SKILL.md",
    ] {
        assert!(
            !fixture.workspace.path().join(stale_path).exists(),
            "{stale_path} is pruned"
        );
    }
    for path in [
        ".agents/skills/context-handover/SKILL.md",
        ".claude/skills/context-handover/SKILL.md",
        ".agents/skills/context-maintenance-deep/SKILL.md",
        ".claude/skills/context-maintenance-deep/SKILL.md",
    ] {
        assert!(
            fixture.workspace.path().join(path).exists(),
            "{path} remains"
        );
    }
    for path in [
        ".agents/skills/context-handover/SKILL.md",
        ".claude/skills/context-handover/SKILL.md",
    ] {
        assert_eq!(
            fixture.read_workspace_file(path),
            format!(
                "---\nname: context-handover\ndescription: 'Use when carrying the psyche''s vision into another session.'\n---\n\n{HANDOVER}"
            ),
            "{path} is the approved exact handover guidance"
        );
    }
}

#[test]
fn kameo_skill_is_removed_from_source_and_generated_surfaces() {
    for source in [
        include_str!("../manifests/active-outputs.nota"),
        include_str!("../manifests/module-dependencies.nota"),
    ] {
        assert!(!source.contains("kameo"));
    }
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/kameo.md")
            .exists()
    );

    let fixture = Fixture::new();
    for stale_path in [
        ".agents/skills/kameo/SKILL.md",
        ".claude/skills/kameo/SKILL.md",
    ] {
        fixture.write_workspace_file(stale_path, "stale\n");
    }
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current source prunes removed Kameo outputs");
    for stale_path in [
        ".agents/skills/kameo/SKILL.md",
        ".claude/skills/kameo/SKILL.md",
    ] {
        assert!(!fixture.workspace.path().join(stale_path).exists());
    }
}

#[test]
fn repository_visibility_doctrine_defaults_public_without_weakening_privacy() {
    let publication = include_str!("../skills/repository-publication.md");
    let management = include_str!("../skills/repository-management.md");
    assert!(publication.contains("Do not publish private material"));
    assert!(management.contains("public visibility as default"));
}

#[test]
fn skill_designing_replaces_skill_editor_and_composes_once_in_skill_maintainer() {
    const SKILL_BODY: &str = "Write skills with brutal minimalism.\n\
Descriptions say when the skill applies.\n\
State unusual, impactful instructions once and directly.\n\
Flag anything noisy, unclear, unsafe, or misplaced. Explain what each proposed change preserves, changes, or removes.\n";
    const ROLE_BODY: &str = "Before changing a skill, show the psyche the exact diff and get approval. A proposal is not approval.\n\
Change only the approved diff.\n\
Generate, verify, and report.\n";
    const RETIRED_ROLE_LINES: [&str; 7] = [
        "Keep only unusual guidance that changes agent behavior.",
        "Keep distinct instructions separate.",
        "Shorten skills by deleting weak guidance, not by compressing it.",
        "Make a skill only when the same guidance is needed across repositories.",
        "Reject operational guidance and repository-specific facts.",
        "Remove anything repeated, unverified, outdated, or already done without the skill.",
        "Use headings only when they aid navigation; never repeat the skill name.",
    ];

    assert_eq!(include_str!("../skills/skill-designing.md"), SKILL_BODY);
    assert_eq!(include_str!("../roles/skill-maintainer.md"), ROLE_BODY);
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/skill-editor.md")
            .exists()
    );
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("roles/skill-editor.md")
            .exists()
    );

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("approved skill-designing and skill-maintainer surfaces generate");
    for path in [
        ".agents/skills/skill-designing/SKILL.md",
        ".claude/skills/skill-designing/SKILL.md",
    ] {
        assert_eq!(
            fixture.read_workspace_file(path),
            format!(
                "---\nname: skill-designing\ndescription: 'Use when designing a skill.'\n---\n\n{SKILL_BODY}"
            ),
            "{path} is the exact active skill-designing surface"
        );
    }
    for path in [
        ".pi/agents/skill-maintainer.md",
        ".claude/agents/skill-maintainer.md",
        ".codex/agents/skill-maintainer.toml",
    ] {
        let output = fixture.read_workspace_file(path).replace("\\n", "\n");
        assert!(output.contains(ROLE_BODY), "{path} receives its role body");
        assert_eq!(
            output.matches(SKILL_BODY).count(),
            1,
            "{path} composes skill-designing once"
        );
        assert!(
            !output.contains("## Allowed child-role roster"),
            "{path} is a leaf"
        );
        for retired_line in RETIRED_ROLE_LINES {
            assert!(
                !output.contains(retired_line),
                "{path} excludes retired skill-editor role guidance: {retired_line}"
            );
        }
    }
    for retired_path in [
        ".agents/skills/skill-editor/SKILL.md",
        ".claude/skills/skill-editor/SKILL.md",
        ".pi/agents/skill-editor.md",
        ".claude/agents/skill-editor.md",
        ".codex/agents/skill-editor.toml",
    ] {
        assert!(!fixture.workspace.path().join(retired_path).exists());
    }
}

#[test]
fn manager_surfaces_are_retired_while_historical_modules_remain_inactive() {
    const MANAGEMENT: &str = "Delegate assigned work to child workers.\n\
Poll until they finish.\n\
Keep observations, hypotheses, and unknowns distinct.\n\
Return unresolved authority, safety, privacy, or scope to the caller.\n\
Return a concise synthesis to the caller.\n";
    const TENETS: &str = "## Central\nNever pretend to know what you don't know; admit you don't know.\n## Evidence\nKeep observations, hypotheses, and unknowns separate.\nKeep unknown causes unknown.\nSeek disconfirming evidence.\nDo not seed audits with suspected conclusions.\nWeigh evidence by origin, not repetition.\n";

    assert_eq!(include_str!("../skills/management.md"), MANAGEMENT);
    assert_eq!(include_str!("../skills/tenets.md"), TENETS);
    for path in [
        "roles/manager.md",
        "skills/manager.md",
        "manifests/manager-packet-composition.nota",
    ] {
        assert!(
            !Path::new(env!("CARGO_MANIFEST_DIR")).join(path).exists(),
            "{path} is retired"
        );
    }

    let index = include_str!("../manifests/module-dependencies.nota");
    for historical_module in [
        "manager-boundary",
        "manager-intent-classification",
        "manager-safeguards",
        "manager-dispatch",
        "manager-liveness",
        "manager-decisions",
        "manager-communication",
        "manager-synthesis",
        "psyche-facing-commitments",
    ] {
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("skills/{historical_module}.md"))
                .is_file(),
            "{historical_module} source remains available"
        );
        assert!(
            !index.contains(&format!("({historical_module} ")),
            "{historical_module} remains outside active composition"
        );
    }

    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("retired Manager source generates the remaining surfaces");
    for path in [
        ".agents/skills/manager/SKILL.md",
        ".claude/skills/manager/SKILL.md",
        ".claude/agents/manager.md",
        ".codex/agents/manager.toml",
        ".pi/agents/manager.md",
    ] {
        assert!(
            !fixture.workspace.path().join(path).exists(),
            "{path} is absent"
        );
    }
    for path in [
        ".agents/skills/management/SKILL.md",
        ".claude/skills/management/SKILL.md",
    ] {
        let output = fixture.read_workspace_file(path);
        assert!(
            output.contains("description: 'Use when coordinating delegated work for a caller.'")
        );
        assert_eq!(output.matches(TENETS).count(), 1, "{path} has tenets once");
        assert_eq!(
            output.matches(MANAGEMENT).count(),
            1,
            "{path} has management once"
        );
    }

    let active_outputs = NotaSource::new(include_str!("../manifests/active-outputs.nota"))
        .parse::<ActiveOutputs>()
        .expect("active outputs parse");
    for role in active_outputs
        .payload()
        .iter()
        .filter_map(|output| match output {
            skills::schema::assembly::ActiveOutput::Role(role) => Some(role),
            skills::schema::assembly::ActiveOutput::Skill(_) => None,
        })
    {
        for surface in role.role_target_surfaces.payload() {
            let path = match surface {
                RoleTargetSurface::ClaudeAgent => {
                    format!(".claude/agents/{}.md", role.output_identifier.as_ref())
                }
                RoleTargetSurface::CodexAgent => {
                    format!(".codex/agents/{}.toml", role.output_identifier.as_ref())
                }
                RoleTargetSurface::PiAgent => {
                    format!(".pi/agents/{}.md", role.output_identifier.as_ref())
                }
            };
            let packet = fixture.read_workspace_file(&path).replace("\\n", "\n");
            assert_eq!(
                packet
                    .matches("Never pretend to know what you don't know; admit you don't know.")
                    .count(),
                1,
                "{path} has tenets once"
            );
        }
    }
}

#[test]
fn management_is_caller_scoped_and_has_no_psyche_interaction_doctrine() {
    let management = include_str!("../skills/management.md");
    for required in [
        "Delegate assigned work to child workers.",
        "Poll until they finish.",
        "Return unresolved authority, safety, privacy, or scope to the caller.",
        "Return a concise synthesis to the caller.",
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
        ".pi/agents/skill-maintainer.md",
        ".claude/agents/skill-maintainer.md",
        ".codex/agents/skill-maintainer.toml",
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
        assert_eq!(
            output
                .matches("Never pretend to know what you don't know; admit you don't know.")
                .count(),
            1,
            "psyche interaction direct loading includes tenets once"
        );
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
fn generated_recorder_packets_preserve_matter_not_intent_classification() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("recorder packets generate");
    for path in [
        ".pi/agents/intent-recorder.md",
        ".codex/agents/intent-recorder.toml",
        ".claude/agents/intent-recorder.md",
    ] {
        let packet = fixture.read_workspace_file(path).replace("\\n", "\n");
        assert!(packet.contains("Return matter to the caller; do not submit it."));
    }
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
fn role_generation_expands_dependencies_in_order_and_writes_harness_paths() {
    let fixture = Fixture::new();
    fixture.write_role_generation_sources();
    fixture.write_source_file(
        "roles/worker.md",
        "# Role - worker\n\n## Contract\n\nGenerated-file notices stay out.\n",
    );
    fixture.write_source_file(
        "skills/shared.md",
        "# Module - shared\n\n## Shared Rule\n\nDependency first.\n",
    );
    fixture.write_source_file(
        "skills/feature.md",
        "# Module - feature\n\n## Feature Rule\n\nDependent second.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("role generation succeeds");

    let claude = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude.starts_with(
        "---\nname: worker\ndescription: 'Worker role.'\nmodel: claude-test\neffort: high\n---\n\n"
    ));
    assert!(claude.contains("# worker"));
    assert!(claude.contains("## shared"));
    assert!(claude.contains("## feature"));
    assert!(!claude.contains("Role - worker"));
    assert!(!claude.contains("Module - shared"));
    assert!(claude.find("# worker") < claude.find("## shared"));
    assert!(claude.find("## shared") < claude.find("## feature"));
    assert_eq!(claude.matches("## shared").count(), 1);
    assert_eq!(claude.matches("Dependency first.").count(), 1);
    assert!(!claude.contains("@generated"));
    assert!(!claude.contains("generated by"));

    let codex = fixture.read_workspace_file(".codex/agents/worker.toml");
    assert!(codex.contains("name = \"worker\""));
    assert!(codex.contains("description = \"Worker role.\""));
    assert!(codex.contains("model = \"gpt-test\""));
    assert!(codex.contains("model_reasoning_effort = \"high\""));
    assert!(codex.contains("developer_instructions = \"# worker"));
    assert!(codex.contains("## shared"));
    assert!(codex.contains("## feature"));
    assert!(!claude.contains("Skill-read de-duplication"));

    let pi = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi.starts_with("---\nname: worker\ndescription: 'Worker role.'\nmodel: 'openai-codex/gpt-test'\nthinking: high\nprojectRoleIdentity: worker\nprojectRoleDispatchKind: leaf\n---\n\n"));
    assert!(!pi.contains("Skill-read de-duplication"));

    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.nota");
    assert!(inventory.contains(".claude/agents/worker.md"));
    assert!(inventory.contains(".codex/agents/worker.toml"));
    assert!(inventory.contains(".pi/agents/worker.md"));
}

#[test]
fn generation_rejects_configured_execution_limit_fields_in_agent_packets() {
    let fixture = Fixture::new();
    fixture.write_role_generation_sources();
    fixture.write_source_file(
        "roles/worker.md",
        "# Role - worker\n\n## Contract\n\ntimeoutMs: 1\n",
    );
    fixture.write_source_file(
        "skills/shared.md",
        "# Module - shared\n\n## Shared Rule\n\nShared rule.\n",
    );
    fixture.write_source_file(
        "skills/feature.md",
        "# Module - feature\n\n## Feature Rule\n\nFeature rule.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("execution-limit field rejects agent packet generation");

    assert!(matches!(
        error,
        Error::GeneratedAgentExecutionLimit { field_name, .. } if field_name == "timeoutMs"
    ));
}

#[test]
fn visualization_reports_role_kinds_composition_and_virtual_output_sizes() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill])) (Role (planner planner [shared] [Planner role.] [PiAgent])) (Role (worker worker [] [Worker role.] [PiAgent]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example skills/example.md [] RuntimeSkill) (shared skills/shared.md [] RoleComposition) (planner roles/planner.md [] RoleSource) (worker roles/worker.md [] RoleSource)]\n",
    );
    fixture.write_role_metadata(&["planner", "worker"]);
    fixture.write_source_file(
        "manifests/nested-role-relations.nota",
        "[(planner [(PiAgent gpt-test Medium)] [worker])]\n",
    );
    fixture.write_source_file("skills/example.md", "# Skill - example\n\nExample.\n");
    fixture.write_source_file("skills/shared.md", "# Module - shared\n\nShared.\n");
    for role in ["planner", "worker"] {
        fixture.write_source_file(
            &format!("roles/{role}.md"),
            &format!("# Role - {role}\n\n## Contract\n\nRole body.\n"),
        );
    }

    let visualization = fixture.visualize().expect("visualization succeeds");
    let roles = visualization.role_visualizations.payload();
    let planner = roles
        .iter()
        .find(|role| role.output_identifier.as_ref() == "planner")
        .expect("nested role visualization exists");
    assert_eq!(
        planner.role_generation_kind,
        RoleGenerationKind::DispatchableNestedRole
    );
    let planner_packet = &planner.role_packet_compositions.payload()[0];
    assert_eq!(
        planner_packet
            .modules
            .payload()
            .iter()
            .map(|module| module.as_ref())
            .collect::<Vec<_>>(),
        ["roles/planner.md", "skills/shared.md"]
    );
    assert_eq!(
        planner_packet
            .dispatchable_roles
            .payload()
            .iter()
            .map(|role| role.as_ref())
            .collect::<Vec<_>>(),
        ["worker"]
    );
    assert_eq!(
        roles
            .iter()
            .find(|role| role.output_identifier.as_ref() == "worker")
            .expect("leaf role visualization exists")
            .role_generation_kind,
        RoleGenerationKind::DispatchableLeafRole
    );

    let generated = fixture
        .generate(GenerationMode::Write)
        .expect("generation succeeds");
    let generated_sizes = generated
        .payload()
        .payload()
        .iter()
        .map(|file| (file.output_path.as_ref(), file.byte_count.payload()))
        .collect::<BTreeMap<_, _>>();
    let visualized_sizes = visualization
        .generated_output_visualizations
        .payload()
        .iter()
        .map(|file| (file.output_path.as_ref(), file.byte_count.payload()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(visualized_sizes, generated_sizes);
    for output in visualization.generated_output_visualizations.payload() {
        let rendered = fixture.read_workspace_file(output.output_path.as_ref());
        assert_eq!(
            *output.line_count.payload(),
            rendered.matches('\n').count() as u64
        );
    }
}

#[test]
fn pi_project_role_frontmatter_matches_extension_parser_contract_fixture() {
    let fixture = Fixture::new();
    fixture.write_project_role_contract_sources();
    fixture
        .generate(GenerationMode::Write)
        .expect("project-role contract fixture generates");

    let generated = fixture.read_workspace_file(".pi/agents/planner.md");
    let contract_fixture = include_str!("fixtures/pi-project-role-frontmatter-contract.md");
    assert_eq!(
        frontmatter_block(&generated),
        frontmatter_block(contract_fixture)
    );
    assert_eq!(
        project_role_contract(&generated, "planner"),
        ParsedProjectRoleContract {
            project_role_identity: "planner".to_owned(),
            project_role_dispatch_kind: "nested".to_owned(),
            allowed_child_role_names: vec!["reader".to_owned(), "writer".to_owned()],
        }
    );
    for leaf in ["reader", "writer"] {
        let packet = fixture.read_workspace_file(&format!(".pi/agents/{leaf}.md"));
        assert_eq!(
            project_role_contract(&packet, leaf),
            ParsedProjectRoleContract {
                project_role_identity: leaf.to_owned(),
                project_role_dispatch_kind: "leaf".to_owned(),
                allowed_child_role_names: Vec::new(),
            }
        );
    }
}

#[test]
fn nested_role_schema_preserves_child_rosters_without_model_upgrades() {
    let relations = NotaSource::new(include_str!("../manifests/nested-role-relations.nota"))
        .parse::<NestedRoleRelations>()
        .expect("nested role relations parse");
    let observed: BTreeMap<_, _> = relations
        .payload()
        .iter()
        .map(|relation| {
            (
                relation.output_identifier.as_ref(),
                relation
                    .allowed_leaf_roles
                    .payload()
                    .iter()
                    .map(|role| role.as_ref())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        observed,
        BTreeMap::from([
            (
                "generalist",
                vec![
                    "scout",
                    "repo-scaffolder",
                    "general-code-implementer",
                    "rust-auditor",
                    "nix-auditor",
                    "repository-closeout",
                    "tracker-weaver",
                ],
            ),
            (
                "operating-system-implementer",
                vec![
                    "scout",
                    "general-code-implementer",
                    "rust-auditor",
                    "nix-auditor",
                    "repository-closeout",
                ],
            ),
        ])
    );
    for relation in relations.payload() {
        for minimum in relation.nested_role_minimum_models.payload() {
            assert_eq!(minimum.effort_level, EffortLevel::Medium);
            match minimum.role_target_surface {
                RoleTargetSurface::ClaudeAgent => {
                    assert_eq!(minimum.model_identifier.as_ref(), "claude-sonnet-5")
                }
                RoleTargetSurface::CodexAgent | RoleTargetSurface::PiAgent => {
                    assert_eq!(minimum.model_identifier.as_ref(), "gpt-5.6-luna")
                }
            }
        }
    }
    let active_outputs = NotaSource::new(include_str!("../manifests/active-outputs.nota"))
        .parse::<ActiveOutputs>()
        .expect("active outputs parse");
    assert!(active_outputs.payload().iter().all(|output| {
        match output {
            skills::schema::assembly::ActiveOutput::Role(role) => !role
                .output_identifier
                .as_ref()
                .starts_with("crucial-greenfield-"),
            skills::schema::assembly::ActiveOutput::Skill(_) => true,
        }
    }));
}
#[test]
fn generated_packets_keep_rosters_and_exclude_disallowed_worker_models() {
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("current manifests generate");

    let roster = |path: &str| {
        let packet = fixture.read_workspace_file(path).replace("\\n", "\n");
        let roster_body = packet
            .split("## Allowed child-role roster")
            .nth(1)
            .expect("generated roster heading exists");
        roster_body
            .split("## optional skills")
            .next()
            .expect("role roster has content")
            .lines()
            .filter_map(|line| line.strip_prefix("- `")?.split('`').next())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        roster(".pi/agents/generalist.md"),
        [
            "scout",
            "repo-scaffolder",
            "general-code-implementer",
            "rust-auditor",
            "nix-auditor",
            "repository-closeout",
            "tracker-weaver",
        ]
    );
    assert_eq!(
        roster(".pi/agents/operating-system-implementer.md"),
        [
            "scout",
            "general-code-implementer",
            "rust-auditor",
            "nix-auditor",
            "repository-closeout",
        ]
    );
    for role in [
        "generalist",
        "intent-translator",
        "operating-system-implementer",
        "skill-maintainer",
        "intent-curator",
    ] {
        assert!(
            fixture
                .read_workspace_file(&format!(".pi/agents/{role}.md"))
                .contains("model: 'openai-codex/gpt-5.6-terra'\nthinking: xhigh"),
            "{role} has the Terra xhigh Pi assignment"
        );
    }
    let active_roles = [
        "generalist",
        "intent-recorder",
        "intent-translator",
        "scout",
        "repo-scaffolder",
        "general-code-implementer",
        "operating-system-implementer",
        "rust-auditor",
        "nix-auditor",
        "skill-maintainer",
        "trivial-task",
        "intent-curator",
        "repository-closeout",
        "tracker-weaver",
    ];
    for role in active_roles {
        let pi = fixture.read_workspace_file(&format!(".pi/agents/{role}.md"));
        let codex = fixture.read_workspace_file(&format!(".codex/agents/{role}.toml"));
        assert!(!pi.contains("gpt-5.6-sol"), "{role} has no Pi Sol model");
        assert!(
            !codex.contains("model = \"gpt-5.6-sol\""),
            "{role} has no Codex Sol model"
        );
        let claude = fixture.read_workspace_file(&format!(".claude/agents/{role}.md"));
        assert!(
            !claude.contains("model: fable-5"),
            "{role} has no Claude Fable model"
        );
    }
    let assignment_source = include_str!("../manifests/role-model-assignments.nota");
    let trivial_assignment = assignment_source
        .lines()
        .find(|line| line.contains("trivial-task"))
        .expect("trivial-task has exactly one role-model assignment");
    assert_eq!(
        trivial_assignment.trim(),
        "(Profile (trivial-task minimalFastEconomical))"
    );
    assert!(!include_str!("../roles/trivial-task.md").contains("gpt-5.6-luna"));
    assert!(!include_str!("../roles/trivial-task.md").contains("claude-sonnet-5"));
    for path in [
        ".pi/agents/trivial-task.md",
        ".codex/agents/trivial-task.toml",
        ".claude/agents/trivial-task.md",
    ] {
        let output = fixture.read_workspace_file(path);
        assert!(
            !output.contains("minimalFastEconomical"),
            "{path} resolves the semantic profile before rendering"
        );
    }
    assert!(
        fixture
            .read_workspace_file(".pi/agents/trivial-task.md")
            .contains("model: 'openai-codex/gpt-5.6-luna'\nthinking: medium")
    );
    assert!(
        fixture
            .read_workspace_file(".codex/agents/trivial-task.toml")
            .contains("model = \"gpt-5.6-luna\"\nmodel_reasoning_effort = \"medium\"")
    );
    assert!(
        fixture
            .read_workspace_file(".claude/agents/trivial-task.md")
            .contains("model: claude-sonnet-5\neffort: medium")
    );

    let claude_briefness = include_str!("../skills/psyche-interraction-claude-briefness.md").trim();
    assert!(
        fixture
            .read_workspace_file(".claude/skills/psyche-interraction/SKILL.md")
            .contains(claude_briefness)
    );
    for path in [
        ".agents/skills/management/SKILL.md",
        ".agents/skills/psyche-interraction/SKILL.md",
        ".claude/agents/generalist.md",
        ".claude/agents/intent-recorder.md",
        ".claude/agents/intent-translator.md",
        ".claude/agents/scout.md",
        ".claude/agents/repo-scaffolder.md",
        ".claude/agents/general-code-implementer.md",
        ".claude/agents/operating-system-implementer.md",
        ".claude/agents/rust-auditor.md",
        ".claude/agents/nix-auditor.md",
        ".claude/agents/skill-maintainer.md",
        ".claude/agents/trivial-task.md",
        ".claude/agents/intent-curator.md",
        ".claude/agents/repository-closeout.md",
        ".claude/agents/tracker-weaver.md",
    ] {
        assert!(!fixture.read_workspace_file(path).contains(claude_briefness));
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
fn cross_session_intercom_training_reaches_every_nested_role() {
    const RULES: [&str; 3] = [
        "Cross-session intercom is prohibited unless the target explicitly invited contact or the psyche explicitly authorized that exact contact.",
        "Apparent status, availability, or topic relevance never grants permission.",
        "Parent-child communication is exempt.",
    ];

    let general = include_str!("../skills/general-instructions.md");
    for rule in RULES {
        assert!(general.contains(rule), "shared source contains {rule}");
    }

    let nested_role_relations =
        NotaSource::new(include_str!("../manifests/nested-role-relations.nota"))
            .parse::<NestedRoleRelations>()
            .expect("nested-role relations parse");
    let dispatch_capable_roles: BTreeSet<&str> = nested_role_relations
        .payload()
        .iter()
        .map(|relation| relation.output_identifier.as_ref())
        .collect();

    let active_outputs = NotaSource::new(include_str!("../manifests/active-outputs.nota"))
        .parse::<ActiveOutputs>()
        .expect("active outputs parse");
    let fixture = Fixture::new();
    fixture
        .generate_from_repo(GenerationMode::Write)
        .expect("active role packets generate");

    for role in active_outputs
        .payload()
        .iter()
        .filter_map(|output| match output {
            skills::schema::assembly::ActiveOutput::Role(role)
                if dispatch_capable_roles.contains(role.output_identifier.as_ref()) =>
            {
                Some(role)
            }
            skills::schema::assembly::ActiveOutput::Skill(_)
            | skills::schema::assembly::ActiveOutput::Role(_) => None,
        })
    {
        for surface in role.role_target_surfaces.payload() {
            let path = match surface {
                RoleTargetSurface::ClaudeAgent => {
                    format!(".claude/agents/{}.md", role.output_identifier.as_ref())
                }
                RoleTargetSurface::CodexAgent => {
                    format!(".codex/agents/{}.toml", role.output_identifier.as_ref())
                }
                RoleTargetSurface::PiAgent => {
                    format!(".pi/agents/{}.md", role.output_identifier.as_ref())
                }
            };
            let packet = fixture.read_workspace_file(&path).replace("\\n", "\n");
            for rule in RULES {
                assert!(packet.contains(rule), "{path} contains {rule}");
            }
        }
    }
}

#[test]
fn nested_model_resolution_uses_strongest_assignment_and_ordinary_wins_ties() {
    let tie = Fixture::new();
    tie.write_model_resolution_sources("Medium");
    tie.generate(GenerationMode::Write)
        .expect("equal-strength ordinary assignments generate");
    assert!(
        tie.read_workspace_file(".pi/agents/parent.md")
            .contains("model: 'ordinary-provider/gpt-ordinary'\nthinking: medium")
    );
    assert!(
        tie.read_workspace_file(".claude/agents/parent.md")
            .contains("model: claude-ordinary\neffort: medium")
    );

    let stronger_floor = Fixture::new();
    stronger_floor.write_model_resolution_sources("High");
    stronger_floor
        .generate(GenerationMode::Write)
        .expect("stronger minimum assignments generate");
    assert!(
        stronger_floor
            .read_workspace_file(".pi/agents/parent.md")
            .contains("model: 'openai-codex/gpt-5.6-sol'\nthinking: high")
    );
    assert!(
        stronger_floor
            .read_workspace_file(".codex/agents/parent.toml")
            .contains("model = \"gpt-5.6-sol\"\nmodel_reasoning_effort = \"high\"")
    );
    assert!(
        stronger_floor
            .read_workspace_file(".claude/agents/parent.md")
            .contains("model: fable-5\neffort: high")
    );
}

#[test]
fn nested_model_resolution_uses_typed_cross_model_strength_not_effort_rank() {
    let fixture = Fixture::new();
    fixture.write_cross_model_floor_sources();
    fixture
        .generate(GenerationMode::Write)
        .expect("cross-model floors generate");

    assert!(
        fixture
            .read_workspace_file(".pi/agents/parent.md")
            .contains("model: 'openai-codex/gpt-5.6-sol'\nthinking: medium")
    );
    assert!(
        fixture
            .read_workspace_file(".codex/agents/parent.toml")
            .contains("model = \"gpt-5.6-sol\"\nmodel_reasoning_effort = \"medium\"")
    );
    assert!(
        fixture
            .read_workspace_file(".claude/agents/parent.md")
            .contains("model: fable-5\neffort: medium")
    );
}

#[test]
fn nested_role_validation_rejects_child_and_recursion_inconsistencies() {
    let missing = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [])]",
    );
    assert!(matches!(missing, Error::MissingNestedRoleChild { .. }));
    let duplicate_relation = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child]) (parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        duplicate_relation,
        Error::DuplicateNestedRoleRelation { .. }
    ));
    let inactive_parent =
        nested_relation_error("[(inactive [(ClaudeAgent claude-test Medium)] [child])]");
    assert!(matches!(inactive_parent, Error::InactiveNestedRole { .. }));
    let duplicate_child = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child child])]",
    );
    assert!(matches!(
        duplicate_child,
        Error::DuplicateNestedRoleChild { .. }
    ));
    let inactive_child = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [inactive])]",
    );
    assert!(matches!(
        inactive_child,
        Error::InactiveNestedRoleChild { .. }
    ));
    let incompatible_child = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [claude-child])]",
    );
    assert!(matches!(
        incompatible_child,
        Error::TargetIncompatibleNestedRoleChild { .. }
    ));
    let self_edge = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [parent])]",
    );
    assert!(matches!(self_edge, Error::NestedRoleSelfEdge { .. }));
    let nested_edge = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [nested-two]) (nested-two [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        nested_edge,
        Error::NestedRoleChildCannotBeNested { .. }
    ));
}

#[test]
fn nested_role_validation_rejects_minimum_model_target_inconsistencies() {
    let missing = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        missing,
        Error::MissingNestedRoleMinimumModel { .. }
    ));
    let duplicate = nested_relation_error(
        "[(parent [(ClaudeAgent claude-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        duplicate,
        Error::DuplicateNestedRoleMinimumModel { .. }
    ));
    let inactive_target = nested_relation_error(
        "[(claude-child [(ClaudeAgent claude-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        inactive_target,
        Error::NestedRoleMinimumForInactiveTarget { .. }
    ));
    let wrong_family = nested_relation_error(
        "[(parent [(ClaudeAgent gpt-test Medium) (CodexAgent gpt-test Medium) (PiAgent gpt-test Medium)] [child])]",
    );
    assert!(matches!(
        wrong_family,
        Error::NestedRoleMinimumModelFamilyMismatch { .. }
    ));
}

#[test]
fn role_profiles_and_optional_skills_render_without_preloading_skill_bodies() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill ClaudeSkill])) (Role (worker worker [] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example skills/example.md [] RuntimeSkill) (worker roles/worker.md [] RoleSource)]\n",
    );
    fixture.write_source_file(
        "skills/example.md",
        "# Skill - example\n\n## Example Rule\n\nThis body must not be preloaded.\n",
    );
    fixture.write_source_file(
        "roles/worker.md",
        "# Role - worker\n\n## Contract\n\nRole body.\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [example])]\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("profiled role with optional skill generates");

    let claude = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude.contains("model: claude-test\neffort: high"));
    assert!(claude.contains("## optional skills"));
    assert!(claude.contains("- `example`"));
    assert!(!claude.contains("This body must not be preloaded."));

    let pi = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi.contains("model: 'openai-codex/gpt-test'\nthinking: high\nprojectRoleIdentity: worker\nprojectRoleDispatchKind: leaf\nskills: example"));
    assert!(pi.contains("## optional skills"));
    assert!(!pi.contains("This body must not be preloaded."));

    let codex = fixture.read_workspace_file(".codex/agents/worker.toml");
    assert!(codex.contains("model = \"gpt-test\""));
    assert!(codex.contains("model_reasoning_effort = \"high\""));
    assert!(codex.contains("## optional skills"));
    assert!(!codex.contains("This body must not be preloaded."));
}

#[test]
fn role_model_assignments_reject_missing_duplicate_stale_and_duplicate_catalog_entries() {
    let missing = Fixture::new();
    missing.write_role_generation_sources();
    missing.write_source_file("manifests/role-model-assignments.nota", "[]\n");
    let error = missing
        .generate(GenerationMode::Write)
        .expect_err("missing assignment fails");
    assert!(
        matches!(error, Error::MissingRoleModelAssignment { .. }),
        "{error:?}"
    );

    let duplicate = Fixture::new();
    duplicate.write_role_generation_sources();
    duplicate.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(Direct (worker (gpt-test High) (claude-test High))) (Direct (worker (gpt-test High) (claude-test High)))]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate assignment fails");
    assert!(
        matches!(error, Error::DuplicateRoleModelAssignment { .. }),
        "{error:?}"
    );

    let stale = Fixture::new();
    stale.write_role_generation_sources();
    stale.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(Direct (worker (gpt-test High) (claude-test High))) (Direct (retired-role (gpt-test High) (claude-test High)))]\n",
    );
    let error = stale
        .generate(GenerationMode::Write)
        .expect_err("stale assignment fails");
    assert!(
        matches!(error, Error::StaleRoleModelAssignment { .. }),
        "{error:?}"
    );

    let duplicate_catalog = Fixture::new();
    duplicate_catalog.write_role_generation_sources();
    duplicate_catalog.write_source_file(
        "manifests/model-catalog.nota",
        "[(ChatGpt (gpt-test openai-codex [(High 30)])) (ChatGpt (gpt-test openai-codex [(High 30)])) (Claude (claude-test [(High 30)]))]\n",
    );
    let error = duplicate_catalog
        .generate(GenerationMode::Write)
        .expect_err("duplicate catalog entry fails");
    assert!(
        matches!(error, Error::DuplicateModelCatalogEntry { .. }),
        "{error:?}"
    );

    let duplicate_effort = Fixture::new();
    duplicate_effort.write_role_generation_sources();
    duplicate_effort.write_source_file(
        "manifests/model-catalog.nota",
        "[(ChatGpt (gpt-test openai-codex [(High 30) (High 40)])) (Claude (claude-test [(High 30)]))]\n",
    );
    let error = duplicate_effort
        .generate(GenerationMode::Write)
        .expect_err("duplicate model effort fails");
    assert!(
        matches!(error, Error::DuplicateModelCatalogEffort { .. }),
        "{error:?}"
    );
}

#[test]
fn role_model_assignments_reject_unsupported_effort_and_family_mismatch() {
    let unsupported = Fixture::new();
    unsupported.write_role_generation_sources();
    unsupported.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(Direct (worker (unknown-model High) (claude-test High)))]\n",
    );
    let error = unsupported
        .generate(GenerationMode::Write)
        .expect_err("unknown model fails");
    assert!(
        matches!(error, Error::UnsupportedRoleModel { .. }),
        "{error:?}"
    );

    let effort = Fixture::new();
    effort.write_role_generation_sources();
    effort.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(Direct (worker (gpt-test Xhigh) (claude-test High)))]\n",
    );
    let error = effort
        .generate(GenerationMode::Write)
        .expect_err("unsupported effort fails");
    assert!(
        matches!(error, Error::UnsupportedRoleModelEffort { .. }),
        "{error:?}"
    );

    let family = Fixture::new();
    family.write_role_generation_sources();
    family.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(Direct (worker (claude-test High) (gpt-test High)))]\n",
    );
    let error = family
        .generate(GenerationMode::Write)
        .expect_err("family mismatch fails");
    assert!(
        matches!(error, Error::RoleModelFamilyMismatch { .. }),
        "{error:?}"
    );
}

#[test]
fn named_role_model_profiles_resolve_and_reject_duplicate_unknown_and_stale_profiles() {
    let resolved = Fixture::new();
    resolved.write_role_generation_sources();
    resolved.write_source_file("roles/worker.md", "Worker role.\n");
    resolved.write_source_file("skills/shared.md", "Shared role module.\n");
    resolved.write_source_file("skills/feature.md", "Feature role module.\n");
    resolved.write_source_file(
        "manifests/role-model-profiles.nota",
        "[(minimalFastEconomical (gpt-test Medium) (claude-test Medium))]\n",
    );
    resolved.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(Profile (worker minimalFastEconomical))]\n",
    );
    resolved
        .generate(GenerationMode::Write)
        .expect("named model profile resolves");
    assert!(
        resolved
            .read_workspace_file(".pi/agents/worker.md")
            .contains("model: 'openai-codex/gpt-test'\nthinking: medium")
    );
    assert!(
        resolved
            .read_workspace_file(".claude/agents/worker.md")
            .contains("model: claude-test\neffort: medium")
    );
    for path in [
        ".pi/agents/worker.md",
        ".codex/agents/worker.toml",
        ".claude/agents/worker.md",
    ] {
        assert!(
            !resolved
                .read_workspace_file(path)
                .contains("minimalFastEconomical"),
            "{path} does not expose the source profile name"
        );
    }

    let duplicate = Fixture::new();
    duplicate.write_role_generation_sources();
    duplicate.write_source_file(
        "manifests/role-model-profiles.nota",
        "[(minimalFastEconomical (gpt-test Medium) (claude-test Medium)) (minimalFastEconomical (gpt-test Medium) (claude-test Medium))]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate profile fails");
    assert!(
        matches!(error, Error::DuplicateNamedRoleModelProfile { .. }),
        "{error:?}"
    );

    let unknown = Fixture::new();
    unknown.write_role_generation_sources();
    unknown.write_source_file(
        "manifests/role-model-assignments.nota",
        "[(Profile (worker unknownProfile))]\n",
    );
    let error = unknown
        .generate(GenerationMode::Write)
        .expect_err("unknown profile fails");
    assert!(
        matches!(error, Error::UnknownNamedRoleModelProfile { .. }),
        "{error:?}"
    );

    let stale = Fixture::new();
    stale.write_role_generation_sources();
    stale.write_source_file(
        "manifests/role-model-profiles.nota",
        "[(minimalFastEconomical (gpt-test Medium) (claude-test Medium))]\n",
    );
    let error = stale
        .generate(GenerationMode::Write)
        .expect_err("unreferenced profile fails");
    assert!(
        matches!(error, Error::StaleNamedRoleModelProfile { .. }),
        "{error:?}"
    );
}

#[test]
fn optional_skill_metadata_rejects_missing_duplicate_stale_and_inactive_references() {
    let missing = Fixture::new();
    missing.write_role_generation_sources();
    missing.write_source_file("manifests/role-optional-skills.nota", "[]\n");
    let error = missing
        .generate(GenerationMode::Write)
        .expect_err("missing optional metadata fails");
    assert!(
        matches!(error, Error::MissingRoleOptionalSkills { .. }),
        "{error:?}"
    );

    let duplicate = Fixture::new();
    duplicate.write_role_generation_sources();
    duplicate.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker []) (worker [])]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate optional metadata fails");
    assert!(
        matches!(error, Error::DuplicateRoleOptionalSkills { .. }),
        "{error:?}"
    );

    let stale = Fixture::new();
    stale.write_role_generation_sources();
    stale.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker []) (retired-role [])]\n",
    );
    let error = stale
        .generate(GenerationMode::Write)
        .expect_err("stale optional metadata fails");
    assert!(
        matches!(error, Error::StaleRoleOptionalSkills { .. }),
        "{error:?}"
    );

    let inactive = Fixture::new();
    inactive.write_role_generation_sources();
    inactive.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [renamed-skill])]\n",
    );
    let error = inactive
        .generate(GenerationMode::Write)
        .expect_err("inactive optional skill fails");
    assert!(
        matches!(error, Error::MissingOptionalSkill { .. }),
        "{error:?}"
    );
}

#[test]
fn optional_skill_metadata_rejects_duplicate_and_target_incompatible_skills() {
    let duplicate = Fixture::new();
    duplicate.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [AgentsSkill ClaudeSkill])) (Role (worker worker [] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    duplicate.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example skills/example.md [] RuntimeSkill) (worker roles/worker.md [] RoleSource)]\n",
    );
    duplicate.write_role_metadata(&["worker"]);
    duplicate.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [example example])]\n",
    );
    let error = duplicate
        .generate(GenerationMode::Write)
        .expect_err("duplicate optional skill fails");
    assert!(
        matches!(error, Error::DuplicateOptionalSkill { .. }),
        "{error:?}"
    );

    let incompatible = Fixture::new();
    incompatible.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (example example Craft Topic [Example skill.] [ClaudeSkill])) (Role (worker worker [] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    incompatible.write_source_file(
        "manifests/module-dependencies.nota",
        "[(example skills/example.md [] RuntimeSkill) (worker roles/worker.md [] RoleSource)]\n",
    );
    incompatible.write_role_metadata(&["worker"]);
    incompatible.write_source_file(
        "manifests/role-optional-skills.nota",
        "[(worker [example])]\n",
    );
    let error = incompatible
        .generate(GenerationMode::Write)
        .expect_err("target-incompatible skill fails");
    assert!(
        matches!(error, Error::TargetIncompatibleOptionalSkill { .. }),
        "{error:?}"
    );
}

#[test]
fn universal_role_modules_expand_into_every_role_packet_without_per_role_manifest_entries() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [feature] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker.md [] RoleSource) (universal skills/universal.md [] RoleComposition) (feature skills/feature.md [] RoleComposition)]\n",
    );
    fixture.write_source_file("manifests/universal-role-modules.nota", "[universal]\n");
    fixture.write_source_file(
        "roles/worker.md",
        "# Role - worker\n\n## Contract\n\nRole body.\n",
    );
    fixture.write_source_file(
        "skills/universal.md",
        "# Module - universal\n\n## Universal Rule\n\nUniversal doctrine.\n",
    );
    fixture.write_source_file(
        "skills/feature.md",
        "# Module - feature\n\n## Feature Rule\n\nPer-role doctrine.\n",
    );

    fixture
        .generate(GenerationMode::Write)
        .expect("universal role modules generate");

    let claude = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude.contains("Universal doctrine."));
    assert!(claude.contains("Per-role doctrine."));
    assert!(claude.find("Role body.") < claude.find("Universal doctrine."));
    assert!(claude.find("Universal doctrine.") < claude.find("Per-role doctrine."));
    assert_eq!(claude.matches("Universal doctrine.").count(), 1);
    assert_eq!(
        fixture
            .read_workspace_file(".pi/agents/worker.md")
            .matches("Universal doctrine.")
            .count(),
        1
    );
    assert_eq!(
        fixture
            .read_workspace_file(".codex/agents/worker.toml")
            .matches("Universal doctrine.")
            .count(),
        1
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
        "[(Skill (management management Meta Mechanism [Management skill] [AgentsSkill ClaudeSkill])) (Role (worker worker [management] [Worker role] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker.md [] RoleSource) (management skills/management.md [] RuntimeSkill) (claude-management skills/claude-management.md [] RuntimeSkill)]\n",
    );
    fixture.write_source_file(
        "manifests/target-module-insertions.nota",
        "[(management ClaudeSkill [claude-management]) (management ClaudeAgent [claude-management])]\n",
    );
    fixture.write_source_file(
        "roles/worker.md",
        "# Role - worker\n\n## Contract\n\nRole body.\n",
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

    let claude_role = fixture.read_workspace_file(".claude/agents/worker.md");
    assert!(claude_role.contains("Shared management."));
    assert!(claude_role.contains("Target overlay."));

    let codex_role = fixture.read_workspace_file(".codex/agents/worker.toml");
    assert!(codex_role.contains("Shared management."));
    assert!(!codex_role.contains("Target overlay."));

    let pi_role = fixture.read_workspace_file(".pi/agents/worker.md");
    assert!(pi_role.contains("Shared management."));
    assert!(!pi_role.contains("Target overlay."));
}

#[test]
fn role_generation_rejects_retired_current_destination_prose() {
    for phrase in [
        "Repo Operator",
        "Weave Operator",
        "Intent Maintainer",
        "workspace essence",
        "workspace intent",
    ] {
        let fixture = Fixture::new();
        fixture.write_role_generation_sources();
        fixture.write_source_file(
            "roles/worker.md",
            &format!(
                "# Role - worker\n\n## Contract\n\nDo not assign current closeout to {phrase}.\n"
            ),
        );
        fixture.write_source_file(
            "skills/shared.md",
            "# Module - shared\n\n## Shared Rule\n\nDependency first.\n",
        );
        fixture.write_source_file(
            "skills/feature.md",
            "# Module - feature\n\n## Feature Rule\n\nDependent second.\n",
        );

        let error = fixture
            .generate(GenerationMode::Write)
            .expect_err("retired title-case current-destination prose fails role generation");

        assert!(
            matches!(
                error,
                Error::RetiredCurrentDestinationProse { phrase: ref found, .. } if found == phrase
            ),
            "{error:?}"
        );
    }
}

#[test]
fn generation_rejects_direct_module_dependency_cycle() {
    let fixture = Fixture::new();
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
fn generation_rejects_duplicate_role_output_paths_before_write() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [] [Worker role.] [ClaudeAgent ClaudeAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker.md [] RoleSource)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("duplicate role output path fails before rendering");

    assert!(
        matches!(
            error,
            Error::DuplicateOutputPath {
                ref relative_path,
                ..
            } if relative_path == ".claude/agents/worker.md"
        ),
        "{error:?}"
    );
    assert!(
        !fixture
            .workspace
            .path()
            .join(".claude/agents/worker.md")
            .exists()
    );
}

#[test]
fn generation_rejects_role_composition_module_as_skill_output() {
    let fixture = Fixture::new();
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
fn generation_rejects_runtime_module_as_role_source() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [] [Worker role.] [ClaudeAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker skills/worker.md [] RuntimeSkill)]\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("role source modules are typed separately");

    assert!(
        matches!(
            error,
            Error::InvalidModuleKind {
                ref module_identifier,
                ref expected,
                ref actual,
            } if module_identifier == "worker"
                && expected == "RoleSource"
                && actual == "RuntimeSkill"
        ),
        "{error:?}"
    );
}

#[test]
fn generation_rejects_role_required_module_missing_from_dependency_index() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Role (worker worker [spirit-query] [Worker role.] [ClaudeAgent]))]\n",
    );
    fixture.write_role_metadata(&["worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(worker roles/worker.md [] RoleSource)]\n",
    );
    fixture.write_source_file(
        "roles/worker.md",
        "# Role - worker\n\n## Contract\n\nBody.\n",
    );

    let error = fixture
        .generate(GenerationMode::Write)
        .expect_err("role-required modules must resolve before packet generation");

    assert!(
        matches!(
            error,
            Error::MissingModule {
                ref module_identifier,
            } if module_identifier == "spirit-query"
        ),
        "{error:?}"
    );
}

#[test]
fn write_mode_removes_only_inventory_owned_stale_role_outputs() {
    let fixture = Fixture::new();
    fixture.write_role_generation_sources();
    fixture.write_source_file(
        "roles/worker.md",
        "# Role - worker\n\n## Contract\n\nBody.\n",
    );
    fixture.write_source_file(
        "skills/shared.md",
        "# Module - shared\n\n## Shared Rule\n\nBody.\n",
    );
    fixture.write_source_file(
        "skills/feature.md",
        "# Module - feature\n\n## Feature Rule\n\nBody.\n",
    );
    fixture.write_workspace_file(
        "skills/generated-role-outputs.nota",
        "[.claude/agents/old.md]\n",
    );
    fixture.write_workspace_file(".claude/agents/old.md", "stale generated role\n");
    fixture.write_workspace_file(".claude/agents/human.md", "human-owned role\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("write mode prunes stale inventory-owned role path");

    assert!(
        !fixture
            .workspace
            .path()
            .join(".claude/agents/old.md")
            .exists()
    );
    assert!(
        fixture
            .workspace
            .path()
            .join(".claude/agents/human.md")
            .exists()
    );
    assert!(
        fixture
            .workspace
            .path()
            .join(".claude/agents/worker.md")
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
fn write_mode_prunes_removed_or_renamed_skill_and_role_outputs() {
    let fixture = Fixture::new();
    fixture.write_source_file(
        "manifests/active-outputs.nota",
        "[(Skill (new-skill new-skill Craft Topic [New skill.] [AgentsSkill ClaudeSkill])) (Role (new-worker new-worker [] [New worker.] [ClaudeAgent CodexAgent PiAgent]))]\n",
    );
    fixture.write_role_metadata(&["new-worker"]);
    fixture.write_source_file(
        "manifests/module-dependencies.nota",
        "[(new-skill skills/new-skill.md [] RuntimeSkill) (new-worker roles/new-worker.md [] RoleSource)]\n",
    );
    fixture.write_source_file(
        "skills/new-skill.md",
        "# Skill — new-skill\n\n## Rule\n\nGenerated.\n",
    );
    fixture.write_source_file(
        "roles/new-worker.md",
        "# Role - new-worker\n\n## Contract\n\nGenerated.\n",
    );
    fixture.write_workspace_file(".agents/skills/old-skill/SKILL.md", "stale skill\n");
    fixture.write_workspace_file(".claude/skills/old-skill/SKILL.md", "stale skill\n");
    fixture.write_workspace_file(
        "skills/generated-role-outputs.nota",
        "[.claude/agents/old-worker.md .codex/agents/old-worker.toml .pi/agents/old-worker.md]\n",
    );
    fixture.write_workspace_file(".claude/agents/old-worker.md", "stale role\n");
    fixture.write_workspace_file(".codex/agents/old-worker.toml", "stale role\n");
    fixture.write_workspace_file(".pi/agents/old-worker.md", "stale role\n");
    fixture.write_workspace_file(".claude/agents/human-owned.md", "human-owned role\n");

    fixture
        .generate(GenerationMode::Write)
        .expect("write mode prunes removed or renamed generated outputs");

    for stale_path in [
        ".agents/skills/old-skill/SKILL.md",
        ".claude/skills/old-skill/SKILL.md",
        ".claude/agents/old-worker.md",
        ".codex/agents/old-worker.toml",
        ".pi/agents/old-worker.md",
    ] {
        assert!(
            !fixture.workspace.path().join(stale_path).exists(),
            "{stale_path} is pruned"
        );
    }
    for active_path in [
        ".agents/skills/new-skill/SKILL.md",
        ".claude/skills/new-skill/SKILL.md",
        ".claude/agents/new-worker.md",
        ".codex/agents/new-worker.toml",
        ".pi/agents/new-worker.md",
        ".claude/agents/human-owned.md",
    ] {
        assert!(
            fixture.workspace.path().join(active_path).exists(),
            "{active_path} remains or is generated"
        );
    }
    let inventory = fixture.read_workspace_file("skills/generated-role-outputs.nota");
    assert!(!inventory.contains("old-worker"));
    assert!(inventory.contains("new-worker"));
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

fn nested_relation_error(relations: &str) -> Error {
    let fixture = Fixture::new();
    fixture.write_nested_validation_sources(relations);
    fixture
        .generate(GenerationMode::Write)
        .expect_err("invalid nested-role relation fails generation")
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
    }

    fn write_role_generation_sources(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (worker worker [shared feature] [Worker role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(worker roles/worker.md [] RoleSource) (shared skills/shared.md [] RoleComposition) (feature skills/feature.md [shared] RoleComposition)]\n",
        );
        self.write_role_metadata(&["worker"]);
    }

    fn write_project_role_contract_sources(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (planner planner [] [Planner role.] [PiAgent])) (Role (reader reader [] [Reader role.] [PiAgent])) (Role (writer writer [] [Writer role.] [PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(planner roles/planner.md [] RoleSource) (reader roles/reader.md [] RoleSource) (writer roles/writer.md [] RoleSource)]\n",
        );
        self.write_role_metadata(&["planner", "reader", "writer"]);
        self.write_source_file(
            "manifests/nested-role-relations.nota",
            "[(planner [(PiAgent gpt-test Medium)] [reader writer])]\n",
        );
        self.write_source_file(
            "roles/planner.md",
            "# Role - planner\n\n## Contract\n\nPlan work.\n",
        );
        for role in ["reader", "writer"] {
            self.write_source_file(
                &format!("roles/{role}.md"),
                &format!("# Role - {role}\n\n## Contract\n\n{role} work.\n"),
            );
        }
    }

    fn write_nested_validation_sources(&self, relations: &str) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (parent parent [] [Parent role.] [ClaudeAgent CodexAgent PiAgent])) (Role (nested-two nested-two [] [Nested two.] [ClaudeAgent CodexAgent PiAgent])) (Role (child child [] [Child role.] [ClaudeAgent CodexAgent PiAgent])) (Role (claude-child claude-child [] [Claude child.] [ClaudeAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(parent roles/parent.md [] RoleSource) (nested-two roles/nested-two.md [] RoleSource) (child roles/child.md [] RoleSource) (claude-child roles/claude-child.md [] RoleSource)]\n",
        );
        self.write_role_metadata(&["parent", "nested-two", "child", "claude-child"]);
        self.write_source_file("manifests/nested-role-relations.nota", relations);
    }

    fn write_cross_model_floor_sources(&self) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (parent parent [] [Parent role.] [ClaudeAgent CodexAgent PiAgent])) (Role (child child [] [Child role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(parent roles/parent.md [] RoleSource) (child roles/child.md [] RoleSource)]\n",
        );
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(ChatGpt (gpt-5.6-sol openai-codex [(Medium 50)])) (ChatGpt (gpt-5.6-terra openai-codex [(High 30)])) (Claude (fable-5 [(Medium 50)])) (Claude (claude-opus-4-8 [(Xhigh 40)]))]\n",
        );
        self.write_source_file(
            "manifests/role-model-assignments.nota",
            "[(Direct (parent (gpt-5.6-terra High) (claude-opus-4-8 Xhigh))) (Direct (child (gpt-5.6-terra High) (claude-opus-4-8 Xhigh)))]\n",
        );
        self.write_source_file(
            "manifests/role-optional-skills.nota",
            "[(parent []) (child [])]\n",
        );
        self.write_source_file(
            "manifests/nested-role-relations.nota",
            "[(parent [(ClaudeAgent fable-5 Medium) (CodexAgent gpt-5.6-sol Medium) (PiAgent gpt-5.6-sol Medium)] [child])]\n",
        );
        for role in ["parent", "child"] {
            self.write_source_file(
                &format!("roles/{role}.md"),
                &format!("# Role - {role}\n\n## Contract\n\nRole body.\n"),
            );
        }
    }

    fn write_model_resolution_sources(&self, minimum_effort: &str) {
        self.write_source_file(
            "manifests/active-outputs.nota",
            "[(Role (parent parent [] [Parent role.] [ClaudeAgent CodexAgent PiAgent])) (Role (child child [] [Child role.] [ClaudeAgent CodexAgent PiAgent]))]\n",
        );
        self.write_source_file(
            "manifests/module-dependencies.nota",
            "[(parent roles/parent.md [] RoleSource) (child roles/child.md [] RoleSource)]\n",
        );
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(ChatGpt (gpt-ordinary ordinary-provider [(Medium 50)])) (ChatGpt (gpt-5.6-sol openai-codex [(Medium 50) (High 60)])) (Claude (claude-ordinary [(Medium 50)])) (Claude (fable-5 [(Medium 50) (High 60)]))]\n",
        );
        self.write_source_file(
            "manifests/role-model-assignments.nota",
            "[(Direct (parent (gpt-ordinary Medium) (claude-ordinary Medium))) (Direct (child (gpt-ordinary Medium) (claude-ordinary Medium)))]\n",
        );
        self.write_source_file(
            "manifests/role-optional-skills.nota",
            "[(parent []) (child [])]\n",
        );
        self.write_source_file(
            "manifests/nested-role-relations.nota",
            &format!(
                "[(parent [(ClaudeAgent fable-5 {minimum_effort}) (CodexAgent gpt-5.6-sol {minimum_effort}) (PiAgent gpt-5.6-sol {minimum_effort})] [child])]\n"
            ),
        );
        for role in ["parent", "child"] {
            self.write_source_file(
                &format!("roles/{role}.md"),
                &format!("# Role - {role}\n\n## Contract\n\nRole body.\n"),
            );
        }
    }

    fn write_role_metadata(&self, role_identifiers: &[&str]) {
        self.write_source_file(
            "manifests/model-catalog.nota",
            "[(ChatGpt (gpt-test openai-codex [(Medium 20) (High 30)])) (Claude (claude-test [(Medium 20) (High 30)]))]\n",
        );
        let assignments = role_identifiers
            .iter()
            .map(|role| format!("(Direct ({role} (gpt-test High) (claude-test High)))"))
            .collect::<Vec<_>>()
            .join(" ");
        self.write_source_file(
            "manifests/role-model-assignments.nota",
            &format!("[{assignments}]\n"),
        );
        let optional_skills = role_identifiers
            .iter()
            .map(|role| format!("({role} [])"))
            .collect::<Vec<_>>()
            .join(" ");
        self.write_source_file(
            "manifests/role-optional-skills.nota",
            &format!("[{optional_skills}]\n"),
        );
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
