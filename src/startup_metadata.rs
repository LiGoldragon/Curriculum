//! Conservative flattening: execution metadata is never silently discarded.
use sha2::{Digest, Sha256};

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Reads the header/body boundary without interpreting execution metadata.
pub(crate) fn framed_source(source: &str) -> Result<(&str, &str), String> {
    let (header, body) = match source.strip_prefix("---\n") {
        Some(rest) => rest
            .split_once("\n---\n")
            .ok_or("unterminated module frontmatter")?,
        None => ("", source),
    };
    if body.trim().is_empty() {
        return Err("empty startup module body".into());
    }
    Ok((header, body))
}

pub(crate) fn instructions(source: &str) -> Result<&str, String> {
    let (header, body) = framed_source(source)?;
    let mut keys = std::collections::BTreeSet::new();
    let mut continuation = false;
    for line in header.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if continuation {
                continue;
            }
            return Err("unsupported frontmatter continuation".into());
        }
        let (key, value) = line
            .split_once(':')
            .ok_or("unsupported frontmatter syntax")?;
        if !keys.insert(key) {
            return Err("duplicate frontmatter key".into());
        }
        continuation = matches!(key, "description" | "dependencies");
        match key {
            "description" | "dependencies" => (),
            "placement" if matches!(value.trim(), "system" | "user" | "on-demand") => (),
            "user-only" | "disable-model-invocation"
                if matches!(value.trim(), "true" | "false") => {}
            "user-invocable" if value.trim() == "true" => (),
            _ => return Err(format!("unsupported startup execution metadata: {key}")),
        }
    }
    Ok(body)
}

/// Empty means the source leaves placement to its explicit consumer contract.
pub(crate) fn declared_placement(source: &str) -> &str {
    source
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .and_then(|(header, _)| {
            header
                .lines()
                .find_map(|line| line.strip_prefix("placement:").map(str::trim))
        })
        .unwrap_or("")
}

/// The bounded scalar forms used by authored description frontmatter.
/// Quoted scalars occupy one line; plain continuation lines fold with spaces.
/// This does not interpret YAML documents, collections, or block scalars.
enum DescriptionScalar {
    Plain(String),
    Quoted(String),
}

trait DescriptionScalarDecoding: Sized {
    fn parse(raw: &str) -> Result<Self, String>;
    fn plain_fragment(value: &str) -> &str;
    fn continue_with(&mut self, line: &str) -> Result<(), String>;
    fn into_text(self) -> String;
}

impl DescriptionScalarDecoding for DescriptionScalar {
    fn parse(raw: &str) -> Result<Self, String> {
        let value = raw.trim();
        let Some(quote) = value.chars().next().filter(|c| matches!(c, '\'' | '"')) else {
            if value.starts_with(['>', '|', '[', '{']) {
                return Err("unsupported description block or collection".into());
            }
            return Ok(Self::Plain(Self::plain_fragment(value).to_owned()));
        };
        let mut characters = value[quote.len_utf8()..].char_indices().peekable();
        let mut decoded = String::new();
        while let Some((offset, character)) = characters.next() {
            if character == quote {
                if quote == '\'' && characters.peek().is_some_and(|(_, c)| *c == '\'') {
                    characters.next();
                    decoded.push('\'');
                    continue;
                }
                let suffix = &value[quote.len_utf8() + offset + character.len_utf8()..];
                if suffix.is_empty()
                    || (suffix.starts_with(char::is_whitespace)
                        && (suffix.trim().is_empty() || suffix.trim_start().starts_with('#')))
                {
                    return Ok(Self::Quoted(decoded));
                }
                return Err("unsupported trailing description tokens".into());
            }
            if quote == '"' && character == '\\' {
                let (_, escape) = characters.next().ok_or("unterminated description escape")?;
                let escaped = match escape {
                    '0' => '\0',
                    'a' => '\u{7}',
                    'b' => '\u{8}',
                    't' | '\t' => '\t',
                    'n' => '\n',
                    'v' => '\u{b}',
                    'f' => '\u{c}',
                    'r' => '\r',
                    'e' => '\u{1b}',
                    ' ' => ' ',
                    '"' => '"',
                    '/' => '/',
                    '\\' => '\\',
                    'N' => '\u{85}',
                    '_' => '\u{a0}',
                    'L' => '\u{2028}',
                    'P' => '\u{2029}',
                    'x' | 'u' | 'U' => {
                        let width = match escape {
                            'x' => 2,
                            'u' => 4,
                            'U' => 8,
                            _ => unreachable!(),
                        };
                        let mut codepoint = 0_u32;
                        for _ in 0..width {
                            let (_, digit) = characters
                                .next()
                                .ok_or("short description Unicode escape")?;
                            let digit = digit
                                .to_digit(16)
                                .ok_or("invalid description Unicode escape")?;
                            codepoint = codepoint
                                .checked_mul(16)
                                .and_then(|value| value.checked_add(digit))
                                .ok_or("invalid description Unicode scalar")?;
                        }
                        char::from_u32(codepoint).ok_or("invalid description Unicode scalar")?
                    }
                    _ => return Err("unsupported description escape".into()),
                };
                decoded.push(escaped);
            } else {
                decoded.push(character);
            }
        }
        Err("unterminated quoted description".into())
    }

    fn plain_fragment(value: &str) -> &str {
        let mut previous_is_space = true;
        for (offset, character) in value.char_indices() {
            if character == '#' && previous_is_space {
                return value[..offset].trim_end();
            }
            previous_is_space = character.is_whitespace();
        }
        value
    }

    fn continue_with(&mut self, line: &str) -> Result<(), String> {
        match self {
            Self::Plain(value) => {
                let fragment = Self::plain_fragment(line.trim());
                if !fragment.is_empty() {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(fragment);
                }
                Ok(())
            }
            Self::Quoted(_) => Err("unsupported continued quoted description".into()),
        }
    }

    fn into_text(self) -> String {
        match self {
            Self::Plain(value) | Self::Quoted(value) => value,
        }
    }
}

/// Reads only the description scalar; other metadata retains its own validation.
/// Full-line comments and blank lines are ignored, and absent descriptions are empty.
pub(crate) fn description(source: &str) -> Result<String, String> {
    let (header, _) = framed_source(source)?;
    let mut owner = None;
    let mut found: Option<DescriptionScalar> = None;
    for line in header.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            match owner {
                Some("description") => {
                    if let Some(description) = &mut found {
                        description.continue_with(line)?;
                    }
                    continue;
                }
                Some(_) => continue,
                None => return Err("unsupported orphan frontmatter continuation".into()),
            }
        }
        let Some((key, raw)) = line.split_once(':') else {
            return Err("unsupported frontmatter syntax".into());
        };
        owner = Some(key);
        if key != "description" {
            continue;
        }
        if found.is_some() {
            return Err("duplicate description".into());
        }
        found = Some(DescriptionScalar::parse(raw)?);
    }
    Ok(found.map(DescriptionScalar::into_text).unwrap_or_default())
}
