//! Per-type document views (spec §3–§6).
//!
//! `doc::parse` builds the generic structure (H1, preamble, H2 sections,
//! fences) and then hands each type scanner a [`ScanCtx`] — the frozen
//! intra-team interface. Scanners push §9 diagnostics through the context;
//! `doc::parse` sorts everything at the end (anchor line, then catalog
//! order).

pub mod brief;
pub mod post;
pub mod profile;
pub mod state;

use std::ops::Range;

use chrono::{DateTime, Utc};

use crate::doc::{Body, Section};
use crate::error::Diagnostic;
use crate::text::{FenceBlock, LineIndex, Span};

/// Read-only scan context over one parsed document (frozen interface; type
/// scanners never touch the raw line/fence structures directly).
pub(crate) struct ScanCtx<'a> {
    source: &'a str,
    lines: &'a LineIndex,
    inside: &'a [bool],
    blocks: &'a [FenceBlock],
    pub(crate) now: DateTime<Utc>,
    diag: &'a mut Vec<Diagnostic>,
}

impl<'a> ScanCtx<'a> {
    pub(crate) fn new(
        source: &'a str,
        lines: &'a LineIndex,
        inside: &'a [bool],
        blocks: &'a [FenceBlock],
        now: DateTime<Utc>,
        diag: &'a mut Vec<Diagnostic>,
    ) -> Self {
        ScanCtx {
            source,
            lines,
            inside,
            blocks,
            now,
            diag,
        }
    }

    /// Append a diagnostic. Ordering (anchor line, catalog order) happens
    /// once in `doc::parse`; scanners push in whatever order they find
    /// things.
    pub(crate) fn push(&mut self, d: Diagnostic) {
        self.diag.push(d);
    }

    /// The `now` handed to `doc::parse` (validity checks, W-STATE-EXPIRED).
    pub(crate) fn now(&self) -> DateTime<Utc> {
        self.now
    }

    /// Whole source text (original bytes, valid UTF-8, BOM included).
    pub(crate) fn source(&self) -> &str {
        self.source
    }

    /// Text of a byte span.
    pub(crate) fn text(&self, span: &Span) -> &str {
        &self.source[span.start..span.end]
    }

    /// 0-based line index of a byte offset.
    pub(crate) fn idx_at(&self, offset: usize) -> usize {
        self.lines.idx_at(offset)
    }

    /// Line content without its terminator. (A leading BOM on line 0 is only
    /// stripped for H1 matching, which `doc::parse` does itself; type
    /// scanners never see line 0 because their regions start after the H1.)
    pub(crate) fn line_str(&self, idx: usize) -> &str {
        let l = self.lines.line(idx);
        &self.source[l.start..l.end]
    }

    /// Span of the line content (terminator excluded).
    pub(crate) fn line_span(&self, idx: usize) -> Span {
        let l = self.lines.line(idx);
        l.start..l.end
    }

    /// Span of the whole line including its terminator (what a deletion of
    /// the line removes, spec §6.4.1).
    pub(crate) fn line_full_span(&self, idx: usize) -> Span {
        let l = self.lines.line(idx);
        l.start..l.next
    }

    /// 0-based line indices whose bytes intersect `span` (a line whose start
    /// equals `span.end` is excluded). An unterminated last line is included
    /// when `span` runs to EOF; a phantom trailing-newline line is not.
    pub(crate) fn lines_between(&self, span: &Span) -> Range<usize> {
        self.lines.lines_in(span.start..span.end)
    }

    /// True when the line is part of a fence block (open/close/content).
    pub(crate) fn fenced(&self, idx: usize) -> bool {
        self.inside[idx]
    }

    /// True when the line contains an isolated `\r`: it matches no structure
    /// grammar at all (spec §2.1.2).
    pub(crate) fn lone_cr(&self, idx: usize) -> bool {
        self.line_str(idx).contains('\r')
    }

    /// First closed fence block whose OPEN line lies in `lines` — the body
    /// selection rule of spec §2.5.5 (info string not inspected).
    pub(crate) fn first_fence_in(&self, lines: &Range<usize>) -> Option<Body> {
        self.blocks
            .iter()
            .find(|b| b.open >= lines.start && b.open < lines.end)
            .map(|b| Body::from_block(self.source, b))
    }

    /// Lines of a section excluding its H2: `[head+1, section_end)`.
    pub(crate) fn section_body_lines(&self, section: &Section) -> Range<usize> {
        let all = self.lines_between(&section.span);
        let head = self.idx_at(section.head.start);
        (head + 1)..all.end
    }

    /// Nonempty non-comment preamble lines (spec §2.6.2, §5.2.4). Fences
    /// stay opaque; a `> ` reader comment is the sole exemption; a line
    /// with an isolated `\r` matches no comment grammar.
    pub(crate) fn stray_preamble(&mut self, lines: Range<usize>) {
        for idx in lines {
            if self.fenced(idx) {
                continue;
            }
            let line = self.line_str(idx);
            if is_blank_line(line) {
                continue;
            }
            if is_reader_comment(line.as_bytes()) {
                continue;
            }
            self.push(Diagnostic::new("W-STRAY", idx + 1).with("n", (idx + 1).to_string()));
        }
    }
}

/// `- <key>: <value>` property line (spec §2.6.1). Returns `(key, value)`
/// where value is everything after the `": "` separator (possibly empty).
/// The line must not contain `\r` (checked by callers).
pub(crate) fn property_parts(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- ")?;
    let colon = rest.find(':')?;
    let key = &rest[..colon];
    if key.is_empty() || !key.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        return None;
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return None;
    }
    // the separator is exactly `": "`; everything after it is the value
    let after = rest.get(colon + 1..)?;
    let value = after.strip_prefix(' ')?;
    Some((key, value))
}

/// Blank line: empty or only U+0020 and tabs (spec §1).
pub(crate) fn is_blank_line(s: &str) -> bool {
    s.chars().all(|c| c == ' ' || c == '\t')
}

/// Reader-comment line: prefix `> ` and no isolated `\r` (spec §2.1.2,
/// §2.6.2). `line` is logical content after a real LF / CRLF terminator
/// has already been removed; a remaining `\r` (including a terminal
/// lone CR) matches no comment grammar.
pub(crate) fn is_reader_comment(line: &[u8]) -> bool {
    line.starts_with(b"> ") && !line.contains(&b'\r')
}
