//! The `state` type view (spec §6): status sections, comma-split entry
//! declarations with clean/dirty line classification, the per-nick
//! selection used by read commands, and expiry diagnostics over those
//! selected declarations (§6.2.5). The complete pairwise `locked`
//! conflict report is derived only when emitted by `check` (§6.2.7)
//! or a qualifying wait hit, not by ordinary parsing or scans.

use std::collections::{HashMap, HashSet};

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

/// Effective declarations across the whole file, including repeated status
/// sections: latest `until` per (status, nick), earliest occurrence on ties,
/// returned in the selected declarations' original file order (§6.2.4).
pub fn selected(sections: &[StateSection]) -> Vec<(&StateSection, &Declaration)> {
    select_latest(sections.iter().flat_map(|section| {
        section.entries.iter().map(move |entry| {
            (
                section.status.as_str(),
                entry.nick.as_str(),
                entry.until,
                (section, entry),
            )
        })
    }))
}

/// Append the complete same-file conflict diagnostics for [`selected`]'s
/// effective view. Only report consumers (`check` and qualifying wait hits)
/// call this; ordinary parsing never expands the pairwise report. `now` is
/// the same command clock used to parse the document. Pairs retain selected
/// file order and anchor on the later declaration; the caller stable-sorts
/// these together with the parse diagnostics by line and catalog order.
pub(crate) fn append_conflict_diagnostics(
    effective: &[(&StateSection, &Declaration)],
    now: DateTime<Utc>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let locked: Vec<&Declaration> = effective
        .iter()
        .filter(|(section, declaration)| {
            section.status == "locked" && section.valid_name && declaration.valid(now)
        })
        .map(|(_, declaration)| *declaration)
        .collect();
    for x in 0..locked.len() {
        for y in (x + 1)..locked.len() {
            if locked[x].nick != locked[y].nick {
                diagnostics.push(
                    Diagnostic::new("W-STATE-CONFLICT", locked[y].line)
                        .with("a", locked[x].nick.clone())
                        .with("b", locked[y].nick.clone()),
                );
            }
        }
    }
}

