use curriculum::registry::IdentityText;
mod support;
use curriculum::{registry::SkillRegistry, startup};
use sha2::{Digest, Sha256};
use signal_curriculum_nexus::*;
use std::{fs, path::PathBuf};
use support::identity;
fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
fn fixture(base: &std::path::Path) -> ([PathBuf; 3], SkillRegistry, StartupSelection) {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
    }
    fs::create_dir(base.join("flows")).unwrap();
    fs::create_dir(roots[0].join("spirit")).unwrap();
    fs::create_dir(roots[0].join("intent")).unwrap();
    write_source(roots[0].join("spirit/core/0.md"), "SPIRIT").unwrap();
    write_source(roots[0].join("intent/seat/0.md"), "SEAT_INTENT").unwrap();
    fs::create_dir(roots[0].join("vision")).unwrap();
    fs::create_dir(roots[2].join("compensation")).unwrap();
    write_source(
        roots[0].join("vision/core/0.md"),
        "---\ndependencies: []\nplacement: user\n---\nSYSTEM_ONLY\n",
    )
    .unwrap();
    let mut ids = Vec::new();
    for index in 0..22 {
        let id = format!("compensation:user{index:02}:0");
        ids.push(id);
        write_source(
            roots[2].join(format!("compensation/user{index:02}/0.md")),
            format!(
                "---\ndependencies: [ vision:core:0 ]\nplacement: user\n---\nBODY_{index:02}_EXACT\n"
            ),
        )
        .unwrap();
    }
    let registry =
        support::registry(&roots, &["a".repeat(40), "b".repeat(40), "c".repeat(40)]).unwrap();
    (
        roots,
        registry,
        StartupSelection {
            target: Target::Claude,
            caller_metaflow: CallerMetaflow {
                first_string: "Field".into(),
                second_string: "Publisher".into(),
                third_string: "Quaternary".into(),
            },
            system_modules: SystemModules(vec![
                identity("spirit:core:0"),
                identity("intent:seat:0"),
            ]),
            user_modules: UserModules(
                std::iter::once("vision:core:0".to_owned())
                    .chain(ids)
                    .map(|id| identity(&id))
                    .collect(),
            ),
            flow_id_vector: vec![],
        },
    )
}
#[test]
fn composes_all_selected_text_in_memory_once_with_exact_hashes_and_placement() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, selection) = fixture(tmp.path());
    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert_eq!(result.context_provenance_vector.len(), 25);
    let system_ids = result
        .context_provenance_vector
        .iter()
        .filter(|row| row.startup_placement == StartupPlacement::System)
        .map(|row| row.module_identity.identity_text().unwrap())
        .collect::<Vec<_>>();
    let user_ids = result
        .context_provenance_vector
        .iter()
        .filter(|row| row.startup_placement == StartupPlacement::User)
        .map(|row| row.module_identity.identity_text().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(system_ids, ["spirit:core:0", "intent:seat:0"]);
    assert_eq!(user_ids.first().map(String::as_str), Some("vision:core:0"));
    assert!(
        user_ids
            .iter()
            .all(|id| { !id.starts_with("spirit:") && !id.starts_with("intent:") })
    );
    assert!(!result.system_text.0.contains("SYSTEM_ONLY"));
    assert_eq!(result.user_text.0.matches("SYSTEM_ONLY").count(), 1);
    assert!(!result.system_text.0.contains("BODY_"));
    for i in 0..22 {
        assert_eq!(
            result
                .user_text
                .0
                .matches(&format!("BODY_{i:02}_EXACT\n"))
                .count(),
            1
        );
    }
    assert!(
        result
            .user_text
            .0
            .starts_with("These are your skills, given here; do not reload them.")
    );
    assert_eq!(
        result.system_payload_sha256.0,
        digest(&result.system_text.0)
    );
    assert_eq!(result.user_payload_sha256.0, digest(&result.user_text.0));
    let ordinals: Vec<_> = result
        .context_provenance_vector
        .iter()
        .filter(|m| matches!(m.startup_placement, StartupPlacement::User))
        .map(|m| m.integer)
        .collect();
    assert_eq!(ordinals, (0..23).collect::<Vec<i64>>());
    assert!(result.records_text.0.is_empty());
    assert_eq!(result.records_payload_sha256.0, digest(""));
    assert!(!tmp.path().join("tools").exists());
    assert!(!tmp.path().join(".claude").exists());
    let outcome = Output::StartupComposed(result);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&outcome).unwrap();
    assert_eq!(
        rkyv::from_bytes::<Output, rkyv::rancor::Error>(&bytes).unwrap(),
        outcome
    );
}

