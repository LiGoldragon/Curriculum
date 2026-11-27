//! Fixture sources are written as files under kind/topic/version.md; each
//! fixture root is then given a modules.datom declaring every such file, so
//! the registry reads them by their declarations.
#![allow(dead_code)]
use curriculum::registry::{MODULE_KINDS, RegistryError, SkillRegistry, Worded};
use std::{fs, path::Path, path::PathBuf};

pub fn identity(text: &str) -> signal_curriculum_nexus::ModuleIdentity {
    let mut fields = text.split(':');
    let Some(kind) = fields.next() else {
        return invalid_identity(text);
    };
    let module_kind = match kind {
        "spirit" => signal_curriculum_nexus::ModuleKind::Spirit,
        "intent" => signal_curriculum_nexus::ModuleKind::Intent,
        "vision" => signal_curriculum_nexus::ModuleKind::Vision,
        "knowledge" => signal_curriculum_nexus::ModuleKind::Knowledge,
        "inquiry" => signal_curriculum_nexus::ModuleKind::Inquiry,
        "compensation" => signal_curriculum_nexus::ModuleKind::Compensation,
        "role" => signal_curriculum_nexus::ModuleKind::Role,
        "raw" => signal_curriculum_nexus::ModuleKind::Raw,
        "notion" => signal_curriculum_nexus::ModuleKind::Notion,
        "state" => signal_curriculum_nexus::ModuleKind::State,
        _ => return invalid_identity(text),
    };
    let Some(subject) = fields.next() else {
        return invalid_identity(text);
    };
    let Some(version_text) = fields.next() else {
        return invalid_identity(text);
    };
    if (version_text.len() > 1 && version_text.starts_with('0')) || fields.next().is_some() {
        return invalid_identity(text);
    }
    let Some(version) = version_text.parse().ok() else {
        return invalid_identity(text);
    };
    let module_subject = signal_curriculum_nexus::ModuleSubject(subject.into());
    signal_curriculum_nexus::ModuleIdentity {
        module_kind,
        module_subject,
        versions: signal_curriculum_nexus::Versions::Single(signal_curriculum_nexus::Version(
            version,
        )),
    }
}

fn invalid_identity(text: &str) -> signal_curriculum_nexus::ModuleIdentity {
    signal_curriculum_nexus::ModuleIdentity {
        module_kind: signal_curriculum_nexus::ModuleKind::Vision,
        module_subject: signal_curriculum_nexus::ModuleSubject(text.into()),
        versions: signal_curriculum_nexus::Versions::Single(signal_curriculum_nexus::Version(0)),
    }
}

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
                    "{{ {{ {:?} {} Single.{version} }} «{}» }}",
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
