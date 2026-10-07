use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{generated::RolePlan, registry::ResolvedSkill, roles::RendersRolePacketPlan};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Harness {
    Claude,
    Codex,
    Pi,
    OpenCode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Surface {
    directory: &'static str,
    harness: Harness,
    codex_policy: bool,
}

const SURFACES: [Surface; 5] = [
    Surface {
        directory: ".agents/skills",
        harness: Harness::Codex,
        codex_policy: true,
    },
    Surface {
        directory: ".claude/skills",
        harness: Harness::Claude,
        codex_policy: false,
    },
    Surface {
        directory: ".codex/skills",
        harness: Harness::Codex,
        codex_policy: true,
    },
    Surface {
        directory: ".pi/skills",
        harness: Harness::Pi,
        codex_policy: false,
    },
    Surface {
        directory: ".opencode/skills",
        harness: Harness::OpenCode,
        codex_policy: false,
    },
];

#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("cannot read generated skill path {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot write generated skill path {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("generated skill projection differs at {0}")]
    Different(PathBuf),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillProjector {
    workspace: PathBuf,
}

pub trait ConstructsSkillProjector {
    fn at(workspace: PathBuf) -> Self;
}

impl ConstructsSkillProjector for SkillProjector {
    fn at(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

pub trait ProjectsSkills {
    fn write(&self, skills: &[ResolvedSkill], role_plan: &RolePlan) -> Result<(), ProjectionError>;
    fn check(&self, skills: &[ResolvedSkill], role_plan: &RolePlan) -> Result<(), ProjectionError>;
}

impl ProjectsSkills for SkillProjector {
    fn write(&self, skills: &[ResolvedSkill], role_plan: &RolePlan) -> Result<(), ProjectionError> {
        let expected = outputs(skills, role_plan)?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let stage = self
            .workspace
            .join(format!(".curriculum-stage-{}-{nonce}", std::process::id()));
        let backup = self
            .workspace
            .join(format!(".curriculum-backup-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&stage).map_err(|source| ProjectionError::Write {
            path: stage.clone(),
            source,
        })?;
        for (relative, body) in &expected {
            let path = stage.join(&relative);
            let parent = path.parent().expect("generated skill parent");
            fs::create_dir_all(parent).map_err(|source| ProjectionError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
            fs::write(&path, body).map_err(|source| ProjectionError::Write { path, source })?;
        }
        for surface in SURFACES {
            let current = self.workspace.join(surface.directory);
            if let Some(parent) = current.parent() {
                fs::create_dir_all(parent).map_err(|source| ProjectionError::Write {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
        }
        let mut saved = Vec::new();
        for surface in SURFACES {
            let current = self.workspace.join(surface.directory);
            if current.exists() {
                let previous = backup.join(surface.directory);
                if let Some(parent) = previous.parent() {
                    fs::create_dir_all(parent).map_err(|source| ProjectionError::Write {
                        path: parent.to_path_buf(),
                        source,
                    })?;
                }
                if let Err(source) = fs::rename(&current, &previous) {
                    restore_surfaces(&self.workspace, &backup, &saved);
                    let _ = fs::remove_dir_all(&stage);
                    return Err(ProjectionError::Write {
                        path: current,
                        source,
                    });
                }
                saved.push(surface);
            }
        }
        let mut saved_files = Vec::new();
        for relative in managed_role_paths(role_plan) {
            let current = self.workspace.join(&relative);
            if !current.exists() {
                continue;
            }
            let previous = backup.join(&relative);
            if let Some(parent) = previous.parent() {
                if let Err(source) = fs::create_dir_all(parent) {
                    restore_surfaces(&self.workspace, &backup, &saved);
                    restore_files(&self.workspace, &backup, &saved_files);
                    let _ = fs::remove_dir_all(&stage);
                    return Err(ProjectionError::Write {
                        path: parent.to_path_buf(),
                        source,
                    });
                }
            }
            if let Err(source) = fs::rename(&current, &previous) {
                restore_surfaces(&self.workspace, &backup, &saved);
                restore_files(&self.workspace, &backup, &saved_files);
                let _ = fs::remove_dir_all(&stage);
                return Err(ProjectionError::Write {
                    path: current,
                    source,
                });
            }
            saved_files.push(relative);
        }
        let mut installed = Vec::<Surface>::new();
        for surface in SURFACES {
            let staged = stage.join(surface.directory);
            let current = self.workspace.join(surface.directory);
            if let Err(source) = fs::rename(&staged, &current) {
                for installed_surface in installed.iter().rev() {
                    let path = self.workspace.join(installed_surface.directory);
                    let _ = fs::remove_dir_all(path);
                }
                restore_surfaces(&self.workspace, &backup, &saved);
                restore_files(&self.workspace, &backup, &saved_files);
                let _ = fs::remove_dir_all(&stage);
                let _ = fs::remove_dir_all(&backup);
                return Err(ProjectionError::Write {
                    path: current,
                    source,
                });
            }
            installed.push(surface);
        }
        let mut installed_files = Vec::new();
        for relative in expected.keys().filter(|path| !is_skill_output(path)) {
            let staged = stage.join(relative);
            let current = self.workspace.join(relative);
            if let Some(parent) = current.parent() {
                if let Err(source) = fs::create_dir_all(parent) {
                    rollback_files(&self.workspace, &installed_files);
                    rollback_surfaces(&self.workspace, &installed);
                    restore_surfaces(&self.workspace, &backup, &saved);
                    restore_files(&self.workspace, &backup, &saved_files);
                    let _ = fs::remove_dir_all(&stage);
                    let _ = fs::remove_dir_all(&backup);
                    return Err(ProjectionError::Write {
                        path: parent.to_path_buf(),
                        source,
                    });
                }
            }
            if let Err(source) = fs::rename(&staged, &current) {
                rollback_files(&self.workspace, &installed_files);
                rollback_surfaces(&self.workspace, &installed);
                restore_surfaces(&self.workspace, &backup, &saved);
                restore_files(&self.workspace, &backup, &saved_files);
                let _ = fs::remove_dir_all(&stage);
                let _ = fs::remove_dir_all(&backup);
                return Err(ProjectionError::Write {
                    path: current,
                    source,
                });
            }
            installed_files.push(relative.clone());
        }
        let _ = fs::remove_dir_all(&backup);
        let _ = fs::remove_dir_all(&stage);
        Ok(())
    }

    fn check(&self, skills: &[ResolvedSkill], role_plan: &RolePlan) -> Result<(), ProjectionError> {
        let expected = outputs(skills, role_plan)?;
        let names = skills
            .iter()
            .map(|skill| skill.name.clone())
            .collect::<BTreeSet<_>>();
        for surface in SURFACES {
            let directory = self.workspace.join(surface.directory);
            let actual_names = if directory.exists() {
                fs::read_dir(&directory)
                    .map_err(|source| ProjectionError::Read {
                        path: directory.clone(),
                        source,
                    })?
                    .filter_map(Result::ok)
                    .filter(|item| item.path().join("SKILL.md").is_file())
                    .map(|item| item.file_name().to_string_lossy().into_owned())
                    .collect::<BTreeSet<_>>()
            } else {
                BTreeSet::new()
            };
            if actual_names != names {
                return Err(ProjectionError::Different(directory));
            }
            for skill in skills {
                let policy = directory.join(&skill.name).join("agents/openai.yaml");
                if policy.exists() && !surface.codex_policy {
                    return Err(ProjectionError::Different(policy));
                }
                if policy.exists() && surface.codex_policy && !user_only(&skill.body) {
                    return Err(ProjectionError::Different(policy));
                }
            }
        }
        for previous in &role_plan.second_string_vector {
            validate_role_path(previous)?;
            if !expected.contains_key(Path::new(previous)) && self.workspace.join(previous).exists()
            {
                return Err(ProjectionError::Different(self.workspace.join(previous)));
            }
        }
        for (relative, expected_body) in expected {
            let path = self.workspace.join(&relative);
            let actual = fs::read_to_string(&path).map_err(|source| ProjectionError::Read {
                path: path.clone(),
                source,
            })?;
            if actual != expected_body {
                return Err(ProjectionError::Different(path));
            }
        }
        Ok(())
    }
}

fn restore_surfaces(workspace: &Path, backup: &Path, surfaces: &[Surface]) {
    for surface in surfaces.iter().rev() {
        let previous = backup.join(surface.directory);
        if !previous.exists() {
            continue;
        }
        let current = workspace.join(surface.directory);
        if let Some(parent) = current.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::rename(previous, current);
    }
}

fn rollback_surfaces(workspace: &Path, surfaces: &[Surface]) {
    for surface in surfaces.iter().rev() {
        let _ = fs::remove_dir_all(workspace.join(surface.directory));
    }
}

fn restore_files(workspace: &Path, backup: &Path, files: &[PathBuf]) {
    for relative in files.iter().rev() {
        let previous = backup.join(relative);
        if !previous.exists() {
            continue;
        }
        let current = workspace.join(relative);
        if let Some(parent) = current.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::rename(previous, current);
    }
}

fn rollback_files(workspace: &Path, files: &[PathBuf]) {
    for relative in files.iter().rev() {
        let _ = fs::remove_file(workspace.join(relative));
    }
}

fn managed_role_paths(plan: &RolePlan) -> BTreeSet<PathBuf> {
    let mut paths = plan
        .second_string_vector
        .iter()
        .chain(
            plan.role_packet_plan_vector
                .iter()
                .map(|packet| &packet.first_string),
        )
        .map(PathBuf::from)
        .collect::<BTreeSet<_>>();
    paths.insert(PathBuf::from("skills/generated-role-outputs.datom"));
    paths.insert(PathBuf::from("tools/standing-skill-selection.mjs"));
    paths
}

fn validate_role_path(path: &str) -> Result<(), ProjectionError> {
    let parsed = Path::new(path);
    let normal = parsed
        .components()
        .all(|component| matches!(component, std::path::Component::Normal(_)));
    let allowed = [".claude/agents/", ".codex/agents/", ".pi/agents/"]
        .iter()
        .any(|prefix| path.starts_with(prefix) && path.len() > prefix.len())
        || path == "skills/generated-role-outputs.datom"
        || path == "tools/standing-skill-selection.mjs";
    if normal && allowed {
        Ok(())
    } else {
        Err(ProjectionError::Different(PathBuf::from(path)))
    }
}

fn is_skill_output(path: &Path) -> bool {
    SURFACES
        .iter()
        .any(|surface| path.starts_with(surface.directory))
}

trait RendersSkill {
    fn render(&self, skill: &ResolvedSkill) -> String;
}

impl RendersSkill for Surface {
    fn render(&self, skill: &ResolvedSkill) -> String {
        let rendered = render_body(self.harness, &skill.body);
        if self.harness == Harness::OpenCode {
            let name = format!("name: {}\n", skill.name);
            return match rendered.strip_prefix("---\n") {
                Some(rest) => format!("---\n{name}{rest}"),
                None => format!("---\n{name}---\n{rendered}"),
            };
        }
        if self.harness == Harness::Claude && user_only(&skill.body) {
            return rendered.replace("user-only: true", "disable-model-invocation: true");
        }
        rendered
    }
}

fn outputs(
    skills: &[ResolvedSkill],
    role_plan: &RolePlan,
) -> Result<BTreeMap<PathBuf, String>, ProjectionError> {
    let mut outputs = BTreeMap::new();
    for skill in skills {
        for surface in SURFACES {
            let root = PathBuf::from(surface.directory).join(&skill.name);
            outputs.insert(root.join("SKILL.md"), surface.render(skill));
            if surface.codex_policy && user_only(&skill.body) {
                outputs.insert(
                    root.join("agents/openai.yaml"),
                    "policy:\n  allow_implicit_invocation: false\n".into(),
                );
            }
        }
    }
    let standing_bodies = standing_closure(skills, &role_plan.first_string_vector)?
        .iter()
        .map(|skill| {
            let body = skill
                .body
                .split_once("\n---\n")
                .map(|(_, body)| body)
                .unwrap_or(&skill.body)
                .trim();
            if body.is_empty() {
                return Err(ProjectionError::Different(PathBuf::from(format!(
                    "empty standing skill {}",
                    skill.name
                ))));
            }
            Ok(body.to_owned())
        })
        .collect::<Result<Vec<_>, ProjectionError>>()?;
    for packet in &role_plan.role_packet_plan_vector {
        validate_role_path(&packet.first_string)?;
        outputs.insert(
            PathBuf::from(&packet.first_string),
            packet.render(&standing_bodies),
        );
    }
    validate_role_path("skills/generated-role-outputs.datom")?;
    validate_role_path("tools/standing-skill-selection.mjs")?;
    outputs.insert(
        PathBuf::from("skills/generated-role-outputs.datom"),
        role_plan.string.clone(),
    );
    outputs.insert(
        PathBuf::from("tools/standing-skill-selection.mjs"),
        standing_skill_selection_module(&role_plan.first_string_vector),
    );
    for previous in &role_plan.second_string_vector {
        validate_role_path(previous)?;
    }
    Ok(outputs)
}

fn standing_closure<'a>(
    skills: &'a [ResolvedSkill],
    roots: &[String],
) -> Result<Vec<&'a ResolvedSkill>, ProjectionError> {
    fn visit<'a>(
        name: &str,
        by_name: &BTreeMap<&str, &'a ResolvedSkill>,
        visiting: &mut Vec<String>,
        visited: &mut BTreeSet<String>,
        output: &mut Vec<&'a ResolvedSkill>,
    ) -> Result<(), ProjectionError> {
        if visited.contains(name) {
            return Ok(());
        }
        if let Some(start) = visiting.iter().position(|seen| seen == name) {
            let mut cycle = visiting[start..].to_vec();
            cycle.push(name.to_owned());
            return Err(ProjectionError::Different(PathBuf::from(format!(
                "standing skill dependency cycle: {}",
                cycle.join(" -> ")
            ))));
        }
        let skill = by_name.get(name).copied().ok_or_else(|| {
            ProjectionError::Different(PathBuf::from(format!("missing standing skill {name}")))
        })?;
        visiting.push(name.to_owned());
        for dependency in &skill.dependencies {
            visit(dependency, by_name, visiting, visited, output)?;
        }
        visiting.pop();
        visited.insert(name.to_owned());
        output.push(skill);
        Ok(())
    }

    let by_name = skills
        .iter()
        .map(|skill| (skill.name.as_str(), skill))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = Vec::new();
    let mut visited = BTreeSet::new();
    let mut output = Vec::new();
    for root in roots {
        visit(root, &by_name, &mut visiting, &mut visited, &mut output)?;
    }
    Ok(output)
}

fn standing_skill_selection_module(names: &[String]) -> String {
    let names = names
        .iter()
        .map(|name| format!("  {:?},", name))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "// Generated by curriculum-deploy from Curriculum roles.datom. Do not edit.\nexport const STANDING_SKILLS = Object.freeze([\n{names}\n]);\n"
    )
}

fn user_only(body: &str) -> bool {
    body.strip_prefix("---\n")
        .and_then(|header| header.split_once("\n---").map(|(header, _)| header))
        .is_some_and(|header| header.lines().any(|line| line.trim() == "user-only: true"))
}

fn render_body(harness: Harness, body: &str) -> String {
    let mut rendered = String::new();
    let mut conditions = Vec::<(bool, bool)>::new();
    let mut raw = false;
    for line in body.split_inclusive('\n') {
        let directive = line.trim();
        if raw {
            if directive == "{% endraw %}" {
                raw = false;
            } else if conditions.iter().all(|(selected, _)| *selected) {
                rendered.push_str(line);
            }
            continue;
        }
        let selected = match directive {
            "{% if claude %}" => Some(harness == Harness::Claude),
            "{% if codex %}" => Some(harness == Harness::Codex),
            "{% if pi %}" => Some(harness == Harness::Pi),
            "{% if opencode %}" => Some(harness == Harness::OpenCode),
            "{% raw %}" => {
                raw = true;
                continue;
            }
            "{% endraw %}" => continue,
            "{% else %}" => {
                if let Some((selected, else_seen)) = conditions.last_mut() {
                    if !*else_seen {
                        *selected = !*selected;
                        *else_seen = true;
                    }
                }
                continue;
            }
            "{% endif %}" => {
                conditions.pop();
                continue;
            }
            _ => None,
        };
        if let Some(selected) = selected {
            conditions.push((selected, false));
            continue;
        }
        if conditions.iter().all(|(selected, _)| *selected) {
            rendered.push_str(line);
        }
    }
    rendered
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::tempdir;

    use super::{ConstructsSkillProjector, ProjectionError, ProjectsSkills, SkillProjector};
    use crate::generated::RolePlan;
    use crate::registry::ResolvedSkill;

    #[test]
    fn all_five_harness_trees_follow_changed_added_and_deleted_skills() {
        let workspace = tempdir().expect("workspace");
        let projector = SkillProjector::at(workspace.path().to_path_buf());
        let role_plan = RolePlan {
            role_packet_plan_vector: Vec::new(),
            first_string_vector: Vec::new(),
            second_string_vector: Vec::new(),
            string: "GeneratedRoleOutputs.{ [] }".into(),
        };
        let skills = vec![
            ResolvedSkill {
                path: PathBuf::from("source/operation-main-flow.md"),
                name: "operation-main-flow".into(),
                body: "---\ndescription: main\nuser-only: true\n---\n\nMain v1.\n".into(),
                dependencies: vec![],
            },
            ResolvedSkill {
                path: PathBuf::from("source/compensation-behavior.md"),
                name: "compensation-behavior".into(),
                body: "Behavior.\n".into(),
                dependencies: vec![],
            },
        ];
        projector
            .write(&skills, &role_plan)
            .expect("project all surfaces");
        projector
            .check(&skills, &role_plan)
            .expect("projection is current");
        for surface in [
            ".agents/skills",
            ".claude/skills",
            ".codex/skills",
            ".pi/skills",
            ".opencode/skills",
        ] {
            assert!(
                workspace
                    .path()
                    .join(surface)
                    .join("operation-main-flow/SKILL.md")
                    .is_file()
            );
            assert!(
                workspace
                    .path()
                    .join(surface)
                    .join("compensation-behavior/SKILL.md")
                    .is_file()
            );
        }
        assert!(
            fs::read_to_string(
                workspace
                    .path()
                    .join(".claude/skills/operation-main-flow/SKILL.md")
            )
            .expect("Claude skill")
            .contains("disable-model-invocation: true")
        );
        assert!(
            workspace
                .path()
                .join(".agents/skills/operation-main-flow/agents/openai.yaml")
                .is_file()
        );
        assert!(
            workspace
                .path()
                .join(".codex/skills/operation-main-flow/agents/openai.yaml")
                .is_file()
        );
        let changed = vec![ResolvedSkill {
            path: PathBuf::from("source/operation-main-flow.md"),
            name: "operation-main-flow".into(),
            body: "Main v2.\n".into(),
            dependencies: vec![],
        }];
        projector
            .write(&changed, &role_plan)
            .expect("update and remove");
        projector
            .check(&changed, &role_plan)
            .expect("updated projection is current");
        for surface in [
            ".agents/skills",
            ".claude/skills",
            ".codex/skills",
            ".pi/skills",
            ".opencode/skills",
        ] {
            let root = workspace.path().join(surface);
            assert!(!root.join("compensation-behavior").exists());
            let projected = fs::read_to_string(root.join("operation-main-flow/SKILL.md"))
                .expect("updated skill");
            if surface == ".opencode/skills" {
                assert_eq!(projected, "---\nname: operation-main-flow\n---\nMain v2.\n");
            } else {
                assert_eq!(projected, "Main v2.\n");
            }
        }
        assert!(matches!(
            projector.check(&skills, &role_plan),
            Err(ProjectionError::Different(_))
        ));
    }
}
