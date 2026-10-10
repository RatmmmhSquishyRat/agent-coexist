//! The single renderer (spec §8): turns a [`Report`] plus the invocation's
//! [`diag::Context`] into the final stdout bytes, row by row.
//!
//! Contract (frozen in `agent_coexist::report`):
//!
//! - First line: `<status>  <command>  <object>` — two-space separators,
//!   status via `Status::as_str`. An empty object is shown as `/` so the
//!   column remains present under `/ {2,}/`.
//! - `Row::Fields`: indent spaces, key left-padded to `width` by Unicode
//!   scalar count (no padding when already longer), two spaces, values
//!   joined by two spaces. An empty key or value is shown as `/`. A row
//!   with no values is indent + key only, with no trailing spaces and no
//!   synthetic value column, so every line splits reliably on `/ {2,}/`
//!   (spec §8.1.4–5).
//! - `Row::Text`: a line with two spaces + the key, then one `    | ` line
//!   per content line, verbatim — never folded, never truncated, never
//!   slash-substituted (spec §8.1.7). Lines are the content's
//!   `\n`-terminated lines: a final `\n` ends the last line instead of
//!   opening an empty one, so a block shows as many lines as its
//!   `L<a>..L<b>` range; empty content is the key line alone.
//! - `Row::Diagnostic`: two spaces + level padded to 5 + two spaces +
//!   `L<line>` + two spaces + message, then `why` / `fix` lines at a
//!   9-space indent (spec §8.1.6). Message / why / fix come from the diag
//!   catalog through [`diag::render`]; the level is `error` when
//!   [`diag::is_error`] says so, else `warn`.
//!
//! Rows are streamed straight into the writer; no whole-screen string is
//! ever built. Every line ends with a bare LF, on Windows too (spec §8.1.1).

use std::io::Write;

use agent_coexist::diag;
use agent_coexist::report::{Report, Row};

/// Render `report` into `out` (spec §8). The first line and every row end
/// with `\n`.
pub fn render(report: &Report, dctx: &diag::Context, out: &mut dyn Write) -> std::io::Result<()> {
    writeln!(
        out,
        "{}  {}  {}",
        report.status.as_str(),
        report.command,
        column(&report.object)
    )?;
    for row in &report.rows {
        match row {
            Row::Fields {
                key,
                values,
                indent,
                width,
            } => {
                write!(out, "{}", " ".repeat(*indent))?;
                if values.is_empty() {
                    writeln!(out, "{}", column(key))?;
                } else {
                    write_padded(out, column(key), *width)?;
                    for value in values {
                        write!(out, "  {}", column(value))?;
                    }
                    writeln!(out)?;
                }
            }
            Row::Text { key, content } => {
                writeln!(out, "  {key}")?;
                for line in content.split_terminator('\n') {
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
                write!(out, "  ")?;
                write_padded(out, level, 5)?;
                writeln!(out, "  L{}  {}", diagnostic.line, rendered.message)?;
                writeln!(out, "         why  {}", rendered.why)?;
                writeln!(out, "         fix  {}", rendered.fix)?;
            }
        }
    }
    Ok(())
}

/// Visible stand-in for an empty scalar column. Nonempty text is returned
/// unchanged so ordinary output stays byte-for-byte. Shared by stdout
/// reports and the wait ready line; not an identity rewrite.
pub(crate) fn column(text: &str) -> &str {
    if text.is_empty() { "/" } else { text }
}

/// Write `text` left-aligned in `width` columns counted by Unicode
/// scalars (`chars().count()`); a key already at or past `width` is
/// written unchanged (spec §8.1.5, §13 Q14).
fn write_padded(out: &mut dyn Write, text: &str, width: usize) -> std::io::Result<()> {
    write!(out, "{text}")?;
    let len = text.chars().count();
    if len < width {
        write!(out, "{}", " ".repeat(width - len))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered_block(content: &str) -> String {
        let mut report = Report::new("read-brief", "docs/b");
        report.text("content", content);
        let mut out = Vec::new();
        render(&report, &diag::Context::default(), &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    /// A block shows the content's lines: a final LF closes the last line
    /// (so a one-line target reported as `L1..L1` shows one line), a real
    /// blank line stays visible, and empty content is the key alone.
    #[test]
    fn text_block_lines_follow_content_lines_not_the_terminator() {
        let head = "ok  read-brief  docs/b\n  content\n";
        assert_eq!(
            rendered_block("fn a(){}\n"),
            format!("{head}    | fn a(){{}}\n")
        );
        assert_eq!(
            rendered_block("fn a(){}"),
            format!("{head}    | fn a(){{}}\n")
        );
        assert_eq!(rendered_block("a\n\n"), format!("{head}    | a\n    | \n"));
        assert_eq!(
            rendered_block("a\n\nb"),
            format!("{head}    | a\n    | \n    | b\n")
        );
        assert_eq!(rendered_block(""), head);
    }
}
