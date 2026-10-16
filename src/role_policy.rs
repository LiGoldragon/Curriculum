//! Explicit role context and dispatch declarations. Enforcement belongs to the
//! installed dispatcher; exporting this artifact does not authorize a caller.
use std::collections::BTreeSet;

use crate::generated::{RoleCallers, RoleContext, RolePacketPlan, RolePolicy, RoleTools, Roles, Surface};

pub const DISPATCH_POLICY_PATH: &str = "tools/role-dispatch-policy.json";

pub trait ValidatesRolePolicy {
    fn validate_for(&self, name: &str, surface: &Surface) -> Result<(), String>;
}

impl ValidatesRolePolicy for RolePolicy {
    fn validate_for(&self, name: &str, surface: &Surface) -> Result<(), String> {
        if self.string != name || !identifier(name) {
            return Err("invalid role policy target".into());
        }
        // Other harnesses do not yet consume this tool/caller contract.
        if !matches!(surface, Surface::ClaudeAgent) {
            return Err("explicit role policies currently require ClaudeAgent".into());
        }
        if let RoleContext::Explicit(names) = &self.role_context {
            unique_names(names, false)?;
        }
        if let RoleTools::Explicit(names) = &self.role_tools {
            let mut seen = BTreeSet::new();
            for name in names {
                if !matches!(name.as_str(), "Read" | "Grep" | "Glob" | "Bash" | "Edit" | "Write" | "WebFetch" | "WebSearch" | "LSP" | "Agent" | "Artifact" | "ToolSearch" | "ArtifactData" | "ArtifactComments") || !seen.insert(name) {
                    return Err(format!("invalid or duplicate explicit tool {name}"));
                }
            }
        }
        if let RoleCallers::Only(callers) = &self.role_callers {
            if callers.is_empty() {
                return Err("caller allowlist must not be empty".into());
            }
            let mut seen = BTreeSet::new();
            for caller in callers {
                let key = (&caller.first_string, &caller.second_string, &caller.third_string);
                if !word(&caller.first_string) || !word(&caller.second_string)
                    || !matches!(caller.third_string.as_str(), "Primary" | "Secondary" | "Tertiary" | "Quaternary")
                    || !seen.insert(key)
                {
                    return Err("invalid or duplicate caller metaflow".into());
                }
            }
        }
        Ok(())
    }
}

pub trait PlansRolePolicies {
    fn validate_policies(&self) -> Result<(), String>;
    fn validate_policy_targets(&self, packets: &[RolePacketPlan]) -> Result<(), String>;
}

impl PlansRolePolicies for Roles {
    fn validate_policies(&self) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        for policy in &self.role_policy_vector {
            policy.validate_for(&policy.string, &Surface::ClaudeAgent)?;
            if !seen.insert(&policy.string) {
                return Err(format!("duplicate policy for {}", policy.string));
            }
            if let RoleContext::Explicit(names) = &policy.role_context {
                for name in names {
                    let mut matches = self.role_module_vector.iter().filter(|module| &module.first_string == name);
                    let module = matches.next().ok_or_else(|| format!("missing explicit module {name}"))?;
                    if module.second_string.trim().is_empty() || matches.next().is_some() {
                        return Err(format!("empty or ambiguous explicit module {name}"));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_policy_targets(&self, packets: &[RolePacketPlan]) -> Result<(), String> {
        for policy in &self.role_policy_vector {
            let targets: Vec<_> = packets.iter().filter(|packet| packet.second_string == policy.string).collect();
            if targets.len() != 1 || !targets[0].first_string.starts_with(".claude/agents/") {
                return Err(format!("policy target must name exactly one Claude subagent: {}", policy.string));
            }
        }
        Ok(())
    }
}

pub fn explicit_context(packet: &RolePacketPlan) -> bool {
    packet.role_policy_option.as_ref().is_some_and(|policy| matches!(&policy.role_context, RoleContext::Explicit(_)))
}

/// Stable JSON, written transactionally with the role definition by the projector.
/// The guard must reject unknown versions and bind session identity independently.
pub fn dispatch_policy(packets: &[RolePacketPlan]) -> Result<String, String> {
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    for packet in packets {
        let Some(policy) = &packet.role_policy_option else { continue };
        policy.validate_for(&packet.second_string, &packet.surface)?;
        if packet.first_string != format!(".claude/agents/{}.md", packet.second_string)
            || !seen.insert(&packet.second_string)
        {
            return Err("invalid or duplicate dispatch policy path".into());
        }
        let context = match &policy.role_context {
            RoleContext::Inherit => "{\"kind\":\"inherit\"}".into(),
            RoleContext::Explicit(names) => format!("{{\"kind\":\"explicit\",\"modules\":{}}}", string_array(names)),
        };
        let tools = match &policy.role_tools {
            RoleTools::Inherit => "{\"kind\":\"inherit\"}".into(),
            RoleTools::Explicit(names) => format!("{{\"kind\":\"explicit\",\"names\":{}}}", string_array(names)),
        };
        let callers = match &policy.role_callers {
            RoleCallers::Any => "{\"kind\":\"any\"}".into(),
            RoleCallers::Only(callers) => {
                let tuples = callers.iter().map(|caller| format!("{{\"aspect\":{},\"topic\":{},\"layer\":{}}}", quote(&caller.first_string), quote(&caller.second_string), quote(&caller.third_string))).collect::<Vec<_>>();
                format!("{{\"kind\":\"only\",\"metaflows\":[{}]}}", tuples.join(","))
            }
        };
        entries.push((packet.second_string.clone(), format!("{{\"name\":{},\"surface\":\"ClaudeAgent\",\"definitionPath\":{},\"model\":{},\"allowModelOverride\":false,\"allowNestedCaller\":false,\"context\":{},\"tools\":{},\"callers\":{}}}", quote(&packet.second_string), quote(&packet.first_string), quote(&packet.fourth_string), context, tools, callers)));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(format!("{{\"schemaVersion\":1,\"roles\":[{}]}}\n", entries.into_iter().map(|(_, value)| value).collect::<Vec<_>>().join(",")))
}

fn identifier(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn word(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn unique_names(names: &[String], allow_empty: bool) -> Result<(), String> {
    if names.is_empty() && !allow_empty {
        return Err("explicit context must name at least one module".into());
    }
    let mut seen = BTreeSet::new();
    for name in names {
        if !identifier(name) || !seen.insert(name) {
            return Err(format!("invalid or duplicate module {name}"));
        }
    }
    Ok(())
}

pub fn string_array(values: &[String]) -> String {
    format!("[{}]", values.iter().map(|value| quote(value)).collect::<Vec<_>>().join(","))
}

fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            c if c <= '\u{1f}' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}
