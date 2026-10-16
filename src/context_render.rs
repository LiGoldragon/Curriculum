//! Existing target/raw syntax, now strict about malformed and unknown directives.
use crate::generated::Target;
pub fn render(target: &Target, body: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut conditions = Vec::<(bool, bool)>::new();
    let mut raw = false;
    for line in body.split_inclusive('\n') {
        let directive = line.trim();
        if raw {
            if directive == "{% endraw %}" {
                raw = false;
            } else if conditions.iter().all(|(selected, _)| *selected) {
                output.push_str(line);
            }
            continue;
        }
        let selected = match directive {
            "{% if claude %}" => Some(matches!(target, Target::Claude)),
            "{% if codex %}" => Some(matches!(target, Target::Codex)),
            "{% if pi %}" => Some(matches!(target, Target::Pi)),
            "{% if opencode %}" => Some(matches!(target, Target::OpenCode)),
            "{% raw %}" => {
                raw = true;
                continue;
            }
            "{% else %}" => {
                let Some((selected, seen)) = conditions.last_mut() else {
                    return Err("else without condition".into());
                };
                if *seen {
                    return Err("duplicate else".into());
                }
                *selected = !*selected;
                *seen = true;
                continue;
            }
            "{% endif %}" => {
                if conditions.pop().is_none() {
                    return Err("endif without condition".into());
                }
                continue;
            }
            _ => None,
        };
        if let Some(selected) = selected {
            conditions.push((selected, false));
            continue;
        }
        if directive.starts_with("{%") || directive == "{% endraw %}" {
            return Err("unsupported context template directive".into());
        }
        if conditions.iter().all(|(selected, _)| *selected) {
            if line.contains("{{") || line.contains("}}") {
                return Err(
                    "unresolved context variable; no caller-inline substitution is allowed".into(),
                );
            }
            output.push_str(line);
        }
    }
    if raw || !conditions.is_empty() {
        return Err("unterminated context template".into());
    }
    Ok(output)
}
