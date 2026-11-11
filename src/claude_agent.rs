//! Native output from registry composition, never caller-authored prose.
use crate::{
    generated::{Effort, Permission, RoleComposition, RoleTools, Surface},
    startup_metadata::sha256,
};

pub trait RendersClaudeAgent {
    fn claude_agent_markdown(&self) -> Result<String, String>;
}

impl RendersClaudeAgent for RoleComposition {
    fn claude_agent_markdown(&self) -> Result<String, String> {
        if self.surface != Surface::ClaudeAgent {
            return Err("native rendering requires a Claude role".into());
        }
        if self.permission == Permission::Restricted && self.role_tools == RoleTools::Inherit {
            return Err("a restricted Claude role names its tools".into());
        }
        if sha256(self.description_text.as_bytes()) != self.description_payload_sha256
            || sha256(self.context_text.as_bytes()) != self.context_payload_sha256
            || self.context_provenance_vector.is_empty()
        {
            return Err("composed role payload verification failed".into());
        }
        let model = self.model_id.plain_name()?;
        let effort = self.effort.claude_effort();
        let tools_header = match &self.role_tools {
            RoleTools::Inherit => String::new(),
            RoleTools::Explicit(tools) => format!(
                "tools: [{}]\n",
                tools
                    .iter()
                    .map(|tool| tool.plain_name())
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            ),
        };
        let description = self.description_text.yaml_string();
        Ok(format!(
            "---\nname: {}\ndescription: {description}\nmodel: {model}\neffort: {effort}\n{tools_header}---\n\n{}",
            self.role_name.plain_name()?,
            self.context_text,
        ))
    }
}

trait NamesClaudeEffort {
    fn claude_effort(&self) -> &'static str;
}

impl NamesClaudeEffort for Effort {
    fn claude_effort(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}

/// A role, model or tool name goes into the frontmatter unquoted, so it is
/// ASCII letters, digits, `-`, `_` and `.` only.
trait ChecksPlainName {
    fn plain_name(&self) -> Result<&str, String>;
}

impl ChecksPlainName for str {
    fn plain_name(&self) -> Result<&str, String> {
        if !self.is_empty()
            && self
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            Ok(self)
        } else {
            Err(format!(
                "not a plain name for a native role header: {self:?}"
            ))
        }
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
