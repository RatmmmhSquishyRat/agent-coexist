//! The `post` type view (spec §3): the preamble subscribers line, comment
//! sections with seq tracking, and the mentions/replies derived from fenced
//! bodies (derived data is never serialized back, §3.4.2).

use std::collections::HashSet;
use std::ops::Range;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::doc::Section;
use crate::error::Diagnostic;
use crate::names;
use crate::text::Span;
use crate::types::{is_blank_line, is_reader_comment, property_parts, ScanCtx};

/// A whole post document (spec §3).
#[derive(Debug, Clone)]
pub struct Post {
    /// Comment sections in file order. Sections whose header fails the
    /// grammar (E-HEAD) never become comments (spec §3.3.2); duplicate seqs
    /// (E-SEQ-DUP) do — the ambiguity is reported, not hidden.
    pub comments: Vec<Comment>,
    /// Nicks of the first `- subscribers:` line, deduplicated with first
    /// occurrence kept (spec §3.2.2). Empty when the file has none.
    pub subscribers: Vec<String>,
    /// Full span (terminator included) of the subscribers line so
    /// `subscribe` / `--cancel` can splice it in place; `None` when the
    /// file has no subscribers line.
    pub subscribers_span: Option<Span>,
}

/// One comment: a `## #<seq> <nick> (<created>…)` section (spec §3.3).
#[derive(Debug, Clone)]
pub struct Comment {
    pub seq: u64,
    pub nick: String,
    pub created: DateTime<Utc>,
    pub edited: Option<DateTime<Utc>>,
    /// Index of this section in `Document::sections`.
    pub section: usize,
    /// `@nick` mentions derived from the fenced body (spec §3.4.2),
    /// deduplicated in first-occurrence order.
    pub mentions: Vec<String>,
    /// `@#n` reply targets derived from the fenced body (spec §3.4.2),
    /// deduplicated in first-occurrence order.
    pub replies: Vec<u64>,
}

impl Comment {
    /// When the comment was last touched: `edited` when present, else
    /// `created`.
    pub fn activity(&self) -> DateTime<Utc> {
        self.edited.unwrap_or(self.created)
    }
}

static MENTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@([\p{L}\p{N}_][\p{L}\p{N}_-]*)").unwrap());
static MENTION_PREFIX_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{N}_@]$").unwrap());
static REPLY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"@#([1-9][0-9]*)").unwrap());

/// Mentions and replies of a body text (spec §3.4.2), each deduplicated
/// preserving first-occurrence order.
///
/// A mention is an `@` at start of text or preceded by any character that
/// is not a Unicode letter/number/`_`/`@`, followed by a maximal run of
/// NICK characters of length 1..=64 (longer runs are not mentions). A
/// preceding `-` counts, including one that just ended a previous nick
/// (`@alice-@bob` yields `alice-` and `bob`). A reply is `@#` followed by
/// `[1-9][0-9]*`; a value that overflows `u64` is skipped, not truncated.
pub fn references(text: &str) -> (Vec<String>, Vec<u64>) {
    let mut mentions: Vec<String> = Vec::new();
    let mut seen_mentions: HashSet<&str> = HashSet::new();
    for caps in MENTION_RE.captures_iter(text) {
        let whole = caps.get(0).expect("whole match");
        if !mention_boundary(text, whole.start()) {
            continue;
        }
        let m = caps.get(1).expect("capture group 1").as_str();
        if m.chars().count() <= 64 && seen_mentions.insert(m) {
            mentions.push(m.to_string());
        }
    }
    let mut replies: Vec<u64> = Vec::new();
    let mut seen_replies: HashSet<u64> = HashSet::new();
    for caps in REPLY_RE.captures_iter(text) {
        let digits = caps.get(1).expect("capture group 1").as_str();
        if let Ok(n) = digits.parse::<u64>() {
            if seen_replies.insert(n) {
                replies.push(n);
            }
        }
    }
    (mentions, replies)
}

/// Whether `@` at byte `at` has a mention boundary: start of text, or a
/// previous scalar outside letter/number/`_`/`@`. Uses the preceding
/// scalar rather than a consuming prefix so a hyphen-ended nick does not
/// swallow the next mention's delimiter.
fn mention_boundary(text: &str, at: usize) -> bool {
    if at == 0 {
        return true;
    }
    let ch = text[..at].chars().next_back().expect("at > 0");
    let mut buf = [0u8; 4];
    !MENTION_PREFIX_BLOCK.is_match(ch.encode_utf8(&mut buf))
}

