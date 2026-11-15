//! A source's modules are what its modules.datom declares; a file's own name
//! and folder mean nothing.
use curriculum::registry::{DeclarationFinding, ExpandsSkills, RegistryError, SkillRegistry};
use curriculum::registry::{IdentityText, ModuleIdentityConversion};
use std::{fs, path::Path, path::PathBuf};

fn roots(base: &Path) -> [PathBuf; 3] {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("modules.datom"), "[]\n").unwrap();
    }
    roots
}
fn revisions() -> [String; 3] {
    ["a".repeat(40), "b".repeat(40), "c".repeat(40)]
}
fn file(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}
fn declare(root: &Path, declarations: &str) {
    fs::write(root.join("modules.datom"), declarations).unwrap();
}
fn read(roots: &[PathBuf; 3]) -> SkillRegistry {
    SkillRegistry::from_context_repositories(roots, &revisions()).unwrap()
}

#[test]
fn empty_registry_has_empty_default_and_verbose_inventory() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    let registry = read(&r);
    assert!(registry.listed_modules().unwrap().is_empty());
    assert!(registry.listed_modules_verbose().unwrap().is_empty());
}

#[test]
fn listing_is_unique_typed_and_verbose_is_opt_in() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "authored/0.md", "Plain body.\n");
    file(&r[0], "authored/0-2.md", "Span body.\n");
    declare(
        &r[0],
        "[ { { Vision topic Single.0 } «authored/0.md» } { { Vision topic Span.{ 0 2 } } «authored/0-2.md» } ]\n",
    );
    let registry = read(&r);
    let listed = registry.listed_modules().unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(
        listed[0].module_identity.identity_text().unwrap(),
        "vision:topic:0"
    );
    assert!(listed[0].module_file.0.starts_with('/'));
    let verbose = registry.listed_modules_verbose().unwrap();
    assert_eq!(verbose[0].description_text.0, "");
}

#[test]
fn verbose_listing_applies_conservative_description_grammar() {
    for frontmatter in [
        "description: \"quoted\"\n",
        "description: plain # ambiguous\n",
        "description: >\n  folded\n",
        "description: |\n  literal\n",
        "description: 'quoted'\n",
        "description: first\n  continued\n",
        "description: first\ndescription: second\n",
        " description: indented\n",
        "# comment\ndescription: plain\n",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        file(
            &r[0],
            "vision/topic/0.md",
            &format!("---\n{frontmatter}---\nBody.\n"),
        );
        declare(
            &r[0],
            "[ { { Vision topic Single.0 } «vision/topic/0.md» } ]\n",
        );
        let registry = read(&r);
        assert!(registry.listed_modules().is_ok());
        let result = registry.listed_modules_verbose();
        if frontmatter.starts_with("#") {
            assert!(result.is_ok());
        } else {
            let error = result.unwrap_err().to_string();
            assert!(error.contains("vision:topic:0") && error.contains("vision/topic/0.md"));
        }
    }
}

#[test]
fn verbose_refuses_empty_and_populated_block_scalar_indicators_and_modifiers() {
    for indicator in [
        ">", "|", ">-", ">+", "|-", "|+", ">2", "|2", ">2-", ">-2", "|2+", "|+2",
    ] {
        for continuation in ["", "  content\n"] {
            let temp = tempfile::tempdir().unwrap();
            let r = roots(temp.path());
            file(
                &r[0],
                "vision/topic/0.md",
                &format!("---\ndescription: {indicator}\n{continuation}---\nBody.\n"),
            );
            declare(
                &r[0],
                "[ { { Vision topic Single.0 } «vision/topic/0.md» } ]\n",
            );
            let registry = read(&r);
            let listed = registry.listed_modules().unwrap();
            assert_eq!(listed.len(), 1);
            assert_eq!(
                listed[0].module_identity.identity_text().unwrap(),
                "vision:topic:0"
            );
            assert!(listed[0].module_file.0.ends_with("vision/topic/0.md"));
            let error = registry.listed_modules_verbose().unwrap_err().to_string();
            assert!(
                error.contains("vision:topic:0") && error.contains("vision/topic/0.md"),
                "{indicator}: {error}"
            );
        }
    }
}

