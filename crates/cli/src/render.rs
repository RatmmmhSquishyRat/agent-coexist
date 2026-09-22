//! The single renderer (spec §8): turns a [`Report`] plus the invocation's
//! [`diag::Context`] into the final stdout bytes, row by row.
//!
//! Contract (frozen in `coexist_core::report`):
//!
//! - First line: `<status>  <command>  <object>` — two-space separators,
//!   status via `Status::as_str`.
//! - `Row::Fields`: indent spaces, key left-padded to `width` by Unicode
//!   scalar count (no padding when already longer), two spaces, values
//!   joined by two spaces. A row with no values is indent + key only, with
//!   no trailing spaces, so every line splits reliably on `/ {2,}/`
//!   (spec §8.1.4–5).
//! - `Row::Text`: a line with two spaces + the key, then one `    | ` line
//!   per content line, split on `\n`, verbatim — never folded, never
//!   truncated (spec §8.1.7).
//! - `Row::Diagnostic`: two spaces + level padded to 5 + two spaces +
//!   `L<line>` + two spaces + message, then `why` / `fix` lines at a
//!   9-space indent (spec §8.1.6). Message / why / fix come from the diag
//!   catalog through [`diag::render`]; the level is `error` when
//!   [`diag::is_error`] says so, else `warn`.
//!
//! Rows are streamed straight into the writer; no whole-screen string is
//! ever built. Every line ends with a bare LF, on Windows too (spec §8.1.1).

use std::io::Write;

use coexist_core::diag;
use coexist_core::report::{Report, Row};

/// Render `report` into `out` (spec §8). The first line and every row end
/// with `\n`.
pub fn render(report: &Report, dctx: &diag::Context, out: &mut dyn Write) -> std::io::Result<()> {
    writeln!(
        out,
        "{}  {}  {}",
        report.status.as_str(),
        report.command,
        report.object
    )?;
    for row in &report.rows {
        match row {
            Row::Fields {
                key,
                values,
                indent,
                width,
            } => {
                if values.is_empty() {
                    writeln!(out, "{}{}", " ".repeat(*indent), key)?;
                } else {
                    let mut line = " ".repeat(*indent);
                    line.push_str(&pad_end(key, *width));
                    line.push_str("  ");
                    line.push_str(&values.join("  "));
                    writeln!(out, "{line}")?;
                }
            }
            Row::Text { key, content } => {
                writeln!(out, "  {key}")?;
                for line in content.split('\n') {
                    writeln!(out, "    | {line}")?;
                }
            }
            Row::Diagnostic(diagnostic) => {
                let level = if diag::is_error(diagnostic.id) {
                    "error"
                } else {
                    "warn"
                };
                let rendered = diag::render(diagnostic, dctx);
                writeln!(
                    out,
                    "  {}  L{}  {}",
                    pad_end(level, 5),
                    diagnostic.line,
                    rendered.message
                )?;
                writeln!(out, "         why  {}", rendered.why)?;
                writeln!(out, "         fix  {}", rendered.fix)?;
            }
        }
    }
    Ok(())
}

/// Left-pad `text` with spaces to `width` columns counted by Unicode
/// scalars (`chars().count()`); a key already at or past `width` is
/// returned unchanged (spec §8.1.5, §13 Q14).
fn pad_end(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len >= width {
        text.to_string()
    } else {
        format!("{text}{}", " ".repeat(width - len))
    }
}
