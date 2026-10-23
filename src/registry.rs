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

/// Projects the authoritative registry into plain harness skill directories.
/// All source text comes from the resolved registry; this code never scans a
/// second skill repository or follows generated symlinks.
pub mod workspace {
    use super::{invalid, sha256, ExpandsSkills, RegistryError, ResolvedSkill, SkillRegistry};
    use crate::generated::{ProjectionCandidate, RepositoryRevision};
    use std::{
        collections::BTreeMap,
        fs,
        io::Write,
        path::{Component, Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    pub const RENDERER_VERSION: &str = "curriculum-workspace-projector-v1";

    /// Typed identity of a fully materialized workspace generation. This is
    /// returned only after all projection writes and the canonical inventory.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Target {
        Claude,
        Codex,
        CodexAlt,
        Pi,
        OpenCode,
    }

    impl Target {
        const ALL: [Self; 5] = [
            Self::Claude,
            Self::Codex,
            Self::CodexAlt,
            Self::Pi,
            Self::OpenCode,
        ];

        fn directory(self) -> &'static str {
            match self {
                Self::Claude => ".claude/skills",
                Self::Codex => ".agents/skills",
                Self::CodexAlt => ".codex/skills",
                Self::Pi => ".pi/skills",
                Self::OpenCode => ".opencode/skills",
            }
        }

        fn render(self, skill: &ResolvedSkill) -> Result<String, RegistryError> {
            let mut rendered = String::new();
            let mut branches = Vec::<(bool, bool)>::new();
            let mut raw = false;

            for (index, line) in skill.body.split_inclusive('\n').enumerate() {
                let line_number = index + 1;
                let directive = line.trim();
                if raw {
                    if directive == "{% endraw %}" {
                        raw = false;
                    } else if branches.iter().all(|(selected, _)| *selected) {
                        rendered.push_str(line);
                    }
                    continue;
                }

                let selected = match directive {
                    "{% if claude %}" => Some(self == Self::Claude),
                    "{% if codex %}" => Some(matches!(self, Self::Codex | Self::CodexAlt)),
                    "{% if opencode %}" => Some(self == Self::OpenCode),
                    "{% if pi %}" => Some(self == Self::Pi),
                    "{% raw %}" => {
                        raw = true;
                        continue;
                    }
                    "{% endraw %}" => {
                        return Err(invalid(
                            &skill.path,
                            format!("line {line_number}: endraw without raw"),
                        ));
                    }
                    "{% else %}" => {
                        let Some((selected, saw_else)) = branches.last_mut() else {
                            return Err(invalid(
                                &skill.path,
                                format!("line {line_number}: else without if"),
                            ));
                        };
                        if *saw_else {
                            return Err(invalid(
                                &skill.path,
                                format!("line {line_number}: repeated else"),
                            ));
                        }
                        *selected = !*selected;
                        *saw_else = true;
                        continue;
                    }
                    "{% endif %}" => {
                        if branches.pop().is_none() {
                            return Err(invalid(
                                &skill.path,
                                format!("line {line_number}: endif without if"),
                            ));
                        }
                        continue;
                    }
                    _ => None,
                };
                if let Some(selected) = selected {
                    branches.push((selected, false));
                    continue;
                }
                if branches.iter().all(|(selected, _)| *selected) {
                    if self == Self::Claude && directive == "user-only: true" {
                        rendered.push_str("disable-model-invocation: true\n");
                    } else {
                        rendered.push_str(line);
                    }
                }
            }

            if !branches.is_empty() || raw {
                return Err(invalid(
                    &skill.path,
                    "unclosed target conditional or raw block",
                ));
            }
            if self == Self::OpenCode {
                let name = format!("name: {}\n", skill.name);
                rendered = match rendered.strip_prefix("---\n") {
                    Some(rest) => format!("---\n{name}{rest}"),
                    None => format!("---\n{name}---\n{rendered}"),
                };
            }
            Ok(rendered)
        }
    }

    pub struct WorkspaceProjector {
        root: PathBuf,
    }

    impl WorkspaceProjector {
        pub fn new(root: impl Into<PathBuf>) -> Self {
            Self { root: root.into() }
        }

        pub fn write(&self, registry: &SkillRegistry) -> Result<(), RegistryError> {
            self.project(registry).map(|_| ())
        }

