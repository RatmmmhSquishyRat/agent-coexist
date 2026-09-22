//! Generic raw-byte document structure (spec §2, §7.3, §8.2 check).
//!
//! `parse` keeps every legal byte of the source, indexes lines, scans fences
//! globally, finds the H1 and the H2 sections, then delegates to the per-type
//! scanners ([`crate::types`]). Structural problems become diagnostics on the
//! returned document — only invalid UTF-8 and oversize input are hard
//! failures, because nothing else makes the whole file undecidable (spec
//! §2.7.4, impl_plan M2).

use std::path::Path;

use chrono::{DateTime, Utc};

use crate::consts;
use crate::diag;
use crate::error::{Diagnostic, Fault};
use crate::names;
use crate::text::{FenceScan, LineIndex, Span};
use crate::types::{self, ScanCtx};

/// The four managed file types (spec §1). Recorded only by the H1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Post,
    Brief,
    Profile,
    State,
}

impl Kind {
    /// The type word used in H1s, `list` output and `check`'s `type` row
    /// (`profile` is only a command-name alias, never a type word).
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Post => "post",
            Kind::Brief => "brief",
            Kind::Profile => "agent-profile",
            Kind::State => "state",
        }
    }
}

/// A fenced body. `span` covers the fence lines (open line content through
/// close line content, terminators excluded); `content` excludes the fence
/// lines (spec §2.5.5).
#[derive(Debug, Clone)]
pub struct Body {
    pub span: Span,
    pub content: Span,
}

/// One H2 section (spec §1): an H2 line and everything up to the next H2
/// line (fenced lines are never section boundaries).
#[derive(Debug, Clone)]
pub struct Section {
    /// Folded H2 title (spec §4.3.1, §10.9.1).
    pub title: String,
    /// The H2 line content, terminator excluded.
    pub head: Span,
    /// Whole section: H2 line start through the byte before the next H2 line
    /// (or EOF) — terminators included, ready for whole-section deletion
    /// (spec §10.3.5).
    pub span: Span,
    /// First closed fence block in the section, if any.
    pub body: Option<Body>,
}

/// A parsed document: original bytes plus typed views and diagnostics.
pub struct Document {
    /// The whole file as read; every span below indexes into this string.
    pub source: String,
    /// `None` when the H1 is missing or malformed (E-H1).
    pub kind: Option<Kind>,
    /// The H1 name: empty string for a valid `# .state`.
    pub name: Option<String>,
    /// The H1 line content (terminator excluded); `None` on E-H1.
    pub h1: Option<Span>,
    /// From the end of the H1 line's terminator (or byte 0 when the H1 is
    /// invalid) to the first H2 line (or EOF).
    pub preamble: Span,
    pub sections: Vec<Section>,
    /// All diagnostics, sorted by anchor line then §9 catalog order.
    pub diagnostics: Vec<Diagnostic>,
    pub post: types::post::Post,
    pub brief: types::brief::Brief,
    pub profile: Vec<types::profile::ProfileSection>,
    pub state: Vec<types::state::StateSection>,
    /// Kept for `line_at`; not part of the frozen surface.
    lines: LineIndex,
}

impl Document {
    /// Raw text of a span (original bytes, terminators where the span
    /// includes them).
    pub fn text(&self, span: &Span) -> &str {
        &self.source[span.start..span.end]
    }

    /// 1-based line number of a byte offset.
    pub fn line_at(&self, offset: usize) -> usize {
        self.lines.line_at(offset)
    }

    /// The section's body content (first fence, fence lines excluded).
    pub fn body(&self, section: usize) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|s| s.body.as_ref())
            .map(|b| &self.source[b.content.start..b.content.end])
    }

    /// The subset of diagnostics that blocks every write command (spec §1).
    pub fn blocking(&self) -> Vec<Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| diag::is_blocking(d.id))
            .cloned()
            .collect()
    }
}

fn too_large() -> Diagnostic {
    Diagnostic::new("E-TOO-LARGE", 1).with(
        "limit",
        (consts::MAX_MANAGED_FILE_BYTES / (1024 * 1024)).to_string(),
    )
}

