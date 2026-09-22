//! The `post` type view (spec §3): the preamble subscribers line, comment
//! sections with seq tracking, and the mentions/replies derived from fenced
//! bodies (derived data is never serialized back, §3.4.2).

use std::ops::Range;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::doc::Section;
use crate::error::Diagnostic;
use crate::names;
use crate::text::Span;
use crate::types::{is_blank_line, property_parts, ScanCtx};

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
    LazyLock::new(|| Regex::new(r"(?:^|[^\p{L}\p{N}_@])@([\p{L}\p{N}_][\p{L}\p{N}_-]*)").unwrap());
static REPLY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"@#([1-9][0-9]*)").unwrap());

/// Mentions and replies of a body text (spec §3.4.2), each deduplicated
/// preserving first-occurrence order.
///
/// A mention is an `@` at start of text or preceded by any character that
/// is not a Unicode letter/number/`_`/`@`, followed by a maximal run of
/// NICK characters of length 1..=64 (longer runs are not mentions). A
/// reply is `@#` followed by `[1-9][0-9]*`; a value that overflows `u64`
/// is skipped, not truncated.
pub fn references(text: &str) -> (Vec<String>, Vec<u64>) {
    let mut mentions: Vec<String> = Vec::new();
    for caps in MENTION_RE.captures_iter(text) {
        let m = caps.get(1).expect("capture group 1").as_str();
        if m.chars().count() <= 64 && !mentions.iter().any(|x| x.as_str() == m) {
            mentions.push(m.to_string());
        }
    }
    let mut replies: Vec<u64> = Vec::new();
    for caps in REPLY_RE.captures_iter(text) {
        let digits = caps.get(1).expect("capture group 1").as_str();
        if let Ok(n) = digits.parse::<u64>() {
            if !replies.contains(&n) {
                replies.push(n);
            }
        }
    }
    (mentions, replies)
}

/// `<NICK>(, <NICK>)*` (spec §3.2.2): split on `,`, trim each fragment,
/// every fragment must be a non-empty valid NICK. Returns the
/// deduplicated list (first occurrence kept) and whether any nick
/// repeated in the line.
fn parse_subscribers(value: &str) -> Option<(Vec<String>, bool)> {
    let mut list: Vec<String> = Vec::new();
    let mut dup = false;
    for frag in value.split(',') {
        let frag = frag.trim();
        if frag.is_empty() || !names::valid_nick(frag) {
            return None;
        }
        if list.iter().any(|n| n.as_str() == frag) {
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
    // E-HEAD / E-SEQ-DUP need the `{next}` / `{exhausted}` param which is
    // only known after every comment was seen (§9 suppl. 3). They are held
    // back and pushed at the end; `doc::parse` sorts by (line, catalog
    // order), so the later push cannot change the rendered order.
    let mut deferred: Vec<Diagnostic> = Vec::new();
    // 1-based header line of each comment, for W-REPLY-DANGLING anchors
    let mut head_lines: Vec<usize> = Vec::new();
    let mut seen: Vec<u64> = Vec::new();
    let mut prev: Option<u64> = None;
    let mut max_legal: u64 = 0;

    // --- preamble (§3.2) --------------------------------------------------
    for idx in preamble {
        if ctx.fenced(idx) {
            continue;
        }
        let line = ctx.line_str(idx);
        if is_blank_line(line) || line.starts_with("> ") {
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
                continue; // the section is not a comment; nothing else derives from it
            }
        };
        // seq tracking (§3.3.1), in file order
        if seen.contains(&seq) {
            deferred.push(Diagnostic::new("E-SEQ-DUP", head_no).with("n", seq.to_string()));
        } else if let Some(p) = prev {
            if seq > p + 1 {
                ctx.push(
                    Diagnostic::new("W-SEQ-GAP", head_no)
                        .with("a", p.to_string())
                        .with("b", seq.to_string())
                        .with("m", (p + 1).to_string()),
                );
            } else if seq < p {
                ctx.push(
                    Diagnostic::new("W-SEQ-ORDER", head_no)
                        .with("n", seq.to_string())
                        .with("m", p.to_string()),
                );
            }
        }
        seen.push(seq);
        prev = Some(seq);
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
    // `{next}` / `{exhausted}` for the deferred E-HEAD / E-SEQ-DUP (§9
    // suppl. 3): the largest legal seq plus one, unless numbering is
    // exhausted (largest legal seq already u64::MAX-1)
    for mut d in deferred {
        if max_legal == u64::MAX - 1 {
            d = d.with("exhausted", "1");
        } else {
            d = d.with("next", (max_legal + 1).to_string());
        }
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
    }

    #[test]
    fn subscribers_line_grammar() {
        let (list, dup) = parse_subscribers("alice, bob").unwrap();
        assert_eq!(list, vec!["alice".to_string(), "bob".to_string()]);
        assert!(!dup);
        let (list, dup) = parse_subscribers("alice, bob ,alice").unwrap();
        assert_eq!(list, vec!["alice".to_string(), "bob".to_string()]);
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
        assert!(dup.params.contains(&("n", "2".to_string())));
        // max legal seq in the file is 4, so {next} = max legal + 1 = 5
        // (§9 suppl. 3)
        assert!(dup.params.contains(&("next", "5".to_string())));
    }
}