#[test]
fn verbose_continuations_follow_the_owning_key_and_preserve_plain_description() {
    for (header, expected) in [
        ("# comment\ndescription:   plain text  \n\n", "plain text"),
        ("description:\n", ""),
        (
            "dependencies: []\n  unrelated continuation\ndescription: plain\n",
            "plain",
        ),
        (
            "description: plain\ndependencies: []\n  unrelated continuation\n",
            "plain",
        ),
        ("dependencies: []\n  description: >-\n", ""),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        file(
            &r[0],
            "vision/topic/0.md",
            &format!("---\n{header}---\nBody.\n"),
        );
        declare(
            &r[0],
            "[ { { Vision topic Single.0 } «vision/topic/0.md» } ]\n",
        );
        let registry = read(&r);
        assert_eq!(registry.listed_modules().unwrap().len(), 1);
        assert_eq!(
            registry.listed_modules_verbose().unwrap()[0]
                .description_text
                .0,
            expected
        );
    }
}

#[test]
fn listing_refuses_source_drift_and_read_failure() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "vision/topic/0.md", "Body.\n");
    declare(
        &r[0],
        "[ { { Vision topic Single.0 } «vision/topic/0.md» } ]\n",
    );
    let registry = read(&r);
    file(&r[0], "vision/topic/0.md", "Changed.\n");
    assert!(registry.listed_modules().is_err());
    fs::remove_file(r[0].join("vision/topic/0.md")).unwrap();
    assert!(registry.listed_modules_verbose().is_err());
}

#[test]
fn malformed_typed_identity_is_refused_in_shared_codecs() {
    let malformed = signal_curriculum_nexus::ModuleIdentity {
        module_kind: signal_curriculum_nexus::ModuleKind::Vision,
        module_subject: signal_curriculum_nexus::ModuleSubject("Bad Topic".into()),
        versions: signal_curriculum_nexus::Versions::Single(signal_curriculum_nexus::Version(-1)),
    };
    assert!(malformed.identity_text().is_err());
    assert!("vision:topic:1-1".module_identity().is_err());
    assert!("unknown:topic:0".module_identity().is_err());
}

#[test]
fn the_declaration_names_the_module_from_kind_topic_and_version_file_name() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "anywhere/0.md", "Core.\n");
    file(&r[0], "spirit/core/0.md", "Never declared.\n");
    declare(&r[0], "[ { { Spirit core Single.0 } «anywhere/0.md» } ]\n");
    let registry = read(&r);
    assert_eq!(registry.skill_names().unwrap(), ["spirit:core:0"]);
    let resolved = registry
        .resolved_sources(&["spirit:core:0".into()])
        .unwrap();
    assert_eq!(resolved[0].string, "Core.\n");
    assert_eq!(resolved[0].source_relative_path.0, "anywhere/0.md");
    assert!(registry.reports().is_empty());
}

#[test]
fn numeric_binding_selects_singleton_and_every_containing_span_with_actual_provenance() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "singles/0.md", "Zero.\n");
    file(&r[0], "singles/1.md", "One.\n");
    file(&r[0], "spans/0-1.md", "Zero through one.\n");
    declare(
        &r[0],
        "[ { { Vision ethos Single.0 } «singles/0.md» }
  { { Vision ethos Single.1 } «singles/1.md» }
  { { Vision ethos Span.{ 0 1 } } «spans/0-1.md» } ]\n",
    );
    let registry = read(&r);
    let resolved = registry
        .resolved_exact_sources(&["vision:ethos:1".into()])
        .unwrap();
    assert_eq!(
        resolved
            .iter()
            .map(|module| module.module_identity.identity_text().unwrap())
            .collect::<Vec<_>>(),
        ["vision:ethos:1", "vision:ethos:0-1"]
    );
    assert_eq!(
        resolved
            .iter()
            .map(|module| module.source_relative_path.0.as_str())
            .collect::<Vec<_>>(),
        ["singles/1.md", "spans/0-1.md"]
    );
}

