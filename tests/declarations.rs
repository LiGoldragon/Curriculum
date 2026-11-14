//! A source's modules are what its modules.datom declares; a file's own name
//! and folder mean nothing.
use curriculum::registry::{DeclarationFinding, ExpandsSkills, SkillRegistry};
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
    declare(&r[0], "[ { Spirit core [ 0 ] «anywhere/at all.md» } ]\n");
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
fn one_declaration_holds_for_every_version_it_names() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "ethos.md", "Ethos.\n");
    file(
        &r[1],
        "consumer.md",
        "---\ndependencies: [ vision:ethos:1 ]\n---\nConsumer.\n",
    );
    declare(&r[0], "[ { Vision ethos [ 0 1 ] «ethos.md» } ]\n");
    declare(&r[1], "[ { Knowledge consumer [ 0 ] «consumer.md» } ]\n");
    let registry = read(&r);
    assert_eq!(
        registry.skill_names().unwrap(),
        ["knowledge:consumer:0", "vision:ethos:0", "vision:ethos:1"]
    );
    let ids: Vec<_> = registry
        .resolved_sources(&["knowledge:consumer:0".into()])
        .unwrap()
        .into_iter()
        .map(|module| module.module_identity.0)
        .collect();
    assert_eq!(ids, ["vision:ethos:1", "knowledge:consumer:0"]);
}

#[test]
fn a_name_declared_twice_is_given_by_the_first_and_reported() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    file(&r[0], "first.md", "First.\n");
    file(&r[0], "second.md", "Second.\n");
    file(&r[2], "third.md", "Third.\n");
    declare(
        &r[0],
        "[ { Knowledge topic [ 0 ] «first.md» }\n  { Knowledge topic [ 0 1 ] «second.md» } ]\n",
    );
    declare(&r[2], "[ { Knowledge topic [ 1 ] «third.md» } ]\n");
    let registry = read(&r);
    let body = |id: &str| {
        registry.resolve_exact(&[id.into()]).unwrap()[0]
            .body
            .clone()
    };
    assert_eq!(body("knowledge:topic:0"), "First.\n");
    assert_eq!(body("knowledge:topic:1"), "Second.\n");
    let reported: Vec<_> = registry
        .reports()
        .iter()
        .map(|report| (report.name.as_str(), report.finding.clone()))
        .collect();
    let root = fs::canonicalize(&r[0]).unwrap();
    assert_eq!(
        reported,
        [
            (
                "knowledge:topic:0",
                DeclarationFinding::HeldBefore(root.join("first.md"))
            ),
            (
                "knowledge:topic:1",
                DeclarationFinding::HeldBefore(root.join("second.md"))
            ),
        ]
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
        "[ { Knowledge kept [ 0 ] «kept.md» }
  { Knowledge absent [ 0 ] «absent.md» }
  { Knowledge linked [ 0 ] «linked.md» }
  { Knowledge escaping [ 0 ] «../outside.md» }
  { Knowledge rooted [ 0 ] «/etc/hostname» }
  { Knowledge Upper [ 0 ] «kept.md» }
  { Knowledge none [] «kept.md» } ]\n",
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
            ("knowledge:none", DeclarationFinding::NoVersion),
        ]
    );
}

#[test]
fn a_source_without_readable_declarations_refuses() {
    for declarations in [
        None,
        Some("{ not a vector }\n"),
        Some("[ { Unknown topic [ 0 ] «x.md» } ]\n"),
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
