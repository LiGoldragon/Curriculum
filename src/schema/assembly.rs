//! The Dotos contract spoken by the skill assembler.
//!
//! These are the concrete values the program reads, writes, and returns. The
//! binding deliberately contains no generator-facing vocabulary or secondary
//! schema representation.

pub type String = std::string::String;
pub type Integer = u64;
pub type Path = String;

#[cfg(feature = "dotos-text")]
pub use dotos::{DotosDecodeError, DotosEncode, DotosSource};

macro_rules! contract {
    ($item:item) => {
        #[cfg_attr(
            feature = "dotos-text",
            derive(dotos::DotosDecode, dotos::DotosDecodeTraced, dotos::DotosEncode)
        )]
        $item
    };
}

macro_rules! record {
    ($name:ident { $($field:ident : $kind:ty),* $(,)? }) => {
        contract! {
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
            pub struct $name {
                $(pub $field: $kind,)*
            }
        }
    };
}

macro_rules! vector {
    ($name:ident($kind:ty)) => {
        contract! {
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
            pub struct $name(Vec<$kind>);
        }

        impl $name {
            pub fn new(payload: Vec<$kind>) -> Self { Self(payload) }
            pub fn payload(&self) -> &Vec<$kind> { &self.0 }
            pub fn into_payload(self) -> Vec<$kind> { self.0 }
        }

        impl From<Vec<$kind>> for $name {
            fn from(payload: Vec<$kind>) -> Self { Self::new(payload) }
        }
    };
}

macro_rules! string {
    ($name:ident) => {
        contract! {
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
            pub struct $name(String);
        }

        impl $name {
            pub fn new(payload: impl Into<String>) -> Self { Self(payload.into()) }
            pub fn payload(&self) -> &String { &self.0 }
            pub fn into_payload(self) -> String { self.0 }
        }

        impl From<String> for $name {
            fn from(payload: String) -> Self { Self::new(payload) }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str { self.0.as_str() }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool { self.0 == *other }
        }
    };
    ($name:ident ordered) => {
        contract! {
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
            #[rkyv(derive(PartialEq, Eq, PartialOrd, Ord))]
            pub struct $name(String);
        }

        impl $name {
            pub fn new(payload: impl Into<String>) -> Self { Self(payload.into()) }
            pub fn payload(&self) -> &String { &self.0 }
            pub fn into_payload(self) -> String { self.0 }
        }

        impl From<String> for $name {
            fn from(payload: String) -> Self { Self::new(payload) }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str { self.0.as_str() }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool { self.0 == *other }
        }
    };
}

macro_rules! integer {
    ($name:ident) => {
        contract! {
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
            pub struct $name(Integer);
        }

        impl $name {
            pub fn new(payload: Integer) -> Self { Self(payload) }
            pub fn payload(&self) -> &Integer { &self.0 }
            pub fn into_payload(self) -> Integer { self.0 }
        }

        impl From<Integer> for $name {
            fn from(payload: Integer) -> Self { Self::new(payload) }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl PartialEq<u64> for $name {
            fn eq(&self, other: &u64) -> bool { self.0 == *other }
        }

        impl PartialOrd<u64> for $name {
            fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }
    };
}

macro_rules! choice {
    ($name:ident { $($variant:ident $(($payload:ty))?),* $(,)? }) => {
        contract! {
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
            pub enum $name { $($variant $(($payload))?,)* }
        }
    };
}

macro_rules! copy_choice {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        contract! {
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
            pub enum $name { $($variant,)* }
        }
    };
}

string!(SourceRoot);
string!(WorkspaceRoot);
string!(ManifestPath);
string!(OutputPath);
string!(FrontmatterKey);
string!(FrontmatterValue);
string!(ModulePath);
string!(OutputIdentifier ordered);
string!(ModuleIdentifier ordered);
string!(RoleDescription);
string!(ModelIdentifier ordered);
string!(RolePermissionIdentifier ordered);
string!(PermissionBody);
string!(RoleDepthIdentifier ordered);
integer!(ByteCount);
integer!(LineCount);