/// Parse the raw bytes of a managed file (spec §2–§6).
///
/// Invalid UTF-8 and oversize input return `Fault::Diagnostics` (E-UTF8 at
/// the first illegal byte, E-TOO-LARGE at L1). Every other structural
/// problem is a diagnostic on the returned document, which keeps whatever
/// could be parsed (spec §10.2.3: an empty file yields E-H1, not an error).
/// `path` supplies the stem for W-NAME / W-NAME-STATE; `now` is the single
/// command clock used for state validity.
pub fn parse(bytes: Vec<u8>, path: &Path, now: DateTime<Utc>) -> Result<Document, Fault> {
    if bytes.len() > consts::MAX_MANAGED_FILE_BYTES {
        return Err(Fault::Diagnostics(vec![too_large()]));
    }
    let source = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(err) => {
            let at = err.utf8_error().valid_up_to();
            let line = 1 + err.as_bytes()[..at].iter().filter(|&&b| b == b'\n').count();
            return Err(Fault::Diagnostics(vec![Diagnostic::new("E-UTF8", line)]));
        }
    };

    let lines = LineIndex::new(&source);
    let fence = FenceScan::scan(&source, &lines);
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    if source.starts_with('\u{feff}') {
        diagnostics.push(Diagnostic::new("W-BOM", 1));
    }
    for idx in 0..lines.count() {
        let l = lines.line(idx);
        if source[l.end..l.next].starts_with("\r\n") {
            diagnostics.push(Diagnostic::new("W-CRLF", idx + 1));
            break;
        }
    }
    if let Some((open, ticks)) = fence.unclosed {
        diagnostics.push(
            Diagnostic::new("E-FENCE", open + 1)
                .with("n", (open + 1).to_string())
                .with("ticks", ticks.to_string()),
        );
    }

    let stem: String = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    // --- H1: the first non-blank line (spec §2.2), BOM stripped. ----------
    let mut h1: Option<Span> = None;
    let mut h1_idx = 0usize;
    let mut kind: Option<Kind> = None;
    let mut name: Option<String> = None;
    let mut candidate = None;
    for idx in 0..lines.count() {
        let l = lines.line(idx);
        let mut content = &source[l.start..l.end];
        if idx == 0 {
            content = content.strip_prefix('\u{feff}').unwrap_or(content);
        }
        if !types::is_blank_line(content) {
            candidate = Some(idx);
            break;
        }
    }
    let bad_h1 = |raw: &str, line_no: usize| {
        Diagnostic::new("E-H1", line_no)
            .with("line", names::display_text(raw))
            .with("stem", stem.clone())
    };
    match candidate {
        None => diagnostics.push(bad_h1("", 1)),
        Some(idx) => {
            let l = lines.line(idx);
            let mut raw = &source[l.start..l.end];
            if idx == 0 {
                raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
            }
            if raw.contains('\r') {
                // spec §2.1.2: a line with an isolated `\r` matches no
                // structure grammar, H1 included
                diagnostics.push(bad_h1(raw, idx + 1));
            } else {
                match parse_h1(raw) {
                    Ok((n, k)) => {
                        match k {
                            Kind::State => {
                                if stem != "state" {
                                    diagnostics.push(
                                        Diagnostic::new("W-NAME-STATE", idx + 1)
                                            .with("stem", stem.clone()),
                                    );
                                }
                            }
                            _ => {
                                if stem != n {
                                    diagnostics.push(
                                        Diagnostic::new("W-NAME", idx + 1)
                                            .with("stem", stem.clone())
                                            .with("name", n.clone())
                                            .with("type", k.as_str()),
                                    );
                                }
                            }
                        }
                        h1 = Some(l.start..l.end);
                        h1_idx = idx;
                        kind = Some(k);
                        name = Some(n);
                    }
                    Err(()) => diagnostics.push(bad_h1(raw, idx + 1)),
                }
            }
        }
    }

    // --- H2 sections (spec §1); fenced lines are opaque. ------------------
    let mut h2s: Vec<usize> = Vec::new();
    for idx in 0..lines.count() {
        if fence.inside[idx] {
            continue;
        }
        let l = lines.line(idx);
        let content = &source[l.start..l.end];
        if content.contains('\r') {
            continue;
        }
        if content.starts_with("## ") {
            h2s.push(idx);
        }
    }
    let mut sections: Vec<Section> = Vec::with_capacity(h2s.len());
    for (si, &h) in h2s.iter().enumerate() {
        let end_idx = h2s.get(si + 1).copied().unwrap_or(lines.count());
        let hl = lines.line(h);
        let span_end = if end_idx == lines.count() {
            source.len()
        } else {
            lines.line(end_idx).start
        };
        let body = fence
            .blocks
            .iter()
            .find(|b| b.open > h && b.open < end_idx)
            .map(|b| Body {
                span: b.block.clone(),
                content: b.content.clone(),
            });
        sections.push(Section {
            title: names::fold(&source[hl.start + 3..hl.end]),
            head: hl.start..hl.end,
            span: hl.start..span_end,
            body,
        });
    }

    // --- preamble geometry -------------------------------------------------
    let pre_start = if h1.is_some() {
        lines.line(h1_idx).next
    } else {
        0
    };
    let pre_end = sections
        .first()
        .map(|s| s.head.start)
        .unwrap_or(source.len());
    let preamble = pre_start..pre_end;
    let preamble_lines = lines.idx_at(pre_start)..lines.idx_at(pre_end).max(lines.idx_at(pre_start));

    // --- per-type views -----------------------------------------------------
    let empty_post = || types::post::Post {
        comments: Vec::new(),
        subscribers: Vec::new(),
        subscribers_span: None,
    };
    let empty_brief = || types::brief::Brief {
        description: None,
        entries: Vec::new(),
    };
    let (post, brief, profile, state) = match kind {
        Some(Kind::Post) => {
            let mut ctx = ScanCtx::new(
                &source,
                &lines,
                &fence.inside,
                &fence.blocks,
                now,
                &mut diagnostics,
            );
            let post = types::post::scan(&mut ctx, preamble_lines.clone(), &sections);
            (post, empty_brief(), Vec::new(), Vec::new())
        }
        Some(Kind::Brief) => {
            let mut ctx = ScanCtx::new(
                &source,
                &lines,
                &fence.inside,
                &fence.blocks,
                now,
                &mut diagnostics,
            );
            let brief = types::brief::scan(&mut ctx, preamble_lines.clone(), &sections);
            (empty_post(), brief, Vec::new(), Vec::new())
        }
        Some(Kind::Profile) => {
            let mut ctx = ScanCtx::new(
                &source,
                &lines,
                &fence.inside,
                &fence.blocks,
                now,
                &mut diagnostics,
            );
            let profile = types::profile::scan(&mut ctx, &sections);
            if sections.is_empty() {
                // W-PROFILE-EMPTY anchors at the H1 (spec §5.2.3)
                diagnostics.push(Diagnostic::new("W-PROFILE-EMPTY", h1_idx + 1));
            }
            (empty_post(), empty_brief(), profile, Vec::new())
        }
        Some(Kind::State) => {
            let mut ctx = ScanCtx::new(
                &source,
                &lines,
                &fence.inside,
                &fence.blocks,
                now,
                &mut diagnostics,
            );
            let state = types::state::scan(&mut ctx, &sections);
            (empty_post(), empty_brief(), Vec::new(), state)
        }
        None => (empty_post(), empty_brief(), Vec::new(), Vec::new()),
    };

    diagnostics.sort_by_key(|d| (d.line, diag::order(d.id)));

    Ok(Document {
        source,
        kind,
        name,
        h1,
        preamble,
        sections,
        diagnostics,
        post,
        brief,
        profile,
        state,
        lines,
    })
}

