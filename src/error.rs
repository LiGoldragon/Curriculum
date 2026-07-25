use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("expected exactly one NOTA argument or manifest file path, found {count}")]
    ArgumentCount { count: usize },

    #[error("read {path}: {source}")]
    ReadFile { path: PathBuf, source: io::Error },

    #[error("write {path}: {source}")]
    WriteFile { path: PathBuf, source: io::Error },

    #[error("remove generated path {path}: {source}")]
    RemovePath { path: PathBuf, source: io::Error },

    #[error("create directory {path}: {source}")]
    CreateDirectory { path: PathBuf, source: io::Error },

    #[error("decode NOTA from {path}: {source}")]
    DecodeNota {
        path: PathBuf,
        source: nota::NotaDecodeError,
    },

    #[error("decode NOTA argument: {0}")]
    DecodeNotaArgument(nota::NotaDecodeError),

    #[error("environment variable {variable} must name a generation root")]
    MissingEnvironmentRoot { variable: String },

    #[error("module `{module_identifier}` is listed more than once in module dependencies")]
    DuplicateModule { module_identifier: String },

    #[error("module `{module_identifier}` is missing from module dependencies")]
    MissingModule { module_identifier: String },

    #[error("module `{module_identifier}` source path `{actual}` must be `{expected}`")]
    InvalidModuleSourcePath {
        module_identifier: String,
        expected: String,
        actual: String,
    },

    #[error("module `{module_identifier}` has kind `{actual}`, expected {expected}")]
    InvalidModuleKind {
        module_identifier: String,
        expected: String,
        actual: String,
    },

    #[error("module dependency cycle: {}", module_identifiers.join(" -> "))]
    ModuleDependencyCycle { module_identifiers: Vec<String> },

    #[error("model `{model_identifier}` is listed more than once in the model catalog")]
    DuplicateModelCatalogEntry { model_identifier: String },

    #[error("model `{model_identifier}` lists effort `{effort}` more than once")]
    DuplicateModelCatalogEffort {
        model_identifier: String,
        effort: String,
    },

    #[error("skill `{skill_identifier}` is listed more than once in skill module compositions")]
    DuplicateSkillModuleComposition { skill_identifier: String },

    #[error("skill module composition names inactive skill `{skill_identifier}`")]
    StaleSkillModuleComposition { skill_identifier: String },

    #[error("role permissions must list at least one permission")]
    MissingRolePermissions,

    #[error("role depths must list at least one depth")]
    MissingRoleDepths,

    #[error("permission `{permission_identifier}` is listed more than once in role permissions")]
    DuplicateRolePermission { permission_identifier: String },

    #[error("depth `{depth_identifier}` is listed more than once in role depths")]
    DuplicateRoleDepth { depth_identifier: String },

    #[error(
        "role descriptions list permission `{permission_identifier}` with depth `{depth_identifier}` more than once"
    )]
    DuplicateRoleDescription {
        permission_identifier: String,
        depth_identifier: String,
    },

    #[error(
        "role descriptions have no cell for permission `{permission_identifier}` with depth `{depth_identifier}`"
    )]
    MissingRoleDescription {
        permission_identifier: String,
        depth_identifier: String,
    },

    #[error(
        "role descriptions carry cell `{permission_identifier}` with depth `{depth_identifier}`, which is outside the permission-by-depth cross product"
    )]
    StaleRoleDescription {
        permission_identifier: String,
        depth_identifier: String,
    },

    #[error("depth `{depth_identifier}` assigns unsupported model `{model_identifier}`")]
    UnsupportedRoleModel {
        depth_identifier: String,
        model_identifier: String,
    },

    #[error(
        "depth `{depth_identifier}` assigns `{model_identifier}` as {expected_provider}, but the catalog marks it {actual_provider}"
    )]
    RoleModelProviderMismatch {
        depth_identifier: String,
        model_identifier: String,
        expected_provider: String,
        actual_provider: String,
    },

    #[error(
        "depth `{depth_identifier}` assigns unsupported effort `{effort}` to model `{model_identifier}`"
    )]
    UnsupportedRoleModelEffort {
        depth_identifier: String,
        model_identifier: String,
        effort: String,
    },

    #[error(
        "depth `{depth_identifier}` assigns no effort to model `{model_identifier}`, which accepts one"
    )]
    MissingRoleModelEffort {
        depth_identifier: String,
        model_identifier: String,
    },

    #[error(
        "depth `{depth_identifier}` assigns effort `{effort}` to model `{model_identifier}`, which accepts none"
    )]
    EffortlessRoleModelCarriesEffort {
        depth_identifier: String,
        model_identifier: String,
        effort: String,
    },

    #[error(
        "generated output path `{relative_path}` resolves to duplicate physical path {physical_path}"
    )]
    DuplicateOutputPath {
        relative_path: String,
        physical_path: PathBuf,
    },

    #[error("duplicate markdown heading `{heading}` in {path}")]
    DuplicateHeading { path: PathBuf, heading: String },

    #[error("markdown output {path} may contain at most one level-one title; found {count}")]
    InvalidTitleCount { path: PathBuf, count: usize },

    #[error("markdown heading jumps from level {previous} to {current} in {path}: `{heading}`")]
    HeadingLevelJump {
        path: PathBuf,
        previous: usize,
        current: usize,
        heading: String,
    },

    #[error("harness skill {path} must define YAML frontmatter")]
    MissingHarnessFrontmatter { path: PathBuf },

    #[error("harness skill {path} frontmatter must define `{key}`")]
    MissingHarnessFrontmatterKey { path: PathBuf, key: String },

    #[error("frontmatter is allowed only at the start of {path}")]
    NestedFrontmatter { path: PathBuf },

    #[error("frontmatter key `{key}` in {path} contains unsupported characters")]
    InvalidFrontmatterKey { path: PathBuf, key: String },

    #[error("frontmatter value for `{key}` in {path} must be a single line")]
    InvalidFrontmatterValue { path: PathBuf, key: String },

    #[error(
        "generated skill `{skill_name}` serialized block at `{location}` is {byte_count} bytes, exceeding the {limit} byte limit"
    )]
    GeneratedSkillBlockTooLarge {
        skill_name: String,
        location: String,
        byte_count: usize,
        limit: usize,
    },

    #[error("retired current-destination prose `{phrase}` appears in generated role output {path}")]
    RetiredCurrentDestinationProse { path: PathBuf, phrase: String },

    #[error(
        "generated agent packet {path} configures forbidden execution limit field `{field_name}`"
    )]
    GeneratedAgentExecutionLimit { path: PathBuf, field_name: String },

    #[error("relative path {path} escapes the workspace root {root}")]
    PathEscapesRoot { root: PathBuf, path: PathBuf },

    #[error(
        "generated output is stale: {path}. Update the locked `skills` input, run `nix run .#generate-skills -- <workspace-root>`, then rerun `nix run .#check-skills -- <workspace-root>`."
    )]
    StaleOutput { path: PathBuf },

    #[error(
        "stale generated archived/deleted skill output remains: {path}. Update the locked `skills` input, run `nix run .#generate-skills -- <workspace-root>`, then rerun `nix run .#check-skills -- <workspace-root>`."
    )]
    StaleGeneratedOutput { path: PathBuf },

    #[error(
        "skills source checkout {source_root} is not a descendant of the fetched remote trunk; regenerating from it would silently revert corrections already landed on trunk. Rebase your checkout onto the latest trunk first (jj: `jj git fetch`, then rebase your work onto `trunk()`), then retry."
    )]
    SourceNotDescendantOfTrunk { source_root: PathBuf },

    #[error("verify source trunk descent in {source_root}: run `jj {command}`: {source}")]
    TrunkGuardCommand {
        command: String,
        source_root: PathBuf,
        source: io::Error,
    },

    #[error("verify source trunk descent in {source_root}: `jj {command}` failed: {stderr}")]
    TrunkGuardCommandFailed {
        command: String,
        source_root: PathBuf,
        stderr: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
