//! Per-target conditionals inside skill and role source fragments.
//!
//! A source fragment is a template only in the narrow sense that it may gate
//! lines on the harness the output is rendered for. The accepted grammar is
//! closed to `{% if <target> %}`, `{% else %}`, and `{% endif %}` so that
//! validation is total and a stray brace can never reach a generated file.

use minijinja::{Environment, UndefinedBehavior, context};

use crate::error::{Error, Result};

/// The single harness one generated file is rendered for.
///
/// Target is one value rather than a set of flags, so "exactly one target is
/// true" needs no assertion: a render with two targets true has no spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderTarget {
    Claude,
    Codex,
    Pi,
}

impl RenderTarget {
    /// The target names a conditional may test, in report order.
    pub const NAMES: [&'static str; 3] = ["claude", "codex", "pi"];

    fn known_names() -> String {
        Self::NAMES.join(", ")
    }
}

/// One source fragment together with the path that names it in diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetTemplate<'a> {
    source_path: &'a str,
    text: &'a str,
}

impl<'a> TargetTemplate<'a> {
    pub fn new(source_path: &'a str, text: &'a str) -> Self {
        Self { source_path, text }
    }

    /// Render this fragment for one target.
    ///
    /// Brace-free text is returned verbatim, which makes the engine a
    /// byte-identical no-op for every fragment that carries no conditional.
    pub fn render(&self, target: RenderTarget) -> Result<String> {
        if !self.text.contains(['{', '}']) {
            return Ok(self.text.to_owned());
        }
        self.validate_grammar()?;
        let mut environment = Environment::new();
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment.set_trim_blocks(true);
        environment.set_lstrip_blocks(true);
        environment.set_keep_trailing_newline(true);
        let rendered = environment
            .render_named_str(
                self.source_path,
                self.text,
                context! {
                    claude => target == RenderTarget::Claude,
                    codex => target == RenderTarget::Codex,
                    pi => target == RenderTarget::Pi,
                },
            )
            .map_err(|source| Error::TemplateRender {
                source_path: self.source_path.to_owned(),
                line: source.line(),
                detail: source.to_string(),
            })?;
        Ok(BlankLineRuns::new(rendered).collapsed())
    }

    /// Reject every brace that is not part of the closed conditional grammar.
    ///
    /// This runs before rendering so that a misspelled target, an open-grammar
    /// construct, and a brace in prose each fail with the source line rather
    /// than as a render error or, worse, as shipped doctrine.
    fn validate_grammar(&self) -> Result<()> {
        for (index, line) in self.text.lines().enumerate() {
            let line_number = index + 1;
            if !line.contains(['{', '}']) {
                continue;
            }
            let tag = ConditionalTag::recognize(line).ok_or_else(|| Error::TemplateSyntax {
                source_path: self.source_path.to_owned(),
                line: line_number,
                line_text: line.to_owned(),
                known_targets: RenderTarget::known_names(),
            })?;
            if let Some(target_name) = tag.target_name()
                && !RenderTarget::NAMES.contains(&target_name)
            {
                return Err(Error::UnknownTemplateTarget {
                    source_path: self.source_path.to_owned(),
                    line: line_number,
                    target_name: target_name.to_owned(),
                    known_targets: RenderTarget::known_names(),
                });
            }
        }
        Ok(())
    }
}

/// A whole source line that is exactly one conditional tag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConditionalTag<'a> {
    If(&'a str),
    Else,
    EndIf,
}

impl<'a> ConditionalTag<'a> {
    /// Recognize a line the grammar accepts, or nothing.
    ///
    /// This is a closed-set recognizer over three fixed literal shapes, not a
    /// parser for the template format; minijinja parses and renders the source.
    /// The tag must be the entire line apart from indentation, which is what
    /// lets a false block leave no residue behind.
    fn recognize(line: &'a str) -> Option<Self> {
        let body = line.trim().strip_prefix("{%")?.strip_suffix("%}")?.trim();
        match body {
            "else" => Some(Self::Else),
            "endif" => Some(Self::EndIf),
            _ => body
                .strip_prefix("if ")
                .map(str::trim)
                .filter(|target_name| {
                    !target_name.is_empty()
                        && target_name
                            .chars()
                            .all(|character| character.is_ascii_lowercase())
                })
                .map(Self::If),
        }
    }

    fn target_name(&self) -> Option<&'a str> {
        match self {
            Self::If(target_name) => Some(target_name),
            Self::Else | Self::EndIf => None,
        }
    }
}

/// Rendered text whose blank-line runs still show where a false block stood.
#[derive(Clone, Debug, Eq, PartialEq)]
struct BlankLineRuns {
    text: String,
}

impl BlankLineRuns {
    fn new(text: String) -> Self {
        Self { text }
    }

    /// Collapse every run of two or more blank lines to one and end with
    /// exactly one newline.
    ///
    /// `trim_blocks` and `lstrip_blocks` remove a block tag's own line, but not
    /// the blank line an author wrote before a block that renders empty. That
    /// leftover is visible damage in line-oriented doctrine.
    fn collapsed(&self) -> String {
        let mut lines: Vec<&str> = Vec::new();
        for line in self.text.lines() {
            let blank = line.trim().is_empty();
            if blank && lines.last().is_some_and(|last| last.trim().is_empty()) {
                continue;
            }
            lines.push(line);
        }
        while lines.last().is_some_and(|last| last.trim().is_empty()) {
            lines.pop();
        }
        if lines.is_empty() {
            return String::new();
        }
        let mut collapsed = lines.join("\n");
        collapsed.push('\n');
        collapsed
    }
}
