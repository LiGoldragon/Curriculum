//! Native output from registry composition, never caller-authored prose.
use crate::{
    generated::{Effort, Permission, RoleCallers, RoleComposition, RoleTools, Surface},
    startup_metadata::sha256,
};

pub trait RendersClaudeAgent {
    fn claude_agent_markdown(&self) -> Result<String, String>;
}

impl RendersClaudeAgent for RoleComposition {
    fn claude_agent_markdown(&self) -> Result<String, String> {
        if self.surface != Surface::ClaudeAgent || self.role_callers != RoleCallers::Any {
            return Err("native rendering requires an Any-caller Claude role".into());
        }
        let (expected_tools, tools_header) = match self.role_name.as_str() {
            "book" => (
                vec![
                    "Read".into(),
                    "Bash".into(),
                    "Agent".into(),
                    "Artifact".into(),
                ],
                "[Read, Bash, Agent, Artifact]",
            ),
            "illustrator" => (vec![], "[]"),
            _ => return Err("unsupported native role".into()),
        };
        if self.model_id != "sonnet"
            || self.effort != Effort::Medium
            || self.permission != Permission::Unrestricted
            || self.role_tools != RoleTools::Explicit(expected_tools)
        {
            return Err(
                "unsupported native role metadata; qualify a changed contract first".into(),
            );
        }
        if sha256(self.description_text.as_bytes()) != self.description_payload_sha256
            || sha256(self.context_text.as_bytes()) != self.context_payload_sha256
            || self.context_provenance_vector.is_empty()
        {
            return Err("composed role payload verification failed".into());
        }
        let description = self.description_text.yaml_string();
        Ok(format!(
            "---\nname: {}\ndescription: {description}\nmodel: sonnet\neffort: medium\ntools: {tools_header}\n---\n\n{}",
            self.role_name, self.context_text
        ))
    }
}

trait QuotesYaml {
    fn yaml_string(&self) -> String;
}

impl QuotesYaml for str {
    fn yaml_string(&self) -> String {
        let mut quoted = String::from("\"");
        for c in self.chars() {
            match c {
                '"' => quoted.push_str("\\\""),
                '\\' => quoted.push_str("\\\\"),
                '\n' => quoted.push_str("\\n"),
                '\r' => quoted.push_str("\\r"),
                '\t' => quoted.push_str("\\t"),
                c if c.is_control() || c == '\u{2028}' || c == '\u{2029}' => {
                    quoted.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => quoted.push(c),
            }
        }
        quoted.push('"');
        quoted
    }
}