copy_choice!(GenerationMode { Write, Check });
copy_choice!(OutputKind {
    Markdown,
    Dotos,
    Toml
});
copy_choice!(OutputSurface {
    Workspace,
    AgentsSkill,
    ClaudeSkill,
    ClaudeAgent,
    CodexAgent,
    PiAgent
});
copy_choice!(RoleTargetSurface {
    ClaudeAgent,
    CodexAgent,
    PiAgent
});
copy_choice!(ProviderSurface { Claude, ChatGpt });
copy_choice!(EffortLevel {
    Low,
    Medium,
    High,
    Xhigh
});
copy_choice!(ToolRestriction {
    Restricted,
    Unrestricted
});
copy_choice!(ModuleKind {
    RuntimeSkill,
    RoleComposition
});
copy_choice!(SkillCategory {
    Architecture,
    Craft,
    Programming,
    Workflow,
    Meta
});
copy_choice!(SkillTier {
    Apex,
    Keystroke,
    Topic,
    Mechanism
});
copy_choice!(TargetSurface {
    AgentsSkill,
    ClaudeSkill
});

choice!(Operation { Generate(GenerationRequest), Visualize(VisualizationRequest) });
choice!(GenerationOutcome { Generated(GenerationReport), Visualized(VisualizationReport) });
choice!(ActiveOutput { Skill(ActiveSkill) });

record!(GenerationRequest {
    source_root: SourceRoot,
    workspace_root: WorkspaceRoot,
    manifest_path: ManifestPath,
    generation_mode: GenerationMode,
});
record!(VisualizationRequest {
    source_root: SourceRoot,
    workspace_root: WorkspaceRoot,
    manifest_path: ManifestPath,
});
record!(GeneratedFile {
    output_path: OutputPath,
    byte_count: ByteCount
});
record!(GeneratedOutputVisualization {
    output_path: OutputPath,
    byte_count: ByteCount,
    line_count: LineCount,
});
record!(VisualizationReport {
    role_visualizations: RoleVisualizations,
    generated_output_visualizations: GeneratedOutputVisualizations,
});
record!(RoleVisualization {
    output_identifier: OutputIdentifier,
    role_packet_compositions: RolePacketCompositions,
});
record!(RolePacketComposition {
    output_path: OutputPath,
    output_surface: OutputSurface,
    modules: Modules,
});
record!(Manifest {
    output_path: OutputPath,
    output_kind: OutputKind,
    output_surface: OutputSurface,
    frontmatter: Frontmatter,
    modules: Modules,
});
record!(FrontmatterEntry {
    frontmatter_key: FrontmatterKey,
    frontmatter_value: FrontmatterValue,
});
record!(ActiveSkill {
    output_identifier: OutputIdentifier,
    module_identifier: ModuleIdentifier,
    skill_category: SkillCategory,
    skill_tier: SkillTier,
    target_surfaces: TargetSurfaces,
});
record!(ModelCatalogEntry {
    model_identifier: ModelIdentifier,
    provider_surface: ProviderSurface,
    accepted_effort_levels: AcceptedEffortLevels,
});
record!(RolePermission {
    role_permission_identifier: RolePermissionIdentifier,
    permission_body: PermissionBody,
    tool_restriction: ToolRestriction,
});
record!(RoleDepth {
    role_depth_identifier: RoleDepthIdentifier,
    claude_depth_model: ClaudeDepthModel,
    chat_gpt_depth_model: ChatGptDepthModel,
});
record!(ClaudeDepthModel {
    model_identifier: ModelIdentifier,
    depth_effort_level: DepthEffortLevel,
});
record!(ChatGptDepthModel {
    model_identifier: ModelIdentifier,
    depth_effort_level: DepthEffortLevel,
});
record!(RoleDescriptionCell {
    role_permission_identifier: RolePermissionIdentifier,
    role_depth_identifier: RoleDepthIdentifier,
    role_description: RoleDescription,
});
record!(SkillModuleComposition {
    output_identifier: OutputIdentifier,
    included_modules: IncludedModules,
});
record!(ModuleDependency {
    module_identifier: ModuleIdentifier,
    module_path: ModulePath,
    module_kind: ModuleKind,
});
record!(TargetModuleInsertion {
    module_identifier: ModuleIdentifier,
    output_surface: OutputSurface,
    included_modules: IncludedModules,
});

