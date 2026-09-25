//! The `state` type view (spec §6): status sections, comma-split entry
//! declarations with clean/dirty line classification, the per-nick
//! selection used by read commands, and the expiry / in-file `locked`
//! conflict diagnostics — both derived once, here, over the selected
//! declarations (§6.2.5, §6.2.7), so no consumer re-derives or
//! duplicates them.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::doc::Section;
use crate::error::Diagnostic;
use crate::names;
use crate::text::Span;
use crate::types::{is_blank_line, ScanCtx};

/// One H2 status section of a state file (spec §6.2.2).
#[derive(Debug, Clone)]
pub struct StateSection {
    /// Folded H2 title, e.g. `locked`, `observing`, or any custom status.
    pub status: String,
    /// Index of this section in `Document::sections`.
    pub section: usize,
    /// Whether the title matches STATUS (§2.3). Entries under an invalid
    /// title are shown but never treated as `locked` (§6.2.2).
    pub valid_name: bool,
    /// Declarations in file order, from clean and dirty lines alike —
    /// legal fragments on dirty lines still count for display, validity
    /// and conflicts (§6.2.6).
    pub entries: Vec<Declaration>,
    /// Full spans (terminators included) of the section's clean lines —
    /// the only lines a state write may rewrite or delete (§6.4.1).
    pub clean_lines: Vec<Span>,
}

/// One `<nick> <start>..<until>` declaration (spec §6.2.3).
#[derive(Debug, Clone)]
pub struct Declaration {
    pub nick: String,
    pub start: DateTime<Utc>,
    pub until: DateTime<Utc>,
    /// 1-based line number, the anchor of every per-entry diagnostic.
    pub line: usize,
    /// Byte span of the trimmed fragment in the document (without the
    /// separating `,`), the unit a `state --remove` splice takes out.
    pub span: Span,
    /// Whether this declaration sits on a clean line; legal fragments on
    /// a dirty line are kept but never merged or cleaned by writes
    /// (§6.2.6).
    pub clean: bool,
}

impl Declaration {
    /// A valid entry: `start <= until && now < until` (spec §1, §6.2.5) —
    /// `now == until` already counts as expired.
    pub fn valid(&self, now: DateTime<Utc>) -> bool {
        self.start <= self.until && now < self.until
    }
}

/// The declarations a read command reports for a section: grouped by
/// nick, keeping the declaration with the latest `until` (earliest wins
/// ties, §6.2.4), returned in file order (§6.2.6).
pub fn selected(section: &StateSection) -> Vec<&Declaration> {
    let mut slot_of: HashMap<&str, usize> = HashMap::new();
    let mut best: Vec<usize> = Vec::new();
    for (i, d) in section.entries.iter().enumerate() {
        match slot_of.get(d.nick.as_str()) {
            None => {
                slot_of.insert(d.nick.as_str(), best.len());
                best.push(i);
            }
            Some(&slot) => {
                if d.until > section.entries[best[slot]].until {
                    best[slot] = i;
                }
            }
        }
    }
    best.sort_unstable();
    best.into_iter().map(|i| &section.entries[i]).collect()
}

/// `{lowercased}` for W-STATUS-NAME (§9 suppl. 8): the folded status
/// name, ASCII-lowercased, every space to `-`, characters outside
/// `[a-z0-9-]` dropped, leading characters dropped until the first
/// `a-z`, truncated to 32 chars; `status` when nothing survives.
fn suggest_status(folded: &str) -> String {
    let lowered: String = folded
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .map(|c| if c == ' ' { '-' } else { c })
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect();
    let start = lowered
        .find(|c: char| c.is_ascii_lowercase())
        .unwrap_or(lowered.len());
    let out: String = lowered[start..].chars().take(32).collect();
    if out.is_empty() {
        "status".to_string()
    } else {
        out
    }
}

