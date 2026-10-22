//! The sole authored module registry: three server-configured repository roots.
use crate::startup_metadata::{declared_placement, instructions, sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use thiserror::Error;

pub const MODULE_TYPES: &[&str] = &[
    "vision",
    "intent",
    "spirit",
    "knowledge",
    "inquiry",
    "compensation",
    "role",
    "raw",
];
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillEntry {
    pub path: PathBuf,
    pub name: String,
    pub body: String,
    pub dependencies: Vec<String>,
}
pub type ResolvedSkill = SkillEntry;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillRegistry {
    entries: BTreeMap<String, SkillEntry>,
    roots: [PathBuf; 3],
    revisions: [String; 3],
}
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("invalid context source {path}: {message}")]
    InvalidSource { path: PathBuf, message: String },
    #[error("missing module {0}")]
    Missing(String),
    #[error("duplicate module {0}")]
    Duplicate(String),
    #[error("dependency cycle at {0}")]
    Cycle(String),
}
fn invalid(path: &Path, message: impl ToString) -> RegistryError {
    RegistryError::InvalidSource {
        path: path.into(),
        message: message.to_string(),
    }
}
fn canonical(path: &Path) -> Result<PathBuf, RegistryError> {
    fs::canonicalize(path).map_err(|e| invalid(path, e))
}
pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceConfiguration {
    pub roots: [PathBuf; 3],
    pub refresh_program: PathBuf,
}
impl SourceConfiguration {
    pub fn from_environment() -> Result<Self, String> {
        if std::env::var("CURRICULUM_CONTEXT_LAYOUT").as_deref() != Ok("2") {
            return Err("only authored context layout 2 is supported".into());
        }
        let mut roots: [PathBuf; 3] = Default::default();
        for (i, root) in [
            "CURRICULUM_PSYCHES_REPOSITORY_DIR",
            "CURRICULUM_MIND_CONTEXT_DIR",
            "CURRICULUM_FIELD_CONTEXT_DIR",
        ]
        .iter()
        .enumerate()
        {
            roots[i] = std::env::var_os(root)
                .map(PathBuf::from)
                .ok_or_else(|| format!("missing {root}"))?;
        }
        let refresh_program = std::env::var_os("CURRICULUM_CONTEXT_REFRESH_PROGRAM")
            .map(PathBuf::from)
            .ok_or_else(|| "missing CURRICULUM_CONTEXT_REFRESH_PROGRAM".to_owned())?;
        Ok(Self {
            roots,
            refresh_program,
        })
    }
    pub fn read(&self) -> Result<SkillRegistry, RegistryError> {
        let refreshed = Command::new(&self.refresh_program).status().map_err(|e| {
            invalid(
                &self.refresh_program,
                format!("refresh current context: {e}"),
            )
        })?;
        if !refreshed.success() {
            return Err(invalid(&self.refresh_program, "context refresh failed"));
        }
        let revisions = self
            .roots
            .iter()
            .map(|root| repository_revision(root))
            .collect::<Result<Vec<_>, _>>()?;
        let revisions: [String; 3] = revisions.try_into().expect("three source roots");
        SkillRegistry::from_context_repositories(&self.roots, &revisions)
    }
}

fn repository_revision(root: &Path) -> Result<String, RegistryError> {
    let head_path = root.join(".git/HEAD");
    let head = fs::read_to_string(&head_path)
        .map_err(|e| invalid(&head_path, format!("read current Git revision: {e}")))?;
    let value = head.trim();
    let revision = if let Some(reference) = value.strip_prefix("ref: ") {
        if !reference.starts_with("refs/heads/")
            || reference
                .split('/')
                .any(|part| part == ".." || part.is_empty())
        {
            return Err(invalid(root, "Git HEAD is not a local branch"));
        }
        let reference_path = root.join(".git").join(reference);
        match fs::read_to_string(&reference_path) {
            Ok(revision) => revision.trim().to_owned(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let packed = fs::read_to_string(root.join(".git/packed-refs"))
                    .map_err(|e| invalid(root, format!("read packed Git refs: {e}")))?;
                packed
                    .lines()
                    .find_map(|line| {
                        let mut fields = line.split_whitespace();
                        let revision = fields.next()?;
                        let name = fields.next()?;
                        (name == reference).then(|| revision.to_owned())
                    })
                    .ok_or_else(|| invalid(root, "current Git branch has no commit"))?
            }
            Err(error) => return Err(invalid(&reference_path, error)),
        }
    } else {
        value.to_owned()
    };
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid(
            root,
            "current Git revision must be 40 lowercase hex",
        ));
    }
    Ok(revision)
}

