//! Per-target conditionals inside skill and role source fragments.
//!
//! A source fragment is a template only in the narrow sense that it may gate
//! lines on the harness the output is rendered for.

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

impl RenderTarget {}

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
    pub fn render(&self, target: RenderTarget) -> Result<String> {
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
