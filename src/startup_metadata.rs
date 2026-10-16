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
            "user-only" | "disable-model-invocation" if matches!(value.trim(), "true" | "false") => (),
            "user-invocable" if value.trim() == "true" => (),
            _ => return Err(format!("unsupported startup execution metadata: {key}")),
        }
    }
    if body.trim().is_empty() { return Err("empty startup module body".into()); }
    Ok(body)
}

pub(crate) fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"'); out
}