/// `^# (?P<name>\S*)\.(?P<type>post|brief|agent-profile|state)\s*$`
/// with the name rules of spec §2.2.2 (state: empty name; others: NAME).
/// The caller already rejected lines containing `\r`.
fn parse_h1(raw: &str) -> Result<(String, Kind), ()> {
    let rest = raw.strip_prefix("# ").ok_or(())?;
    let trimmed = rest.trim_end();
    // `\S*` and the type word contain no whitespace; only trailing padding
    // is allowed, and `trim_end` here is ASCII-space/tab only in practice
    // because the line has no `\r`.
    if trimmed.chars().any(|c| c.is_whitespace()) {
        return Err(());
    }
    let dot = trimmed.rfind('.').ok_or(())?;
    let (name, type_word) = (&trimmed[..dot], &trimmed[dot + 1..]);
    let kind = match type_word {
        "post" => Kind::Post,
        "brief" => Kind::Brief,
        "agent-profile" => Kind::Profile,
        "state" => Kind::State,
        _ => return Err(()),
    };
    match kind {
        Kind::State => {
            if !name.is_empty() {
                return Err(());
            }
        }
        _ => {
            if !names::valid_name(name) {
                return Err(());
            }
        }
    }
    Ok((name.to_string(), kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        "2026-09-17T10:00:00Z".parse().unwrap()
    }

    fn parse_str(s: &str, stem: &str) -> Document {
        parse(s.as_bytes().to_vec(), Path::new(stem), now()).unwrap()
    }

    fn ids(d: &Document) -> Vec<&'static str> {
        d.diagnostics.iter().map(|x| x.id).collect()
    }

    #[test]
    fn minimal_post_h1_and_section_geometry() {
        let src = "# standup.post\n\n## #1 alice (2026-09-17T09:12:00Z)\n\n```md\nhi @bob\n```\n";
        let d = parse_str(src, "standup");
        assert_eq!(d.kind, Some(Kind::Post));
        assert_eq!(d.name.as_deref(), Some("standup"));
        assert!(d.diagnostics.is_empty());
        assert_eq!(d.sections.len(), 1);
        let sec = &d.sections[0];
        assert_eq!(d.text(&sec.head), "## #1 alice (2026-09-17T09:12:00Z)");
        // section spans through EOF including terminators
        assert_eq!(&src[sec.span.clone()], &src[16..]);
        // body content excludes the fence lines
        assert_eq!(d.body(0), Some("hi @bob"));
        // preamble: after the H1 terminator up to the H2 line
        assert_eq!(&src[d.preamble.clone()], "\n");
        assert_eq!(d.line_at(14), 1);
    }

    #[test]
    fn bom_blank_prefix_and_mixed_terminators_are_preserved() {
        let src = "\u{feff}\n\n# a.post\r\n\r\n## s\r\n```md\r\nx\r\n```\r\n";
        let d = parse_str(src, "a");
        assert_eq!(d.kind, Some(Kind::Post));
        assert_eq!(ids(&d), vec!["W-BOM", "W-CRLF"]);
        assert_eq!(d.diagnostics[0].line, 1);
        assert_eq!(d.diagnostics[1].line, 3);
        assert_eq!(d.sections.len(), 1);
        // inner line terminators stay in the raw content span
        assert_eq!(d.body(0), Some("x\r\n"));
        assert_eq!(d.text(&d.sections[0].head), "## s");
    }

    #[test]
    fn h1_failures() {
        let empty = parse_str("", "x");
        assert_eq!(ids(&empty), vec!["E-H1"]);
        assert_eq!(empty.diagnostics[0].line, 1);
        assert!(empty.kind.is_none());

        let junk = parse_str("# Notes\n", "notes");
        assert_eq!(ids(&junk), vec!["E-H1"]);
        // folded line text is embedded in the message param
        assert_eq!(
            junk.diagnostics[0]
                .params
                .iter()
                .find(|(k, _)| *k == "line")
                .unwrap()
                .1,
            "# Notes"
        );

        let state_named = parse_str("# x.state\n", "state");
        assert_eq!(ids(&state_named), vec!["E-H1"]);
        let post_empty = parse_str("# .post\n", "x");
        assert_eq!(ids(&post_empty), vec!["E-H1"]);
        let bad_name = parse_str("# has space.post\n", "x");
        assert_eq!(ids(&bad_name), vec!["E-H1"]);
        // isolated \r breaks H1 matching (spec §2.1.2)
        let cr = parse_str("# a\rpost\n", "a");
        assert_eq!(ids(&cr), vec!["E-H1"]);

        let ok = parse_str("# a.b.post\n", "a.b");
        assert_eq!(ok.kind, Some(Kind::Post)); // NAME may contain dots
        assert_eq!(ok.name.as_deref(), Some("a.b"));
    }

    #[test]
    fn name_mismatch_diagnostics() {
        let d = parse_str("# alice.post\n", "bob");
        assert_eq!(ids(&d), vec!["W-NAME"]);
        let d = parse_str("# .state\n", "State");
        assert_eq!(ids(&d), vec!["W-NAME-STATE"]);
        let d = parse_str("# .state\n", "state");
        assert_eq!(ids(&d), Vec::<&str>::new());
    }

    #[test]
    fn invalid_utf8_and_oversize_are_typed_faults() {
        let err = parse(vec![0x23, 0x20, 0x61, 0xFF, 0x2E], Path::new("a"), now()).unwrap_err();
        match err {
            Fault::Diagnostics(d) => {
                assert_eq!(d[0].id, "E-UTF8");
                assert_eq!(d[0].line, 1);
            }
            other => panic!("unexpected: {other:?}"),
        }
        let err = parse(vec![0x61; 16 * 1024 * 1024 + 1], Path::new("a"), now()).unwrap_err();
        match err {
            Fault::Diagnostics(d) => {
                assert_eq!(d[0].id, "E-TOO-LARGE");
                assert_eq!(d[0].line, 1);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn fence_rules() {
        // unclosed fence truncates structure and blocks; the section
        // before it survives without a body (W-BODY from the post scan)
        let src = "# a.post\n\n## #1 alice (2026-09-17T09:12:00Z)\n\n```md\nnever closed\n";
        let d = parse_str(src, "a");
        assert_eq!(ids(&d), vec!["W-BODY", "E-FENCE"]);
        assert_eq!(d.diagnostics[1].line, 5);
        assert_eq!(d.diagnostics[1].params[1].1, "3"); // ticks
        assert_eq!(d.sections.len(), 1);
        assert!(d.sections[0].body.is_none());

        // a close with MORE backticks closes; the info string may contain
        // spaces but no backtick; a preamble fence is not a section body
        let src = "# a.post\n\n```md x\nbody\n````\n\n## #1 alice (2026-09-17T09:12:00Z)\n";
        let d = parse_str(src, "a");
        assert_eq!(ids(&d), vec!["W-BODY"]);
        assert_eq!(d.sections.len(), 1);
        assert!(d.sections[0].body.is_none());

        // an open fence with a backtick in the info string is NOT a fence;
        // the trailing ``` then really opens an unclosed fence
        let src = "# a.post\n\n``` md`x\nplain text\n```\n";
        let d = parse_str(src, "a");
        assert_eq!(ids(&d), vec!["W-STRAY", "W-STRAY", "E-FENCE"]);

        // H2 inside a fence is not a section boundary
        let src = "# a.post\n\n## #1 alice (2026-09-17T09:12:00Z)\n\n```md\n## not a section\n```\n";
        let d = parse_str(src, "a");
        assert_eq!(d.sections.len(), 1);
        assert_eq!(d.body(0), Some("## not a section"));
    }

    #[test]
    fn blocking_subset_and_sorting() {
        // an invalid H1 leaves no type view, so no W-STRAY is derived;
        // blocking() keeps only the §1 set
        let d = parse_str("# x\n\njunk\n", "x");
        assert_eq!(ids(&d), vec!["E-H1"]);
        let blocking = d.blocking();
        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0].id, "E-H1");
    }
}