/// Shared selection rule for the parsed document and the raw scan-history
/// view. The payload is either a borrowed declaration or its raw position;
/// selection never drops history from the source view.
pub(crate) fn select_latest<'a, T>(
    entries: impl IntoIterator<Item = (&'a str, &'a str, DateTime<Utc>, T)>,
) -> Vec<T> {
    let mut latest = HashMap::new();
    for (order, (status, nick, until, value)) in entries.into_iter().enumerate() {
        match latest.entry((status, nick)) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert((order, until, value));
            }
            std::collections::hash_map::Entry::Occupied(mut slot) => {
                if until > slot.get().1 {
                    slot.insert((order, until, value));
                }
            }
        }
    }
    let mut selected: Vec<_> = latest.into_values().collect();
    selected.sort_unstable_by_key(|(order, _, _)| *order);
    selected.into_iter().map(|(_, _, value)| value).collect()
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
/// ranges — per-entry facts. Time expiry (§6.2.5) uses the whole file's
/// selected declarations (§6.2.4). Pairwise `locked` diagnostics are
/// deferred to report consumers, which use the same selected view.
/// Parse diagnostics are pushed through `ctx`; `doc::parse` sorts them
/// once at the end.
pub(crate) fn scan(ctx: &mut ScanCtx, sections: &[Section]) -> Vec<StateSection> {
    let now = ctx.now();
    let mut out: Vec<StateSection> = Vec::with_capacity(sections.len());
    // folded titles already seen, for E-STATUS-DUP at later occurrences
    let mut seen_titles = HashSet::new();
    // titles heading more than one section: their W-STATUS-NAME repair is
    // the same merge E-STATUS-DUP asks for, not one new name per copy
    let mut once = HashSet::new();
    let repeated: HashSet<&str> = sections
        .iter()
        .map(|sec| sec.title.as_str())
        .filter(|title| !once.insert(*title))
        .collect();

    for (si, sec) in sections.iter().enumerate() {
        let status = sec.title.clone();
        let head_no = ctx.idx_at(sec.head.start) + 1;
        let valid_name = names::valid_status(&status);
        if !valid_name {
            let d = Diagnostic::new("W-STATUS-NAME", head_no)
                .with("name", names::display_text(&status))
                .with("n", head_no.to_string())
                .with("lowercased", suggest_status(&status));
            ctx.push(if repeated.contains(status.as_str()) {
                d.with("repeated", "1")
            } else {
                d
            });
        }
        if !seen_titles.insert(sec.title.as_str()) {
            ctx.push(Diagnostic::new("E-STATUS-DUP", head_no).with("status", status.clone()));
        }

        let mut entries: Vec<Declaration> = Vec::new();
        let mut clean_lines: Vec<Span> = Vec::new();

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

        out.push(StateSection {
            status,
            section: si,
            valid_name,
            entries,
            clean_lines,
        });
    }

    // Duplicate identities and inverted ranges are raw per-entry facts,
    // including identities repeated under a later section of the same name.
    let mut seen = std::collections::HashSet::new();
    for section in &out {
        for d in &section.entries {
            if !seen.insert((section.status.as_str(), d.nick.as_str())) {
                ctx.push(
                    Diagnostic::new("W-STATE-DUP", d.line)
                        .with("nick", d.nick.clone())
                        .with("status", section.status.clone()),
                );
            }
            if d.start > d.until {
                ctx.push(
                    Diagnostic::new("W-STATE-RANGE", d.line)
                        .with("nick", d.nick.clone())
                        .with("status", section.status.clone()),
                );
            }
        }
    }

    // Expiry uses one effective view across every section. A superseded
    // declaration cannot add an expired row; inverted ranges retain
    // W-STATE-RANGE only.
    let effective = selected(&out);
    for (section, d) in &effective {
        if d.start <= d.until && now >= d.until {
            ctx.push(
                Diagnostic::new("W-STATE-EXPIRED", d.line)
                    .with("nick", d.nick.clone())
                    .with("status", section.status.clone())
                    .with("until", crate::time::format(d.until)),
            );
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
        let rows: Vec<(&str, usize)> = selected(std::slice::from_ref(&section))
            .iter()
            .map(|(_, d)| (d.nick.as_str(), d.line))
            .collect();
        assert_eq!(rows, vec![("bob", 3), ("alice", 5)]);
    }

    #[test]
    fn repeated_sections_share_selection() {
        let src = concat!(
            "# .state\n\n## locked\n",
            "alice 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z\n",
            "\n## locked\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, ",
            "bob 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z\n",
        );
        let doc = crate::doc::parse(
            src.as_bytes().to_vec(),
            std::path::Path::new("state.agent-besties.md"),
            crate::time::parse("2026-09-17T10:00:00Z").unwrap(),
        )
        .unwrap();
        let entries: Vec<_> = selected(&doc.state)
            .into_iter()
            .map(|(_, entry)| (entry.nick.as_str(), entry.line))
            .collect();
        assert_eq!(entries, vec![("alice", 7), ("bob", 7)]);
        assert!(doc.diagnostics.iter().any(|d| d.id == "E-STATUS-DUP"));
        assert!(doc.diagnostics.iter().any(|d| d.id == "W-STATE-DUP"));
        assert!(!doc.diagnostics.iter().any(|d| d.id == "W-STATE-EXPIRED"));
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
        let rows: Vec<(&str, usize)> = selected(&doc.state)
            .into_iter()
            .filter(|(section, _)| section.status == "locked")
            .map(|(_, d)| (d.nick.as_str(), d.line))
            .collect();
        assert_eq!(
            rows,
            vec![("bob", 6), ("alice", 7), ("carol", 9), ("gina", 17)]
        );

        // Parse diagnostics retain per-entry issues and selected expiry.
        // Pairwise conflict reporting belongs to report consumers.
        let rows: Vec<(&str, usize)> = doc.diagnostics.iter().map(|d| (d.id, d.line)).collect();
        assert_eq!(
            rows,
            vec![
                ("W-STATE-DUP", 7),     // alice again under locked
                ("W-STATE-ENTRY", 8),   // broken entry
                ("W-STATE-EXPIRED", 9), // carol: now == until (selected)
                ("W-STATE-ENTRY", 10),  // lone \r: folded fragment
                ("W-STATUS-NAME", 12),
                ("W-STATE-RANGE", 14), // fred: starts after it ends
                ("E-STATUS-DUP", 16),
            ]
        );
        let expired = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "W-STATE-EXPIRED")
            .unwrap();
        assert!(expired
            .params
            .contains(&("until", "2026-09-17T10:00:00Z".into())));
        let status_name = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "W-STATUS-NAME")
            .unwrap();
        assert!(status_name
            .params
            .contains(&("lowercased", "bad-name".into())));
        let entry10 = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "W-STATE-ENTRY" && d.line == 10)
            .unwrap();
        assert!(entry10.params.contains(&(
            "fragment",
            "dave 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z note".into()
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
