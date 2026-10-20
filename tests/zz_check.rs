use curriculum::registry::{ExpandsSkills, SkillRegistry};
use std::{fs, path::PathBuf};
#[test]
fn psyche_ids() {
    let p = PathBuf::from(std::env::var("PSY").unwrap());
    let t = tempfile::tempdir().unwrap();
    let m = t.path().join("m"); let f = t.path().join("f");
    fs::create_dir_all(&m).unwrap(); fs::create_dir_all(&f).unwrap();
    let revs = ["a".repeat(40), "b".repeat(40), "c".repeat(40)];
    let r = SkillRegistry::from_context_repositories(&[p, m, f], &revs).unwrap();
    let mut n = r.skill_names().unwrap(); n.sort();
    fs::write(std::env::var("OUT").unwrap(), n.join("\n")).unwrap();
}