        pub fn project(
            &self,
            registry: &SkillRegistry,
        ) -> Result<ProjectionCandidate, RegistryError> {
            fs::create_dir_all(&self.root).map_err(|e| invalid(&self.root, e))?;
            let ids = registry.skill_names()?;
            let skills = registry.resolve_exact(&ids)?;
            let modules = registry.resolved_exact_sources(&ids)?;
            let mut outputs = BTreeMap::<PathBuf, String>::new();
            for target in Target::ALL {
                self.remove_stale(target, &ids)?;
            }
            for skill in &skills {
                for target in Target::ALL {
                    let relative = Path::new(target.directory())
                        .join(&skill.name)
                        .join("SKILL.md");
                    outputs.insert(relative, target.render(skill)?);
                }
                if user_only(&skill.body) {
                    for surface in [".agents/skills", ".codex/skills"] {
                        outputs.insert(
                            Path::new(surface)
                                .join(&skill.name)
                                .join("agents/openai.yaml"),
                            "policy:\n  allow_implicit_invocation: false\n".into(),
                        );
                    }
                }
            }
            let inventory = modules
                .iter()
                .map(|m| {
                    format!(
                        "{} {} {}\n",
                        m.module_identity, m.source_revision, m.source_relative_path
                    )
                })
                .collect::<String>();
            outputs.insert(PathBuf::from("modules"), inventory.clone());
            let mut content_manifest = String::new();
            for (relative, contents) in &outputs {
                content_manifest.push_str(&format!(
                    "{} {}\n",
                    relative.to_string_lossy(),
                    sha256(contents.as_bytes())
                ));
            }
            for (relative, contents) in &outputs {
                let path = self.safe(relative)?;
                let parent = path.parent().expect("skill output parent");
                fs::create_dir_all(parent).map_err(|e| invalid(parent, e))?;
                write_atomic(&path, contents.clone())?;
            }
            let destination = fs::canonicalize(&self.root).map_err(|e| invalid(&self.root, e))?;
            let repositories = registry
                .roots
                .iter()
                .zip(registry.revisions.iter())
                .enumerate()
                .map(|(index, (_root, revision))| RepositoryRevision {
                    repository_name: ["psyche-context", "mind-context", "field-context"][index]
                        .into(),
                    source_revision: revision.clone(),
                })
                .collect();
            Ok(ProjectionCandidate {
                deployment_destination: destination.to_string_lossy().into_owned(),
                repository_revision_vector: repositories,
                published_inventory_digest: sha256(inventory.as_bytes()),
                published_content_digest: sha256(content_manifest.as_bytes()),
                renderer_version: RENDERER_VERSION.into(),
            })
        }

        fn remove_stale(&self, target: Target, current: &[String]) -> Result<(), RegistryError> {
            // A prior inventory is the ownership receipt. Without one, existing
            // directories may belong to the user and are left untouched.
            let inventory = self.safe(Path::new("modules"))?;
            let previous = match fs::read_to_string(&inventory) {
                Ok(text) => text
                    .lines()
                    .filter_map(|line| line.split_whitespace().next().map(str::to_owned))
                    .collect::<Vec<_>>(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(invalid(&inventory, error)),
            };
            let surface = self.safe(Path::new(target.directory()))?;
            let entries = match fs::read_dir(&surface) {
                Ok(entries) => entries,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(e) => return Err(invalid(&surface, e)),
            };
            for entry in entries {
                let entry = entry.map_err(|e| invalid(&surface, e))?;
                let name = entry.file_name().to_string_lossy().into_owned();
                let path = entry.path();
                let metadata = fs::symlink_metadata(&path).map_err(|e| invalid(&path, e))?;
                if previous.iter().any(|id| id == &name)
                    && !current.iter().any(|id| id == &name)
                    && !metadata.file_type().is_symlink()
                    && metadata.is_dir()
                {
                    fs::remove_dir_all(&path).map_err(|e| invalid(&path, e))?;
                }
            }
            Ok(())
        }

        fn safe(&self, relative: &Path) -> Result<PathBuf, RegistryError> {
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
            {
                return Err(invalid(relative, "workspace output escapes its root"));
            }
            let root = fs::canonicalize(&self.root).map_err(|e| invalid(&self.root, e))?;
            let components = relative.components().collect::<Vec<_>>();
            let mut path = root;
            for (index, component) in components.iter().enumerate() {
                path.push(component.as_os_str());
                match fs::symlink_metadata(&path) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        return Err(invalid(&path, "workspace output crosses a symlink"));
                    }
                    Ok(metadata) if index + 1 < components.len() && !metadata.is_dir() => {
                        return Err(invalid(&path, "workspace output parent is not a directory"));
                    }
                    Ok(_) => (),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                    Err(error) => return Err(invalid(&path, error)),
                }
            }
            Ok(path)
        }
    }

    fn write_atomic(path: &Path, contents: String) -> Result<(), RegistryError> {
        let parent = path.parent().expect("workspace output parent");
        let mut temporary = None;
        for _ in 0..16 {
            let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            let candidate = parent.join(format!(".curriculum-{}-{serial}.tmp", std::process::id()));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate)
            {
                Ok(mut file) => {
                    if let Err(error) = file.write_all(contents.as_bytes()) {
                        let _ = fs::remove_file(&candidate);
                        return Err(invalid(&candidate, error));
                    }
                    if let Err(error) = fs::rename(&candidate, path) {
                        let _ = fs::remove_file(&candidate);
                        return Err(invalid(path, error));
                    }
                    temporary = Some(());
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(invalid(&candidate, error)),
            }
        }
        if temporary.is_none() {
            return Err(invalid(path, "cannot allocate an atomic output file"));
        }
        Ok(())
    }

    fn user_only(body: &str) -> bool {
        body.strip_prefix("---\n")
            .and_then(|body| body.split_once("\n---\n"))
            .is_some_and(|(frontmatter, _)| {
                frontmatter
                    .lines()
                    .any(|line| line.trim() == "user-only: true")
            })
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