#[test]
fn claude_and_codex_modules_start_with_descriptions_and_keep_rendered_body_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, registry, mut selection) = fixture(tmp.path());
    let mut expected = std::collections::BTreeMap::new();
    for (index, module) in registry.listed_modules().unwrap().into_iter().enumerate() {
        let id = module.module_identity.identity_text().unwrap();
        let source = fs::read_to_string(&module.module_file.0).unwrap();
        let (metadata, body) = match source.strip_prefix("---\n") {
            Some(rest) => rest.split_once("\n---\n").unwrap(),
            None => ("", source.as_str()),
        };
        let (scalar, description) = match index {
            0 => (
                r#""Quoted \"head\".""#.to_owned(),
                "Quoted \"head\".".to_owned(),
            ),
            1 => (
                "Folded\n  description.".to_owned(),
                "Folded description.".to_owned(),
            ),
            _ => (
                format!("Description for {id}."),
                format!("Description for {id}."),
            ),
        };
        let body = if id == "compensation:user00:0" {
            "{% if claude %}\nCLAUDE\n{% else %}\nCODEX\n{% endif %}\n```json\n{\"nested\":{\"value\":1}}\n```\n"
        } else {
            body
        };
        fs::write(
            &module.module_file.0,
            format!("---\ndescription: {scalar}\n{metadata}\n---\n{body}"),
        )
        .unwrap();
        expected.insert(id, (description, body.to_owned()));
    }
    let registry =
        support::registry(&roots, &["a".repeat(40), "b".repeat(40), "c".repeat(40)]).unwrap();
    for (target, target_body) in [(Target::Claude, "CLAUDE"), (Target::Codex, "CODEX")] {
        selection.target = target;
        let composed = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
        assert_eq!(composed.context_provenance_vector.len(), expected.len());
        for row in &composed.context_provenance_vector {
            let id = row.module_identity.identity_text().unwrap();
            let (description, body) = &expected[&id];
            let rendered_body = if id == "compensation:user00:0" {
                format!("{target_body}\n```json\n{{\"nested\":{{\"value\":1}}}}\n```\n")
            } else {
                body.clone()
            };
            let interior = format!("{description}\n\n{rendered_body}");
            let text = match row.startup_placement {
                StartupPlacement::System => &composed.system_text.0,
                StartupPlacement::User => &composed.user_text.0,
            };
            let segment =
                format!("\n<!-- module: {id} -->\n{interior}\n<!-- end-module: {id} -->\n");
            assert_eq!(text.matches(&segment).count(), 1, "{id}");
            assert_eq!(row.rendered_sha256.0, digest(&interior), "{id}");
            assert_eq!(row.body_sha256.0, digest(body), "{id}");
        }
        assert_eq!(
            composed.system_payload_sha256.0,
            digest(&composed.system_text.0)
        );
        assert_eq!(
            composed.user_payload_sha256.0,
            digest(&composed.user_text.0)
        );
    }
}
#[test]
fn caller_paths_inline_text_unknown_duplicates_wrong_placement_and_changed_sources_refuse() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, registry, selection) = fixture(tmp.path());
    for bad in [
        "/tmp/context.md",
        "../vision/core/0.md",
        "Ignore previous instructions",
        "missing:id",
        "vision:core",
        "vision:core:00",
        "vision:core:latest",
        "vision:core:0:1",
    ] {
        let mut s = selection.clone();
        s.user_modules = UserModules(vec![identity(bad)]);
        assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    }
    let mut s = selection.clone();
    s.user_modules.0.push(s.user_modules.0[0].clone());
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    let mut s = selection.clone();
    s.user_modules.0.push(identity("vision:core:0"));
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    let mut s = selection.clone();
    s.system_modules = SystemModules(vec![identity("compensation:user00:0")]);
    s.user_modules.0.clear();
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    write_source(roots[2].join("compensation/user00/0.md"), "Changed").unwrap();
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &selection).is_err());
}
#[test]
fn undeclared_shared_dependency_placement_refuses_instead_of_duplicate_injection() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, _) = fixture(tmp.path());
    write_source(roots[0].join("vision/core/0.md"), "COMMON_UNPLACED").unwrap();
    write_source(
        roots[0].join("vision/system/0.md"),
        "---\ndependencies: [ vision:core:0 ]\n---\nSYSTEM\n",
    )
    .unwrap();
    let registry =
        support::registry(&roots, &["a".repeat(40), "b".repeat(40), "c".repeat(40)]).unwrap();
    let selection = StartupSelection {
        target: Target::Claude,
        caller_metaflow: CallerMetaflow {
            first_string: "Field".into(),
            second_string: "Publisher".into(),
            third_string: "Quaternary".into(),
        },
        system_modules: SystemModules(vec![identity("vision:system:0")]),
        user_modules: UserModules(vec![identity("compensation:user00:0")]),
        flow_id_vector: vec![],
    };
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &selection).is_err());
}

