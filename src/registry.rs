//! The sole authored module registry: three server-configured repository roots, each holding the modules its modules.datom declares.
use crate::emission::{Emittable, RefreshOutcome, RefreshProgram};
use crate::generated::ModuleDeclarations;
use crate::refresh::ProcessLauncher;
use crate::startup_metadata::{
    declared_placement, description, framed_source, instructions, sha256,
};
use signal_curriculum_nexus::{
    BodySha256, DeclaredPlacement, Module, ModuleIdentity, ModuleKind, ModuleSubject,
    RepositoryName, ResolvedModule, SourceRelativePath, SourceRevision, SourceRoot, SourceSha256,
    Versions,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Every module kind, in the order the kinds are declared.
pub const MODULE_KINDS: [ModuleKind; 10] = [
    ModuleKind::Spirit,
    ModuleKind::Intent,
    ModuleKind::Vision,
    ModuleKind::Knowledge,
    ModuleKind::Inquiry,
    ModuleKind::Compensation,
    ModuleKind::Role,
    ModuleKind::Raw,
    ModuleKind::Notion,
    ModuleKind::Panel,
];

/// The file at a source's root that declares the modules the source holds.
pub const DECLARATIONS_FILE: &str = "modules.datom";

/// A module kind as it is written in a module's name.
pub trait Worded {
    fn word(&self) -> &'static str;
}

pub trait IdentityText {
    fn identity_text(&self) -> Result<String, String>;
}
impl IdentityText for ModuleIdentity {
    fn identity_text(&self) -> Result<String, String> {
        let version = self.versions.version_name()?;
        if !valid_segment(&self.module_subject.0) {
            return Err(format!(
                "invalid module subject {:?}",
                self.module_subject.0
            ));
        }
        Ok(format!(
            "{}:{}:{version}",
            self.module_kind.word(),
            self.module_subject.0
        ))
    }
}

/// Converts the registry's canonical textual key to the wire identity.
/// This is the only legacy-key parser used by service, roles and startup.
pub trait ModuleIdentityConversion {
    fn module_identity(&self) -> Result<ModuleIdentity, String>;
}
impl ModuleIdentityConversion for str {
    fn module_identity(&self) -> Result<ModuleIdentity, String> {
        let parsed = self
            .parsed_identity()
            .ok_or_else(|| format!("invalid module identity {self}"))?;
        let module_kind = MODULE_KINDS
            .iter()
            .find(|kind| kind.word() == parsed.kind)
            .cloned()
            .ok_or_else(|| format!("invalid module kind in {self}"))?;
        let versions = match parsed.version {
            BoundVersion::Single(v) => Versions::Single(signal_curriculum_nexus::Version(v)),
            BoundVersion::Span { first, last } => {
                Versions::Span(signal_curriculum_nexus::Span_Data {
                    first: signal_curriculum_nexus::First(signal_curriculum_nexus::Version(first)),
                    last: signal_curriculum_nexus::Last(signal_curriculum_nexus::Version(last)),
                })
            }
        };
        Ok(ModuleIdentity {
            module_kind,
            module_subject: ModuleSubject(parsed.topic.into()),
            versions,
        })
    }
}
impl Worded for ModuleKind {
    fn word(&self) -> &'static str {
        match self {
            ModuleKind::Spirit => "spirit",
            ModuleKind::Intent => "intent",
            ModuleKind::Vision => "vision",
            ModuleKind::Knowledge => "knowledge",
            ModuleKind::Inquiry => "inquiry",
            ModuleKind::Compensation => "compensation",
            ModuleKind::Role => "role",
            ModuleKind::Raw => "raw",
            ModuleKind::Notion => "notion",
            ModuleKind::Panel => "panel",
        }
    }
}

