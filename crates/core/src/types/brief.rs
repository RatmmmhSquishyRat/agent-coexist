//! brief entry view (spec §4).
//!
//! Every H2 section is an entry. The property area is the run of property
//! and blank lines directly after the H2; the note is the section's first
//! fence (reached through `Document::body`). All spans are original bytes;
//! nothing here rewrites the file.

use std::collections::HashSet;
use std::ops::Range;

use crate::doc::{Body, Section};
use crate::error::Diagnostic;
use crate::names;
use crate::text::Span;
use crate::types::{is_blank_line, property_parts, ScanCtx};

/// One `- <key>: <value>` line of a brief entry (spec §2.6.1, §4.3).
#[derive(Debug, Clone)]
pub struct Property {
    pub key: String,
    /// Raw value bytes after `": "` (possibly empty).
    pub value: String,
    /// Whole property line, terminator excluded.
    pub span: Span,
    /// Value text only.
    pub value_span: Span,
    /// 1-based line number.
    pub line: usize,
}

/// The brief document view: description fence plus entries in file order.
#[derive(Debug, Clone, Default)]
pub struct Brief {
    /// First fence of the preamble, if any (spec §4.2).
    pub description: Option<Body>,
    pub entries: Vec<Entry>,
}

/// One brief entry = one H2 section (spec §4.3).
#[derive(Debug, Clone)]
pub struct Entry {
    /// Folded title (§4.3.1); `--entry` matches this form.
    pub title: String,
    /// Index into `Document::sections`.
    pub section: usize,
    /// 1-based H2 line — the anchor for E-ENTRY-* / W-ENTRY-SHIFTED /
    /// W-ENTRY-STALE rows.
    pub head_line: usize,
    /// First `- path:` line. `None` means E-ENTRY-PATH and the entry is
    /// `unchecked  no path`; never an empty string stand-in.
    pub path: Option<Property>,
    /// First `- hash:` line (kept even when its value is malformed — the
    /// refresh form replaces the value in place).
    pub hash: Option<Property>,
    /// First `- regex:` line.
    pub regex: Option<Property>,
    /// Property lines with other keys, in file order (each reported as
    /// W-ENTRY-KEY at parse).
    pub attributes: Vec<Property>,
    /// Offset where NEW property lines are inserted: the start of the first
    /// non-property non-blank line after the H2, or the section end (spec
    /// §10.3.2 “属性区末尾”). Blank lines belong to the property area.
    pub attrs_end: usize,
}