/// `<NICK>(, <NICK>)*` (spec §3.2.2): split on `,`, trim each fragment,
/// every fragment must be a non-empty valid NICK. Returns the
/// deduplicated list (first occurrence kept) and whether any nick
/// repeated in the line.
fn parse_subscribers(value: &str) -> Option<(Vec<String>, bool)> {
    let mut list: Vec<String> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut dup = false;
    for frag in value.split(',') {
        let frag = frag.trim();
        if frag.is_empty() || !names::valid_nick(frag) {
            return None;
        }
        if !seen.insert(frag) {
            dup = true;
        } else {
            list.push(frag.to_string());
        }
    }
    Some((list, dup))
}

/// One parsed comment header (§3.3): the fields of `#<seq> <nick>
/// (<created>[, edited <edited>])`.
struct ParsedHeader {
    seq: u64,
    nick: String,
    created: DateTime<Utc>,
    edited: Option<DateTime<Utc>>,
}

/// The `#…` part of a comment-header H2 (everything after `## `; the line
/// is known to start with `#`). Grammar (spec §3.3):
/// `#<seq> <nick> (<created>)` or `#<seq> <nick> (<created>, edited
/// <edited>)`. `seq` is `[1-9][0-9]*` and must stay below `u64::MAX` —
/// at or above it the header is malformed (E-HEAD, §3.3.1). Only
/// whitespace may follow the closing `)`. Returns `None` on any
/// deviation.
fn parse_header(text: &str) -> Option<ParsedHeader> {
    let rest = text.strip_prefix('#')?;
    let b = rest.as_bytes();
    if b.is_empty() || b[0] < b'1' || b[0] > b'9' {
        return None; // seq is [1-9][0-9]*: no leading zero
    }
    let mut i = 1usize;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    // legal seqs are 1..=u64::MAX-1 (§3.3.1): a value that does not fit or
    // equals u64::MAX is a malformed header, not a comment
    let seq: u64 = rest[..i].parse().ok()?;
    if seq == u64::MAX {
        return None;
    }
    // exactly one space, then the maximal nick run
    let after_seq = rest[i..].strip_prefix(' ')?;
    let nick = names::nick_run(after_seq);
    if nick.is_empty() || !names::valid_nick(nick) {
        return None;
    }
    // exactly one space, then '('
    let in_paren = after_seq[nick.len()..]
        .strip_prefix(' ')?
        .strip_prefix('(')?;
    // created: up to the first ',' or ')', no whitespace inside
    let split = in_paren.find([',', ')'])?;
    let created_s = &in_paren[..split];
    if created_s.chars().any(char::is_whitespace) {
        return None;
    }
    let created = crate::time::parse(created_s).ok()?;
    let edited = if in_paren.as_bytes()[split] == b',' {
        let after_edit = in_paren[split + 1..].strip_prefix(" edited ")?;
        let close = after_edit.find(')')?;
        let edited_s = &after_edit[..close];
        if edited_s.chars().any(char::is_whitespace) {
            return None;
        }
        let edited = crate::time::parse(edited_s).ok()?;
        if !after_edit[close + 1..].trim_end().is_empty() {
            return None; // only whitespace may follow ')'
        }
        Some(edited)
    } else {
        if !in_paren[split + 1..].trim_end().is_empty() {
            return None; // only whitespace may follow ')'
        }
        None
    };
    Some(ParsedHeader {
        seq,
        nick: nick.to_string(),
        created,
        edited,
    })
}