/// A declaration that put no module in the registry, or a name it could not give.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationReport {
    pub declarations: PathBuf,
    pub name: String,
    pub finding: DeclarationFinding,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationFinding {
    /// The declared file does not exist.
    Absent(PathBuf),
    /// The declared file is not a plain file inside the source, reached through no link.
    Unplaceable(PathBuf),
    /// The declared kind, subject and version make no valid name.
    InvalidName,
}
impl std::fmt::Display for DeclarationReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let declarations = self.declarations.display();
        let name = &self.name;
        match &self.finding {
            DeclarationFinding::Absent(file) => {
                write!(
                    formatter,
                    "{declarations}: {name}: no file {}",
                    file.display()
                )
            }
            DeclarationFinding::Unplaceable(file) => write!(
                formatter,
                "{declarations}: {name}: {} is not a plain file inside the source",
                file.display()
            ),
            DeclarationFinding::InvalidName => {
                write!(
                    formatter,
                    "{declarations}: {name} is not a valid module name"
                )
            }
        }
    }
}

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
    reports: Vec<DeclarationReport>,
}
#[derive(Debug, Error, Eq, PartialEq)]
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
pub fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub fn valid_version(value: &str) -> bool {
    !value.is_empty()
        && (value == "0" || !value.starts_with('0'))
        && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundVersion {
    Single(i64),
    Span { first: i64, last: i64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParsedIdentity<'a> {
    kind: &'a str,
    topic: &'a str,
    version: BoundVersion,
}

trait ParsesModuleIdentity {
    fn parsed_identity(&self) -> Option<ParsedIdentity<'_>>;
}

impl ParsesModuleIdentity for str {
    fn parsed_identity(&self) -> Option<ParsedIdentity<'_>> {
        let fields: Vec<_> = self.split(':').collect();
        let [kind, topic, version] = fields.as_slice() else {
            return None;
        };
        if !MODULE_KINDS.iter().any(|known| known.word() == *kind) || !valid_segment(topic) {
            return None;
        }
        let version = if let Some((first, last)) = version.split_once('-') {
            if !valid_version(first) || !valid_version(last) {
                return None;
            }
            let first = first.parse().ok()?;
            let last = last.parse().ok()?;
            (first < last).then_some(BoundVersion::Span { first, last })?
        } else {
            if !valid_version(version) {
                return None;
            }
            BoundVersion::Single(version.parse().ok()?)
        };
        Some(ParsedIdentity {
            kind,
            topic,
            version,
        })
    }
}

trait CoversVersion {
    fn covers(&self, version: i64) -> bool;
}

impl CoversVersion for BoundVersion {
    fn covers(&self, version: i64) -> bool {
        match self {
            Self::Single(single) => *single == version,
            Self::Span { first, last } => *first <= version && version <= *last,
        }
    }
}

trait NamesModuleVersion {
    fn version_name(&self) -> Result<String, String>;
    fn names_span(&self) -> bool;
}

impl NamesModuleVersion for Versions {
    fn version_name(&self) -> Result<String, String> {
        match self {
            Self::Single(version) if version.0 >= 0 => Ok(version.0.to_string()),
            Self::Single(version) => Err(format!("version {} must be nonnegative", version.0)),
            Self::Span(bounds)
                if bounds.first.0.0 >= 0
                    && bounds.last.0.0 >= 0
                    && bounds.first.0.0 < bounds.last.0.0 =>
            {
                Ok(format!("{}-{}", bounds.first.0.0, bounds.last.0.0))
            }
            Self::Span(bounds) => Err(format!(
                "span {}-{} must have nonnegative increasing bounds",
                bounds.first.0.0, bounds.last.0.0
            )),
        }
    }

    fn names_span(&self) -> bool {
        matches!(self, Self::Span(_))
    }
}

trait DeclaresModuleIdentity {
    fn declared_identity(&self) -> Result<String, String>;
}

impl DeclaresModuleIdentity for Module {
    fn declared_identity(&self) -> Result<String, String> {
        self.module_identity.versions.version_name().map(|version| {
            format!(
                "{}:{}:{version}",
                self.module_identity.module_kind.word(),
                self.module_identity.module_subject.0
            )
        })
    }
}

pub fn valid_id(value: &str) -> bool {
    value.parsed_identity().is_some()
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceConfiguration {
    pub roots: [PathBuf; 3],
    pub refresh_program: RefreshProgram,
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
            .ok_or_else(|| "missing CURRICULUM_CONTEXT_REFRESH_PROGRAM".to_owned())?
            .into_string()
            .map(RefreshProgram)
            .map_err(|_| "CURRICULUM_CONTEXT_REFRESH_PROGRAM is not UTF-8".to_owned())?;
        Ok(Self {
            roots,
            refresh_program,
        })
    }
    pub fn read(&self) -> Result<SkillRegistry, RegistryError> {
        let program = Path::new(&self.refresh_program.0);
        match self.refresh_program.emit(&ProcessLauncher) {
            RefreshOutcome::Refreshed => {}
            RefreshOutcome::Failed => return Err(invalid(program, "context refresh failed")),
            RefreshOutcome::Unstarted(error) => {
                return Err(invalid(
                    program,
                    format!("refresh current context: {error}"),
                ));
            }
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
        let mut declarations = Vec::new();
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
            declarations.extend(
                read_declarations(root)?
                    .into_iter()
                    .map(|declaration| (i, declaration)),
            );
        }
        let mut identities = BTreeSet::new();
        let mut declared_files = BTreeMap::new();
        for (root_index, declaration) in &declarations {
            let identity = declaration
                .declared_identity()
                .map_err(|message| invalid(&roots[*root_index].join(DECLARATIONS_FILE), message))?;
            if valid_id(&identity) && !identities.insert(identity.clone()) {
                return Err(RegistryError::Duplicate(identity));
            }
            let file = (*root_index, declaration.module_file.0.clone());
            if let Some(first) = declared_files.insert(file, identity.clone()) {
                return Err(invalid(
                    &roots[*root_index].join(DECLARATIONS_FILE),
                    format!("{first} and {identity} declare the same source file"),
                ));
            }
        }
        let mut entries = BTreeMap::new();
        let mut reports = Vec::new();
        for (root_index, declaration) in declarations {
            admit(&roots[root_index], declaration, &mut entries, &mut reports)?;
        }
        let registry = Self {
            entries,
            roots,
            revisions: revisions.clone(),
            reports,
        };
        registry.expand_skills(&registry.skill_names()?)?;
        Ok(registry)
    }
    /// The declarations that put no module in the registry, as reported when the sources were read.
    pub fn reports(&self) -> &[DeclarationReport] {
        &self.reports
    }
    pub fn resolve_exact(&self, ids: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError> {
        let mut seen_bindings = BTreeSet::new();
        let mut seen_modules = BTreeSet::new();
        let mut resolved = Vec::new();
        for id in ids {
            if !seen_bindings.insert(id) {
                return Err(RegistryError::Duplicate(id.clone()));
            }
            for module in self.resolve_binding(id)? {
                if seen_modules.insert(module.name.clone()) {
                    resolved.push(module.clone());
                }
            }
        }
        Ok(resolved)
    }
    pub fn resolved_exact_sources(
        &self,
        ids: &[String],
    ) -> Result<Vec<ResolvedModule>, RegistryError> {
        self.resolve_exact(ids)?
            .into_iter()
            .map(|s| self.describe(s))
            .collect()
    }
    pub fn resolved_sources(&self, ids: &[String]) -> Result<Vec<ResolvedModule>, RegistryError> {
        self.expand_skills(ids)?
            .into_iter()
            .map(|s| self.describe(s))
            .collect()
    }

    /// Lists each admitted module once, preserving its actual identity and file.
    pub fn listed_modules(&self) -> Result<Vec<Module>, RegistryError> {
        self.entries
            .values()
            .filter(|skill| !skill.name.starts_with("panel:"))
            .map(|skill| {
                if canonical(&skill.path)? != skill.path
                    || fs::read(&skill.path).map_err(|e| invalid(&skill.path, e))?
                        != skill.body.as_bytes()
                {
                    return Err(invalid(
                        &skill.path,
                        "source changed since registry admission",
                    ));
                }
                let module_identity = skill
                    .name
                    .as_str()
                    .module_identity()
                    .map_err(|e| invalid(&skill.path, e))?;
                Ok(Module {
                    module_identity,
                    module_file: signal_curriculum_nexus::ModuleFile(
                        skill.path.to_string_lossy().into_owned(),
                    ),
                })
            })
            .collect()
    }

    pub fn listed_modules_verbose(
        &self,
    ) -> Result<Vec<signal_curriculum_nexus::ModuleDescription>, RegistryError> {
        self.entries
            .values()
            .filter(|skill| !skill.name.starts_with("panel:"))
            .map(|skill| {
                if canonical(&skill.path)? != skill.path
                    || fs::read(&skill.path).map_err(|e| invalid(&skill.path, e))?
                        != skill.body.as_bytes()
                {
                    return Err(invalid(
                        &skill.path,
                        "source changed since registry admission",
                    ));
                }
                let module = Module {
                    module_identity: skill
                        .name
                        .as_str()
                        .module_identity()
                        .map_err(|e| invalid(&skill.path, e))?,
                    module_file: signal_curriculum_nexus::ModuleFile(
                        skill.path.to_string_lossy().into_owned(),
                    ),
                };
                let value = description(&skill.body)
                    .map_err(|e| invalid(&skill.path, format!("{} ({})", skill.name, e)))?;
                Ok(signal_curriculum_nexus::ModuleDescription {
                    module,
                    description_text: signal_curriculum_nexus::DescriptionText(value),
                })
            })
            .collect()
    }
    fn describe(&self, skill: ResolvedSkill) -> Result<ResolvedModule, RegistryError> {
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
        let module_identity = skill
            .name
            .as_str()
            .module_identity()
            .map_err(|e| invalid(&skill.path, e))?;
        let text = instructions(&skill.body).map_err(|e| invalid(&skill.path, e))?;
        Ok(ResolvedModule {
            module_identity,
            repository_name: RepositoryName(
                ["psyche-belief", "mind-belief", "field-belief"][i].into(),
            ),
            source_revision: SourceRevision(self.revisions[i].clone()),
            source_root: SourceRoot(self.roots[i].to_string_lossy().into_owned()),
            source_relative_path: SourceRelativePath(
                skill
                    .path
                    .strip_prefix(&self.roots[i])
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            ),
            declared_placement: DeclaredPlacement(declared_placement(&skill.body).into()),
            source_sha256: SourceSha256(sha256(skill.body.as_bytes())),
            body_sha256: BodySha256(sha256(text.as_bytes())),
            string: text.to_owned(),
            string_vector: skill.dependencies,
        })
    }
    fn visit(
        &self,
        identity: &str,
        visiting: &mut BTreeSet<String>,
        seen: &mut BTreeSet<String>,
        output: &mut Vec<ResolvedSkill>,
    ) -> Result<(), RegistryError> {
        if seen.contains(identity) {
            return Ok(());
        }
        if !visiting.insert(identity.into()) {
            return Err(RegistryError::Cycle(identity.into()));
        }
        let source = self
            .entries
            .get(identity)
            .ok_or_else(|| RegistryError::Missing(identity.into()))?;
        for dependency in &source.dependencies {
            for actual in self.resolve_binding(dependency)? {
                // A numeric binding can incidentally contain the span module
                // that declares it. That candidate is already being visited;
                // a dependency naming the actual identity itself remains a
                // genuine cycle, and every other candidate is traversed.
                if actual.name != identity {
                    self.visit(&actual.name, visiting, seen, output)?;
                } else if dependency == identity {
                    return Err(RegistryError::Cycle(identity.into()));
                }
            }
        }
        visiting.remove(identity);
        seen.insert(identity.into());
        output.push(source.clone());
        Ok(())
    }
}

trait ResolvesBindings {
    fn resolve_binding(&self, binding: &str) -> Result<Vec<&SkillEntry>, RegistryError>;
}

impl ResolvesBindings for SkillRegistry {
    fn resolve_binding(&self, binding: &str) -> Result<Vec<&SkillEntry>, RegistryError> {
        let requested = binding
            .parsed_identity()
            .ok_or_else(|| RegistryError::Missing(binding.into()))?;
        if matches!(requested.version, BoundVersion::Span { .. }) {
            return self
                .entries
                .get(binding)
                .map(|module| vec![module])
                .ok_or_else(|| RegistryError::Missing(binding.into()));
        }
        let BoundVersion::Single(version) = requested.version else {
            unreachable!("span returned above")
        };
        let mut modules = self
            .entries
            .values()
            .filter(|module| {
                module.name.parsed_identity().is_some_and(|actual| {
                    actual.kind == requested.kind
                        && actual.topic == requested.topic
                        && actual.version.covers(version)
                })
            })
            .collect::<Vec<_>>();
        modules.sort_by_key(|module| (module.name != binding, module.name.as_str()));
        if modules.is_empty() {
            Err(RegistryError::Missing(binding.into()))
        } else {
            Ok(modules)
        }
    }
}

pub trait ExpandsSkills {
    fn expand_skills(&self, roots: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError>;
    fn skill_names(&self) -> Result<Vec<String>, RegistryError>;
    fn actual_skills(&self) -> Vec<ResolvedSkill>;
}
impl ExpandsSkills for SkillRegistry {
    fn expand_skills(&self, roots: &[String]) -> Result<Vec<ResolvedSkill>, RegistryError> {
        let mut visiting = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut output = Vec::new();
        for root in roots {
            for actual in self.resolve_binding(root)? {
                self.visit(&actual.name, &mut visiting, &mut seen, &mut output)?;
            }
        }
        Ok(output)
    }
    fn skill_names(&self) -> Result<Vec<String>, RegistryError> {
        Ok(self
            .entries
            .keys()
            .filter(|name| !name.starts_with("panel:"))
            .cloned()
            .collect())
    }
    fn actual_skills(&self) -> Vec<ResolvedSkill> {
        self.entries
            .values()
            .filter(|skill| !skill.name.starts_with("panel:"))
            .cloned()
            .collect()
    }
}

/// Projects the authoritative registry into plain harness skill directories.
/// All source text comes from the resolved registry; this code never scans a
/// second skill repository or follows generated symlinks.
pub mod workspace {
    use super::{
        ExpandsSkills, IdentityText, RegistryError, ResolvedSkill, SkillRegistry, invalid, sha256,
    };
    use crate::generated::{
        DeploymentDestination, ProjectionCandidate, PublishedContentDigest,
        PublishedInventoryDigest, RendererVersion, RepositoryRevision, SkillDirectory,
    };
    use crate::panels::{PanelPrompt, PanelRendering};
    use signal_curriculum_nexus::{RepositoryName, SourceRevision, Target};
    use std::{
        collections::BTreeMap,
        fs,
        io::Write,
        path::{Component, Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    pub const RENDERER_VERSION: &str = "curriculum-workspace-projector-v1";

    /// Every directory a projection writes each module into.
    const SKILL_DIRECTORIES: [SkillDirectory; 5] = [
        SkillDirectory::Claude,
        SkillDirectory::Agents,
        SkillDirectory::Codex,
        SkillDirectory::Pi,
        SkillDirectory::OpenCode,
    ];

    /// A skill directory sits at its path under the workspace, and the
    /// harness reading it is the target its modules are rendered for.
    trait PlacesSkills {
        fn path(&self) -> &'static str;
        fn target(&self) -> Target;
    }

    impl PlacesSkills for SkillDirectory {
        fn path(&self) -> &'static str {
            match self {
                SkillDirectory::Claude => ".claude/skills",
                SkillDirectory::Agents => ".agents/skills",
                SkillDirectory::Codex => ".codex/skills",
                SkillDirectory::Pi => ".pi/skills",
                SkillDirectory::OpenCode => ".opencode/skills",
            }
        }

        fn target(&self) -> Target {
            match self {
                SkillDirectory::Claude => Target::Claude,
                SkillDirectory::Agents | SkillDirectory::Codex => Target::Codex,
                SkillDirectory::Pi => Target::Pi,
                SkillDirectory::OpenCode => Target::OpenCode,
            }
        }
    }

    /// A module's whole source, frontmatter included, rendered for a target.
    trait RendersSkill {
        fn render(&self, skill: &ResolvedSkill) -> Result<String, RegistryError>;
    }

    impl RendersSkill for Target {
        fn render(&self, skill: &ResolvedSkill) -> Result<String, RegistryError> {
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
                    "{% if claude %}" => Some(*self == Target::Claude),
                    "{% if codex %}" => Some(*self == Target::Codex),
                    "{% if opencode %}" => Some(*self == Target::OpenCode),
                    "{% if pi %}" => Some(*self == Target::Pi),
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
                    if *self == Target::Claude && directive == "user-only: true" {
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
            if *self == Target::OpenCode {
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
            let skills = registry.actual_skills();
            let modules = skills
                .iter()
                .cloned()
                .map(|skill| registry.describe(skill))
                .collect::<Result<Vec<_>, _>>()?;
            let mut outputs = BTreeMap::<PathBuf, String>::new();
            for directory in &SKILL_DIRECTORIES {
                self.remove_stale(directory, &ids)?;
            }
            let mut missing_panels = signal_curriculum_nexus::MissingPanels(vec![]);
            for skill in &skills {
                for directory in &SKILL_DIRECTORIES {
                    let relative = Path::new(directory.path())
                        .join(&skill.name)
                        .join("SKILL.md");
                    let panels = PanelPrompt::for_registry(registry);
                    let mut merged = skill.clone();
                    merged.body = panels
                        .render(&directory.target(), &skill.body)
                        .map_err(|error| invalid(&skill.path, error))?;
                    for name in panels
                        .missing()
                        .map_err(|error| invalid(&skill.path, error))?
                        .0
                    {
                        if !missing_panels.0.contains(&name) {
                            missing_panels.0.push(name);
                        }
                    }
                    outputs.insert(relative, directory.target().render(&merged)?);
                }
                if user_only(&skill.body) {
                    for directory in SKILL_DIRECTORIES
                        .iter()
                        .filter(|directory| directory.target() == Target::Codex)
                    {
                        outputs.insert(
                            Path::new(directory.path())
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
                    let identity = m
                        .module_identity
                        .identity_text()
                        .map_err(|e| invalid(&self.root, e))?;
                    Ok::<_, RegistryError>(format!(
                        "{} {} {}\n",
                        identity, m.source_revision.0, m.source_relative_path.0
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
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
                    repository_name: RepositoryName(
                        ["psyche-context", "mind-context", "field-context"][index].into(),
                    ),
                    source_revision: SourceRevision(revision.clone()),
                })
                .collect();
            Ok(ProjectionCandidate {
                missing_panels,
                deployment_destination: DeploymentDestination(
                    destination.to_string_lossy().into_owned(),
                ),
                repository_revision_vector: repositories,
                published_inventory_digest: PublishedInventoryDigest(sha256(inventory.as_bytes())),
                published_content_digest: PublishedContentDigest(sha256(
                    content_manifest.as_bytes(),
                )),
                renderer_version: RendererVersion(RENDERER_VERSION.into()),
            })
        }

        fn remove_stale(
            &self,
            directory: &SkillDirectory,
            current: &[String],
        ) -> Result<(), RegistryError> {
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
            let surface = self.safe(Path::new(directory.path()))?;
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
/// Reads the declarations of the source at this root: its modules.datom, whole.
#[cfg(feature = "datom")]
fn read_declarations(root: &Path) -> Result<Vec<Module>, RegistryError> {
    use datom_codec::{Actualizing, Budget, Datomizable, Potential};
    use protos::{Protosizable, ReaderBudget, Textualizable};
    let path = root.join(DECLARATIONS_FILE);
    let metadata = fs::symlink_metadata(&path).map_err(|e| invalid(&path, e))?;
    if !metadata.is_file() {
        return Err(invalid(&path, "declarations must be a plain file"));
    }
    let text = fs::read_to_string(&path).map_err(|e| invalid(&path, e))?;
    Potential::<ModuleDeclarations>::from(text)
        .actualize(&mut Budget {
            remaining: 1_048_576,
            reader: ReaderBudget::try_from(1_048_576).expect("a constant reader budget"),
            depth: 0,
            maximum_depth: 1024,
        })
        .map(|ModuleDeclarations(declarations)| declarations)
        .map_err(|e| invalid(&path, e.datomize(vec![]).protosize().textualize()))
}
#[cfg(not(feature = "datom"))]
fn read_declarations(root: &Path) -> Result<Vec<Module>, RegistryError> {
    Err(invalid(
        &root.join(DECLARATIONS_FILE),
        "source declarations require the datom package feature",
    ))
}

/// Gives the declaration's identity to its correspondingly named file.
/// Recoverable singleton findings are reported; malformed declarations,
/// duplicate identities and every inadmissible span file refuse the registry.
fn admit(
    root: &Path,
    declaration: Module,
    entries: &mut BTreeMap<String, SkillEntry>,
    reports: &mut Vec<DeclarationReport>,
) -> Result<(), RegistryError> {
    let declarations = root.join(DECLARATIONS_FILE);
    let mut report = |name: String, finding| {
        reports.push(DeclarationReport {
            declarations: declarations.clone(),
            name,
            finding,
        })
    };
    let name = declaration
        .declared_identity()
        .map_err(|message| invalid(&declarations, message))?;
    let version_name = declaration
        .module_identity
        .versions
        .version_name()
        .map_err(|message| invalid(&declarations, message))?;
    let span = declaration.module_identity.versions.names_span();
    let relative = Path::new(&declaration.module_file.0);
    let expected_file_name = format!("{version_name}.md");
    if relative.file_name().and_then(|file| file.to_str()) != Some(expected_file_name.as_str()) {
        return Err(invalid(
            &declarations,
            format!(
                "{name}: declared file {} must be named {expected_file_name}",
                relative.display()
            ),
        ));
    }
    let path = root.join(relative);
    let placeable = !declaration.module_file.0.is_empty()
        && relative
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)));
    let placed = match fs::canonicalize(&path) {
        _ if !placeable => Err(DeclarationFinding::Unplaceable(relative.into())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(DeclarationFinding::Absent(relative.into()))
        }
        Err(e) => return Err(invalid(&path, e)),
        Ok(actual) if actual != path || !actual.is_file() => {
            Err(DeclarationFinding::Unplaceable(relative.into()))
        }
        Ok(actual) => Ok(actual),
    };
    let path = match placed {
        Ok(path) => path,
        Err(DeclarationFinding::Absent(file)) if span => {
            return Err(invalid(
                &declarations,
                format!("{name}: no file {}", file.display()),
            ));
        }
        Err(DeclarationFinding::Unplaceable(file)) if span => {
            return Err(invalid(
                &declarations,
                format!(
                    "{name}: {} is not a plain file inside the source",
                    file.display()
                ),
            ));
        }
        Err(finding) => {
            report(name, finding);
            return Ok(());
        }
    };
    if !valid_id(&name) {
        report(name, DeclarationFinding::InvalidName);
        return Ok(());
    }
    if entries.contains_key(&name) {
        return Err(RegistryError::Duplicate(name));
    }
    if let Some(held) = entries.values().find(|entry| entry.path == path) {
        return Err(invalid(
            &declarations,
            format!("{} and {name} declare the same source file", held.name),
        ));
    }
    let text = fs::read_to_string(&path).map_err(|e| invalid(&path, e))?;
    framed_source(&text).map_err(|e| invalid(&path, e))?;
    let dependencies = dependencies(&path, &text)?;
    entries.insert(
        name.clone(),
        SkillEntry {
            path,
            name,
            body: text,
            dependencies,
        },
    );
    Ok(())
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
