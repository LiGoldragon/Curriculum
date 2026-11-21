use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use datom_codec::Potential;
use sha2::{Digest, Sha256};
use signal_curriculum_nexus::{
    Effort, Input, Output as CurriculumOutput, Permission, RoleComposition, RoleSelection,
    RoleTools, Surface,
};
use thiserror::Error;

use crate::{
    client::{BudgetedActualizing, ClientError, DatomFaulting, PresentsOutput, exchange},
    projection_types::{
        GeneratedRoleOutputDocument, GeneratedRoleOutputs, GeneratedRoleProvenance,
        GeneratedRoleProvenanceDocument, ModuleIdentityText, ProjectionOutput, ProjectionRequest,
        RoleOutputPath, RoleOutputProvenance, RoleOutputSha256, RoleProjectionConfiguration,
        RoleSourceProvenance,
    },
    registry::IdentityText,
};

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error("invalid or reserved role output path: {0}")]
    Path(PathBuf),
    #[error("duplicate role output path: {0}")]
    Duplicate(PathBuf),
    #[error("cannot read {0}: {1}")]
    Read(PathBuf, io::Error),
    #[error("cannot write {0}: {1}")]
    Write(PathBuf, io::Error),
    #[error("read-back differs at {0}")]
    Different(PathBuf),
    #[error("role composition was refused: {0}")]
    RoleRejected(String),
    #[error("Curriculum returned a different role, surface or reply")]
    RoleMismatch,
    #[error("invalid module identity in role provenance: {0}")]
    Identity(String),
}

pub fn run(args: impl IntoIterator<Item = String>) -> (std::process::ExitCode, String) {
    let arguments: Vec<_> = args.into_iter().collect();
    let result = match arguments.as_slice() {
        [argument] => Potential::<ProjectionRequest>::from(argument.as_str())
            .actualize_with_limit(1_048_576)
            .map_err(|error| Error::Client(error.datom_fault()))
            .and_then(|ProjectionRequest::ProjectRoles(configuration)| project(configuration)),
        _ => Err(ClientError::Arguments.into()),
    };
    let (status, output) = match result {
        Ok(count) => (
            std::process::ExitCode::SUCCESS,
            ProjectionOutput::RolesProjected(count),
        ),
        Err(error) => (
            std::process::ExitCode::FAILURE,
            ProjectionOutput::ProjectionRejected(error.to_string()),
        ),
    };
    (status, output.datom_text())
}

const INVENTORY: &str = "skills/generated-role-outputs.datom";
const PROVENANCE: &str = "skills/generated-role-provenance.datom";
static TEMPORARY: AtomicU64 = AtomicU64::new(0);

struct Output {
    relative: PathBuf,
    text: String,
    provenance: RoleOutputProvenance,
}