/// Scan a post (spec §3): the preamble subscribers line (§3.2), then every
/// H2 section as a comment header attempt (§3.3), body strays (§2.5.5) and
/// the derived references (§3.4). Diagnostics are pushed through `ctx`;
/// `doc::parse` sorts them once at the end.
pub(crate) fn scan(ctx: &mut ScanCtx, preamble: Range<usize>, sections: &[Section]) -> Post {
    let mut subscribers: Vec<String> = Vec::new();
    let mut subscribers_span: Option<Span> = None;
    let mut comments: Vec<Comment> = Vec::new();
    // E-HEAD / E-SEQ-DUP need `{next}` / `{exhausted}` only after every
    // stored legal seq is known (§9 suppl. 3). Held back and pushed at
    // the end; `doc::parse` sorts by (line, catalog order), so the later
    // push cannot change the rendered order.
    let mut deferred: Vec<Diagnostic> = Vec::new();
    // 1-based header line of each comment, for W-REPLY-DANGLING anchors
    let mut head_lines: Vec<usize> = Vec::new();
    let mut seen: HashSet<u64> = HashSet::new();
    let mut prev: u64 = 0;
    let mut max_legal: u64 = 0;

    // --- preamble (§3.2) --------------------------------------------------
    for idx in preamble {
        if ctx.fenced(idx) {
            continue;
        }
        let line = ctx.line_str(idx);
        if is_blank_line(line) || is_reader_comment(line.as_bytes()) {
            continue; // blank or reader comment line (§2.6.2): no diagnostic
        }
        let subs = if ctx.lone_cr(idx) {
            None // matches no structure grammar (§2.1.2)
        } else {
            match property_parts(line) {
                Some(("subscribers", value)) => parse_subscribers(value),
                _ => None,
            }
        };
        match subs {
            Some((list, dup)) => {
                if subscribers_span.is_none() {
                    // the first subscribers line wins (§3.2.2)
                    if dup {
                        ctx.push(Diagnostic::new("W-SUB-DUP", idx + 1));
                    }
                    subscribers = list;
                    subscribers_span = Some(ctx.line_full_span(idx));
                } else {
                    ctx.push(Diagnostic::new("W-SUB-DUP", idx + 1));
                }
            }
            None => {
                ctx.push(Diagnostic::new("W-STRAY", idx + 1).with("n", (idx + 1).to_string()));
            }
        }
    }

    // --- sections (§3.3) --------------------------------------------------
    for (si, sec) in sections.iter().enumerate() {
        let head_idx = ctx.idx_at(sec.head.start);
        let head_no = head_idx + 1;
        let raw = ctx.line_str(head_idx);
        let text = raw.strip_prefix("## ").unwrap_or(raw);
        if !text.starts_with('#') {
            // a plain H2: the section is kept but is not a comment (§3.3.2)
            ctx.push(Diagnostic::new("W-STRAY", head_no).with("n", head_no.to_string()));
            scan_body_strays(ctx, sec);
            continue;
        }
        // looks like `## #…`: comment header attempt; a lone-`\r` line
        // never matches any structure grammar (§2.1.2)
        let parsed = if ctx.lone_cr(head_idx) {
            None
        } else {
            parse_header(text)
        };
        let ParsedHeader {
            seq,
            nick,
            created,
            edited,
        } = match parsed {
            Some(ok) => ok,
            None => {
                let disp = names::display_text(raw);
                deferred.push(Diagnostic::new("E-HEAD", head_no).with("line", disp));
                // not a comment (§3.3.2); still scan unfenced body text (§2.5.5)
                scan_body_strays(ctx, sec);
                continue;
            }
        };
        // seq tracking (§3.3.1), in file order. Numbering must start at 1;
        // prev begins at 0 so a first legal seq > 1 is the existing
        // nonblocking W-SEQ-GAP (no renumbering, no write refusal).
        if !seen.insert(seq) {
            deferred.push(Diagnostic::new("E-SEQ-DUP", head_no).with("n", seq.to_string()));
        } else if seq > prev + 1 {
            ctx.push(
                Diagnostic::new("W-SEQ-GAP", head_no)
                    .with("a", prev.to_string())
                    .with("b", seq.to_string())
                    .with("m", (prev + 1).to_string()),
            );
        } else if seq < prev {
            ctx.push(
                Diagnostic::new("W-SEQ-ORDER", head_no)
                    .with("n", seq.to_string())
                    .with("m", prev.to_string()),
            );
        }
        prev = seq;
        if seq > max_legal {
            max_legal = seq;
        }
        // stray text outside fences (§2.5.5)
        scan_body_strays(ctx, sec);
        // body and derived references (§3.4); no fence -> W-BODY
        let (mentions, replies) = match &sec.body {
            Some(body) => references(ctx.text(&body.content)),
            None => {
                ctx.push(
                    Diagnostic::new("W-BODY", head_no)
                        .with("unit", format!("comment #{seq}"))
                        .with("n", head_no.to_string())
                        .with("seq", seq.to_string())
                        .with("author", nick.clone()),
                );
                (Vec::new(), Vec::new())
            }
        };
        comments.push(Comment {
            seq,
            nick,
            created,
            edited,
            section: si,
            mentions,
            replies,
        });
        head_lines.push(head_no);
    }

    // --- post pass --------------------------------------------------------
    // reply targets that do not exist (§3.4.3)
    for (c, &head_no) in comments.iter().zip(&head_lines) {
        for &m in &c.replies {
            if !seen.contains(&m) {
                ctx.push(
                    Diagnostic::new("W-REPLY-DANGLING", head_no)
                        .with("n", c.seq.to_string())
                        .with("m", m.to_string()),
                );
            }
        }
    }
    // `{next}` / `{exhausted}` for deferred E-HEAD / E-SEQ-DUP (§9
    // suppl. 3). After stored legal seqs are known, assign distinct
    // unused numbers in source order from max_legal+1. Each emitted
    // repair reserves its number. Do not rewrite stored seqs, wrap, or
    // suggest u64::MAX; later repairs use the exhausted form.
    let mut assigned = max_legal;
    for mut d in deferred {
        d = match assigned.checked_add(1) {
            Some(n) if n < u64::MAX => {
                assigned = n;
                d.with("next", n.to_string())
            }
            _ => d.with("exhausted", "1"),
        };
        ctx.push(d);
    }

    Post {
        comments,
        subscribers,
        subscribers_span,
    }
}

