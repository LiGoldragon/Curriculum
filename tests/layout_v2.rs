use curriculum::registry::{ExpandsSkills, SkillRegistry};
use std::{fs, path::PathBuf};

fn roots(base: &std::path::Path) -> [PathBuf; 3] {
    let roots = [base.join("psyche"), base.join("mind"), base.join("field")];
    for root in &roots {
        fs::create_dir_all(root).unwrap();
    }
    roots
}
fn revisions() -> [String; 3] {
    ["a".repeat(40), "b".repeat(40), "c".repeat(40)]
}
fn source(root: &std::path::Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn source_api_exposes_registry_address_and_declared_placement() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[0], "vision/book.md", "Book vision.\n");
    source(
        &r[2],
        "compensation/illustration.md",
        "---\ndependencies: [ vision-book ]\nplacement: system\n---\nExact approved text.\n",
    );
    let registry = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap();
    let ids = vec!["compensation-illustration".into()];
    let resolved = registry.resolved_sources(&ids).unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[1].repository_name, "field-context");
    assert_eq!(
        resolved[1].source_relative_path,
        "compensation/illustration.md"
    );
    assert_eq!(resolved[1].source_revision, "c".repeat(40));
    assert_eq!(resolved[1].declared_placement, "system");
    assert_eq!(resolved[1].string, "Exact approved text.\n");
    assert_eq!(registry.resolve_exact(&ids).unwrap().len(), 1);
}

#[test]
fn mixed_duplicate_case_nested_and_escape_layouts_refuse() {
    for case in 0..7 {
        let temp = tempfile::tempdir().unwrap();
        let r = roots(temp.path());
        source(&r[1], "knowledge/topic.md", "Known.\n");
        match case {
            0 => {
                fs::create_dir(r[2].join("skills")).unwrap();
            }
            1 => source(&r[2], "knowledge/topic.md", "Duplicate.\n"),
            2 => source(&r[1], "knowledge/Topic.md", "Wrong case.\n"),
            3 => source(&r[1], "knowledge/nested/topic.md", "Nested.\n"),
            4 => {
                fs::write(temp.path().join("outside.md"), "Outside.").unwrap();
                std::os::unix::fs::symlink(
                    temp.path().join("outside.md"),
                    r[1].join("knowledge/escape.md"),
                )
                .unwrap();
            }
            5 => {
                fs::create_dir(temp.path().join("outside")).unwrap();
                std::os::unix::fs::symlink(temp.path().join("outside"), r[2].join("compensation"))
                    .unwrap();
            }
            _ => source(
                &r[1],
                "knowledge/missing.md",
                "---\ndependencies: [ knowledge-absent ]\n---\nMissing.\n",
            ),
        }
        assert!(
            SkillRegistry::from_context_repositories(&r, &revisions()).is_err(),
            "case {case}"
        );
    }
}

#[test]
fn same_subject_different_types_are_distinct() {
    let temp = tempfile::tempdir().unwrap();
    let r = roots(temp.path());
    source(&r[1], "knowledge/topic.md", "Knowledge.");
    source(&r[1], "inquiry/topic.md", "Inquiry.");
    let registry = SkillRegistry::from_context_repositories(&r, &revisions()).unwrap();
    assert_eq!(
        registry.skill_names().unwrap(),
        vec!["inquiry-topic", "knowledge-topic"]
    );
    assert!(registry.resolve_exact(&["topic".into()]).is_err());
}