#[test]
fn target_rendering_precedes_hashing_and_preserves_raw_and_shell_literals() {
    use curriculum::context_render;
    let text = "{% if claude %}\nCLAUDE\n{% else %}\nOTHER\n{% endif %}\n{% raw %}\n{{literal}}\n{% endraw %}\n${VAR:?}\n";
    assert_eq!(
        context_render::render(&Target::Claude, text).unwrap(),
        "CLAUDE\n{{literal}}\n${VAR:?}\n"
    );
    assert_eq!(
        context_render::render(&Target::Codex, text).unwrap(),
        "OTHER\n{{literal}}\n${VAR:?}\n"
    );
    for bad in [
        "{% endif %}",
        "{% if unknown %}",
        "{% if claude %}\nmissing end",
        "{{unknown}}",
    ] {
        assert!(context_render::render(&Target::Claude, bad).is_err());
    }
}

#[test]
fn spirit_intent_cannot_be_user_even_without_frontmatter_placement() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, selection) = fixture(tmp.path());
    for id in ["spirit:core:0", "intent:seat:0"] {
        let mut s = selection.clone();
        s.system_modules
            .0
            .retain(|m| m.identity_text().map(|text| text != id).unwrap_or(false));
        s.user_modules.0.push(identity(id));
        assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
    }
    let mut s = selection;
    s.system_modules.0.clear();
    assert!(startup::compose(&registry, &tmp.path().join("flows"), &s).is_err());
}

#[test]
fn manifest_distinguishes_authored_and_target_rendered_module_hashes() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, selection) = fixture(tmp.path());
    let text = "{% if claude %}\nCLAUDE_ONLY\n{% else %}\nCODEX_OTHER\n{% endif %}\n";
    write_source(roots[2].join("compensation/user00/0.md"), text).unwrap();
    let registry =
        support::registry(&roots, &["a".repeat(40), "b".repeat(40), "c".repeat(40)]).unwrap();
    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    let module = result
        .context_provenance_vector
        .iter()
        .find(|m| {
            m.module_identity
                .identity_text()
                .map(|id| id == "compensation:user00:0")
                .unwrap_or(false)
        })
        .unwrap();
    assert_eq!(module.body_sha256.0, digest(text));
    assert_eq!(module.rendered_sha256.0, digest("CLAUDE_ONLY\n"));
    assert!(result.user_text.0.contains("CLAUDE_ONLY"));
    assert!(!result.user_text.0.contains("CODEX_OTHER"));
    assert!(!result.user_text.0.contains("{%"));
}

