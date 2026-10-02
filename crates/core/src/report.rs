//! Structured command results (spec §8).
//!
//! A `Report` is pure data: the first-line triple (`status`, `command`,
//! `object`), the process exit code, and ordered rows. Rendering happens
//! exactly once, in the CLI crate; nothing here formats columns or derives
//! status from already-formatted strings (spec §8.1.11 — status is decided
//! from structured facts by the code that builds the report).

use crate::consts;
use crate::error::Diagnostic;

/// First-line status (spec §8.1.3). `Warn` still exits 0 (spec §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warn,
    Error,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Warn => "warn",
            Status::Error => "error",
        }
    }
}

/// One output row.
///
/// Rendering contract (implemented once by the CLI renderer):
///
/// - `Fields`: `indent` spaces, then the key left-padded (by Unicode scalar
///   count) to `width` columns (no padding when the key is already longer),
///   then two spaces, then the values joined by two spaces. An empty key or
///   value renders as `/` so it remains a column under `/ {2,}/`. A row with
///   no values renders as indent + key only, not a synthetic value column.
///   This makes every row splittable on `/ {2,}/` (spec §8.1.4).
/// - `Text`: a line with the key alone, then one `    | ` prefixed line per
///   content line, verbatim, never folded or truncated (spec §8.1.7). A
///   final `\n` terminates the last line rather than adding an empty one;
///   empty content renders as the key line alone.
/// - `Diagnostic`: the raw diagnostic; the renderer renders message / why /
///   fix through the diag catalog with the invocation's `diag::Context`.
#[derive(Clone, Debug)]
pub enum Row {
    Fields {
        key: String,
        values: Vec<String>,
        indent: usize,
        width: usize,
    },
    Text {
        key: String,
        content: String,
    },
    Diagnostic(Diagnostic),
}

/// The complete result of one command invocation.
#[derive(Clone, Debug)]
pub struct Report {
    /// Subcommand name as shown in the first line (e.g. `read-notification`).
    pub command: &'static str,
    /// Display object: PATH without suffix or DIR (`.` when omitted), per
    /// spec §8.1.3 / §8.1.9.
    pub object: String,
    /// First-line status; the single authority for ok/warn/error.
    pub status: Status,
    /// Process exit code (spec §11): 0 for ok/warn, 1 for error, 3 for the
    /// retryable lock failures.
    pub exit: u8,
    /// Ordered output rows (diagnostics already in anchor order per §9.4).
    pub rows: Vec<Row>,
}

impl Report {
    /// A new successful report for `command` / `object`.
    pub fn new(command: &'static str, object: impl Into<String>) -> Self {
        Report {
            command,
            object: object.into(),
            status: Status::Ok,
            exit: 0,
            rows: Vec::new(),
        }
    }

    /// Key/value row: two-space indent, key padded to `KEY_WIDTH` (spec §8.1.5).
    pub fn kv(&mut self, key: impl Into<String>, values: Vec<String>) -> &mut Self {
        self.record(key, values, 2, consts::KEY_WIDTH)
    }

    /// Generic record row with explicit indent and key-column width
    /// (tree rows, notification records, state entry rows, …).
    pub fn record(
        &mut self,
        key: impl Into<String>,
        values: Vec<String>,
        indent: usize,
        width: usize,
    ) -> &mut Self {
        self.rows.push(Row::Fields {
            key: key.into(),
            values,
            indent,
            width,
        });
        self
    }

    /// Text block row (spec §8.1.7): key alone, then verbatim `| ` lines.
    pub fn text(&mut self, key: impl Into<String>, content: impl Into<String>) -> &mut Self {
        self.rows.push(Row::Text {
            key: key.into(),
            content: content.into(),
        });
        self
    }

    /// Append a raw diagnostic row (rendered by the CLI via the catalog).
    pub fn diagnostic(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.rows.push(Row::Diagnostic(diagnostic));
        self
    }

    /// Promote a successful report to `warn`; the exit code stays 0 and an
    /// already-failed report is not downgraded (spec §8.1.11, §11).
    pub fn warn(&mut self) -> &mut Self {
        if self.status == Status::Ok {
            self.status = Status::Warn;
        }
        self
    }

    /// Closing `total` row for list-style outputs (spec §8.1.10).
    pub fn total(&mut self, count: usize) -> &mut Self {
        self.kv("total", vec![count.to_string()])
    }
}
