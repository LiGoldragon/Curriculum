use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    path::PathBuf,
};

use nota::{NotaDecode, NotaEncode, NotaSource};

use crate::{
    error::{Error, Result},
    markdown::{MarkdownAssembly, MarkdownFragment},
    schema::assembly::{
        ActiveOutput, ActiveOutputs, ActiveSkill, ByteCount, EffortLevel, Frontmatter,
        FrontmatterEntry, FrontmatterKey, FrontmatterValue, GeneratedFile, GeneratedFiles,
        GeneratedOutputVisualization, GeneratedOutputVisualizations, GeneratedRoleOutputs,
        GenerationMode, GenerationOutcome, GenerationReport, GenerationRequest, IncludedModules,
        LineCount, Manifest, ManifestPath, ModelCatalog, ModelCatalogEntry, ModelIdentifier,
        ModuleDependencies, ModuleDependency, ModuleIdentifier, ModuleKind, ModulePath, Modules,
        Operation, OutputIdentifier, OutputKind, OutputPath, OutputSurface, ProviderSurface,
        RoleDepth, RoleDepthIdentifier, RoleDepths, RoleDescription, RoleDescriptionCell,
        RoleDescriptions, RolePacketComposition, RolePacketCompositions, RolePermission,
        RolePermissionIdentifier, RolePermissions, RoleTargetSurface, RoleVisualization,
        RoleVisualizations, SkillModuleComposition, SkillModuleCompositions, TargetModuleInsertion,
        TargetModuleInsertions, TargetSurface, ToolRestriction, UniversalRoleModules,
        VisualizationReport, VisualizationRequest,
    },
    template::RenderTarget,
    trunk_guard::TrunkDescendantGuard,
    workspace_path::WorkspacePath,
};