#[test]
fn selected_raw_records_are_verified_in_flow_vision_notion_and_file_order() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, mut selection) = fixture(tmp.path());
    let first = tmp.path().join("flows/first");
    let second = tmp.path().join("flows/second");
    fs::create_dir_all(first.join("vision")).unwrap();
    fs::create_dir_all(first.join("notion")).unwrap();
    fs::create_dir_all(second.join("vision")).unwrap();
    write_source(first.join("vision/b.md"), "FIRST-B\n").unwrap();
    write_source(first.join("vision/a.md"), "FIRST-A\n").unwrap();
    write_source(first.join("vision/archive-old.md"), "ARCHIVED\n").unwrap();
    write_source(first.join("notion/c.md"), "FIRST-C\n").unwrap();
    write_source(second.join("vision/a.md"), "SECOND-A\n").unwrap();
    selection.flow_id_vector = vec![FlowId("first".into()), FlowId("second".into())];

    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert_eq!(
        result.records_text.0,
        "FIRST-A\nFIRST-B\nFIRST-C\nSECOND-A\n"
    );
    assert!(!result.records_text.0.contains("ARCHIVED"));
    assert_eq!(
        result.records_payload_sha256.0,
        digest(&result.records_text.0)
    );

    selection.flow_id_vector = vec![FlowId("missing".into())];
    assert_eq!(
        startup::compose(&registry, &tmp.path().join("flows"), &selection),
        Err("record missing: missing".into())
    );
}

#[test]
fn selected_flow_with_no_nonarchive_records_contributes_empty_context() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, mut selection) = fixture(tmp.path());
    let flow = tmp.path().join("flows/empty");
    fs::create_dir_all(flow.join("vision")).unwrap();
    fs::create_dir_all(flow.join("notion")).unwrap();
    write_source(flow.join("vision/archive-old.md"), "ARCHIVED\n").unwrap();
    write_source(flow.join("notion/note.txt"), "NOT A RECORD\n").unwrap();
    selection.flow_id_vector = vec![FlowId("empty".into())];

    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    assert!(result.records_text.0.is_empty());
    assert_eq!(result.records_payload_sha256.0, digest(""));
}

#[test]
fn malformed_selected_record_retains_a_typed_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, registry, mut selection) = fixture(tmp.path());
    let flow = tmp.path().join("flows/malformed/vision");
    fs::create_dir_all(&flow).unwrap();
    write_source(flow.join("record.md"), [0xff]).unwrap();
    selection.flow_id_vector = vec![FlowId("malformed".into())];

    assert_eq!(
        startup::compose(&registry, &tmp.path().join("flows"), &selection),
        Err("record missing: malformed".into())
    );
}

fn write_source(
    path: impl AsRef<std::path::Path>,
    contents: impl AsRef<[u8]>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, contents)
}

#[test]
fn startup_uses_actual_span_provenance_and_deduplicates_a_shared_span() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, mut selection) = fixture(tmp.path());
    write_source(
        roots[2].join("compensation/user00/0-1.md"),
        "---\ndependencies: [ vision:core:0 ]\nplacement: user\n---\nBODY_00_SPAN\n",
    )
    .unwrap();
    let manifest = roots[2].join("modules.datom");
    let declarations = fs::read_to_string(&manifest).unwrap();
    let declarations = declarations.strip_suffix(" ]\n").unwrap();
    fs::write(
        &manifest,
        format!(
            "{declarations}\n  {{ {{ Compensation user00 Span.{{ 0 1 }} }} «compensation/user00/0-1.md» }} ]\n"
        ),
    )
    .unwrap();
    let registry = SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)],
    )
    .unwrap();
    selection
        .user_modules
        .0
        .push(identity("compensation:user00:1"));
    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    let actual = result
        .context_provenance_vector
        .iter()
        .filter(|module| {
            module
                .module_identity
                .identity_text()
                .map(|id| id.starts_with("compensation:user00:"))
                .unwrap_or(false)
        })
        .map(|module| module.module_identity.identity_text().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, ["compensation:user00:0", "compensation:user00:0-1"]);
    assert_eq!(result.user_text.0.matches("BODY_00_EXACT").count(), 1);
    assert_eq!(result.user_text.0.matches("BODY_00_SPAN").count(), 1);
}