/// One trimmed comma fragment: `<NICK> <start>..<until>` (spec §6.2.3).
/// The fragment arrives already trimmed, so the grammar's leading and
/// trailing `\s*` are gone; at least one whitespace character must
/// separate the nick from the first time, and both times must parse as
/// RFC 3339 (§2.8.2). Returns `None` for anything else (W-STATE-ENTRY).
fn parse_entry(frag: &str) -> Option<(&str, DateTime<Utc>, DateTime<Utc>)> {
    let nick = names::nick_run(frag);
    if nick.is_empty() || !names::valid_nick(nick) {
        return None;
    }
    let rest = &frag[nick.len()..];
    let times = rest.trim_start();
    if times.len() == rest.len() {
        return None; // `\s+` required between nick and start
    }
    let sep = times.find("..")?;
    let start = crate::time::parse(&times[..sep]).ok()?;
    let until = crate::time::parse(&times[sep + 2..]).ok()?;
    Some((nick, start, until))
}

/// Scan a state file (spec §6.2): per section the status name checks,
/// the entry lines and the per-(status, nick) duplicates and reversed
/// ranges — per-entry facts. Time expiry (§6.2.5) and the in-file
/// `locked` conflicts (§6.2.7) are derived once, over each section's
/// selected declarations (§6.2.4), and pushed here as well: every
/// consumer of `Document::diagnostics` sees one coherent derivation,
/// never a parse-time set plus a preview-time duplicate. Diagnostics
/// are pushed through `ctx`; `doc::parse` sorts them once at the end.
pub(crate) fn scan(ctx: &mut ScanCtx, sections: &[Section]) -> Vec<StateSection> {
    let now = ctx.now();
    let mut out: Vec<StateSection> = Vec::with_capacity(sections.len());
    // folded titles already seen, for E-STATUS-DUP at later occurrences
    let mut seen_titles: Vec<&str> = Vec::new();

    for (si, sec) in sections.iter().enumerate() {
        let status = sec.title.clone();
        let head_no = ctx.idx_at(sec.head.start) + 1;
        let valid_name = names::valid_status(&status);
        if !valid_name {
            ctx.push(
                Diagnostic::new("W-STATUS-NAME", head_no)
                    .with("name", names::display_text(&status))
                    .with("n", head_no.to_string())
                    .with("lowercased", suggest_status(&status)),
            );
        }
        if seen_titles.contains(&sec.title.as_str()) {
            ctx.push(Diagnostic::new("E-STATUS-DUP", head_no).with("status", status.clone()));
        } else {
            seen_titles.push(sec.title.as_str());
        }

        let mut entries: Vec<Declaration> = Vec::new();
        let mut clean_lines: Vec<Span> = Vec::new();
        let mut nicks: Vec<String> = Vec::new();

        for idx in ctx.section_body_lines(sec) {
            if ctx.fenced(idx) || is_blank_line(ctx.line_str(idx)) {
                continue;
            }
            let line_no = idx + 1;
            if ctx.lone_cr(idx) {
                // matches no structure grammar (§2.1.2): every comma
                // fragment is unparseable and the whole line is dirty
                // (folded first so the line borrow ends before pushing)
                let folded: Vec<String> = ctx
                    .line_str(idx)
                    .split(',')
                    .map(names::display_text)
                    .collect();
                for fragment in folded {
                    ctx.push(
                        Diagnostic::new("W-STATE-ENTRY", line_no)
                            .with("fragment", fragment)
                            .with("status", status.clone()),
                    );
                }
                continue;
            }
            // pass 1: byte spans of the trimmed non-empty fragments
            let base = ctx.line_span(idx).start;
            let mut frags: Vec<Span> = Vec::new();
            let mut off = 0usize;
            for piece in ctx.line_str(idx).split(',') {
                let start = base + off;
                off += piece.len() + 1; // the consumed ','
                let lead = piece.len() - piece.trim_start().len();
                let tail = piece.len() - piece.trim_end().len();
                if lead + tail < piece.len() {
                    frags.push((start + lead)..(start + piece.len() - tail));
                }
            }
            // pass 2: parse and diagnose
            let mut all_parsed = true;
            for fs in frags {
                let frag = &ctx.source()[fs.start..fs.end];
                match parse_entry(frag) {
                    Some((nick, start, until)) => entries.push(Declaration {
                        nick: nick.to_string(),
                        start,
                        until,
                        line: line_no,
                        span: fs,
                        clean: true,
                    }),
                    None => {
                        all_parsed = false;
                        ctx.push(
                            Diagnostic::new("W-STATE-ENTRY", line_no)
                                .with("fragment", names::display_text(frag))
                                .with("status", status.clone()),
                        );
                    }
                }
            }
            // clean line: every non-empty fragment parsed (§6.2.6) — only
            // clean lines may be rewritten or deleted by a state write
            if all_parsed {
                clean_lines.push(ctx.line_full_span(idx));
            } else {
                for d in entries.iter_mut().rev() {
                    if d.line != line_no {
                        break;
                    }
                    d.clean = false;
                }
            }
        }

        // per-(status, nick) duplicates and reversed ranges (§6.2.4,
        // §6.2.5): per-entry facts, reported for every declaration
        // alike, selected or not.
        for d in &entries {
            if nicks.iter().any(|n| n == &d.nick) {
                ctx.push(
                    Diagnostic::new("W-STATE-DUP", d.line)
                        .with("nick", d.nick.clone())
                        .with("status", status.clone()),
                );
            } else {
                nicks.push(d.nick.clone());
            }
            if d.start > d.until {
                // W-STATE-RANGE and never also W-STATE-EXPIRED (§6.2.5)
                ctx.push(
                    Diagnostic::new("W-STATE-RANGE", d.line)
                        .with("nick", d.nick.clone())
                        .with("status", status.clone()),
                );
            }
        }

        let section = StateSection {
            status,
            section: si,
            valid_name,
            entries,
            clean_lines,
        };
        // Time expiry over the selected declarations only (§6.2.4's
        // per-nick latest-`until` view, §6.2.5/§6.3.2): a superseded
        // earlier declaration of the same nick is not reported — the
        // merged view is the valid one — and a reversed range keeps the
        // per-entry W-STATE-RANGE above, never an expiry on top.
        for d in selected(&section) {
            if d.start <= d.until && now >= d.until {
                ctx.push(
                    Diagnostic::new("W-STATE-EXPIRED", d.line)
                        .with("nick", d.nick.clone())
                        .with("status", section.status.clone())
                        .with("until", crate::time::format(d.until)),
                );
            }
        }
        out.push(section);
    }

    // In-file conflict (§6.2.7), derived once over the selected
    // declarations of every valid `locked` section: each pair of valid
    // declarations with different nicks, `{a}` the earlier one, `{b}`
    // the later, anchored on the later declaration's line (§9
    // supplement 6).
    let flat: Vec<&Declaration> = out
        .iter()
        .filter(|sec| sec.status == "locked" && sec.valid_name)
        .flat_map(selected)
        .collect();
    for x in 0..flat.len() {
        for y in (x + 1)..flat.len() {
            if flat[x].nick != flat[y].nick && flat[x].valid(now) && flat[y].valid(now) {
                ctx.push(
                    Diagnostic::new("W-STATE-CONFLICT", flat[y].line)
                        .with("a", flat[x].nick.clone())
                        .with("b", flat[y].nick.clone()),
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_fragment_grammar() {
        let (nick, start, until) =
            parse_entry("alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z").unwrap();
        assert_eq!(nick, "alice");
        assert_eq!(crate::time::format(start), "2026-09-17T09:00:00Z");
        assert_eq!(crate::time::format(until), "2026-09-17T11:00:00Z");

        // any RFC 3339 offset / fractional seconds, converted to UTC (§2.8.2)
        let (nick, start, _) =
            parse_entry("bob 2026-09-17T11:30:00.500+01:00..2026-09-17T12:00:00Z").unwrap();
        assert_eq!(nick, "bob");
        assert_eq!(crate::time::format(start), "2026-09-17T10:30:00Z");

        for bad in [
            "",
            "alice",
            "alice 2026-09-17T09:00:00Z",
            "alice2026-09-17T09:00:00Z..2026-09-17T11:00:00Z",
            "-x 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z",
            "alice bad..2026-09-17T11:00:00Z",
            "alice 2026-09-17T09:00:00Z..bad",
            "alice 2026-09-17T09:00:00Z .. 2026-09-17T11:00:00Z",
            "alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z extra",
        ] {
            assert!(parse_entry(bad).is_none(), "should reject {bad:?}");
        }
    }

    #[test]
    fn status_name_suggestion() {
        assert_eq!(suggest_status("Bad Name"), "bad-name");
        assert_eq!(suggest_status("9lives"), "lives");
        assert_eq!(suggest_status("--X-ray 9"), "x-ray-9");
        assert_eq!(suggest_status("LOCKED"), "locked");
        // non-ASCII letters are outside the STATUS alphabet
        assert_eq!(suggest_status("ÜBER"), "ber");
        assert_eq!(suggest_status("***"), "status");
        assert_eq!(suggest_status(""), "status");
        assert_eq!(suggest_status(&"w".repeat(40)).len(), 32);
    }

    #[test]
    fn selected_keeps_latest_until_in_file_order() {
        let t = |s: &str| crate::time::parse(s).unwrap();
        let mk = |nick: &str, until: &str, line: usize| Declaration {
            nick: nick.to_string(),
            start: t("2026-09-17T09:00:00Z"),
            until: t(until),
            line,
            span: 0..0,
            clean: true,
        };
        let section = StateSection {
            status: "locked".to_string(),
            section: 0,
            valid_name: true,
            entries: vec![
                mk("alice", "2026-09-17T11:00:00Z", 3),
                mk("bob", "2026-09-17T12:00:00Z", 3),
                mk("alice", "2026-09-17T10:30:00Z", 4), // earlier until loses
                mk("alice", "2026-09-17T12:00:00Z", 5), // tie: earliest wins
                mk("alice", "2026-09-17T12:00:00Z", 6),
            ],
            clean_lines: Vec::new(),
        };
        let rows: Vec<(&str, usize)> = selected(&section)
            .iter()
            .map(|d| (d.nick.as_str(), d.line))
            .collect();
        assert_eq!(rows, vec![("bob", 3), ("alice", 5)]);
    }

    #[test]
    fn declaration_validity() {
        let t = |s: &str| crate::time::parse(s).unwrap();
        let mk = |start: &str, until: &str| Declaration {
            nick: "alice".to_string(),
            start: t(start),
            until: t(until),
            line: 1,
            span: 0..0,
            clean: true,
        };
        let now = t("2026-09-17T10:00:00Z");
        assert!(mk("2026-09-17T09:00:00Z", "2026-09-17T11:00:00Z").valid(now));
        assert!(!mk("2026-09-17T09:00:00Z", "2026-09-17T10:00:00Z").valid(now)); // now == until
        assert!(!mk("2026-09-17T09:00:00Z", "2026-09-17T09:30:00Z").valid(now));
        assert!(!mk("2026-09-17T11:00:00Z", "2026-09-17T10:00:00Z").valid(now)); // start > until
        assert!(mk("2026-09-17T10:00:00Z", "2026-09-17T10:00:01Z").valid(now));
    }

    #[test]
    fn state_scan_end_to_end() {
        let src = concat!(
            "# .state\n",
            "\n",
            "> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.\n",
            "\n",
            "## locked\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z\n",
            "alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z\n",
            "broken entry\n",
            "carol 2026-09-17T08:00:00Z..2026-09-17T10:00:00Z\n",
            "dave 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\rnote\n",
            "\n",
            "## Bad Name\n",
            "eve 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
            "fred 2026-09-17T14:00:00Z..2026-09-17T13:00:00Z\n",
            "\n",
            "## locked\n",
            "gina 2026-09-17T09:00:00Z..2026-09-17T11:30:00Z\n",
        );
        let now = crate::time::parse("2026-09-17T10:00:00Z").unwrap();
        let doc = crate::doc::parse(
            src.as_bytes().to_vec(),
            std::path::Path::new("state.agent-besties.md"),
            now,
        )
        .expect("parses");

        assert_eq!(doc.state.len(), 3);
        let (locked, bad, dup) = (&doc.state[0], &doc.state[1], &doc.state[2]);
        assert_eq!(locked.status, "locked");
        assert!(locked.valid_name);
        assert!(!bad.valid_name);
        assert_eq!(bad.status, "Bad Name");
        // the duplicate `locked` section is still scanned (E-STATUS-DUP
        // flags it, entries count anyway)
        assert_eq!(dup.status, "locked");
        assert_eq!(dup.entries.len(), 1);
        assert_eq!(dup.entries[0].nick, "gina");

        // entries: 4 under locked (alice, bob, alice again, carol), the
        // lone-\r line contributes none; fred's reversed range still parses
        assert_eq!(locked.entries.len(), 4);
        assert_eq!(locked.entries[1].nick, "bob");
        assert_eq!(
            &doc.source[locked.entries[0].span.start..locked.entries[0].span.end],
            "alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z"
        );
        assert_eq!(locked.entries[1].line, 6);
        // clean lines: L6, L7, L9 (the lone-\r line and the broken line
        // are dirty and untouched by writes)
        assert_eq!(locked.clean_lines.len(), 3);
        assert_eq!(&doc.source[locked.clean_lines[0].clone()], "alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z\n");
        // both Bad Name lines parse, so both are clean
        assert_eq!(bad.clean_lines.len(), 2);

        // selection under locked: per nick the latest until, file order
        let rows: Vec<(&str, usize)> = selected(locked)
            .iter()
            .map(|d| (d.nick.as_str(), d.line))
            .collect();
        assert_eq!(rows, vec![("bob", 6), ("alice", 7), ("carol", 9)]);

        // Diagnostics: exactly one derivation over the selected
        // declarations — no parse-time + preview duplicates. The
        // selected alice under locked is the L7 renewal, so bob's
        // conflict anchors on L7 (§9 supplement 6), and gina conflicts
        // with both bob and the selected alice.
        let rows: Vec<(&str, usize)> = doc.diagnostics.iter().map(|d| (d.id, d.line)).collect();
        assert_eq!(
            rows,
            vec![
                ("W-STATE-DUP", 7),      // alice again under locked
                ("W-STATE-CONFLICT", 7), // bob vs the selected alice (L7 renewal)
                ("W-STATE-ENTRY", 8),    // broken entry
                ("W-STATE-EXPIRED", 9),  // carol: now == until (selected)
                ("W-STATE-ENTRY", 10),   // lone \r: folded fragment
                ("W-STATUS-NAME", 12),
                ("W-STATE-RANGE", 14), // fred: starts after it ends
                ("E-STATUS-DUP", 16),
                ("W-STATE-CONFLICT", 17), // bob vs gina
                ("W-STATE-CONFLICT", 17), // the selected alice vs gina
            ]
        );
        let expired = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "W-STATE-EXPIRED")
            .unwrap();
        assert!(expired
            .params
            .contains(&("until", "2026-09-17T10:00:00Z".to_string())));
        let status_name = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "W-STATUS-NAME")
            .unwrap();
        assert!(status_name
            .params
            .contains(&("lowercased", "bad-name".to_string())));
        let entry10 = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "W-STATE-ENTRY" && d.line == 10)
            .unwrap();
        assert!(entry10.params.contains(&(
            "fragment",
            "dave 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z note".to_string()
        )));
    }

    #[test]
    fn scan_expires_selected_view_not_superseded_declarations() {
        // alice's earlier declaration is expired but superseded by the
        // L5 renewal (latest until, §6.2.4): the merged view is valid,
        // so no W-STATE-EXPIRED; bob's only declaration is expired and
        // selected — exactly one warning.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "alice 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            "bob 2026-09-17T08:00:00Z..2026-09-17T09:30:00Z\n",
        );
        let now = crate::time::parse("2026-09-17T10:00:00Z").unwrap();
        let doc = crate::doc::parse(
            src.as_bytes().to_vec(),
            std::path::Path::new("state.agent-besties.md"),
            now,
        )
        .expect("parses");
        let rows: Vec<(&str, usize)> = doc.diagnostics.iter().map(|d| (d.id, d.line)).collect();
        assert_eq!(
            rows,
            vec![
                ("W-STATE-DUP", 5),     // alice again under locked
                ("W-STATE-EXPIRED", 6), // bob only — the superseded alice (L4) is not reported
            ]
        );
    }

    #[test]
    fn scan_reports_reversed_range_without_expiry_duplicate() {
        // §6.2.5: start > until counts as expired but is reported as
        // W-STATE-RANGE only — never also W-STATE-EXPIRED.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "fred 2026-09-17T14:00:00Z..2026-09-17T13:00:00Z\n",
        );
        let now = crate::time::parse("2026-09-17T10:00:00Z").unwrap();
        let doc = crate::doc::parse(
            src.as_bytes().to_vec(),
            std::path::Path::new("state.agent-besties.md"),
            now,
        )
        .expect("parses");
        let rows: Vec<(&str, usize)> = doc.diagnostics.iter().map(|d| (d.id, d.line)).collect();
        assert_eq!(rows, vec![("W-STATE-RANGE", 4)]);
    }
}