#[test]
fn span_only_and_overlapping_spans_resolve_in_deterministic_order() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    for (path, body) in [("ranges/0-2.md", "0-2"), ("ranges/1-3.md", "1-3")] {
        file(&r[0], path, body);
    }
    declare(
        &r[0],
        "[ { { Vision range Span.{ 0 2 } } «ranges/0-2.md» }
  { { Vision range Span.{ 1 3 } } «ranges/1-3.md» } ]\n",
    );
    let registry = read(&r);
    let identities = registry
        .resolved_exact_sources(&["vision:range:1".into()])
        .unwrap()
        .into_iter()
        .map(|module| module.module_identity.identity_text().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(identities, ["vision:range:0-2", "vision:range:1-3"]);
    assert_eq!(
        registry
            .resolved_exact_sources(&["vision:range:1-3".into()])
            .unwrap()[0]
            .module_identity
            .identity_text()
            .unwrap(),
        "vision:range:1-3"
    );
    assert_eq!(
        registry
            .resolved_exact_sources(&["vision:range:2-3".into()])
            .unwrap_err(),
        RegistryError::Missing("vision:range:2-3".into())
    );
}

#[test]
fn duplicate_identity_refuses_instead_of_keeping_one() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "first/0-1.md", "First.\n");
    file(&r[2], "second/0-1.md", "Second.\n");
    declare(
        &r[0],
        "[ { { Knowledge topic Span.{ 0 1 } } «first/0-1.md» } ]\n",
    );
    declare(
        &r[2],
        "[ { { Knowledge topic Span.{ 0 1 } } «second/0-1.md» } ]\n",
    );
    assert_eq!(
        SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err(),
        RegistryError::Duplicate("knowledge:topic:0-1".into())
    );
}

#[test]
fn declarations_that_cannot_be_placed_are_reported_and_leave_the_rest() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[1], "kept/0.md", "Kept.\n");
    file(&r[1], "upper/0.md", "Upper.\n");
    fs::write(temp.path().join("0.md"), "Outside.\n").unwrap();
    fs::create_dir_all(r[1].join("linked")).unwrap();
    std::os::unix::fs::symlink(temp.path().join("0.md"), r[1].join("linked/0.md")).unwrap();
    declare(
        &r[1],
        "[ { { Knowledge kept Single.0 } «kept/0.md» }
  { { Knowledge absent Single.0 } «absent/0.md» }
  { { Knowledge linked Single.0 } «linked/0.md» }
  { { Knowledge escaping Single.0 } «../0.md» }
  { { Knowledge Upper Single.0 } «upper/0.md» } ]\n",
    );
    let registry = read(&r);
    assert_eq!(registry.skill_names().unwrap(), ["knowledge:kept:0"]);
    let reported: Vec<_> = registry
        .reports()
        .iter()
        .map(|report| (report.name.as_str(), report.finding.clone()))
        .collect();
    assert_eq!(
        reported,
        [
            (
                "knowledge:absent:0",
                DeclarationFinding::Absent("absent/0.md".into())
            ),
            (
                "knowledge:linked:0",
                DeclarationFinding::Unplaceable("linked/0.md".into())
            ),
            (
                "knowledge:escaping:0",
                DeclarationFinding::Unplaceable("../0.md".into())
            ),
            ("knowledge:Upper:0", DeclarationFinding::InvalidName),
        ]
    );
}

#[test]
fn malformed_spans_and_absent_span_files_refuse() {
    {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        file(&r[0], "negative/-1.md", "Single.\n");
        declare(
            &r[0],
            "[ { { Vision single Single.-1 } «negative/-1.md» } ]\n",
        );
        let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
        assert!(error.to_string().contains("must be nonnegative"));
    }
    for (versions, path) in [
        ("Span.{ 0 0 }", "ranges/0-0.md"),
        ("Span.{ 2 1 }", "ranges/2-1.md"),
        ("Span.{ -1 1 }", "ranges/-1-1.md"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        file(&r[0], path, "Range.\n");
        declare(
            &r[0],
            &format!("[ {{ {{ Vision range {versions} }} «{path}» }} ]\n"),
        );
        let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("must have nonnegative increasing bounds"),
            "{versions}"
        );
    }

    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    declare(
        &r[0],
        "[ { { Vision range Span.{ 0 1 } } «missing/0-1.md» } ]\n",
    );
    let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
    assert!(error.to_string().contains("vision:range:0-1"));
    assert!(error.to_string().contains("missing/0-1.md"));
}

