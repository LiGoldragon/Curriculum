//! Fixture sources are written as files under kind/topic/version.md; each
//! fixture root is then given a modules.datom declaring every such file, so
//! the registry reads them by their declarations.
#![allow(dead_code)]
use curriculum::registry::{MODULE_KINDS, RegistryError, SkillRegistry, Worded};
use std::{fs, path::Path, path::PathBuf};

/// Writes the root's modules.datom: one declaration per kind/topic/version.md file.
pub fn declare(root: &Path) {
    let mut declarations = Vec::new();
    for kind in MODULE_KINDS.iter() {
        let Ok(topics) = fs::read_dir(root.join(kind.word())) else {
            continue;
        };
        let mut topics: Vec<_> = topics.map(|entry| entry.unwrap().path()).collect();
        topics.sort();
        for topic in topics.iter().filter(|path| path.is_dir()) {
            let mut files: Vec<_> = fs::read_dir(topic)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            files.sort();
            for file in files {
                let Some(version) = file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| name.strip_suffix(".md"))
                    .and_then(|stem| stem.parse::<u64>().ok())
                else {
                    continue;
                };
                declarations.push(format!(
                    "{{ {:?} {} [ {version} ] «{}» }}",
                    kind,
                    topic.file_name().unwrap().to_str().unwrap(),
                    file.strip_prefix(root).unwrap().display()
                ));
            }
        }
    }
    fs::write(
        root.join("modules.datom"),
        format!("[ {} ]\n", declarations.join("\n  ")),
    )
    .unwrap();
}

/// Declares every fixture file in each root, then reads the registry.
pub fn registry(
    roots: &[PathBuf; 3],
    revisions: &[String; 3],
) -> Result<SkillRegistry, RegistryError> {
    for root in roots {
        declare(root);
    }
    SkillRegistry::from_context_repositories(roots, revisions)
}