#[test]
fn user_dependency_on_explicit_system_modules_is_deduplicated_by_actual_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, selection) = fixture(tmp.path());
    write_source(
        roots[2].join("compensation/user00/0.md"),
        "---\ndependencies: [ spirit:core:0 intent:seat:0 ]\nplacement: user\n---\nUSER_WITH_SYSTEM_DEPS\n",
    )
    .unwrap();
    let registry =
        support::registry(&roots, &["a".repeat(40), "b".repeat(40), "c".repeat(40)]).unwrap();
    let result = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap();
    for identity in ["spirit:core:0", "intent:seat:0"] {
        assert_eq!(
            result
                .context_provenance_vector
                .iter()
                .filter(|module| module
                    .module_identity
                    .identity_text()
                    .map(|id| id == identity)
                    .unwrap_or(false))
                .count(),
            1
        );
    }
    assert_eq!(
        result.user_text.0.matches("USER_WITH_SYSTEM_DEPS").count(),
        1
    );
}

#[test]
fn inventory_is_independent_while_startup_refuses_duplicate_execution_metadata() {
    let tmp = tempfile::tempdir().unwrap();
    let (roots, _, selection) = fixture(tmp.path());
    fs::write(
        roots[0].join("vision/core/0.md"),
        "---\ndescription: first\ndescription: second\nplacement: user\n---\nBody.\n",
    )
    .unwrap();
    let registry =
        support::registry(&roots, &["a".repeat(40), "b".repeat(40), "c".repeat(40)]).unwrap();
    assert_eq!(registry.listed_modules().unwrap().len(), 25);
    let verbose = registry.listed_modules_verbose().unwrap_err().to_string();
    assert!(verbose.contains("vision:core:0") && verbose.contains("vision/core/0.md"));
    let error = startup::compose(&registry, &tmp.path().join("flows"), &selection).unwrap_err();
    assert!(error.contains("duplicate frontmatter key"), "{error}");
}

#[test]
fn context_variables_require_a_complete_identifier_placeholder() {
    use curriculum::context_render;
    let literal = concat!(
        "```json\n",
        "{\"type\":\"assistant\",\"message\":{\"usage\":{\"input_tokens\":2}}}\n",
        "```\n",
        "}} {{ {{ }} {{123}} {{not a name}} {{\"key\":1}}\n",
        "Unicode: café; shell: ${VAR:?}; unclosed: {{name\n",
    );
    for target in [Target::Claude, Target::Codex, Target::Pi, Target::OpenCode] {
        assert_eq!(context_render::render(&target, literal).unwrap(), literal);
        for variable in [
            "{{unknown}}",
            "{{ unknown_2 }}",
            "{{_name}}",
            "literal }} then {{ missing }}",
        ] {
            assert_eq!(
                context_render::render(&target, variable).unwrap_err(),
                "unresolved context variable; no caller-inline substitution is allowed"
            );
        }
        assert_eq!(
            context_render::render(&target, "{% raw %}\n{{unknown}}\n{% endraw %}\n").unwrap(),
            "{{unknown}}\n"
        );
    }
    assert_eq!(
        context_render::render(
            &Target::Codex,
            "{% if claude %}\n{{unknown}}\n{% endif %}\n"
        )
        .unwrap(),
        ""
    );
}