#[test]
fn a_source_without_readable_declarations_refuses() {
    for declarations in [
        None,
        Some("{ not a vector }\n"),
        Some("[ { Unknown topic Single.0 «x.md» } ]\n"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        match declarations {
            None => fs::remove_file(r[2].join("modules.datom")).unwrap(),
            Some(text) => declare(&r[2], text),
        }
        assert!(
            SkillRegistry::from_context_repositories(&r, &revisions()).is_err(),
            "{declarations:?}"
        );
    }
}

#[test]
fn dependency_binding_traverses_each_actual_module_once_without_span_self_cycle() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "topic/1.md", "One.\n");
    file(
        &r[0],
        "topic/0-2.md",
        "---\ndependencies: [ vision:topic:1 ]\n---\nSpan.\n",
    );
    file(
        &r[1],
        "consumer/0.md",
        "---\ndependencies: [ vision:topic:1 ]\n---\nConsumer.\n",
    );
    declare(
        &r[0],
        "[ { { Vision topic Single.1 } «topic/1.md» }
  { { Vision topic Span.{ 0 2 } } «topic/0-2.md» } ]\n",
    );
    declare(
        &r[1],
        "[ { { Knowledge consumer Single.0 } «consumer/0.md» } ]\n",
    );
    let registry = read(&r);
    let identities = registry
        .resolved_sources(&["knowledge:consumer:0".into()])
        .unwrap()
        .into_iter()
        .map(|module| module.module_identity.identity_text().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        ["vision:topic:1", "vision:topic:0-2", "knowledge:consumer:0"]
    );
}

#[test]
fn declaration_version_must_equal_its_file_name() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "range/0-2.md", "Range.\n");
    declare(
        &r[0],
        "[ { { Vision range Span.{ 0 1 } } «range/0-2.md» } ]\n",
    );
    let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
    assert!(error.to_string().contains("vision:range:0-1"));
    assert!(error.to_string().contains("must be named 0-1.md"));
}

#[test]
fn declarations_cannot_share_one_source_file() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "shared/0.md", "Shared.\n");
    declare(
        &r[0],
        "[ { { Vision first Single.0 } «shared/0.md» }
  { { Vision second Single.0 } «shared/0.md» } ]\n",
    );
    let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
    assert!(error.to_string().contains("vision:first:0"));
    assert!(error.to_string().contains("vision:second:0"));
    assert!(error.to_string().contains("same source file"));
}

#[test]
fn every_inadmissible_span_file_refuses_and_names_the_identity() {
    for linked in [true, false] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        let outside = temp.path().join("0-1.md");
        fs::write(&outside, "Outside.\n").unwrap();
        let declared = if linked {
            fs::create_dir_all(r[0].join("range")).unwrap();
            std::os::unix::fs::symlink(&outside, r[0].join("range/0-1.md")).unwrap();
            "range/0-1.md"
        } else {
            "../0-1.md"
        };
        declare(
            &r[0],
            &format!("[ {{ {{ Vision range Span.{{ 0 1 }} }} «{declared}» }} ]\n"),
        );
        let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
        assert!(error.to_string().contains("vision:range:0-1"));
        assert!(
            error
                .to_string()
                .contains("not a plain file inside the source")
        );
    }
}

#[test]
fn duplicate_span_refuses_before_an_inadmissible_file_can_hide_it() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "present/0-1.md", "Present.\n");
    declare(
        &r[0],
        "[ { { Vision range Span.{ 0 1 } } «missing/0-1.md» }
  { { Vision range Span.{ 0 1 } } «present/0-1.md» } ]\n",
    );
    assert_eq!(
        SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err(),
        RegistryError::Duplicate("vision:range:0-1".into())
    );
}

#[test]
fn dependencies_naming_the_actual_singleton_or_span_are_cycles() {
    for (versions, identity, path) in [
        ("Single.1", "vision:topic:1", "topic/1.md"),
        ("Span.{ 0 2 }", "vision:topic:0-2", "topic/0-2.md"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        file(
            &r[0],
            path,
            &format!("---\ndependencies: [ {identity} ]\n---\nSelf.\n"),
        );
        declare(
            &r[0],
            &format!("[ {{ {{ Vision topic {versions} }} «{path}» }} ]\n"),
        );
        assert_eq!(
            SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err(),
            RegistryError::Cycle(identity.into())
        );
    }
}