impl SkillRegistry {
    pub fn from_context_repositories(
        input: &[PathBuf; 3],
        revisions: &[String; 3],
    ) -> Result<Self, RegistryError> {
        let roots = [
            canonical(&input[0])?,
            canonical(&input[1])?,
            canonical(&input[2])?,
        ];
        let mut entries = BTreeMap::new();
        for (i, root) in roots.iter().enumerate() {
            if revisions[i].len() != 40
                || !revisions[i]
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(invalid(
                    root,
                    "immutable source revision must be 40 lowercase hex",
                ));
            }
            if roots
                .iter()
                .enumerate()
                .any(|(j, r)| i != j && (root.starts_with(r) || r.starts_with(root)))
            {
                return Err(invalid(root, "repository roots overlap"));
            }
            if fs::symlink_metadata(root.join("skills")).is_ok() {
                return Err(invalid(root, "legacy skills directory forbidden"));
            }
            for kind in MODULE_TYPES {
                let directory = root.join(kind);
                match fs::symlink_metadata(&directory) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => return Err(invalid(&directory, e)),
                    Ok(m) if !m.is_dir() || m.file_type().is_symlink() => {
                        return Err(invalid(
                            &directory,
                            "type folder must be an actual directory",
                        ));
                    }
                    _ => (),
                }
                for entry in fs::read_dir(&directory).map_err(|e| invalid(&directory, e))? {
                    let path = entry.map_err(|e| invalid(&directory, e))?.path();
                    let metadata = fs::symlink_metadata(&path).map_err(|e| invalid(&path, e))?;
                    if !metadata.is_file() || metadata.file_type().is_symlink() {
                        return Err(invalid(
                            &path,
                            "nested directories and symlinks are forbidden",
                        ));
                    }
                    if path.extension().is_none_or(|e| e != "md") {
                        return Err(invalid(
                            &path,
                            "module type folder contains non-Markdown source",
                        ));
                    }
                    let subject = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default();
                    if !valid_id(subject) {
                        return Err(invalid(&path, "invalid subject"));
                    }
                    let name = format!("{kind}-{subject}");
                    let body = fs::read_to_string(&path).map_err(|e| invalid(&path, e))?;
                    let text = instructions(&body).map_err(|e| invalid(&path, e))?;
                    if text.trim().is_empty() {
                        return Err(invalid(&path, "empty body"));
                    }
                    let dependencies = dependencies(&path, &body)?;
                    if entries
                        .insert(
                            name.clone(),
                            SkillEntry {
                                path,
                                name: name.clone(),
                                body,
                                dependencies,
                            },
                        )
                        .is_some()
                    {
                        return Err(RegistryError::Duplicate(name));
                    }
                }
            }
        }
        let registry = Self {
            entries,
            roots,
            revisions: revisions.clone(),
        };
        registry.expand_skills(&registry.skill_names()?)?;
        Ok(registry)
    }
    pub fn resolve_exact(&self, ids: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError> {
        let mut seen = BTreeSet::new();
        ids.iter()
            .map(|id| {
                if !valid_id(id) {
                    return Err(RegistryError::Missing(id.clone()));
                }
                if !seen.insert(id) {
                    return Err(RegistryError::Duplicate(id.clone()));
                }
                self.entries
                    .get(id)
                    .cloned()
                    .ok_or_else(|| RegistryError::Missing(id.clone()))
            })
            .collect()
    }
    pub fn resolved_exact_sources(
        &self,
        ids: &[String],
    ) -> Result<Vec<crate::generated::ResolvedModule>, RegistryError> {
        self.resolve_exact(ids)?
            .into_iter()
            .map(|s| self.describe(s))
            .collect()
    }
    pub fn resolved_sources(
        &self,
        ids: &[String],
    ) -> Result<Vec<crate::generated::ResolvedModule>, RegistryError> {
        self.expand_skills(ids)?
            .into_iter()
            .map(|s| self.describe(s))
            .collect()
    }
    fn describe(
        &self,
        skill: ResolvedSkill,
    ) -> Result<crate::generated::ResolvedModule, RegistryError> {
        if canonical(&skill.path)? != skill.path
            || fs::read(&skill.path).map_err(|e| invalid(&skill.path, e))? != skill.body.as_bytes()
        {
            return Err(invalid(
                &skill.path,
                "source changed since registry admission",
            ));
        }
        let i = self
            .roots
            .iter()
            .position(|r| skill.path.starts_with(r))
            .ok_or_else(|| invalid(&skill.path, "outside configured roots"))?;
        let (kind, subject) = skill
            .name
            .split_once('-')
            .ok_or_else(|| invalid(&skill.path, "invalid identity"))?;
        let text = instructions(&skill.body).map_err(|e| invalid(&skill.path, e))?;
        Ok(crate::generated::ResolvedModule {
            module_identity: skill.name.clone(),
            repository_name: ["psyche-context", "mind-context", "field-context"][i].into(),
            source_revision: self.revisions[i].clone(),
            source_root: self.roots[i].to_string_lossy().into_owned(),
            source_relative_path: skill
                .path
                .strip_prefix(&self.roots[i])
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            module_type: kind.into(),
            module_subject: subject.into(),
            declared_placement: declared_placement(&skill.body).into(),
            source_sha256: sha256(skill.body.as_bytes()),
            body_sha256: sha256(text.as_bytes()),
            string: text.to_owned(),
            string_vector: skill.dependencies,
        })
    }
    fn visit(
        &self,
        id: &str,
        visiting: &mut BTreeSet<String>,
        seen: &mut BTreeSet<String>,
        output: &mut Vec<ResolvedSkill>,
    ) -> Result<(), RegistryError> {
        if seen.contains(id) {
            return Ok(());
        }
        if !visiting.insert(id.into()) {
            return Err(RegistryError::Cycle(id.into()));
        }
        let source = self
            .entries
            .get(id)
            .ok_or_else(|| RegistryError::Missing(id.into()))?;
        for dependency in &source.dependencies {
            self.visit(dependency, visiting, seen, output)?;
        }
        visiting.remove(id);
        seen.insert(id.into());
        output.push(source.clone());
        Ok(())
    }
}
pub trait ExpandsSkills {
    fn expand_skills(&self, roots: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError>;
    fn skill_names(&self) -> Result<Vec<String>, RegistryError>;
}
impl ExpandsSkills for SkillRegistry {
    fn expand_skills(&self, roots: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError> {
        let mut visiting = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut output = Vec::new();
        for root in roots {
            self.visit(root, &mut visiting, &mut seen, &mut output)?;
        }
        Ok(output)
    }
    fn skill_names(&self) -> Result<Vec<String>, RegistryError> {
        Ok(self.entries.keys().cloned().collect())
    }
}
fn dependencies(path: &Path, body: &str) -> Result<Vec<String>, RegistryError> {
    let Some((header, _)) = body
        .strip_prefix("---\n")
        .and_then(|b| b.split_once("\n---\n"))
    else {
        return Ok(vec![]);
    };
    let Some(value) = header.lines().find_map(|l| l.strip_prefix("dependencies:")) else {
        return Ok(vec![]);
    };
    let value = value.trim();
    let Some(inner) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) else {
        return Err(invalid(path, "dependencies must be an inline ID list"));
    };
    let ids: Vec<String> = inner
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let mut seen = BTreeSet::new();
    if ids.iter().any(|id| !valid_id(id) || !seen.insert(id)) {
        return Err(invalid(path, "invalid or duplicate dependency ID"));
    }
    Ok(ids)
}