/// W-STRAY for every non-blank line after an H2 that is not fenced
/// (§2.5.5). Fence open/close/content lines are opaque.
fn scan_body_strays(ctx: &mut ScanCtx, sec: &Section) {
    for idx in ctx.section_body_lines(sec) {
        if ctx.fenced(idx) || is_blank_line(ctx.line_str(idx)) {
            continue;
        }
        ctx.push(Diagnostic::new("W-STRAY", idx + 1).with("n", (idx + 1).to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_grammar_accepts_and_rejects() {
        let header = parse_header("#1 alice (2026-09-17T09:12:00Z)").expect("plain header parses");
        assert_eq!(header.seq, 1);
        assert_eq!(header.nick, "alice");
        assert_eq!(crate::time::format(header.created), "2026-09-17T09:12:00Z");
        assert!(header.edited.is_none());

        let header =
            parse_header("#2 bob (2026-09-17T09:20:00Z, edited 2026-09-17T09:22:00Z)").unwrap();
        assert_eq!(header.seq, 2);
        assert_eq!(
            crate::time::format(header.edited.unwrap()),
            "2026-09-17T09:22:00Z"
        );

        // any RFC 3339 offset is accepted and converted to UTC (§2.8.2)
        let header = parse_header("#3 c (2026-09-17T11:00:00+02:00)").unwrap();
        assert_eq!(crate::time::format(header.created), "2026-09-17T09:00:00Z");

        // trailing whitespace after ')' is padding; anything else is not
        assert!(parse_header("#1 alice (2026-09-17T09:12:00Z)  ").is_some());

        for bad in [
            "",
            "#",
            "#0 alice (2026-09-17T09:12:00Z)",
            "#01 alice (2026-09-17T09:12:00Z)",
            // u64::MAX and beyond are malformed headers, not comments (§3.3.1)
            "#18446744073709551615 a (2026-09-17T09:12:00Z)",
            "#18446744073709551616 a (2026-09-17T09:12:00Z)",
            "#1  alice (2026-09-17T09:12:00Z)",
            "#1 alice  (2026-09-17T09:12:00Z)",
            "#1 alice(2026-09-17T09:12:00Z)",
            "#1 -bad (2026-09-17T09:12:00Z)",
            "#1 (2026-09-17T09:12:00Z)",
            "#1 alice ()",
            "#1 alice (2026-09-17 09:12:00Z)",
            "#1 alice (2026-09-17T09:12:00Z",
            "#1 alice (2026-09-17T09:12:00Z) tail",
            "#1 alice (2026-09-17T09:12:00Z edited 2026-09-17T09:13:00Z)",
            "#1 alice (2026-09-17T09:12:00Z, edited bad)",
            "#1 alice (2026-09-17T09:12:00Z, edited 2026-09-17T09:13:00Z, x)",
        ] {
            assert!(parse_header(bad).is_none(), "should reject {bad:?}");
        }
    }

    #[test]
    fn references_extracts_mentions_and_replies() {
        let (mentions, replies) = references("hi @bob, @#1 and @#2; @@x, mail@work, a@bob, @#1");
        assert_eq!(mentions, vec!["bob".to_string()]);
        assert_eq!(replies, vec![1, 2]);

        // start-of-text and space/newline-preceded '@' count; a letter
        // before does not
        let (mentions, _) = references("@alice\nsecond @dave\nmail@nope");
        assert_eq!(mentions, vec!["alice".to_string(), "dave".to_string()]);

        // a run longer than 64 nick characters is not a mention (§2.3)
        let (mentions, _) = references(&format!("@{}", "a".repeat(65)));
        assert!(mentions.is_empty());
        let (mentions, _) = references(&format!("@{}", "a".repeat(64)));
        assert_eq!(mentions, vec!["a".repeat(64)]);

        // an overflowing @#n is skipped, not truncated
        let (_, replies) = references("@#99999999999999999999999 @#7");
        assert_eq!(replies, vec![7]);

        // first-occurrence order is canonical; later repeats are dropped
        let (mentions, replies) = references("@carol @bob @carol @alice @bob @#9 @#2 @#9 @#1 @#2");
        assert_eq!(
            mentions,
            vec!["carol".to_string(), "bob".to_string(), "alice".to_string()]
        );
        assert_eq!(replies, vec![9, 2, 1]);
    }

    #[test]
    fn subscribers_line_grammar() {
        let (list, dup) = parse_subscribers("alice, bob").unwrap();
        assert_eq!(list, vec!["alice".to_string(), "bob".to_string()]);
        assert!(!dup);
        let (list, dup) = parse_subscribers("alice, bob ,alice").unwrap();
        assert_eq!(list, vec!["alice".to_string(), "bob".to_string()]);
        assert!(dup);
        let (list, dup) = parse_subscribers("carol, alice, carol, bob, alice").unwrap();
        assert_eq!(
            list,
            vec!["carol".to_string(), "alice".to_string(), "bob".to_string()]
        );
        assert!(dup);
        for bad in ["", "alice,", ",alice", "alice,,bob", "alice,x y", "-x"] {
            assert!(parse_subscribers(bad).is_none(), "should reject {bad:?}");
        }
    }

    #[test]
    fn post_scan_end_to_end() {
        let src = concat!(
            "# t.post\n",
            "- subscribers: alice, bob\n",
            "- subscribers: alice\n",
            "stray\n",
            "> note\n",
            "\n",
            "## #1 alice (2026-09-17T09:00:00Z)\n",
            "\n",
            "```md\n",
            "hi @bob @#2\n",
            "```\n",
            "extra stray\n",
            "\n",
            "## #2 bob (2026-09-17T09:20:00Z, edited 2026-09-17T09:22:00Z)\n",
            "\n",
            "```md\n",
            "@#1 @#9 reply\n",
            "```\n",
            "\n",
            "## #4 carol (2026-09-17T10:00:00Z)\n",
            "\n",
            "## #2 dave (2026-09-17T11:00:00Z)\n",
            "\n",
            "## plain section\n",
            "text\n",
        );
        let now = crate::time::parse("2026-09-17T10:00:00Z").unwrap();
        let doc = crate::doc::parse(
            src.as_bytes().to_vec(),
            std::path::Path::new("t.agent-besties.md"),
            now,
        )
        .expect("parses");

        assert_eq!(
            doc.post.subscribers,
            vec!["alice".to_string(), "bob".to_string()]
        );
        let sp = doc.post.subscribers_span.expect("first line wins");
        assert_eq!(&doc.source[sp.start..sp.end], "- subscribers: alice, bob\n");

        assert_eq!(
            doc.post.comments.iter().map(|c| c.seq).collect::<Vec<_>>(),
            vec![1, 2, 4, 2]
        );
        assert_eq!(doc.post.comments[0].section, 0);
        assert_eq!(doc.post.comments[3].section, 3);
        assert_eq!(doc.post.comments[0].mentions, vec!["bob".to_string()]);
        assert_eq!(doc.post.comments[0].replies, vec![2]);
        // activity: edited when present, else created
        assert_eq!(
            doc.post.comments[1].activity().to_rfc3339(),
            "2026-09-17T09:22:00+00:00"
        );
        assert_eq!(
            doc.post.comments[0].activity().to_rfc3339(),
            "2026-09-17T09:00:00+00:00"
        );
        // both fence-less sections (#4 and the duplicate #2) carry W-BODY
        assert!(doc.post.comments[2].mentions.is_empty());
        assert!(doc.post.comments[2].replies.is_empty());

        let rows: Vec<(&str, usize)> = doc.diagnostics.iter().map(|d| (d.id, d.line)).collect();
        assert_eq!(
            rows,
            vec![
                ("W-SUB-DUP", 3),
                ("W-STRAY", 4),
                ("W-STRAY", 12),
                ("W-REPLY-DANGLING", 14),
                ("W-BODY", 20),
                ("W-SEQ-GAP", 20),
                ("E-SEQ-DUP", 22),
                ("W-BODY", 22),
                ("W-STRAY", 24),
                ("W-STRAY", 25),
            ]
        );
        let dup = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "E-SEQ-DUP")
            .expect("E-SEQ-DUP present");
        assert!(dup.params.contains(&("n", "2".into())));
        // max legal seq in the file is 4, so {next} = max legal + 1 = 5
        // (§9 suppl. 3)
        assert!(dup.params.contains(&("next", "5".into())));
    }

    fn parse_post(src: &str) -> crate::doc::Document {
        let now = crate::time::parse("2026-09-17T10:00:00Z").unwrap();
        crate::doc::parse(
            src.as_bytes().to_vec(),
            std::path::Path::new("foo.agent-besties.md"),
            now,
        )
        .expect("parses")
    }

    fn diag_rows(doc: &crate::doc::Document) -> Vec<(&str, usize)> {
        doc.diagnostics.iter().map(|d| (d.id, d.line)).collect()
    }

    #[test]
    fn preamble_cr_tainted_gt_is_stray_not_comment() {
        let tainted_src = "# foo.post\n\n> hello\rworld\n";
        let tainted = parse_post(tainted_src);
        assert_eq!(diag_rows(&tainted), vec![("W-STRAY", 3)]);
        assert!(tainted.post.subscribers.is_empty());
        assert_eq!(tainted.source, tainted_src);

        let genuine = parse_post("# foo.post\n\n> hello world\n");
        assert!(diag_rows(&genuine).iter().all(|(id, _)| *id != "W-STRAY"));

        let crlf_src = "# foo.post\r\n\r\n> hello world\r\n";
        let crlf = parse_post(crlf_src);
        assert!(diag_rows(&crlf).iter().all(|(id, _)| *id != "W-STRAY"));
        assert!(diag_rows(&crlf).iter().any(|(id, _)| *id == "W-CRLF"));
        assert_eq!(crlf.source, crlf_src);
    }

    #[test]
    fn malformed_header_still_reports_body_stray() {
        let doc = parse_post("# foo.post\n\n## #1 alice (not-a-time)\n\nloose text\n");
        assert_eq!(diag_rows(&doc), vec![("E-HEAD", 3), ("W-STRAY", 5)]);
        assert!(doc.post.comments.is_empty());
        assert!(!doc.diagnostics.iter().any(|d| d.id == "W-BODY"));

        let fenced =
            parse_post("# foo.post\n\n## #1 alice (not-a-time)\n\n```md\nloose text\n```\n");
        assert_eq!(diag_rows(&fenced), vec![("E-HEAD", 3)]);
        assert!(fenced.post.comments.is_empty());

        let quoted = parse_post("# foo.post\n\n## #1 alice (not-a-time)\n\n> note\n");
        assert_eq!(diag_rows(&quoted), vec![("E-HEAD", 3), ("W-STRAY", 5)]);
        assert!(quoted.post.comments.is_empty());
    }

    #[test]
    fn references_hyphen_ended_nick_does_not_consume_next_boundary() {
        let (mentions, replies) = references("@alice-@bob");
        assert_eq!(mentions, vec!["alice-".to_string(), "bob".to_string()]);
        assert!(replies.is_empty());

        // '-' is a boundary; letter, '_', and '@' still are not
        let (mentions, _) = references("a@bob _@carol @@dave -@erin");
        assert_eq!(mentions, vec!["erin".to_string()]);

        let (mentions, _) = references("@alice_@bob");
        assert_eq!(mentions, vec!["alice_".to_string()]);
    }

    #[test]
    fn first_comment_above_one_warns_seq_gap_without_renumbering() {
        let src = "# gap.post\n\n## #2 alice (2026-09-25T11:00:00Z)\n\n```md\nx\n```\n";
        let now = crate::time::parse("2026-09-25T12:00:00Z").unwrap();
        let doc = crate::doc::parse(
            src.as_bytes().to_vec(),
            std::path::Path::new("gap.agent-besties.md"),
            now,
        )
        .expect("parses");
        assert_eq!(doc.source, src);
        assert_eq!(
            doc.post.comments.iter().map(|c| c.seq).collect::<Vec<_>>(),
            vec![2]
        );
        let gap = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "W-SEQ-GAP")
            .expect("W-SEQ-GAP for missing #1");
        assert_eq!(gap.line, 3);
        assert!(gap.params.contains(&("a", "0".into())));
        assert!(gap.params.contains(&("b", "2".into())));
        assert!(gap.params.contains(&("m", "1".into())));
        assert!(!doc
            .diagnostics
            .iter()
            .any(|d| d.id == "E-SEQ-DUP" || d.id == "W-SEQ-ORDER"));

        let start =
            parse_post("# foo.post\n\n## #1 alice (2026-09-17T09:00:00Z)\n\n```md\nx\n```\n");
        assert!(!start
            .diagnostics
            .iter()
            .any(|d| d.id == "W-SEQ-GAP" || d.id == "W-SEQ-ORDER"));
    }

    #[test]
    fn reply_targets_and_seq_membership_keep_first_encounter_order() {
        let src = concat!(
            "# foo.post\n",
            "\n",
            "## #1 alice (2026-09-17T09:00:00Z)\n",
            "\n",
            "```md\n",
            "@#9 @#2 @#8 @#9\n",
            "```\n",
            "\n",
            "## #3 bob (2026-09-17T09:20:00Z)\n",
            "\n",
            "```md\n",
            "@#1\n",
            "```\n",
            "\n",
            "## #2 carol (2026-09-17T10:00:00Z)\n",
            "\n",
            "```md\n",
            "@#7 @#3\n",
            "```\n",
            "\n",
            "## #3 dave (2026-09-17T11:00:00Z)\n",
            "\n",
            "```md\n",
            "@#3 @#99\n",
            "```\n",
        );
        let doc = parse_post(src);
        assert_eq!(
            doc.post.comments.iter().map(|c| c.seq).collect::<Vec<_>>(),
            vec![1, 3, 2, 3]
        );
        assert_eq!(doc.post.comments[0].replies, vec![9, 2, 8]);
        assert_eq!(doc.post.comments[2].replies, vec![7, 3]);
        assert_eq!(doc.post.comments[3].replies, vec![3, 99]);
        // Later and duplicate seqs still exist as reply targets; dangling
        // rows follow comment file order, then first-occurrence reply order
        // on the same header line (stable catalog sort).
        let dangling: Vec<(usize, &str)> = doc
            .diagnostics
            .iter()
            .filter(|d| d.id == "W-REPLY-DANGLING")
            .map(|d| {
                let m = d
                    .params
                    .iter()
                    .find(|(k, _)| *k == "m")
                    .and_then(|(_, v)| v.to_str())
                    .expect("m");
                (d.line, m)
            })
            .collect();
        assert_eq!(dangling, vec![(3, "9"), (3, "8"), (15, "7"), (21, "99")]);
        assert_eq!(
            diag_rows(&doc),
            vec![
                ("W-REPLY-DANGLING", 3),
                ("W-REPLY-DANGLING", 3),
                ("W-SEQ-GAP", 9),
                ("W-SEQ-ORDER", 15),
                ("W-REPLY-DANGLING", 15),
                ("E-SEQ-DUP", 21),
                ("W-REPLY-DANGLING", 21),
            ]
        );
        let dup = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "E-SEQ-DUP")
            .expect("E-SEQ-DUP present");
        assert!(dup.params.contains(&("n", "3".into())));
        // max legal seq is 3, so {next} = 4 even with a later duplicate
        assert!(dup.params.contains(&("next", "4".into())));
    }

    fn param<'a>(d: &'a Diagnostic, key: &str) -> Option<&'a str> {
        d.params
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.to_str())
    }

    #[test]
    fn same_report_emitted_next_repairs_malformed_and_duplicate_headers() {
        let time = "2026-09-17T10:00:00Z";

        let two_invalid = concat!(
            "# two-invalid.post\n",
            "\n",
            "## #1 alice (bad)\n",
            "\n",
            "```md\n",
            "keep-0\n",
            "```\n",
            "\n",
            "## #1 bob (also-bad)\n",
            "\n",
            "```md\n",
            "keep-1\n",
            "```\n",
        );
        let doc = parse_post(two_invalid);
        assert_eq!(
            doc.diagnostics
                .iter()
                .filter(|d| d.id == "E-HEAD")
                .map(|d| d.line)
                .collect::<Vec<_>>(),
            vec![3, 9]
        );
        let heads: Vec<_> = doc
            .diagnostics
            .iter()
            .filter(|d| d.id == "E-HEAD")
            .collect();
        let n0 = param(heads[0], "next").expect("first E-HEAD next");
        let n1 = param(heads[1], "next").expect("second E-HEAD next");
        assert_eq!((n0, n1), ("1", "2"));
        let repaired_src = two_invalid
            .replace("## #1 alice (bad)", &format!("## #{n0} alice ({time})"))
            .replace("## #1 bob (also-bad)", &format!("## #{n1} bob ({time})"));
        let repaired = parse_post(&repaired_src);
        assert!(
            repaired.blocking().is_empty(),
            "copied distinct next repairs must permit writes: {:?}",
            diag_rows(&repaired)
        );
        assert_eq!(
            repaired
                .post
                .comments
                .iter()
                .map(|c| (c.seq, c.nick.as_str()))
                .collect::<Vec<_>>(),
            vec![(1, "alice"), (2, "bob")]
        );
        assert!(repaired_src.contains("```md\nkeep-0\n```"));
        assert!(repaired_src.contains("```md\nkeep-1\n```"));

        let three_dup = concat!(
            "# three-duplicates.post\n",
            "\n",
            "## #1 alice (2026-09-26T09:00:00Z)\n",
            "\n",
            "```md\n",
            "keep-0\n",
            "```\n",
            "\n",
            "## #1 bob (2026-09-26T09:00:00Z)\n",
            "\n",
            "```md\n",
            "keep-1\n",
            "```\n",
            "\n",
            "## #1 carol (2026-09-26T09:00:00Z)\n",
            "\n",
            "```md\n",
            "keep-2\n",
            "```\n",
        );
        let doc = parse_post(three_dup);
        let dups: Vec<_> = doc
            .diagnostics
            .iter()
            .filter(|d| d.id == "E-SEQ-DUP")
            .collect();
        assert_eq!(dups.len(), 2);
        assert_eq!((dups[0].line, dups[1].line), (9, 15));
        let n0 = param(dups[0], "next").expect("first E-SEQ-DUP next");
        let n1 = param(dups[1], "next").expect("second E-SEQ-DUP next");
        assert_eq!((n0, n1), ("2", "3"));
        assert_eq!(doc.post.comments[0].seq, 1);
        let repaired_src = three_dup
            .replace(
                "## #1 bob (2026-09-26T09:00:00Z)",
                &format!("## #{n0} bob (2026-09-26T09:00:00Z)"),
            )
            .replace(
                "## #1 carol (2026-09-26T09:00:00Z)",
                &format!("## #{n1} carol (2026-09-26T09:00:00Z)"),
            );
        assert!(repaired_src.contains("## #1 alice (2026-09-26T09:00:00Z)"));
        let repaired = parse_post(&repaired_src);
        assert!(
            repaired.blocking().is_empty(),
            "copied distinct next repairs must permit writes: {:?}",
            diag_rows(&repaired)
        );
        assert_eq!(
            repaired
                .post
                .comments
                .iter()
                .map(|c| (c.seq, c.nick.as_str()))
                .collect::<Vec<_>>(),
            vec![(1, "alice"), (2, "bob"), (3, "carol")]
        );
        assert!(repaired_src.contains("```md\nkeep-0\n```"));
        assert!(repaired_src.contains("```md\nkeep-1\n```"));
        assert!(repaired_src.contains("```md\nkeep-2\n```"));
    }

    #[test]
    fn near_ceiling_later_repairs_exhaust_without_wrap() {
        let last = u64::MAX - 1;
        let almost = u64::MAX - 2;
        let time = "2026-09-17T09:00:00Z";

        let mixed = format!(
            "# ceil.post\n\n## #{almost} alice ({time})\n\n```md\nkeep-high\n```\n\n## #1 bob (not-a-time)\n\n```md\nkeep-head\n```\n\n## #{almost} carol ({time})\n\n```md\nkeep-dup\n```\n"
        );
        let doc = parse_post(&mixed);
        let head = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "E-HEAD")
            .expect("E-HEAD");
        let dup = doc
            .diagnostics
            .iter()
            .find(|d| d.id == "E-SEQ-DUP")
            .expect("E-SEQ-DUP");
        assert!(
            head.line < dup.line,
            "source order of mixed deferred repairs"
        );
        let last_s = last.to_string();
        assert_eq!(param(head, "next"), Some(last_s.as_str()));
        assert!(param(head, "exhausted").is_none());
        assert_eq!(param(dup, "next"), None);
        assert!(param(dup, "exhausted").is_some());
        let n: u64 = param(head, "next").unwrap().parse().unwrap();
        assert_ne!(n, 0);
        assert_ne!(n, u64::MAX);
        assert!(n < u64::MAX);

        let repaired_src = mixed.replace(
            "## #1 bob (not-a-time)",
            &format!("## #{n} bob (2026-09-17T10:00:00Z)"),
        );
        assert!(repaired_src.contains(&format!("## #{almost} alice ({time})")));
        assert!(repaired_src.contains(&format!("## #{almost} carol ({time})")));
        let repaired = parse_post(&repaired_src);
        assert!(
            repaired.diagnostics.iter().any(|d| d.id == "E-SEQ-DUP"),
            "one remaining slot cannot repair both live headers"
        );
        assert!(repaired
            .post
            .comments
            .iter()
            .all(|c| c.seq != 0 && c.seq != u64::MAX));
        assert!(repaired_src.contains("```md\nkeep-high\n```"));
        assert!(repaired_src.contains("```md\nkeep-head\n```"));
        assert!(repaired_src.contains("```md\nkeep-dup\n```"));

        let exhausted = format!(
            "# ceil.post\n\n## #{last} alice ({time})\n\n```md\nkeep-high\n```\n\n## #1 bob (not-a-time)\n\n```md\nkeep-head\n```\n\n## #{last} carol ({time})\n\n```md\nkeep-dup\n```\n"
        );
        let doc = parse_post(&exhausted);
        let deferred: Vec<_> = doc
            .diagnostics
            .iter()
            .filter(|d| d.id == "E-HEAD" || d.id == "E-SEQ-DUP")
            .collect();
        assert_eq!(deferred.len(), 2);
        for d in deferred {
            assert_eq!(param(d, "next"), None);
            assert!(param(d, "exhausted").is_some());
        }
    }
}