const GENERATED_SKILL_BLOCK_BYTE_LIMIT: usize = 32 * 1024;
const RETIRED_SKILL_INDEX_PATH: &str = "skills/skills.nota";
const RETIRED_CURRENT_DESTINATION_PHRASES: &[&str] = &[
    "Repo Operator",
    "Weave Operator",
    "Intent Maintainer",
    "workspace essence",
    "workspace intent",
];
const EXECUTION_LIMIT_FIELD_PARTS: &[&str] = &["budget", "deadline", "limit", "timeout"];
const EXECUTION_LIMIT_MAXIMUMS: &[&str] = &[
    "cost",
    "iteration",
    "output",
    "retry",
    "runtime",
    "time",
    "token",
    "tool",
    "turn",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandLine {
    arguments: Vec<String>,
}

impl CommandLine {
    pub fn from_environment() -> Self {
        Self {
            arguments: env::args().skip(1).collect(),
        }
    }

    pub fn run(&self) -> Result<GenerationOutcome> {
        let operation = RequestArgument::new(self.arguments.clone())
            .read()?
            .parse()?;
        operation.guard_source()?;
        operation.execute()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequestArgument {
    arguments: Vec<String>,
}

impl RequestArgument {
    fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }

    fn read(self) -> Result<RequestText> {
        if self.arguments.len() != 1 {
            return Err(Error::ArgumentCount {
                count: self.arguments.len(),
            });
        }
        let argument = self.arguments.into_iter().next().expect("length checked");
        if argument.trim_start().starts_with('(') {
            Ok(RequestText::new(argument))
        } else {
            RequestFile::new(PathBuf::from(argument)).read()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequestFile {
    path: PathBuf,
}

impl RequestFile {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn read(self) -> Result<RequestText> {
        fs::read_to_string(&self.path)
            .map(RequestText::new)
            .map_err(|source| Error::ReadFile {
                path: self.path,
                source,
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequestText {
    text: String,
}

impl RequestText {
    fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    fn parse(&self) -> Result<Operation> {
        NotaSource::new(&self.text)
            .parse::<Operation>()
            .map_err(Error::DecodeNotaArgument)
    }
}

impl Operation {
    fn guard_source(&self) -> Result<()> {
        match self {
            Self::Generate(request) => request.guard_source(),
            Self::Visualize(request) => request.guard_source(),
        }
    }

    pub fn execute(self) -> Result<GenerationOutcome> {
        match self {
            Self::Generate(request) => request.generate().map(GenerationOutcome::Generated),
            Self::Visualize(request) => request.visualize().map(GenerationOutcome::Visualized),
        }
    }
}

impl GenerationRequest {
    fn guard_source(&self) -> Result<()> {
        let source_root = RootPath::new(self.source_root.as_ref()).to_path_buf()?;
        TrunkDescendantGuard::new(source_root).verify()
    }

    pub fn generate(&self) -> Result<GenerationReport> {
        let source_root = RootPath::new(self.source_root.as_ref()).to_path_buf()?;
        let workspace_root = RootPath::new(self.workspace_root.as_ref()).to_path_buf()?;
        let configuration =
            GenerationSource::new(source_root.clone(), self.manifest_path.clone()).read()?;
        let jobs = GenerationJobs::new(source_root, workspace_root.clone(), configuration.clone())
            .jobs()?;
        if self.generation_mode == GenerationMode::Write {
            WorkspacePruner::new(workspace_root.clone(), configuration.clone()).prune()?;
        }
        let mut files = Vec::new();
        for job in jobs {
            files.push(job.generate(self.generation_mode)?);
        }
        StaleOutputScan::new(workspace_root, configuration).validate()?;
        Ok(GenerationReport::new(GeneratedFiles::new(files)))
    }
}

impl VisualizationRequest {
    fn guard_source(&self) -> Result<()> {
        let source_root = RootPath::new(self.source_root.as_ref()).to_path_buf()?;
        TrunkDescendantGuard::new(source_root).verify()
    }

    pub fn visualize(&self) -> Result<VisualizationReport> {
        let source_root = RootPath::new(self.source_root.as_ref()).to_path_buf()?;
        let workspace_root = RootPath::new(self.workspace_root.as_ref()).to_path_buf()?;
        let configuration =
            GenerationSource::new(source_root.clone(), self.manifest_path.clone()).read()?;
        let jobs =
            GenerationJobs::new(source_root, workspace_root, configuration.clone()).jobs()?;
        let mut generated_outputs = jobs
            .iter()
            .map(GenerationJob::visualize)
            .collect::<Result<Vec<_>>>()?;
        generated_outputs
            .sort_by(|left, right| left.output_path.as_ref().cmp(right.output_path.as_ref()));
        Ok(VisualizationReport {
            role_visualizations: RoleVisualizations::new(configuration.role_visualizations()?),
            generated_output_visualizations: GeneratedOutputVisualizations::new(generated_outputs),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RootPath<'a> {
    text: &'a str,
}

impl<'a> RootPath<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn to_path_buf(&self) -> Result<PathBuf> {
        if let Some(name) = self.text.strip_prefix('$') {
            return env::var_os(name).map(PathBuf::from).ok_or_else(|| {
                Error::MissingEnvironmentRoot {
                    variable: name.to_owned(),
                }
            });
        }
        Ok(PathBuf::from(self.text))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationSource {
    source_root: PathBuf,
    manifest_path: ManifestPath,
}

impl GenerationSource {
    fn new(source_root: PathBuf, manifest_path: ManifestPath) -> Self {
        Self {
            source_root,
            manifest_path,
        }
    }

    fn read(&self) -> Result<GenerationConfiguration> {
        let active_outputs: ActiveOutputs = SourceFile::new(
            self.source_root.clone(),
            PathBuf::from(self.manifest_path.as_ref()),
        )
        .read()?;
        let dependency_path = PathBuf::from(self.manifest_path.as_ref())
            .parent()
            .map(|parent| parent.join("module-dependencies.nota"))
            .unwrap_or_else(|| PathBuf::from("module-dependencies.nota"));
        let module_dependencies: ModuleDependencies =
            SourceFile::new(self.source_root.clone(), dependency_path).read()?;
        let manifest_directory = PathBuf::from(self.manifest_path.as_ref())
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default();
        let insertion_path = manifest_directory.join("target-module-insertions.nota");
        let target_module_insertions: TargetModuleInsertions =
            SourceFile::new(self.source_root.clone(), insertion_path)
                .read_optional()?
                .unwrap_or_else(|| TargetModuleInsertions::new(Vec::new()));
        let universal_role_modules_path = manifest_directory.join("universal-role-modules.nota");
        let universal_role_modules: UniversalRoleModules =
            SourceFile::new(self.source_root.clone(), universal_role_modules_path)
                .read_optional()?
                .unwrap_or_else(|| UniversalRoleModules::new(Vec::new()));
        let skill_module_compositions_path =
            manifest_directory.join("skill-module-compositions.nota");
        let skill_module_compositions: SkillModuleCompositions =
            SourceFile::new(self.source_root.clone(), skill_module_compositions_path)
                .read_optional()?
                .unwrap_or_else(|| SkillModuleCompositions::new(Vec::new()));
        let model_catalog_path = manifest_directory.join("model-catalog.nota");
        let model_catalog: ModelCatalog =
            SourceFile::new(self.source_root.clone(), model_catalog_path).read()?;
        let role_permissions_path = manifest_directory.join("role-permissions.nota");
        let role_permissions: RolePermissions =
            SourceFile::new(self.source_root.clone(), role_permissions_path).read()?;
        let role_depths_path = manifest_directory.join("role-depths.nota");
        let role_depths: RoleDepths =
            SourceFile::new(self.source_root.clone(), role_depths_path).read()?;
        let role_descriptions_path = manifest_directory.join("role-descriptions.nota");
        let role_descriptions: RoleDescriptions =
            SourceFile::new(self.source_root.clone(), role_descriptions_path).read()?;
        GenerationConfiguration::active(
            active_outputs,
            module_dependencies,
            target_module_insertions,
            universal_role_modules,
            skill_module_compositions,
            RoleGenerationSources {
                model_catalog,
                role_permissions,
                role_depths,
                role_descriptions,
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceFile {
    source_root: PathBuf,
    relative_path: PathBuf,
}

impl SourceFile {
    fn new(source_root: PathBuf, relative_path: PathBuf) -> Self {
        Self {
            source_root,
            relative_path,
        }
    }

    fn read<T>(&self) -> Result<T>
    where
        T: NotaDecode,
    {
        let workspace_path =
            WorkspacePath::new(self.source_root.clone(), self.relative_path.clone())?;
        let path = workspace_path.full_path();
        let text = fs::read_to_string(&path).map_err(|source| Error::ReadFile {
            path: path.clone(),
            source,
        })?;
        NotaSource::new(&text)
            .parse::<T>()
            .map_err(|source| Error::DecodeNota { path, source })
    }

    fn read_optional<T>(&self) -> Result<Option<T>>
    where
        T: NotaDecode,
    {
        let workspace_path =
            WorkspacePath::new(self.source_root.clone(), self.relative_path.clone())?;
        let path = workspace_path.full_path();
        match fs::read_to_string(&path) {
            Ok(text) => NotaSource::new(&text)
                .parse::<T>()
                .map(Some)
                .map_err(|source| Error::DecodeNota { path, source }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(Error::ReadFile { path, source }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationJobs {
    source_root: PathBuf,
    workspace_root: PathBuf,
    configuration: GenerationConfiguration,
}

impl GenerationJobs {
    fn new(
        source_root: PathBuf,
        workspace_root: PathBuf,
        configuration: GenerationConfiguration,
    ) -> Self {
        Self {
            source_root,
            workspace_root,
            configuration,
        }
    }

    fn jobs(&self) -> Result<Vec<GenerationJob>> {
        let mut jobs = Vec::new();
        for manifest in self.configuration.skill_manifests()? {
            jobs.push(GenerationJob::Manifest(ManifestAssembler::new(
                self.source_root.clone(),
                self.workspace_root.clone(),
                manifest,
            )));
        }
        for packet in self.configuration.role_packets()? {
            jobs.push(GenerationJob::Manifest(
                ManifestAssembler::new(
                    self.source_root.clone(),
                    self.workspace_root.clone(),
                    packet.manifest,
                )
                .with_leading_fragment(packet.packet_body),
            ));
        }
        jobs.push(GenerationJob::Rendered(RenderedOutput::new(
            self.workspace_root.clone(),
            RoleOutputInventory::relative_path(),
            self.configuration.role_output_inventory().render(),
        )?));
        OutputPathIndex::new().validate(&jobs)?;
        Ok(jobs)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SkillModuleCompositionIndex {
    entries: BTreeMap<OutputIdentifier, IncludedModules>,
}

impl SkillModuleCompositionIndex {
    fn new(
        skill_module_compositions: SkillModuleCompositions,
        active_outputs: &ActiveOutputs,
    ) -> Result<Self> {
        let active_skills: BTreeSet<OutputIdentifier> = active_outputs
            .payload()
            .iter()
            .map(|output| match output {
                ActiveOutput::Skill(skill) => skill.output_identifier.clone(),
            })
            .collect();
        let mut entries = BTreeMap::new();
        for SkillModuleComposition {
            output_identifier,
            included_modules,
        } in skill_module_compositions.into_payload()
        {
            if !active_skills.contains(&output_identifier) {
                return Err(Error::StaleSkillModuleComposition {
                    skill_identifier: output_identifier.into_payload(),
                });
            }
            if entries
                .insert(output_identifier.clone(), included_modules)
                .is_some()
            {
                return Err(Error::DuplicateSkillModuleComposition {
                    skill_identifier: output_identifier.into_payload(),
                });
            }
        }
        Ok(Self { entries })
    }

    fn included_modules(&self, skill_identifier: &OutputIdentifier) -> Vec<ModuleIdentifier> {
        self.entries
            .get(skill_identifier)
            .map(|composition| composition.payload().clone())
            .unwrap_or_default()
    }
}

/// Every generated role packet ends its own body with these two lines. A
/// permission that carries body text places that text immediately before them.
const SHARED_ROLE_BODY: &str = "The brief is your authority. Decide what it settles; return what it does not.\nFinish everything that does not depend on what you return.";

/// Pi names a ChatGPT model provider-qualified; Codex names the same model
/// bare.
const PI_CHAT_GPT_PROVIDER: &str = "openai-codex";

/// A restricted packet blocks the editing tools by the name each harness uses.
/// Codex role files carry no tool field, so a restricted Codex packet carries
/// no tool restriction.
const CLAUDE_RESTRICTED_TOOLS: &str = "Edit, Write, NotebookEdit";
const PI_RESTRICTED_TOOLS: &str = "edit, write";

const ROLE_TARGET_SURFACES: [RoleTargetSurface; 3] = [
    RoleTargetSurface::ClaudeAgent,
    RoleTargetSurface::CodexAgent,
    RoleTargetSurface::PiAgent,
];

impl EffortLevel {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}

impl ProviderSurface {
    fn name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::ChatGpt => "ChatGpt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CatalogModel {
    provider_surface: ProviderSurface,
    accepted_effort_levels: Vec<EffortLevel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModelCatalogIndex {
    entries: BTreeMap<ModelIdentifier, CatalogModel>,
}

impl ModelCatalogIndex {
    fn new(model_catalog: ModelCatalog) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for ModelCatalogEntry {
            model_identifier,
            provider_surface,
            accepted_effort_levels,
        } in model_catalog.into_payload()
        {
            let accepted_effort_levels = accepted_effort_levels.into_payload();
            Self::validate_accepted_efforts(&model_identifier, &accepted_effort_levels)?;
            if entries
                .insert(
                    model_identifier.clone(),
                    CatalogModel {
                        provider_surface,
                        accepted_effort_levels,
                    },
                )
                .is_some()
            {
                return Err(Error::DuplicateModelCatalogEntry {
                    model_identifier: model_identifier.into_payload(),
                });
            }
        }
        Ok(Self { entries })
    }

    fn validate_accepted_efforts(
        model_identifier: &ModelIdentifier,
        accepted_effort_levels: &[EffortLevel],
    ) -> Result<()> {
        let mut seen = BTreeSet::new();
        for effort_level in accepted_effort_levels {
            if !seen.insert(effort_level.as_str()) {
                return Err(Error::DuplicateModelCatalogEffort {
                    model_identifier: model_identifier.as_ref().to_owned(),
                    effort: effort_level.as_str().to_owned(),
                });
            }
        }
        Ok(())
    }

    fn resolve(
        &self,
        depth_identifier: &RoleDepthIdentifier,
        provider_surface: ProviderSurface,
        model_identifier: &ModelIdentifier,
        effort_level: Option<EffortLevel>,
    ) -> Result<DepthModelProfile> {
        let model =
            self.entries
                .get(model_identifier)
                .ok_or_else(|| Error::UnsupportedRoleModel {
                    depth_identifier: depth_identifier.as_ref().to_owned(),
                    model_identifier: model_identifier.as_ref().to_owned(),
                })?;
        if model.provider_surface != provider_surface {
            return Err(Error::RoleModelProviderMismatch {
                depth_identifier: depth_identifier.as_ref().to_owned(),
                model_identifier: model_identifier.as_ref().to_owned(),
                expected_provider: provider_surface.name().to_owned(),
                actual_provider: model.provider_surface.name().to_owned(),
            });
        }
        match (model.accepted_effort_levels.as_slice(), effort_level) {
            ([], None) => {}
            ([], Some(effort_level)) => {
                return Err(Error::EffortlessRoleModelCarriesEffort {
                    depth_identifier: depth_identifier.as_ref().to_owned(),
                    model_identifier: model_identifier.as_ref().to_owned(),
                    effort: effort_level.as_str().to_owned(),
                });
            }
            (_, None) => {
                return Err(Error::MissingRoleModelEffort {
                    depth_identifier: depth_identifier.as_ref().to_owned(),
                    model_identifier: model_identifier.as_ref().to_owned(),
                });
            }
            (accepted_effort_levels, Some(effort_level)) => {
                if !accepted_effort_levels.contains(&effort_level) {
                    return Err(Error::UnsupportedRoleModelEffort {
                        depth_identifier: depth_identifier.as_ref().to_owned(),
                        model_identifier: model_identifier.as_ref().to_owned(),
                        effort: effort_level.as_str().to_owned(),
                    });
                }
            }
        }
        Ok(DepthModelProfile {
            model_identifier: model_identifier.as_ref().to_owned(),
            effort_level,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DepthModelProfile {
    model_identifier: String,
    effort_level: Option<EffortLevel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedPermission {
    permission_identifier: RolePermissionIdentifier,
    permission_body: String,
    tool_restriction: ToolRestriction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedDepth {
    depth_identifier: RoleDepthIdentifier,
    claude_model: DepthModelProfile,
    chat_gpt_model: DepthModelProfile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleGenerationSources {
    model_catalog: ModelCatalog,
    role_permissions: RolePermissions,
    role_depths: RoleDepths,
    role_descriptions: RoleDescriptions,
}

/// Every generated role is one cell of the permission-by-depth cross product.
#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratedRole {
    output_identifier: OutputIdentifier,
    role_description: RoleDescription,
    packet_body: String,
    tool_restriction: ToolRestriction,
    claude_model: DepthModelProfile,
    chat_gpt_model: DepthModelProfile,
}

impl GeneratedRole {
    fn new(
        permission: &ResolvedPermission,
        depth: &ResolvedDepth,
        role_description: RoleDescription,
    ) -> Self {
        let output_identifier = OutputIdentifier::new(format!(
            "{}-{}",
            permission.permission_identifier.as_ref(),
            depth.depth_identifier.as_ref()
        ));
        let packet_body = Self::render_packet_body(&output_identifier, permission);
        Self {
            output_identifier,
            role_description,
            packet_body,
            tool_restriction: permission.tool_restriction,
            claude_model: depth.claude_model.clone(),
            chat_gpt_model: depth.chat_gpt_model.clone(),
        }
    }

    fn render_packet_body(
        output_identifier: &OutputIdentifier,
        permission: &ResolvedPermission,
    ) -> String {
        let mut packet_body = format!("# Role — {}\n\n", output_identifier.as_ref());
        if !permission.permission_body.is_empty() {
            packet_body.push_str(&permission.permission_body);
            packet_body.push('\n');
        }
        packet_body.push_str(SHARED_ROLE_BODY);
        packet_body.push('\n');
        packet_body
    }

    fn packets(
        &self,
        module_index: &ModuleIndex,
        universal_role_modules: &UniversalRoleModules,
    ) -> Result<Vec<RolePacket>> {
        let mut packets = Vec::new();
        for role_target_surface in ROLE_TARGET_SURFACES {
            let output_surface = OutputSurface::from(&role_target_surface);
            packets.push(RolePacket {
                packet_body: self.packet_body.clone(),
                manifest: Manifest {
                    output_path: OutputPath::new(
                        output_surface.role_path(self.output_identifier.as_ref()),
                    ),
                    output_kind: output_surface.role_output_kind(),
                    output_surface,
                    frontmatter: self.frontmatter(output_surface),
                    modules: Modules::new(module_index.expanded_paths(
                        universal_role_modules.payload(),
                        ModuleUse::RoleContent,
                        output_surface,
                    )?),
                },
            });
        }
        Ok(packets)
    }

    fn frontmatter(&self, output_surface: OutputSurface) -> Frontmatter {
        let mut entries = vec![
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("name"),
                frontmatter_value: FrontmatterValue::new(self.output_identifier.as_ref()),
            },
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("description"),
                frontmatter_value: FrontmatterValue::new(self.role_description.as_ref()),
            },
        ];
        match output_surface {
            OutputSurface::ClaudeAgent => {
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("model"),
                    frontmatter_value: FrontmatterValue::new(&self.claude_model.model_identifier),
                });
                if let Some(effort_level) = self.claude_model.effort_level {
                    entries.push(FrontmatterEntry {
                        frontmatter_key: FrontmatterKey::new("effort"),
                        frontmatter_value: FrontmatterValue::new(effort_level.as_str()),
                    });
                }
                if self.tool_restriction == ToolRestriction::Restricted {
                    entries.push(FrontmatterEntry {
                        frontmatter_key: FrontmatterKey::new("disallowedTools"),
                        frontmatter_value: FrontmatterValue::new(CLAUDE_RESTRICTED_TOOLS),
                    });
                }
            }
            OutputSurface::PiAgent => {
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("model"),
                    frontmatter_value: FrontmatterValue::new(format!(
                        "{PI_CHAT_GPT_PROVIDER}/{}",
                        self.chat_gpt_model.model_identifier
                    )),
                });
                if let Some(effort_level) = self.chat_gpt_model.effort_level {
                    entries.push(FrontmatterEntry {
                        frontmatter_key: FrontmatterKey::new("thinking"),
                        frontmatter_value: FrontmatterValue::new(effort_level.as_str()),
                    });
                }
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("projectRoleIdentity"),
                    frontmatter_value: FrontmatterValue::new(self.output_identifier.as_ref()),
                });
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("projectRoleDispatchKind"),
                    frontmatter_value: FrontmatterValue::new("leaf"),
                });
                if self.tool_restriction == ToolRestriction::Restricted {
                    entries.push(FrontmatterEntry {
                        frontmatter_key: FrontmatterKey::new("disallowed_tools"),
                        frontmatter_value: FrontmatterValue::new(PI_RESTRICTED_TOOLS),
                    });
                }
            }
            OutputSurface::CodexAgent => {
                entries.push(FrontmatterEntry {
                    frontmatter_key: FrontmatterKey::new("model"),
                    frontmatter_value: FrontmatterValue::new(&self.chat_gpt_model.model_identifier),
                });
                if let Some(effort_level) = self.chat_gpt_model.effort_level {
                    entries.push(FrontmatterEntry {
                        frontmatter_key: FrontmatterKey::new("model_reasoning_effort"),
                        frontmatter_value: FrontmatterValue::new(effort_level.as_str()),
                    });
                }
            }
            OutputSurface::Workspace | OutputSurface::AgentsSkill | OutputSurface::ClaudeSkill => {
                unreachable!("role frontmatter renders only for role surfaces")
            }
        }
        Frontmatter::new(entries)
    }

    fn output_paths(&self) -> Vec<OutputPath> {
        ROLE_TARGET_SURFACES
            .iter()
            .map(|role_target_surface| {
                let output_surface = OutputSurface::from(role_target_surface);
                OutputPath::new(output_surface.role_path(self.output_identifier.as_ref()))
            })
            .collect()
    }
}

/// One generated role rendered for one harness surface: the role's own body
/// followed by the manifest that names its universal modules and frontmatter.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RolePacket {
    packet_body: String,
    manifest: Manifest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratedRoleIndex {
    roles: Vec<GeneratedRole>,
}

impl GeneratedRoleIndex {
    fn new(sources: RoleGenerationSources) -> Result<Self> {
        let catalog = ModelCatalogIndex::new(sources.model_catalog)?;
        let permissions = Self::permissions(sources.role_permissions)?;
        let depths = Self::depths(sources.role_depths, &catalog)?;
        let mut descriptions = Self::descriptions(sources.role_descriptions)?;
        let mut roles = Vec::new();
        for permission in &permissions {
            for depth in &depths {
                let cell = (
                    permission.permission_identifier.clone(),
                    depth.depth_identifier.clone(),
                );
                let role_description =
                    descriptions
                        .remove(&cell)
                        .ok_or_else(|| Error::MissingRoleDescription {
                            permission_identifier: permission
                                .permission_identifier
                                .as_ref()
                                .to_owned(),
                            depth_identifier: depth.depth_identifier.as_ref().to_owned(),
                        })?;
                roles.push(GeneratedRole::new(permission, depth, role_description));
            }
        }
        if let Some((permission_identifier, depth_identifier)) = descriptions.into_keys().next() {
            return Err(Error::StaleRoleDescription {
                permission_identifier: permission_identifier.into_payload(),
                depth_identifier: depth_identifier.into_payload(),
            });
        }
        Ok(Self { roles })
    }

    fn permissions(role_permissions: RolePermissions) -> Result<Vec<ResolvedPermission>> {
        let mut seen = BTreeSet::new();
        let mut permissions = Vec::new();
        for RolePermission {
            role_permission_identifier,
            permission_body,
            tool_restriction,
        } in role_permissions.into_payload()
        {
            if !seen.insert(role_permission_identifier.clone()) {
                return Err(Error::DuplicateRolePermission {
                    permission_identifier: role_permission_identifier.into_payload(),
                });
            }
            permissions.push(ResolvedPermission {
                permission_identifier: role_permission_identifier,
                permission_body: permission_body.into_payload(),
                tool_restriction,
            });
        }
        if permissions.is_empty() {
            return Err(Error::MissingRolePermissions);
        }
        Ok(permissions)
    }

    fn depths(role_depths: RoleDepths, catalog: &ModelCatalogIndex) -> Result<Vec<ResolvedDepth>> {
        let mut seen = BTreeSet::new();
        let mut depths = Vec::new();
        for RoleDepth {
            role_depth_identifier,
            claude_depth_model,
            chat_gpt_depth_model,
        } in role_depths.into_payload()
        {
            if !seen.insert(role_depth_identifier.clone()) {
                return Err(Error::DuplicateRoleDepth {
                    depth_identifier: role_depth_identifier.into_payload(),
                });
            }
            let claude_model = catalog.resolve(
                &role_depth_identifier,
                ProviderSurface::Claude,
                &claude_depth_model.model_identifier,
                *claude_depth_model.depth_effort_level.payload(),
            )?;
            let chat_gpt_model = catalog.resolve(
                &role_depth_identifier,
                ProviderSurface::ChatGpt,
                &chat_gpt_depth_model.model_identifier,
                *chat_gpt_depth_model.depth_effort_level.payload(),
            )?;
            depths.push(ResolvedDepth {
                depth_identifier: role_depth_identifier,
                claude_model,
                chat_gpt_model,
            });
        }
        if depths.is_empty() {
            return Err(Error::MissingRoleDepths);
        }
        Ok(depths)
    }

    fn descriptions(
        role_descriptions: RoleDescriptions,
    ) -> Result<BTreeMap<(RolePermissionIdentifier, RoleDepthIdentifier), RoleDescription>> {
        let mut entries = BTreeMap::new();
        for RoleDescriptionCell {
            role_permission_identifier,
            role_depth_identifier,
            role_description,
        } in role_descriptions.into_payload()
        {
            let cell = (role_permission_identifier, role_depth_identifier);
            if entries.insert(cell.clone(), role_description).is_some() {
                return Err(Error::DuplicateRoleDescription {
                    permission_identifier: cell.0.into_payload(),
                    depth_identifier: cell.1.into_payload(),
                });
            }
        }
        Ok(entries)
    }

    fn packets(
        &self,
        module_index: &ModuleIndex,
        universal_role_modules: &UniversalRoleModules,
    ) -> Result<Vec<RolePacket>> {
        let mut packets = Vec::new();
        for role in &self.roles {
            packets.extend(role.packets(module_index, universal_role_modules)?);
        }
        Ok(packets)
    }

    fn output_inventory(&self) -> RoleOutputInventory {
        RoleOutputInventory::new(
            self.roles
                .iter()
                .flat_map(GeneratedRole::output_paths)
                .collect(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationConfiguration {
    active_outputs: ActiveOutputs,
    module_dependencies: ModuleDependencies,
    target_module_insertions: TargetModuleInsertions,
    universal_role_modules: UniversalRoleModules,
    skill_module_compositions: SkillModuleCompositionIndex,
    generated_roles: GeneratedRoleIndex,
}

impl GenerationConfiguration {
    fn active(
        active_outputs: ActiveOutputs,
        module_dependencies: ModuleDependencies,
        target_module_insertions: TargetModuleInsertions,
        universal_role_modules: UniversalRoleModules,
        skill_module_compositions: SkillModuleCompositions,
        role_generation_sources: RoleGenerationSources,
    ) -> Result<Self> {
        let skill_module_compositions =
            SkillModuleCompositionIndex::new(skill_module_compositions, &active_outputs)?;
        let generated_roles = GeneratedRoleIndex::new(role_generation_sources)?;
        Ok(Self {
            active_outputs,
            module_dependencies,
            target_module_insertions,
            universal_role_modules,
            skill_module_compositions,
            generated_roles,
        })
    }

    fn active_skills(&self) -> Vec<ActiveSkill> {
        self.active_outputs
            .payload()
            .iter()
            .map(|output| match output {
                ActiveOutput::Skill(skill) => skill.clone(),
            })
            .collect()
    }

    fn module_index(&self) -> Result<ModuleIndex> {
        ModuleIndex::new(
            self.module_dependencies.clone(),
            self.target_module_insertions.clone(),
        )
    }

    fn skill_manifests(&self) -> Result<Vec<Manifest>> {
        let module_index = self.module_index()?;
        let mut manifests = Vec::new();
        for skill in self.active_skills() {
            for manifest in
                skill.first_class_manifests(&module_index, &self.skill_module_compositions)?
            {
                manifests.push(manifest);
            }
        }
        Ok(manifests)
    }

    fn role_packets(&self) -> Result<Vec<RolePacket>> {
        let module_index = self.module_index()?;
        self.generated_roles
            .packets(&module_index, &self.universal_role_modules)
    }

    fn role_visualizations(&self) -> Result<Vec<RoleVisualization>> {
        let module_index = self.module_index()?;
        let mut visualizations = Vec::new();
        for role in &self.generated_roles.roles {
            let mut packet_compositions = role
                .packets(&module_index, &self.universal_role_modules)?
                .into_iter()
                .map(|packet| RolePacketComposition {
                    output_path: packet.manifest.output_path,
                    output_surface: packet.manifest.output_surface,
                    modules: packet.manifest.modules,
                })
                .collect::<Vec<_>>();
            packet_compositions
                .sort_by(|left, right| left.output_path.as_ref().cmp(right.output_path.as_ref()));
            visualizations.push(RoleVisualization {
                output_identifier: role.output_identifier.clone(),
                role_packet_compositions: RolePacketCompositions::new(packet_compositions),
            });
        }
        visualizations.sort_by(|left, right| {
            left.output_identifier
                .as_ref()
                .cmp(right.output_identifier.as_ref())
        });
        Ok(visualizations)
    }

    fn role_output_inventory(&self) -> RoleOutputInventory {
        self.generated_roles.output_inventory()
    }

    fn expected_outputs(&self) -> Result<BTreeSet<String>> {
        let mut expected = BTreeSet::new();
        for manifest in self.skill_manifests()? {
            expected.insert(manifest.output_path.as_ref().to_owned());
        }
        for packet in self.role_packets()? {
            expected.insert(packet.manifest.output_path.as_ref().to_owned());
        }
        expected.insert(RoleOutputInventory::relative_path().into_payload());
        Ok(expected)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum GenerationJob {
    Manifest(ManifestAssembler),
    Rendered(RenderedOutput),
}

impl GenerationJob {
    fn generate(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        match self {
            Self::Manifest(assembler) => assembler.generate(mode),
            Self::Rendered(output) => output.write(mode),
        }
    }

    fn visualize(&self) -> Result<GeneratedOutputVisualization> {
        let (output_path, rendered) = match self {
            Self::Manifest(assembler) => {
                let output_path = WorkspacePath::new(
                    assembler.workspace_root.clone(),
                    PathBuf::from(assembler.manifest.output_path.as_ref()),
                )?;
                let rendered = assembler.render(&output_path)?;
                (
                    OutputPath::new(output_path.relative_path().to_string_lossy().into_owned()),
                    rendered,
                )
            }
            Self::Rendered(output) => (
                OutputPath::new(
                    output
                        .output_path
                        .relative_path()
                        .to_string_lossy()
                        .into_owned(),
                ),
                output.rendered.clone(),
            ),
        };
        Ok(GeneratedOutputVisualization {
            output_path,
            byte_count: ByteCount::new(rendered.len() as u64),
            line_count: LineCount::new(
                rendered.bytes().filter(|byte| *byte == b'\n').count() as u64
            ),
        })
    }

    fn output_path(&self) -> Result<WorkspacePath> {
        match self {
            Self::Manifest(assembler) => WorkspacePath::new(
                assembler.workspace_root.clone(),
                PathBuf::from(assembler.manifest.output_path.as_ref()),
            ),
            Self::Rendered(output) => Ok(output.output_path.clone()),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OutputPathIndex {
    physical_paths: BTreeMap<PathBuf, String>,
}

impl OutputPathIndex {
    fn new() -> Self {
        Self {
            physical_paths: BTreeMap::new(),
        }
    }

    fn validate(mut self, jobs: &[GenerationJob]) -> Result<()> {
        for job in jobs {
            self.record(job)?;
        }
        Ok(())
    }

    fn record(&mut self, job: &GenerationJob) -> Result<()> {
        let output_path = job.output_path()?;
        let physical_path = output_path.full_path();
        let relative_path = output_path.relative_path().to_string_lossy().into_owned();
        if self
            .physical_paths
            .insert(physical_path.clone(), relative_path.clone())
            .is_some()
        {
            return Err(Error::DuplicateOutputPath {
                relative_path,
                physical_path,
            });
        }
        Ok(())
    }
}

impl ActiveSkill {
    fn first_class_manifests(
        &self,
        module_index: &ModuleIndex,
        skill_module_compositions: &SkillModuleCompositionIndex,
    ) -> Result<Vec<Manifest>> {
        let mut module_identifiers = vec![self.module_identifier.clone()];
        module_identifiers
            .extend(skill_module_compositions.included_modules(&self.output_identifier));
        let mut manifests = Vec::new();
        for surface in self.target_surfaces.payload() {
            let output_surface = OutputSurface::from(surface);
            let modules = Modules::new(module_index.expanded_paths(
                &module_identifiers,
                ModuleUse::SkillContent,
                output_surface,
            )?);
            manifests.push(Manifest {
                output_path: OutputPath::new(
                    output_surface.skill_path(self.output_identifier.as_ref()),
                ),
                output_kind: OutputKind::Markdown,
                output_surface,
                frontmatter: self.frontmatter(),
                modules: modules.clone(),
            });
        }
        Ok(manifests)
    }

    fn frontmatter(&self) -> Frontmatter {
        Frontmatter::new(vec![
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("name"),
                frontmatter_value: FrontmatterValue::new(self.output_identifier.as_ref()),
            },
            FrontmatterEntry {
                frontmatter_key: FrontmatterKey::new("description"),
                frontmatter_value: FrontmatterValue::new(self.skill_description.as_ref()),
            },
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModuleIndex {
    entries: BTreeMap<ModuleIdentifier, ModuleDependency>,
    target_module_insertions: TargetModuleInsertions,
}

impl ModuleIndex {
    fn new(
        module_dependencies: ModuleDependencies,
        target_module_insertions: TargetModuleInsertions,
    ) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for dependency in module_dependencies.into_payload() {
            dependency.require_flat_source_path()?;
            let module_identifier = dependency.module_identifier.clone();
            if entries
                .insert(module_identifier.clone(), dependency)
                .is_some()
            {
                return Err(Error::DuplicateModule {
                    module_identifier: module_identifier.into_payload(),
                });
            }
        }
        Ok(Self {
            entries,
            target_module_insertions,
        })
    }

    fn expanded_paths(
        &self,
        module_identifiers: &[ModuleIdentifier],
        module_use: ModuleUse,
        output_surface: OutputSurface,
    ) -> Result<Vec<ModulePath>> {
        let mut expansion = ModuleExpansion::new(self, module_use, output_surface);
        for module_identifier in module_identifiers {
            expansion.append(module_identifier)?;
        }
        Ok(expansion.into_paths())
    }

    fn dependency(&self, module_identifier: &ModuleIdentifier) -> Result<&ModuleDependency> {
        self.entries
            .get(module_identifier)
            .ok_or_else(|| Error::MissingModule {
                module_identifier: module_identifier.as_ref().to_owned(),
            })
    }

    fn target_modules(
        &self,
        module_identifier: &ModuleIdentifier,
        output_surface: OutputSurface,
    ) -> Vec<ModuleIdentifier> {
        self.target_module_insertions
            .payload()
            .iter()
            .filter(|insertion| insertion.applies_to(module_identifier, output_surface))
            .flat_map(|insertion| insertion.included_modules.payload().iter().cloned())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleUse {
    SkillContent,
    RoleContent,
}

impl ModuleUse {
    fn expected_description(&self) -> &'static str {
        match self {
            Self::SkillContent => "RuntimeSkill",
            Self::RoleContent => "RuntimeSkill or RoleComposition",
        }
    }

    fn accepts(&self, module_kind: ModuleKind) -> bool {
        match self {
            Self::SkillContent => module_kind == ModuleKind::RuntimeSkill,
            Self::RoleContent => matches!(
                module_kind,
                ModuleKind::RuntimeSkill | ModuleKind::RoleComposition
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ModuleExpansion<'a> {
    module_index: &'a ModuleIndex,
    module_use: ModuleUse,
    output_surface: OutputSurface,
    resolved: BTreeSet<ModuleIdentifier>,
    visiting: Vec<ModuleIdentifier>,
    paths: Vec<ModulePath>,
}

impl<'a> ModuleExpansion<'a> {
    fn new(
        module_index: &'a ModuleIndex,
        module_use: ModuleUse,
        output_surface: OutputSurface,
    ) -> Self {
        Self {
            module_index,
            module_use,
            output_surface,
            resolved: BTreeSet::new(),
            visiting: Vec::new(),
            paths: Vec::new(),
        }
    }

    fn append(&mut self, module_identifier: &ModuleIdentifier) -> Result<()> {
        if self.resolved.contains(module_identifier) {
            return Ok(());
        }
        if let Some(position) = self
            .visiting
            .iter()
            .position(|visiting_identifier| visiting_identifier == module_identifier)
        {
            let mut module_identifiers: Vec<String> = self.visiting[position..]
                .iter()
                .map(|identifier| identifier.as_ref().to_owned())
                .collect();
            module_identifiers.push(module_identifier.as_ref().to_owned());
            return Err(Error::ModuleDependencyCycle { module_identifiers });
        }
        let dependency = self.module_index.dependency(module_identifier)?;
        dependency.require_accepted(self.module_use)?;
        self.visiting.push(module_identifier.clone());
        for dependency_identifier in dependency.dependency_modules.payload() {
            self.append(dependency_identifier)?;
        }
        self.visiting.pop();
        self.resolved.insert(module_identifier.clone());
        self.paths.push(dependency.module_path.clone());
        for target_module_identifier in self
            .module_index
            .target_modules(module_identifier, self.output_surface)
        {
            self.append(&target_module_identifier)?;
        }
        Ok(())
    }

    fn into_paths(self) -> Vec<ModulePath> {
        self.paths
    }
}

impl TargetModuleInsertion {
    fn applies_to(
        &self,
        module_identifier: &ModuleIdentifier,
        output_surface: OutputSurface,
    ) -> bool {
        &self.module_identifier == module_identifier && self.output_surface == output_surface
    }
}

impl ModuleDependency {
    fn require_flat_source_path(&self) -> Result<()> {
        let identifier = self.module_identifier.as_ref();
        let expected = format!("skills/{identifier}.md");
        if self.module_path.as_ref() == expected {
            Ok(())
        } else {
            Err(Error::InvalidModuleSourcePath {
                module_identifier: identifier.to_owned(),
                expected,
                actual: self.module_path.as_ref().to_owned(),
            })
        }
    }

    fn require_accepted(&self, module_use: ModuleUse) -> Result<()> {
        if module_use.accepts(self.module_kind) {
            Ok(())
        } else {
            Err(Error::InvalidModuleKind {
                module_identifier: self.module_identifier.as_ref().to_owned(),
                expected: module_use.expected_description().to_owned(),
                actual: self.module_kind.name().to_owned(),
            })
        }
    }
}

impl ModuleKind {
    fn name(&self) -> &'static str {
        match self {
            Self::RuntimeSkill => "RuntimeSkill",
            Self::RoleComposition => "RoleComposition",
        }
    }
}

impl From<&TargetSurface> for OutputSurface {
    fn from(surface: &TargetSurface) -> Self {
        match surface {
            TargetSurface::AgentsSkill => Self::AgentsSkill,
            TargetSurface::ClaudeSkill => Self::ClaudeSkill,
        }
    }
}

impl From<&RoleTargetSurface> for OutputSurface {
    fn from(surface: &RoleTargetSurface) -> Self {
        match surface {
            RoleTargetSurface::ClaudeAgent => Self::ClaudeAgent,
            RoleTargetSurface::CodexAgent => Self::CodexAgent,
            RoleTargetSurface::PiAgent => Self::PiAgent,
        }
    }
}

impl OutputSurface {
    fn skill_path(&self, module_name: &str) -> String {
        match self {
            Self::AgentsSkill => format!(".agents/skills/{module_name}/SKILL.md"),
            Self::ClaudeSkill => format!(".claude/skills/{module_name}/SKILL.md"),
            Self::Workspace | Self::ClaudeAgent | Self::CodexAgent | Self::PiAgent => {
                unreachable!("not a first-class skill surface")
            }
        }
    }

    fn role_path(&self, role_name: &str) -> String {
        match self {
            Self::ClaudeAgent => format!(".claude/agents/{role_name}.md"),
            Self::CodexAgent => format!(".codex/agents/{role_name}.toml"),
            Self::PiAgent => format!(".pi/agents/{role_name}.md"),
            Self::Workspace | Self::AgentsSkill | Self::ClaudeSkill => {
                unreachable!("not a role target surface")
            }
        }
    }

    fn role_output_kind(&self) -> OutputKind {
        match self {
            Self::CodexAgent => OutputKind::Toml,
            Self::ClaudeAgent | Self::PiAgent => OutputKind::Markdown,
            Self::Workspace | Self::AgentsSkill | Self::ClaudeSkill => {
                unreachable!("not a role target surface")
            }
        }
    }

    /// The harness this surface's target conditionals render for.
    ///
    /// `AgentsSkill` renders for Codex. Pi natively scans `.agents/skills` as
    /// well, so a Codex-only block on this surface also reaches Pi. That is
    /// accepted only while Pi is unused; a returning Pi needs its own skill
    /// destination before any further Codex-only skill content is added.
    fn render_target(&self) -> RenderTarget {
        match self {
            Self::ClaudeSkill | Self::ClaudeAgent => RenderTarget::Claude,
            Self::AgentsSkill | Self::CodexAgent => RenderTarget::Codex,
            Self::PiAgent => RenderTarget::Pi,
            Self::Workspace => unreachable!("no manifest renders a workspace surface"),
        }
    }

    fn is_skill(&self) -> bool {
        matches!(self, Self::AgentsSkill | Self::ClaudeSkill)
    }

    fn is_role(&self) -> bool {
        matches!(self, Self::ClaudeAgent | Self::CodexAgent | Self::PiAgent)
    }
}

/// A fully rendered output that must reach the workspace with no brace in it.
///
/// The check is brace-level rather than a scan for the six Jinja markers,
/// because the dangerous case is the near miss: `{ % if codex % }` is not a tag,
/// so it renders as literal text and would ship a conditional to an agent as
/// doctrine while containing no `{%`. No source or generated file carries a
/// brace today, so the cost of the stricter rule is nothing and the guarantee
/// is total.
#[derive(Clone, Debug, Eq, PartialEq)]
struct BraceFreeOutput<'a> {
    path: PathBuf,
    text: &'a str,
}

impl<'a> BraceFreeOutput<'a> {
    fn new(path: PathBuf, text: &'a str) -> Self {
        Self { path, text }
    }

    fn validate(&self) -> Result<()> {
        for (index, line) in self.text.lines().enumerate() {
            if line.contains(['{', '}']) {
                return Err(Error::TemplateLeak {
                    path: self.path.clone(),
                    line: index + 1,
                    line_text: line.to_owned(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SkillName {
    value: String,
}

impl SkillName {
    fn from_frontmatter(frontmatter: &Frontmatter) -> Self {
        let value = frontmatter
            .payload()
            .iter()
            .find(|entry| entry.frontmatter_key.as_ref() == "name")
            .map(|entry| entry.frontmatter_value.as_ref().to_owned())
            .unwrap_or_else(|| "<missing>".to_owned());
        Self { value }
    }

    fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SerializedSkillBlock<'a> {
    skill_name: SkillName,
    location: String,
    body: &'a str,
}

impl<'a> SerializedSkillBlock<'a> {
    fn new(skill_name: SkillName, location: String, body: &'a str) -> Self {
        Self {
            skill_name,
            location,
            body,
        }
    }

    fn validate_size(&self) -> Result<()> {
        let byte_count = self.byte_count();
        if byte_count > GENERATED_SKILL_BLOCK_BYTE_LIMIT {
            return Err(Error::GeneratedSkillBlockTooLarge {
                skill_name: self.skill_name.as_str().to_owned(),
                location: self.location.clone(),
                byte_count,
                limit: GENERATED_SKILL_BLOCK_BYTE_LIMIT,
            });
        }
        Ok(())
    }

    fn byte_count(&self) -> usize {
        format!(
            "<skill name=\"{}\" location=\"{}\">\n{}\n</skill>",
            self.skill_name.as_str(),
            self.location,
            self.body
        )
        .len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetiredCurrentDestinationProse<'a> {
    output_path: WorkspacePath,
    body: &'a str,
}

impl<'a> RetiredCurrentDestinationProse<'a> {
    fn new(output_path: WorkspacePath, body: &'a str) -> Self {
        Self { output_path, body }
    }

    fn validate(&self) -> Result<()> {
        for phrase in RETIRED_CURRENT_DESTINATION_PHRASES {
            if self.body.contains(phrase) {
                return Err(Error::RetiredCurrentDestinationProse {
                    path: self.output_path.full_path(),
                    phrase: (*phrase).to_owned(),
                });
            }
        }
        Ok(())
    }
}

fn validate_agent_execution_limits(output_path: &WorkspacePath, packet: &str) -> Result<()> {
    for line in packet.lines() {
        let Some(field_name) = configured_field_name(line) else {
            continue;
        };
        if is_execution_limit_field(field_name) {
            return Err(Error::GeneratedAgentExecutionLimit {
                path: output_path.full_path(),
                field_name: field_name.to_owned(),
            });
        }
    }
    Ok(())
}

fn configured_field_name(line: &str) -> Option<&str> {
    let line = line.trim();
    let delimiter = line.find([':', '='])?;
    let (field_name, value) = line.split_at(delimiter);
    let field_name = field_name.trim();
    let value = value[1..].trim();
    (!field_name.is_empty()
        && !value.is_empty()
        && field_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')))
    .then_some(field_name)
}

fn is_execution_limit_field(field_name: &str) -> bool {
    let normalized: String = field_name
        .bytes()
        .filter(u8::is_ascii_alphanumeric)
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect();
    EXECUTION_LIMIT_FIELD_PARTS
        .iter()
        .any(|part| normalized.contains(part))
        || (normalized.starts_with("max")
            && EXECUTION_LIMIT_MAXIMUMS
                .iter()
                .any(|maximum| normalized.contains(maximum)))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestAssembler {
    source_root: PathBuf,
    workspace_root: PathBuf,
    manifest: Manifest,
    leading_fragment: Option<String>,
}

impl ManifestAssembler {
    fn new(source_root: PathBuf, workspace_root: PathBuf, manifest: Manifest) -> Self {
        Self {
            source_root,
            workspace_root,
            manifest,
            leading_fragment: None,
        }
    }

    /// A generated role's own body is the packet's first fragment; its
    /// universal modules follow.
    fn with_leading_fragment(mut self, fragment: String) -> Self {
        self.leading_fragment = Some(fragment);
        self
    }

    fn generate(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        let output_path = WorkspacePath::new(
            self.workspace_root.clone(),
            PathBuf::from(self.manifest.output_path.as_ref()),
        )?;
        let rendered = self.render(&output_path)?;
        RenderedOutput::new(
            self.workspace_root.clone(),
            self.manifest.output_path.clone(),
            rendered,
        )?
        .write(mode)
    }

    fn render(&self, output_path: &WorkspacePath) -> Result<String> {
        let rendered = match self.manifest.output_kind {
            OutputKind::Markdown => self.render_markdown(output_path)?,
            OutputKind::Toml => self.render_toml(output_path)?,
            OutputKind::Nota => {
                unreachable!("derived NOTA outputs render without module manifests")
            }
        };
        BraceFreeOutput::new(output_path.full_path(), &rendered).validate()?;
        Ok(rendered)
    }

    fn render_markdown(&self, output_path: &WorkspacePath) -> Result<String> {
        let rendered = MarkdownAssembly::new(
            output_path.clone(),
            self.manifest.output_surface,
            self.manifest.frontmatter.payload().to_vec(),
            self.markdown_fragments()?,
        )
        .render()?;
        if self.manifest.output_surface.is_skill() {
            SerializedSkillBlock::new(
                SkillName::from_frontmatter(&self.manifest.frontmatter),
                output_path.relative_path().to_string_lossy().into_owned(),
                &rendered,
            )
            .validate_size()?;
        }
        if self.manifest.output_surface.is_role() {
            RetiredCurrentDestinationProse::new(output_path.clone(), &rendered).validate()?;
            validate_agent_execution_limits(output_path, &rendered)?;
        }
        Ok(rendered)
    }

    fn render_toml(&self, output_path: &WorkspacePath) -> Result<String> {
        let body = MarkdownAssembly::new(
            output_path.clone(),
            self.manifest.output_surface,
            Vec::new(),
            self.markdown_fragments()?,
        )
        .render()?;
        let developer_instructions = body;
        if self.manifest.output_surface.is_role() {
            RetiredCurrentDestinationProse::new(output_path.clone(), &developer_instructions)
                .validate()?;
        }
        let rendered =
            RoleToml::new(&self.manifest.frontmatter, developer_instructions).render()?;
        if self.manifest.output_surface.is_role() {
            validate_agent_execution_limits(output_path, &rendered)?;
        }
        Ok(rendered)
    }

    fn markdown_fragments(&self) -> Result<Vec<MarkdownFragment>> {
        let mut fragments = Vec::new();
        if let Some(text) = &self.leading_fragment {
            fragments.push(MarkdownFragment::from_text(
                WorkspacePath::new(
                    self.source_root.clone(),
                    PathBuf::from("manifests/role-permissions.nota"),
                )?,
                text,
            ));
        }
        let target = self.manifest.output_surface.render_target();
        for module_path in self.manifest.modules.payload() {
            fragments.push(MarkdownFragment::read(
                self.module_workspace_path(module_path)?,
                target,
            )?);
        }
        Ok(fragments)
    }

    fn module_workspace_path(&self, module_path: &ModulePath) -> Result<WorkspacePath> {
        WorkspacePath::new(
            self.source_root.clone(),
            PathBuf::from(module_path.as_ref()),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleToml {
    name: String,
    description: String,
    model: String,
    model_reasoning_effort: Option<String>,
    developer_instructions: String,
}

impl RoleToml {
    fn new(frontmatter: &Frontmatter, developer_instructions: String) -> Self {
        let mut name = String::new();
        let mut description = String::new();
        let mut model = String::new();
        let mut model_reasoning_effort = None;
        for entry in frontmatter.payload() {
            match entry.frontmatter_key.as_ref() {
                "name" => name = entry.frontmatter_value.as_ref().to_owned(),
                "description" => description = entry.frontmatter_value.as_ref().to_owned(),
                "model" => model = entry.frontmatter_value.as_ref().to_owned(),
                "model_reasoning_effort" => {
                    model_reasoning_effort = Some(entry.frontmatter_value.as_ref().to_owned())
                }
                _ => {}
            }
        }
        Self {
            name,
            description,
            model,
            model_reasoning_effort,
            developer_instructions,
        }
    }

    fn render(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str("name = ");
        output.push_str(&TomlString::new(&self.name).render());
        output.push('\n');
        output.push_str("description = ");
        output.push_str(&TomlString::new(&self.description).render());
        output.push('\n');
        output.push_str("model = ");
        output.push_str(&TomlString::new(&self.model).render());
        output.push('\n');
        if let Some(model_reasoning_effort) = &self.model_reasoning_effort {
            output.push_str("model_reasoning_effort = ");
            output.push_str(&TomlString::new(model_reasoning_effort).render());
            output.push('\n');
        }
        output.push_str("developer_instructions = ");
        output.push_str(&TomlString::new(&self.developer_instructions).render());
        output.push('\n');
        Ok(output)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TomlString<'a> {
    text: &'a str,
}

impl<'a> TomlString<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn render(&self) -> String {
        let mut output = String::from("\"");
        for character in self.text.chars() {
            match character {
                '\\' => output.push_str("\\\\"),
                '"' => output.push_str("\\\""),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                character => output.push(character),
            }
        }
        output.push('"');
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderedOutput {
    output_path: WorkspacePath,
    rendered: String,
}

impl RenderedOutput {
    fn new(workspace_root: PathBuf, output_path: OutputPath, rendered: String) -> Result<Self> {
        Ok(Self {
            output_path: WorkspacePath::new(workspace_root, PathBuf::from(output_path.as_ref()))?,
            rendered,
        })
    }

    fn write(&self, mode: GenerationMode) -> Result<GeneratedFile> {
        match mode {
            GenerationMode::Write => self.write_file()?,
            GenerationMode::Check => self.check_file()?,
        }
        Ok(GeneratedFile {
            output_path: OutputPath::new(
                self.output_path
                    .relative_path()
                    .to_string_lossy()
                    .into_owned(),
            ),
            byte_count: ByteCount::new(self.rendered.len() as u64),
        })
    }

    fn write_file(&self) -> Result<()> {
        let path = self.output_path.full_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::CreateDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&path, &self.rendered).map_err(|source| Error::WriteFile { path, source })
    }

    fn check_file(&self) -> Result<()> {
        let path = self.output_path.full_path();
        let current = fs::read_to_string(&path).map_err(|source| Error::ReadFile {
            path: path.clone(),
            source,
        })?;
        if current == self.rendered {
            Ok(())
        } else {
            Err(Error::StaleOutput { path })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleOutputInventory {
    output_paths: Vec<OutputPath>,
}

impl RoleOutputInventory {
    fn new(output_paths: Vec<OutputPath>) -> Self {
        Self { output_paths }
    }

    fn relative_path() -> OutputPath {
        OutputPath::new("skills/generated-role-outputs.nota")
    }

    fn render(&self) -> String {
        GeneratedRoleOutputs::new(self.output_paths.clone()).to_nota()
    }

    fn contains(&self, output_path: &OutputPath) -> bool {
        self.output_paths.contains(output_path)
    }

    fn remove_stale(&self, active: &Self, pruner: &WorkspacePruner) -> Result<()> {
        for output_path in &self.output_paths {
            if !active.contains(output_path) {
                pruner.remove_relative_path(output_path.as_ref())?;
            }
        }
        Ok(())
    }

    fn validate_no_stale_paths(&self, active: &Self, scan: &StaleOutputScan) -> Result<()> {
        for output_path in &self.output_paths {
            if !active.contains(output_path) {
                scan.require_absent(output_path.as_ref())?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleOutputInventoryFile {
    workspace_root: PathBuf,
}

impl RoleOutputInventoryFile {
    fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    fn read(&self) -> Result<RoleOutputInventory> {
        let workspace_path = WorkspacePath::new(
            self.workspace_root.clone(),
            PathBuf::from(RoleOutputInventory::relative_path().as_ref()),
        )?;
        let path = workspace_path.full_path();
        match fs::read_to_string(&path) {
            Ok(text) => {
                let generated_outputs = NotaSource::new(&text)
                    .parse::<GeneratedRoleOutputs>()
                    .map_err(|source| Error::DecodeNota { path, source })?;
                Ok(RoleOutputInventory::new(generated_outputs.into_payload()))
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                Ok(RoleOutputInventory::new(Vec::new()))
            }
            Err(source) => Err(Error::ReadFile { path, source }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspacePruner {
    workspace_root: PathBuf,
    configuration: GenerationConfiguration,
}

impl WorkspacePruner {
    fn new(workspace_root: PathBuf, configuration: GenerationConfiguration) -> Self {
        Self {
            workspace_root,
            configuration,
        }
    }

    fn prune(&self) -> Result<()> {
        self.remove_relative_path(".agents/skills")?;
        self.remove_relative_path(".claude/skills")?;
        self.remove_relative_path(RETIRED_SKILL_INDEX_PATH)?;
        RoleOutputInventoryFile::new(self.workspace_root.clone())
            .read()?
            .remove_stale(&self.configuration.role_output_inventory(), self)?;
        Ok(())
    }

    fn remove_relative_path(&self, relative_path: &str) -> Result<()> {
        let workspace_path =
            WorkspacePath::new(self.workspace_root.clone(), PathBuf::from(relative_path))?;
        let full_path = workspace_path.full_path();
        match fs::symlink_metadata(&full_path) {
            Ok(metadata) if metadata.is_dir() => {
                fs::remove_dir_all(&full_path).map_err(|source| Error::RemovePath {
                    path: full_path,
                    source,
                })?;
            }
            Ok(_) => {
                fs::remove_file(&full_path).map_err(|source| Error::RemovePath {
                    path: full_path,
                    source,
                })?;
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(Error::ReadFile {
                    path: full_path,
                    source,
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StaleOutputScan {
    workspace_root: PathBuf,
    configuration: GenerationConfiguration,
}

impl StaleOutputScan {
    fn new(workspace_root: PathBuf, configuration: GenerationConfiguration) -> Self {
        Self {
            workspace_root,
            configuration,
        }
    }

    fn validate(&self) -> Result<()> {
        let expected = self.configuration.expected_outputs()?;
        self.require_no_unexpected_skill_files(".agents/skills", &expected)?;
        self.require_no_unexpected_skill_files(".claude/skills", &expected)?;
        self.require_absent(RETIRED_SKILL_INDEX_PATH)?;
        RoleOutputInventoryFile::new(self.workspace_root.clone())
            .read()?
            .validate_no_stale_paths(&self.configuration.role_output_inventory(), self)
    }

    fn require_absent(&self, relative_path: &str) -> Result<()> {
        let workspace_path =
            WorkspacePath::new(self.workspace_root.clone(), PathBuf::from(relative_path))?;
        let full_path = workspace_path.full_path();
        match fs::symlink_metadata(&full_path) {
            Ok(_) => Err(Error::StaleGeneratedOutput { path: full_path }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(Error::ReadFile {
                path: full_path,
                source,
            }),
        }
    }

    fn require_no_unexpected_skill_files(
        &self,
        relative_directory: &str,
        expected: &BTreeSet<String>,
    ) -> Result<()> {
        let workspace_path = WorkspacePath::new(
            self.workspace_root.clone(),
            PathBuf::from(relative_directory),
        )?;
        let full_path = workspace_path.full_path();
        match fs::read_dir(&full_path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.map_err(|source| Error::ReadFile {
                        path: full_path.clone(),
                        source,
                    })?;
                    let skill_path = entry.path().join("SKILL.md");
                    if skill_path.exists() {
                        let relative_path = skill_path
                            .strip_prefix(&self.workspace_root)
                            .expect("skill path comes from workspace root")
                            .to_string_lossy()
                            .into_owned();
                        if !expected.contains(&relative_path) {
                            self.require_absent(&relative_path)?;
                        }
                    }
                }
                Ok(())
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(Error::ReadFile {
                path: full_path,
                source,
            }),
        }
    }
}

impl GenerationReport {
    pub fn to_nota_text(&self) -> String {
        GenerationOutcome::Generated(self.clone()).to_nota()
    }
}
