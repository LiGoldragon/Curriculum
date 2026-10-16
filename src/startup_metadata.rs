//! Conservative flattening: execution metadata is never silently discarded.
use sha2::{Digest, Sha256};

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn instructions(source: &str) -> Result<&str, String> {
    let Some(rest) = source.strip_prefix("---\n") else { return Ok(source) };
    let (header, body) = rest.split_once("\n---\n").ok_or("unterminated module frontmatter")?;
    let mut keys = std::collections::BTreeSet::new();
    let mut continuation = false;
    for line in header.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') { continue; }
        if line.starts_with(' ') || line.starts_with('\t') {
            if continuation { continue; }
            return Err("unsupported frontmatter continuation".into());
        }
        let (key, value) = line.split_once(':').ok_or("unsupported frontmatter syntax")?;
        if !keys.insert(key) { return Err("duplicate frontmatter key".into()); }
        continuation = matches!(key, "description" | "dependencies");
        match key {
            "description" | "dependencies" => (),
            "placement" if matches!(value.trim(), "system" | "user" | "on-demand") => (),
            "user-only" | "disable-model-invocation" if matches!(value.trim(), "true" | "false") => (),
            "user-invocable" if value.trim() == "true" => (),
            _ => return Err(format!("unsupported startup execution metadata: {key}")),
        }
    }
    if body.trim().is_empty() { return Err("empty startup module body".into()); }
    Ok(body)
}

/// Empty means the source leaves placement to its explicit consumer contract.
pub(crate) fn declared_placement(source: &str) -> &str {
    source.strip_prefix("---\n").and_then(|s| s.split_once("\n---\n"))
        .and_then(|(header, _)| header.lines().find_map(|line| line.strip_prefix("placement:").map(str::trim)))
        .unwrap_or("")
}
