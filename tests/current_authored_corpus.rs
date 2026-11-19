use curriculum::registry::{IdentityText, SkillRegistry};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/current-main-25858e")
}

#[test]
fn verbose_lists_every_current_authored_module_with_exact_source_and_description() {
    let base = fixture();
    let roots = ["psyche-belief", "mind-belief", "field-belief"].map(|name| base.join(name));
    let revisions = [
        "faf30d66c36f919242f1cbf9aaf4c5195271b5f7",
        "a27a6a3633caaf2c172e03e6b2e72a86c9112d5c",
        "7eba46a2d1e70e10dead070d6aa6ab33beaa5d5c",
    ]
    .map(String::from);
    let registry = SkillRegistry::from_context_repositories(&roots, &revisions).unwrap();
    let ordinary = registry.listed_modules().unwrap();
    let verbose = registry.listed_modules_verbose().unwrap();
    let oracle = include_str!("fixtures/current-main-25858e/module-oracle.tsv");
    assert_eq!(ordinary.len(), 130);
    assert_eq!(verbose.len(), 130);
    assert_eq!(oracle.lines().count(), 130);
    for line in oracle.lines() {
        let fields: Vec<_> = line.splitn(5, '\t').collect();
        let [repository, identity, relative, hash, description] = fields.as_slice() else {
            panic!("invalid oracle row")
        };
        let path = base.join(repository).join(relative);
        assert_eq!(
            format!("{:x}", Sha256::digest(fs::read(&path).unwrap())),
            *hash
        );
        let found: Vec<_> = verbose
            .iter()
            .filter(|entry| entry.module.module_identity.identity_text().unwrap() == *identity)
            .collect();
        assert_eq!(found.len(), 1, "{identity}");
        assert_eq!(PathBuf::from(&found[0].module.module_file.0), path);
        assert_eq!(found[0].description_text.0, *description, "{identity}");
    }
}

#[test]
fn literal_flat_current_manifests_remain_an_explicit_compatibility_boundary() {
    let temp = tempfile::tempdir().unwrap();
    let roots = ["psyche-belief", "mind-belief", "field-belief"].map(|name| {
        let root = temp.path().join(name);
        fs::create_dir_all(&root).unwrap();
        fs::copy(
            fixture().join(name).join("modules.flat.datom"),
            root.join("modules.datom"),
        )
        .unwrap();
        root
    });
    assert!(SkillRegistry::from_context_repositories(
        &roots,
        &["a".repeat(40), "b".repeat(40), "c".repeat(40)]
    )
    .is_err());
}