/// Is `value` an absolute target path (spec §10.4.3)? `/`-rooted (incl.
/// UNC and `//?/`), `\\`-rooted, or drive-letter paths count.
pub(crate) fn is_absolute_path(value: &str) -> bool {
    let normalized = value.replace('\\', "/");
    let b = normalized.as_bytes();
    if b.first() == Some(&b'/') {
        return true;
    }
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

pub(crate) fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub(crate) fn scan(ctx: &mut ScanCtx, preamble: Range<usize>, sections: &[Section]) -> Brief {
    let description = ctx.first_fence_in(&preamble);

    // preamble: fences, blank lines and `> ` comment lines are fine; any
    // other non-blank line is stray (no property semantics here, §4.2)
    for idx in preamble {
        if ctx.fenced(idx) {
            continue;
        }
        let line = ctx.line_str(idx);
        if is_blank_line(line) || line.starts_with("> ") {
            continue;
        }
        ctx.push(
            Diagnostic::new("W-STRAY", idx + 1).with("n", (idx + 1).to_string()),
        );
    }

    let mut seen_titles: HashSet<&str> = HashSet::new();
    let mut entries = Vec::with_capacity(sections.len());
    for (si, sec) in sections.iter().enumerate() {
        let head_idx = ctx.idx_at(sec.head.start);
        let head_line = head_idx + 1;
        if !seen_titles.insert(sec.title.as_str()) {
            ctx.push(
                Diagnostic::new("E-ENTRY-DUP", head_line)
                    .with("title", names::display_text(&sec.title)),
            );
        }

        let mut path: Option<Property> = None;
        let mut hash: Option<Property> = None;
        let mut regex: Option<Property> = None;
        let mut attributes: Vec<Property> = Vec::new();
        // property area: consecutive property + blank lines (spec §4.3.2)
        let mut attrs_end = sec.span.end;
        let mut after_area = None;
        for idx in ctx.section_body_lines(sec) {
            if ctx.fenced(idx) {
                attrs_end = ctx.line_span(idx).start;
                after_area = Some(idx);
                break;
            }
            let line = ctx.line_str(idx);
            if is_blank_line(line) {
                continue;
            }
            let parts = if ctx.lone_cr(idx) {
                None
            } else {
                property_parts(line)
            };
            match parts {
                Some((key, value)) => {
                    let lspan = ctx.line_span(idx);
                    let value_span = (lspan.end - value.len())..lspan.end;
                    let prop = Property {
                        key: key.to_string(),
                        value: value.to_string(),
                        span: lspan,
                        value_span,
                        line: idx + 1,
                    };
                    match key {
                        "path" => path.get_or_insert(prop),
                        "hash" => hash.get_or_insert(prop),
                        "regex" => regex.get_or_insert(prop),
                        _ => {
                            // unknown key: kept byte-for-byte, flagged
                            ctx.push(
                                Diagnostic::new("W-ENTRY-KEY", prop.line)
                                    .with("title", names::display_text(&sec.title))
                                    .with("key", key.to_string()),
                            );
                            attributes.push(prop)
                        }
                    };
                }
                None => {
                    attrs_end = ctx.line_span(idx).start;
                    after_area = Some(idx);
                    break;
                }
            }
        }

        // entry-level checks (anchors per §9)
        if let Some(p) = &path {
            if is_absolute_path(&p.value) {
                ctx.push(
                    Diagnostic::new("W-ENTRY-ABS", p.line)
                        .with("title", names::display_text(&sec.title)),
                );
            }
        } else {
            ctx.push(
                Diagnostic::new("E-ENTRY-PATH", head_line)
                    .with("title", names::display_text(&sec.title)),
            );
        }
        let hash_valid = hash.as_ref().is_some_and(|h| is_sha256_hex(&h.value));
        if !hash_valid {
            ctx.push(
                Diagnostic::new("E-ENTRY-HASH", head_line)
                    .with("title", names::display_text(&sec.title)),
            );
        }
        if let Some(r) = &regex {
            if let Err(err) = regex::Regex::new(&r.value) {
                // {target} is unavailable when the entry has no path line
                // (§9 supplement 7): show the reader placeholder instead
                let target = path
                    .as_ref()
                    .map(|p| names::display_text(&p.value))
                    .unwrap_or_else(|| "<file>".to_string());
                ctx.push(
                    Diagnostic::new("E-ENTRY-REGEX", r.line)
                        .with("title", names::display_text(&sec.title))
                        .with("err", names::display_text(&err.to_string()))
                        .with("target", target),
                );
            }
        }

        // stray lines after the property area (fence content stays opaque,
        // fence lines themselves are structural)
        if let Some(from) = after_area {
            for idx in from..ctx.section_body_lines(sec).end {
                if ctx.fenced(idx) {
                    continue;
                }
                let line = ctx.line_str(idx);
                if is_blank_line(line) {
                    continue;
                }
                ctx.push(
                    Diagnostic::new("W-STRAY", idx + 1).with("n", (idx + 1).to_string()),
                );
            }
        }

        entries.push(Entry {
            title: sec.title.clone(),
            section: si,
            head_line,
            path,
            hash,
            regex,
            attributes,
            attrs_end,
        });
    }

    Brief {
        description,
        entries,
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::{self, Document, Kind};
    use std::path::Path;

    fn parse_brief(src: &str) -> Document {
        doc::parse(
            src.as_bytes().to_vec(),
            Path::new("onboarding"),
            "2026-09-17T10:00:00Z".parse().unwrap(),
        )
        .unwrap()
    }

    fn ids(d: &Document) -> Vec<&'static str> {
        d.diagnostics.iter().map(|x| x.id).collect()
    }

    const BASE: &str = "# onboarding.brief\n\n```md\nHow to understand this project in 10 minutes.\n```\n\n## parser entry point\n\n- path: src/parser/mod.rs\n- hash: 42b664743ddb6056ca84ab76bcf57d71533713c1bed9a493e8c0e787709e0540\n- regex: pub fn parse\n\n```md\nStart here.\n```\n";

    #[test]
    fn minimal_document_parses_cleanly() {
        let d = parse_brief(BASE);
        assert_eq!(d.kind, Some(Kind::Brief));
        assert_eq!(ids(&d), Vec::<&str>::new());
        assert_eq!(d.brief.entries.len(), 1);
        let e = &d.brief.entries[0];
        assert_eq!(e.title, "parser entry point");
        assert_eq!(e.head_line, 7);
        assert_eq!(e.path.as_ref().unwrap().value, "src/parser/mod.rs");
        assert!(is_sha256_hex(e.hash.as_ref().unwrap().value.as_str()));
        assert_eq!(e.regex.as_ref().unwrap().value, "pub fn parse");
        assert_eq!(d.body(e.section), Some("Start here."));
        assert!(d.brief.description.is_some());
        // property spans point at the raw line bytes
        assert_eq!(
            d.text(&e.path.as_ref().unwrap().span),
            "- path: src/parser/mod.rs"
        );
        assert_eq!(
            d.text(&e.path.as_ref().unwrap().value_span),
            "src/parser/mod.rs"
        );
    }

    #[test]
    fn attrs_end_tracks_property_area() {
        // blank line inside the area, then the note fence ends it
        let src = "# b.brief\n\n## t\n\n- path: x\n\n```md\nn\n```\n";
        let d = parse_brief(src);
        let e = &d.brief.entries[0];
        let fence_start = src.find("```md").unwrap();
        assert_eq!(e.attrs_end, fence_start);
        // no fence at all: area runs to the section end
        let src = "# b.brief\n\n## t\n\n- path: x\n";
        let d = parse_brief(src);
        let e = &d.brief.entries[0];
        assert_eq!(e.attrs_end, src.len());
    }

    #[test]
    fn entry_level_diagnostics_anchor_correctly() {
        let src = "# b.brief\n\n## lexer\n\n- path: /abs/x\n- hash: nothex\n- regex: (\n- extra: kept\n";
        let d = parse_brief(src);
        assert_eq!(
            ids(&d),
            vec!["E-ENTRY-HASH", "W-ENTRY-ABS", "E-ENTRY-REGEX", "W-ENTRY-KEY"]
        );
        assert_eq!(d.diagnostics[0].line, 3); // E-ENTRY-HASH at the H2
        assert_eq!(d.diagnostics[1].line, 5); // W-ENTRY-ABS at the path line
        assert_eq!(d.diagnostics[2].line, 7); // E-ENTRY-REGEX at its line
        assert_eq!(d.diagnostics[3].line, 8); // W-ENTRY-KEY at its line
        let e = &d.brief.entries[0];
        assert_eq!(e.attributes.len(), 1);
        assert_eq!(e.attributes[0].key, "extra");
        // {target} present because the entry has a path
        let p = d.diagnostics[2]
            .params
            .iter()
            .find(|(k, _)| *k == "target")
            .unwrap();
        assert_eq!(p.1, "/abs/x");
        // missing path -> E-ENTRY-PATH and target unavailable in regex diag
        let src = "# b.brief\n\n## t\n\n- regex: (\n";
        let d = parse_brief(src);
        assert_eq!(ids(&d), vec!["E-ENTRY-PATH", "E-ENTRY-HASH", "E-ENTRY-REGEX"]);
        let p = d.diagnostics[2]
            .params
            .iter()
            .find(|(k, _)| *k == "target")
            .unwrap();
        assert_eq!(p.1, "<file>");
    }

    #[test]
    fn duplicate_entries_and_stray_lines() {
        let src = "# b.brief\n\n## same\n\n- path: a\n\n## same\n\n- path: b\n\nloose text\n";
        let d = parse_brief(src);
        assert_eq!(ids(&d), vec!["E-ENTRY-DUP", "W-STRAY"]);
        assert_eq!(d.diagnostics[0].line, 7); // second occurrence head
        assert_eq!(d.diagnostics[1].line, 11);
        assert_eq!(d.brief.entries.len(), 2);
    }

    #[test]
    fn preamble_comment_lines_are_not_stray() {
        let src = "# b.brief\n\n> note for readers\n\n```md\ndesc\n```\n";
        let d = parse_brief(src);
        assert_eq!(ids(&d), Vec::<&str>::new());
        assert!(d.brief.description.is_some());
    }
}
