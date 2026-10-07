use std::{fs, path::Path};

use crate::generated::{Effort, Permission, Provider, RolePacketPlan, RolePlan, Roles, Surface};

pub trait PlansRoles {
    fn role_plan(&self, workspace: &Path) -> Result<RolePlan, String>;
}

impl PlansRoles for Roles {
    fn role_plan(&self, workspace: &Path) -> Result<RolePlan, String> {
        let standing = standing_skills(&self.second_string_vector)?;
        let mut packets = Vec::new();
        for description in &self.role_description_vector {
            for surface in [Surface::ClaudeAgent, Surface::CodexAgent, Surface::PiAgent] {
                packets.push(self.packet(
                    &format!("{}-{}", description.first_string, description.second_string),
                    &description.first_string,
                    &description.second_string,
                    &description.third_string,
                    &surface,
                    workspace,
                )?);
            }
        }
        for alias in &self.role_alias_vector {
            for surface in &alias.surface_vector {
                packets.push(self.packet(
                    &alias.first_string,
                    &alias.second_string,
                    &alias.third_string,
                    &alias.fourth_string,
                    surface,
                    workspace,
                )?);
            }
        }
        Ok(RolePlan {
            role_packet_plan_vector: packets,
            first_string_vector: standing,
            second_string_vector: Vec::new(),
            string: String::new(),
        })
    }
}

impl Roles {
    fn packet(
        &self,
        identifier: &str,
        discipline: &str,
        depth: &str,
        description: &str,
        surface: &Surface,
        workspace: &Path,
    ) -> Result<RolePacketPlan, String> {
        let permission = self
            .role_permission_vector
            .iter()
            .find(|entry| entry.first_string == discipline)
            .ok_or_else(|| format!("missing permission {discipline}"))?;
        let depth_entry = self
            .role_depth_vector
            .iter()
            .find(|entry| entry.string == depth)
            .ok_or_else(|| format!("missing depth {depth}"))?;
        let choice = match surface {
            Surface::ClaudeAgent => &depth_entry.first_model_choice,
            Surface::CodexAgent | Surface::PiAgent => &depth_entry.second_model_choice,
        };
        let model = self
            .model_vector
            .iter()
            .find(|entry| entry.string == choice.string)
            .ok_or_else(|| format!("missing model {}", choice.string))?;
        if !provider_matches(&model.provider, surface)
            || choice.effort_option.as_ref().is_some_and(|effort| {
                !model
                    .effort_vector
                    .iter()
                    .any(|item| same_effort(item, effort))
            })
        {
            return Err(format!("invalid model choice {}", choice.string));
        }
        let mut modules = Vec::<String>::new();
        if restricted(&permission.permission) {
            modules.push(permission.second_string.clone());
        }
        for module_id in &self.first_string_vector {
            modules.push(self.module(module_id)?);
        }
        for insertion in self.target_insertion_vector.iter().filter(|entry| {
            same_surface(&entry.surface, surface)
                && self.first_string_vector.contains(&entry.string)
        }) {
            for module_id in &insertion.string_vector {
                modules.push(self.module(module_id)?);
            }
        }
        let path = match surface {
            Surface::ClaudeAgent => format!(".claude/agents/{identifier}.md"),
            Surface::CodexAgent => format!(".codex/agents/{identifier}.toml"),
            Surface::PiAgent => format!(".pi/agents/{identifier}.md"),
        };
        let procedure = path
            .strip_prefix(".claude/agents/")
            .and_then(|rest| rest.strip_suffix(".md"))
            .map(|name| workspace.join("subagents").join(format!("{name}.md")))
            .filter(|path| path.is_file())
            .map(|path| {
                fs::read_to_string(&path)
                    .map_err(|error| format!("read {}: {error}", path.display()))
            })
            .transpose()?
            .unwrap_or_default();
        Ok(RolePacketPlan {
            surface: surface.clone(),
            first_string: path,
            second_string: identifier.to_owned(),
            third_string: description.to_owned(),
            fourth_string: choice.string.clone(),
            effort: choice.effort_option.clone().unwrap_or(Effort::Low),
            permission: permission.permission.clone(),
            fifth_string: modules.join("\n\n"),
            sixth_string: procedure,
        })
    }