fn project(configuration: RoleProjectionConfiguration) -> Result<i64, Error> {
    let source_socket = PathBuf::from(configuration.curriculum_socket.0);
    let workspace = PathBuf::from(configuration.workspace_root.0);
    let previous = read_inventory(&workspace)?;
    let mut paths = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for path in &previous {
        let relative = safe_role_path(&path.0)?;
        check_destination(&workspace, &relative)?;
        paths.insert(relative);
    }
    for projection in &configuration.role_projection_vector {
        let relative = safe_role_path(&projection.role_output_path.0)?;
        check_destination(&workspace, &relative)?;
        if !seen.insert(relative.clone()) {
            return Err(Error::Duplicate(relative));
        }
        paths.insert(relative);
    }
    for path in &paths {
        if path
            .ancestors()
            .skip(1)
            .any(|parent| paths.contains(parent))
        {
            return Err(Error::Path(path.clone()));
        }
    }

    let mut outputs = Vec::new();
    for projection in configuration.role_projection_vector {
        let relative = PathBuf::from(&projection.role_output_path.0);
        let (composition, text) = composed(&source_socket, &projection.role_selection)?;
        let output_sha = sha256(text.as_bytes());
        let provenance = RoleOutputProvenance {
            role_output_path: projection.role_output_path,
            role_name: composition.role_name.clone(),
            surface: projection.role_selection.surface,
            role_output_sha256: RoleOutputSha256(output_sha),
            description_payload_sha256: composition.description_payload_sha256.clone(),
            context_payload_sha256: composition.context_payload_sha256.clone(),
            role_source_provenance_vector: composition
                .context_provenance_vector
                .iter()
                .map(|item| {
                    Ok(RoleSourceProvenance {
                        module_identity_text: ModuleIdentityText(
                            item.module_identity
                                .identity_text()
                                .map_err(Error::Identity)?,
                        ),
                        repository_name: item.repository_name.clone(),
                        source_revision: item.source_revision.clone(),
                        source_relative_path: item.source_relative_path.clone(),
                        source_sha256: item.source_sha256.clone(),
                        body_sha256: item.body_sha256.clone(),
                        rendered_sha256: item.rendered_sha256.clone(),
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?,
        };
        outputs.push(Output {
            relative,
            text,
            provenance,
        });
    }

    for output in &outputs {
        atomic_write(&workspace, &output.relative, &output.text)?;
    }
    let current: Vec<RoleOutputPath> = outputs
        .iter()
        .map(|output| output.provenance.role_output_path.clone())
        .collect();
    for output in &outputs {
        let path = workspace.join(&output.relative);
        let read = fs::read(&path).map_err(|error| Error::Read(path.clone(), error))?;
        if sha256(&read) != output.provenance.role_output_sha256.0 || read != output.text.as_bytes()
        {
            return Err(Error::Different(path));
        }
    }
    let current_set: BTreeSet<_> = current.iter().cloned().collect();
    for owned in previous {
        if !current_set.contains(&owned) {
            let relative = PathBuf::from(&owned.0);
            let path = workspace.join(relative);
            if path.is_file() {
                fs::remove_file(&path).map_err(|error| Error::Write(path, error))?;
            }
        }
    }

    let inventory = GeneratedRoleOutputDocument::GeneratedRoleOutputs(GeneratedRoleOutputs {
        role_output_path_vector: current.clone(),
    });
    atomic_write(&workspace, Path::new(INVENTORY), &inventory.datom_text())?;
    let provenance =
        GeneratedRoleProvenanceDocument::GeneratedRoleProvenance(GeneratedRoleProvenance {
            role_output_provenance_vector: outputs
                .iter()
                .map(|output| output.provenance.clone())
                .collect(),
        });
    atomic_write(&workspace, Path::new(PROVENANCE), &provenance.datom_text())?;
    read_inventory(&workspace).and_then(|read| {
        if read == current {
            Ok(())
        } else {
            Err(Error::Different(workspace.join(INVENTORY)))
        }
    })?;
    Ok(i64::try_from(outputs.len()).expect("vector fits in i64"))
}

fn composed(socket: &Path, selection: &RoleSelection) -> Result<(RoleComposition, String), Error> {
    let input = Input::ComposeRole(selection.clone());
    match exchange(socket, &input)? {
        CurriculumOutput::RoleComposed(composition)
            if composition.role_name == selection.role_name
                && composition.surface == selection.surface =>
        {
            let text = render(&composition);
            Ok((composition, text))
        }
        CurriculumOutput::Rejected(reason) => Err(Error::RoleRejected(reason)),
        _ => Err(Error::RoleMismatch),
    }
}

fn render(role: &RoleComposition) -> String {
    let effort = match role.effort {
        Effort::Low => "low",
        Effort::Medium => "medium",
        Effort::High => "high",
        Effort::Xhigh => "xhigh",
    };
    let tools = match &role.role_tools {
        RoleTools::Inherit => None,
        RoleTools::Explicit(tools) => Some(tools.join(", ")),
    };
    match role.surface {
        Surface::ClaudeAgent => {
            let tools = tools
                .map(|tools| format!("tools: [{tools}]\n"))
                .unwrap_or_default();
            format!(
                "---\nname: {}\ndescription: {:?}\nmodel: {:?}\neffort: {effort}\n{tools}---\n\n{}",
                role.role_name.0, role.description_text.0, role.model_id.0, role.context_text.0
            )
        }
        Surface::CodexAgent => format!(
            "name = {:?}\ndescription = {:?}\nmodel = {:?}\nmodel_reasoning_effort = {effort:?}\ndeveloper_instructions = {:?}\n",
            role.role_name.0,
            role.description_text.0,
            role.model_id.0,
            role.context_text.0.trim_end()
        ),
        Surface::PiAgent => {
            let restriction = if matches!(role.permission, Permission::Restricted) {
                "disallowed_tools: 'edit, write'\n"
            } else {
                ""
            };
            format!(
                "---\nname: {}\ndescription: {:?}\nmodel: 'openai-codex/{}'\nthinking: {effort}\nprojectRoleIdentity: {}\nprojectRoleDispatchKind: leaf\n{restriction}---\n\n{}",
                role.role_name.0,
                role.description_text.0,
                role.model_id.0,
                role.role_name.0,
                role.context_text.0
            )
        }
    }
}

fn read_inventory(workspace: &Path) -> Result<Vec<RoleOutputPath>, Error> {
    let path = workspace.join(INVENTORY);
    check_destination(workspace, Path::new(INVENTORY))?;
    check_destination(workspace, Path::new(PROVENANCE))?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let source = fs::read_to_string(&path).map_err(|error| Error::Read(path.clone(), error))?;
    let document = Potential::<GeneratedRoleOutputDocument>::from(source)
        .actualize_with_limit(1_048_576)
        .map_err(DatomFaulting::datom_fault)?;
    let GeneratedRoleOutputDocument::GeneratedRoleOutputs(outputs) = document;
    Ok(outputs.role_output_path_vector)
}

fn atomic_write(workspace: &Path, relative: &Path, body: &str) -> Result<(), Error> {
    let path = workspace.join(relative);
    let parent = path.parent().expect("output parent");
    fs::create_dir_all(parent).map_err(|error| Error::Write(parent.to_path_buf(), error))?;
    let serial = TEMPORARY.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".curriculum-project-{}-{serial}.tmp",
        std::process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| Error::Write(temporary.clone(), error))?;
    let result = file
        .write_all(body.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|error| Error::Write(temporary.clone(), error))
        .and_then(|()| fs::rename(&temporary, &path).map_err(|error| Error::Write(path, error)));
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn safe_role_path(path: &str) -> Result<PathBuf, Error> {
    let relative = Path::new(path);
    let normalized: PathBuf = relative.components().collect();
    if relative.as_os_str().is_empty()
        || normalized.as_os_str() != relative.as_os_str()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || [INVENTORY, PROVENANCE].iter().any(|metadata| {
            let metadata = Path::new(metadata);
            metadata.starts_with(relative) || relative.starts_with(metadata)
        })
    {
        return Err(Error::Path(relative.to_owned()));
    }
    Ok(normalized)
}

fn check_destination(workspace: &Path, relative: &Path) -> Result<(), Error> {
    let mut path = workspace.to_owned();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if components.peek().is_some() && metadata.is_dir() => {}
            Ok(metadata) if components.peek().is_none() && metadata.is_file() => {}
            Ok(_) => return Err(Error::Path(path)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(Error::Read(path, error)),
        }
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{read_frame, write_frame},
        projection_types::{CurriculumSocket, RoleProjection, WorkspaceRoot},
    };
    use signal_curriculum_nexus::{
        BodySha256, ContextPayloadSha256, ContextProvenance, ContextText, DescriptionPayloadSha256,
        DescriptionText, ModelId, ModuleIdentity, ModuleKind, ModuleSubject, RenderedSha256,
        RepositoryName, RoleCallers, RoleName, SourceRelativePath, SourceRevision, SourceSha256,
        StartupPlacement, Version, Versions,
    };
    use std::{os::unix::net::UnixListener, thread};
    use tempfile::TempDir;

    #[test]
    fn destinations_stay_beneath_the_workspace() {
        assert!(safe_role_path(".pi/agents/read.md").is_ok());
        assert!(safe_role_path("../outside").is_err());
        assert!(safe_role_path("/outside").is_err());
    }

    fn source(socket: PathBuf, outputs: Vec<CurriculumOutput>) -> thread::JoinHandle<()> {
        let listener = UnixListener::bind(socket).expect("mock Curriculum socket");
        thread::spawn(move || {
            for output in outputs {
                let (mut stream, _) = listener.accept().expect("projection connects");
                let request = read_frame(&mut stream).expect("input frame");
                let _: Input = rkyv::from_bytes::<_, rkyv::rancor::Error>(&request)
                    .expect("typed ComposeRole input");
                let response =
                    rkyv::to_bytes::<rkyv::rancor::Error>(&output).expect("typed output");
                write_frame(&mut stream, &response).expect("outcome frame");
            }
        })
    }

    fn composition(name: &str, surface: Surface) -> CurriculumOutput {
        CurriculumOutput::RoleComposed(RoleComposition {
            role_name: RoleName(name.into()),
            surface,
            model_id: ModelId("model".into()),
            effort: Effort::Medium,
            permission: Permission::Restricted,
            description_text: DescriptionText("reads a bounded target".into()),
            context_text: ContextText("context body\n".into()),
            role_tools: RoleTools::Inherit,
            role_callers: RoleCallers::Any,
            context_provenance_vector: vec![ContextProvenance {
                module_identity: ModuleIdentity {
                    module_kind: ModuleKind::Notion,
                    module_subject: ModuleSubject("module".into()),
                    versions: Versions::Single(Version(0)),
                },
                repository_name: RepositoryName("psyche-skills".into()),
                source_revision: SourceRevision("revision".into()),
                source_relative_path: SourceRelativePath("skills/role-module.md".into()),
                source_sha256: SourceSha256("source".into()),
                body_sha256: BodySha256("body".into()),
                rendered_sha256: RenderedSha256("rendered".into()),
                startup_placement: StartupPlacement::System,
                integer: 0,
            }],
            description_payload_sha256: DescriptionPayloadSha256("description".into()),
            context_payload_sha256: ContextPayloadSha256("context".into()),
        })
    }

    fn configuration(
        socket: &Path,
        workspace: &Path,
        name: &str,
        destination: &str,
    ) -> RoleProjectionConfiguration {
        RoleProjectionConfiguration {
            curriculum_socket: CurriculumSocket(socket.to_string_lossy().into_owned()),
            workspace_root: WorkspaceRoot(workspace.to_string_lossy().into_owned()),
            role_projection_vector: vec![RoleProjection {
                role_selection: RoleSelection {
                    role_name: RoleName(name.into()),
                    surface: Surface::PiAgent,
                },
                role_output_path: RoleOutputPath(destination.into()),
            }],
        }
    }

    #[test]
    fn projection_reads_back_hashes_cleans_only_owned_outputs_and_preserves_unrelated_files() {
        let root = TempDir::new().expect("temporary roots");
        let workspace = root.path().join("workspace");
        fs::create_dir_all(workspace.join(".pi/agents")).expect("workspace");
        let unrelated = workspace.join(".pi/agents/home-owned.md");
        fs::write(&unrelated, "home owned").expect("unrelated file");

        let first_socket = root.path().join("first.sock");
        let first = source(
            first_socket.clone(),
            vec![composition("read-trivial", Surface::PiAgent)],
        );
        let retired = ".pi/agents/read-trivial.md";
        assert_eq!(
            project(configuration(
                &first_socket,
                &workspace,
                "read-trivial",
                retired
            ))
            .expect("first projection"),
            1
        );
        first.join().expect("first source");

        let second_socket = root.path().join("second.sock");
        let second = source(
            second_socket.clone(),
            vec![composition("write-trivial", Surface::PiAgent)],
        );
        let current = ".pi/agents/write-trivial.md";
        project(configuration(
            &second_socket,
            &workspace,
            "write-trivial",
            current,
        ))
        .expect("replacement projection");
        second.join().expect("second source");

        assert!(
            !workspace.join(retired).exists(),
            "recorded owned output cleaned"
        );
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "home owned");
        let body = fs::read_to_string(workspace.join(current)).expect("projected role");
        let provenance = fs::read_to_string(workspace.join(PROVENANCE)).expect("provenance");
        assert!(provenance.contains(&sha256(body.as_bytes())));
        assert!(provenance.contains("revision"));
        assert!(provenance.contains("notion:module:0"));
        assert!(!provenance.contains(first_socket.to_string_lossy().as_ref()));
    }

    #[test]
    fn preflight_refuses_overlapping_outputs() {
        let root = TempDir::new().expect("temporary roots");
        let mut request = configuration(root.path(), root.path(), "first", ".pi/agents/first.md");
        let mut nested = request.role_projection_vector[0].clone();
        nested.role_output_path.0.push_str("/child");
        request.role_projection_vector.push(nested);
        assert!(matches!(project(request), Err(Error::Path(_))));
    }

    #[test]
    fn preflight_preserves_unrelated_files_behind_a_symlink() {
        let root = TempDir::new().expect("temporary roots");
        let workspace = root.path().join("workspace");
        fs::create_dir(&workspace).expect("workspace");
        let unrelated = root.path().join("unrelated.md");
        fs::write(&unrelated, "unrelated").expect("unrelated file");
        std::os::unix::fs::symlink(root.path(), workspace.join("linked")).expect("directory link");
        assert!(matches!(
            project(configuration(
                root.path(),
                &workspace,
                "role",
                "linked/unrelated.md"
            )),
            Err(Error::Path(_))
        ));
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "unrelated");
    }

    #[test]
    fn source_rejection_is_a_typed_failure_and_writes_nothing() {
        let root = TempDir::new().expect("temporary roots");
        let socket = root.path().join("curriculum.sock");
        let server = source(
            socket.clone(),
            vec![
                composition("known", Surface::PiAgent),
                CurriculumOutput::Rejected("unknown role/surface".into()),
            ],
        );
        let workspace = root.path().join("workspace");
        let mut request = configuration(&socket, &workspace, "known", ".pi/agents/known.md");
        request.role_projection_vector.extend(
            configuration(&socket, &workspace, "missing", ".pi/agents/missing.md")
                .role_projection_vector,
        );
        let result = project(request);
        server.join().expect("source");
        assert!(
            matches!(result, Err(Error::RoleRejected(reason)) if reason == "unknown role/surface")
        );
        assert!(!workspace.exists());
    }
}
