//! A source's modules are what its modules.datom declares; a file's own name
//! and folder mean nothing.
use curriculum::registry::{DeclarationFinding, ExpandsSkills, RegistryError, SkillRegistry};
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
fn the_declaration_names_the_module_whatever_its_file_is_called() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "anywhere/at all.md", "Core.\n");
    file(&r[0], "spirit/core/0.md", "Never declared.\n");
    declare(&r[0], "[ { Spirit core Single.0 «anywhere/at all.md» } ]\n");
    let registry = read(&r);
    assert_eq!(registry.skill_names().unwrap(), ["spirit:core:0"]);
    let resolved = registry
        .resolved_sources(&["spirit:core:0".into()])
        .unwrap();
    assert_eq!(resolved[0].string, "Core.\n");
    assert_eq!(resolved[0].source_relative_path.0, "anywhere/at all.md");
    assert!(registry.reports().is_empty());
}

#[test]
fn numeric_binding_selects_singleton_and_every_containing_span_with_actual_provenance() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "zero.md", "Zero.\n");
    file(&r[0], "one.md", "One.\n");
    file(&r[0], "zero-one.md", "Zero through one.\n");
    declare(
        &r[0],
        "[ { Vision ethos Single.0 «zero.md» }
  { Vision ethos Single.1 «one.md» }
  { Vision ethos Span.{ 0 1 } «zero-one.md» } ]\n",
    );
    let registry = read(&r);
    let resolved = registry
        .resolved_exact_sources(&["vision:ethos:1".into()])
        .unwrap();
    assert_eq!(
        resolved
            .iter()
            .map(|module| module.module_identity.0.as_str())
            .collect::<Vec<_>>(),
        ["vision:ethos:1", "vision:ethos:0-1"]
    );
    assert_eq!(
        resolved
            .iter()
            .map(|module| module.source_relative_path.0.as_str())
            .collect::<Vec<_>>(),
        ["one.md", "zero-one.md"]
    );
}

#[test]
fn span_only_and_overlapping_spans_resolve_in_deterministic_order() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    for (path, body) in [("zero-two.md", "0-2"), ("one-three.md", "1-3")] {
        file(&r[0], path, body);
    }
    declare(
        &r[0],
        "[ { Vision range Span.{ 0 2 } «zero-two.md» }
  { Vision range Span.{ 1 3 } «one-three.md» } ]\n",
    );
    let registry = read(&r);
    let identities = registry
        .resolved_exact_sources(&["vision:range:1".into()])
        .unwrap()
        .into_iter()
        .map(|module| module.module_identity.0)
        .collect::<Vec<_>>();
    assert_eq!(identities, ["vision:range:0-2", "vision:range:1-3"]);
    assert_eq!(
        registry
            .resolved_exact_sources(&["vision:range:1-3".into()])
            .unwrap()[0]
            .module_identity
            .0,
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
    file(&r[0], "first.md", "First.\n");
    file(&r[2], "second.md", "Second.\n");
    declare(&r[0], "[ { Knowledge topic Span.{ 0 1 } «first.md» } ]\n");
    declare(&r[2], "[ { Knowledge topic Span.{ 0 1 } «second.md» } ]\n");
    assert_eq!(
        SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err(),
        RegistryError::Duplicate("knowledge:topic:0-1".into())
    );
}

#[test]
fn declarations_that_cannot_be_placed_are_reported_and_leave_the_rest() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[1], "kept.md", "Kept.\n");
    fs::write(temp.path().join("outside.md"), "Outside.\n").unwrap();
    std::os::unix::fs::symlink(temp.path().join("outside.md"), r[1].join("linked.md")).unwrap();
    declare(
        &r[1],
        "[ { Knowledge kept Single.0 «kept.md» }
  { Knowledge absent Single.0 «absent.md» }
  { Knowledge linked Single.0 «linked.md» }
  { Knowledge escaping Single.0 «../outside.md» }
  { Knowledge rooted Single.0 «/etc/hostname» }
  { Knowledge Upper Single.0 «kept.md» } ]\n",
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
                DeclarationFinding::Absent("absent.md".into())
            ),
            (
                "knowledge:linked:0",
                DeclarationFinding::Unplaceable("linked.md".into())
            ),
            (
                "knowledge:escaping:0",
                DeclarationFinding::Unplaceable("../outside.md".into())
            ),
            (
                "knowledge:rooted:0",
                DeclarationFinding::Unplaceable("/etc/hostname".into())
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
        file(&r[0], "single.md", "Single.\n");
        declare(&r[0], "[ { Vision single Single.-1 «single.md» } ]\n");
        let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
        assert!(error.to_string().contains("must be nonnegative"));
    }
    for versions in ["Span.{ 0 0 }", "Span.{ 2 1 }", "Span.{ -1 1 }"] {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        file(&r[0], "range.md", "Range.\n");
        declare(
            &r[0],
            &format!("[ {{ Vision range {versions} «range.md» }} ]\n"),
        );
        let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
        assert!(
            error.to_string().contains("malformed module span"),
            "{versions}"
        );
    }

    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    declare(&r[0], "[ { Vision range Span.{ 0 1 } «missing.md» } ]\n");
    let error = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap_err();
    assert!(error.to_string().contains("vision:range:0-1"));
    assert!(error.to_string().contains("missing.md"));
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
    file(&r[0], "one.md", "One.\n");
    file(
        &r[0],
        "span.md",
        "---\ndependencies: [ vision:topic:1 ]\n---\nSpan.\n",
    );
    file(
        &r[1],
        "consumer.md",
        "---\ndependencies: [ vision:topic:1 ]\n---\nConsumer.\n",
    );
    declare(
        &r[0],
        "[ { Vision topic Single.1 «one.md» }
  { Vision topic Span.{ 0 2 } «span.md» } ]\n",
    );
    declare(&r[1], "[ { Knowledge consumer Single.0 «consumer.md» } ]\n");
    let registry = read(&r);
    let identities = registry
        .resolved_sources(&["knowledge:consumer:0".into()])
        .unwrap()
        .into_iter()
        .map(|module| module.module_identity.0)
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        ["vision:topic:1", "vision:topic:0-2", "knowledge:consumer:0"]
    );
}