    fn module(&self, identifier: &str) -> Result<String, String> {
        self.role_module_vector
            .iter()
            .find(|module| module.first_string == identifier)
            .map(|module| module.second_string.clone())
            .ok_or_else(|| format!("missing role module {identifier}"))
    }
}

fn standing_skills(names: &[String]) -> Result<Vec<String>, String> {
    let mut unique = std::collections::BTreeSet::new();
    for name in names {
        if name.is_empty() {
            return Err("empty standing skill selection".into());
        }
        if !unique.insert(name) {
            return Err(format!("duplicate standing skill {name}"));
        }
    }
    Ok(names.to_vec())
}

fn provider_matches(provider: &Provider, surface: &Surface) -> bool {
    matches!(
        (provider, surface),
        (Provider::Claude, Surface::ClaudeAgent)
            | (Provider::ChatGpt, Surface::CodexAgent | Surface::PiAgent)
    )
}

fn same_effort(left: &Effort, right: &Effort) -> bool {
    matches!(
        (left, right),
        (Effort::Low, Effort::Low)
            | (Effort::Medium, Effort::Medium)
            | (Effort::High, Effort::High)
            | (Effort::Xhigh, Effort::Xhigh)
    )
}

fn restricted(permission: &Permission) -> bool {
    matches!(permission, Permission::Restricted)
}

fn same_surface(left: &Surface, right: &Surface) -> bool {
    matches!(
        (left, right),
        (Surface::ClaudeAgent, Surface::ClaudeAgent)
            | (Surface::CodexAgent, Surface::CodexAgent)
            | (Surface::PiAgent, Surface::PiAgent)
    )
}

fn effort_text(effort: &Effort) -> &'static str {
    match effort {
        Effort::Low => "low",
        Effort::Medium => "medium",
        Effort::High => "high",
        Effort::Xhigh => "xhigh",
    }
}

pub trait RendersRolePacketPlan {
    fn render(&self, standing_bodies: &[String]) -> String;
}

impl RendersRolePacketPlan for RolePacketPlan {
    fn render(&self, standing_bodies: &[String]) -> String {
        let mut body = self.fifth_string.clone();
        body.push('\n');
        for standing in standing_bodies {
            body.push_str("\n\n");
            body.push_str(standing);
            body.push('\n');
        }
        if !self.sixth_string.is_empty() {
            body.push('\n');
            body.push_str(&self.sixth_string);
        }
        let effort = effort_text(&self.effort);
        match self.surface {
            Surface::ClaudeAgent => format!(
                "---\nname: {}\ndescription: '{}'\nmodel: '{}'\neffort: {effort}\n---\n\n{body}",
                self.second_string,
                self.third_string.replace('\'', "''"),
                self.fourth_string
            ),
            Surface::CodexAgent => format!(
                "name = \"{}\"\ndescription = \"{}\"\nmodel = \"{}\"\nmodel_reasoning_effort = \"{effort}\"\ndeveloper_instructions = \"{}\"\n",
                self.second_string,
                self.third_string.replace('"', "\\\"").replace('\n', "\\n"),
                self.fourth_string,
                body.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
            ),
            Surface::PiAgent => {
                let restriction = if restricted(&self.permission) {
                    "disallowed_tools: 'edit, write'\n"
                } else {
                    ""
                };
                format!(
                    "---\nname: {}\ndescription: '{}'\nmodel: 'openai-codex/{}'\nthinking: {effort}\nprojectRoleIdentity: {}\nprojectRoleDispatchKind: leaf\n{restriction}---\n\n{body}",
                    self.second_string,
                    self.third_string.replace('\'', "''"),
                    self.fourth_string,
                    self.second_string
                )
            }
        }
    }
}