vector!(GeneratedFiles(GeneratedFile));
vector!(RoleVisualizations(RoleVisualization));
vector!(RolePacketCompositions(RolePacketComposition));
vector!(GeneratedOutputVisualizations(GeneratedOutputVisualization));
vector!(Frontmatter(FrontmatterEntry));
vector!(Modules(ModulePath));
vector!(ActiveOutputs(ActiveOutput));
vector!(IncludedModules(ModuleIdentifier));
vector!(ModelCatalog(ModelCatalogEntry));
vector!(AcceptedEffortLevels(EffortLevel));
vector!(RolePermissions(RolePermission));
vector!(RoleDepths(RoleDepth));
vector!(RoleDescriptions(RoleDescriptionCell));
vector!(ModuleDependencies(ModuleDependency));
vector!(UniversalRoleModules(ModuleIdentifier));
vector!(SkillModuleCompositions(SkillModuleComposition));
vector!(TargetModuleInsertions(TargetModuleInsertion));
vector!(GeneratedRoleOutputs(OutputPath));
vector!(TargetSurfaces(TargetSurface));

contract! {
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
    pub struct GenerationReport(GeneratedFiles);
}

impl GenerationReport {
    pub fn new(payload: GeneratedFiles) -> Self {
        Self(payload)
    }
    pub fn payload(&self) -> &GeneratedFiles {
        &self.0
    }
    pub fn into_payload(self) -> GeneratedFiles {
        self.0
    }
}

impl From<GeneratedFiles> for GenerationReport {
    fn from(payload: GeneratedFiles) -> Self {
        Self::new(payload)
    }
}

contract! {
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
    pub struct DepthEffortLevel(Option<EffortLevel>);
}

impl DepthEffortLevel {
    pub fn new(payload: Option<EffortLevel>) -> Self {
        Self(payload)
    }
    pub fn payload(&self) -> &Option<EffortLevel> {
        &self.0
    }
    pub fn into_payload(self) -> Option<EffortLevel> {
        self.0
    }
}

impl From<Option<EffortLevel>> for DepthEffortLevel {
    fn from(payload: Option<EffortLevel>) -> Self {
        Self::new(payload)
    }
}

impl Operation {
    pub fn generate(payload: GenerationRequest) -> Self {
        Self::Generate(payload)
    }
    pub fn visualize(payload: VisualizationRequest) -> Self {
        Self::Visualize(payload)
    }
}

impl GenerationOutcome {
    pub fn generated(payload: GeneratedFiles) -> Self {
        Self::Generated(GenerationReport::new(payload))
    }
    pub fn visualized(payload: VisualizationReport) -> Self {
        Self::Visualized(payload)
    }
}

impl ActiveOutput {
    pub fn skill(payload: ActiveSkill) -> Self {
        Self::Skill(payload)
    }
}

impl From<GenerationRequest> for Operation {
    fn from(payload: GenerationRequest) -> Self {
        Self::Generate(payload)
    }
}

impl From<VisualizationRequest> for Operation {
    fn from(payload: VisualizationRequest) -> Self {
        Self::Visualize(payload)
    }
}

impl From<GenerationReport> for GenerationOutcome {
    fn from(payload: GenerationReport) -> Self {
        Self::Generated(payload)
    }
}

impl From<VisualizationReport> for GenerationOutcome {
    fn from(payload: VisualizationReport) -> Self {
        Self::Visualized(payload)
    }
}

impl From<ActiveSkill> for ActiveOutput {
    fn from(payload: ActiveSkill) -> Self {
        Self::Skill(payload)
    }
}
