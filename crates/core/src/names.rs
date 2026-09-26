//! Identifier grammars and display folding (spec §2.3, §8.1.8, §10.9).
//!
//! Validation uses compiled regexes so the Unicode classes match the §2.3
//! grammar exactly; the folding helpers implement §10.9 (whitespace runs
//! collapse to one U+0020, control characters become U+FFFD in display
//! text). These are the ONLY folding/quoting helpers in the crate — callers
//! never write their own copy.
use std::sync::LazyLock;

use crate::consts;
use crate::error::Fault;

static NAME: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[\p{L}\p{N}_-][\p{L}\p{N}_.-]{0,99}$").unwrap());
static NICK: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[\p{L}\p{N}_][\p{L}\p{N}_-]{0,63}$").unwrap());
static STATUS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-z][a-z0-9-]{0,31}$").unwrap());
/// NAME grammar `[\p{L}\p{N}_-][\p{L}\p{N}_.-]*`, length 1..=100 (spec §2.3).
pub fn valid_name(s: &str) -> bool {
    NAME.is_match(s)
}

/// NICK grammar `[\p{L}\p{N}_][\p{L}\p{N}_-]*`, length 1..=64 (spec §2.3).
pub fn valid_nick(s: &str) -> bool {
    NICK.is_match(s)
}

/// STATUS grammar `[a-z][a-z0-9-]*`, length 1..=32 (spec §2.3).
pub fn valid_status(s: &str) -> bool {
    STATUS.is_match(s)
}

/// SECTION shares the NICK grammar, length 1..=64 (spec §2.3).
pub fn valid_section(s: &str) -> bool {
    NICK.is_match(s)
}

/// Maximal run of NICK characters at the start of `s` (first char class,
/// then continuation class). This is a scanner helper for comment headers
/// and state entries; the extracted run still has to pass [`valid_nick`].
/// Unicode classes are approximated with `char::is_alphabetic`/`is_numeric`;
/// combined with the exact [`valid_nick`] check the accept/reject outcome is
/// identical for every input.
pub(crate) fn nick_run(s: &str) -> &str {
    let mut end = s.len();
    let mut first = true;
    for (i, c) in s.char_indices() {
        let ok = c.is_alphabetic() || c.is_numeric() || c == '_' || (c == '-' && !first);
        if !ok {
            end = i;
            break;
        }
        first = false;
    }
    &s[..end]
}

/// Fold: collapse every run of Unicode whitespace (tabs, isolated `\r`, ...)
/// to a single U+0020 and trim both ends (spec §10.9.1). Not reversible
/// (§13 Q5).
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            pending_ws = true;
        } else {
            if pending_ws {
                if !out.is_empty() {
                    out.push(' ');
                }
                pending_ws = false;
            }
            out.push(c);
        }
    }
    out
}

/// Display text for titles, excerpts, path columns: fold, then replace
/// remaining control characters (Unicode Cc except tab, which folding already
/// consumed) with U+FFFD (spec §10.9.3).
pub fn display_text(s: &str) -> String {
    fold(s)
        .chars()
        .map(|c| {
            if c.is_control() && c != '\t' {
                '\u{FFFD}'
            } else {
                c
            }
        })
        .collect()
}

/// Excerpt: the first non-blank line of a body, whitespace-folded; longer
/// than [`consts::EXCERPT_MAX`] chars it is cut to the first 77 chars plus
/// `...` (spec §8.1.8). An empty body excerpts to the empty string.
pub fn excerpt(s: &str) -> String {
    let mut picked: Option<&str> = None;
    for raw_line in s.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if !line.chars().all(|c| c == ' ' || c == '\t') {
            picked = Some(line);
            break;
        }
    }
    let mut text = display_text(picked.unwrap_or(""));
    if text.chars().count() > consts::EXCERPT_MAX {
        text = text.chars().take(77).collect::<String>() + "...";
    }
    text
}

/// POSIX single-quote wrapping for command-form fix values (spec §8.3.4):
/// nonempty words containing only ASCII letters, digits or `_@%+,./-`
/// remain readable as-is. Everything else (including `=` assignment
/// syntax and the empty argument) is wrapped in single quotes, with
/// inner `'` written as `'\''`.
pub fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(b, b'_' | b'@' | b'%' | b'+' | b',' | b'.' | b'/' | b'-')
        })
    {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

/// Normalize a `-m` text before it is written (spec §2.1.3): CRLF becomes LF,
/// trailing newlines and whitespace are removed. Leading whitespace is
/// deliberately preserved — the spec only strips the end. An empty result is
/// rejected as a usage error (exit 2).
pub fn normalize_body(s: &str) -> Result<String, Fault> {
    let lf = s.replace("\r\n", "\n");
    let trimmed = lf.trim_end_matches(|c: char| c.is_whitespace());
    if trimmed.is_empty() {
        return Err(Fault::Usage(
            "-m text is empty after normalization (CRLF folded, trailing whitespace removed)"
                .to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nick_boundaries() {
        assert!(valid_nick("a"));
        assert!(valid_nick(&"a".repeat(64)));
        assert!(!valid_nick(&"a".repeat(65)));
        assert!(!valid_nick(""));
        assert!(!valid_nick("-a"));
        assert!(!valid_nick("a b"));
        assert!(valid_nick("a-b_2"));
        assert!(valid_nick("maría")); // Unicode letters allowed
    }

    #[test]
    fn name_allows_dots_after_first_char() {
        assert!(valid_name("a.b"));
        assert!(!valid_name(".a"));
        assert!(valid_name(&"a".repeat(100)));
        assert!(!valid_name(&"a".repeat(101)));
        assert!(!valid_name("a b"));
    }

    #[test]
    fn status_boundaries() {
        assert!(valid_status("locked"));
        assert!(valid_status(&"l".repeat(32)));
        assert!(!valid_status(&"l".repeat(33)));
        assert!(!valid_status("Locked"));
        assert!(!valid_status("2x"));
    }

    #[test]
    fn fold_collapses_runs_and_trims() {
        assert_eq!(fold("  a \t b  "), "a b");
        assert_eq!(fold("a\rb"), "a b"); // isolated \r is whitespace
        assert_eq!(fold("   "), "");
        assert_eq!(fold("a\n\nb"), "a b");
    }

    #[test]
    fn display_text_replaces_controls() {
        assert_eq!(display_text("a\u{1}b"), "a\u{FFFD}b");
        assert_eq!(display_text("a\tb"), "a b"); // tab folded, not replaced
    }

    #[test]
    fn excerpt_takes_first_non_blank_line_and_truncates() {
        assert_eq!(excerpt("\n\n  \nhello world\nsecond"), "hello world");
        assert_eq!(excerpt("line one\r\nline two"), "line one");
        assert_eq!(excerpt(""), "");
        let exactly = "x".repeat(80);
        assert_eq!(excerpt(&exactly).chars().count(), 80);
        let longer = "y".repeat(81);
        let cut = excerpt(&longer);
        assert_eq!(cut.chars().count(), 80);
        assert!(cut.ends_with("..."));
        assert_eq!(cut.trim_end_matches('.').chars().count(), 77);
    }

    #[test]
    fn normalize_body_rules() {
        assert_eq!(normalize_body("a\r\nb\r\n").unwrap(), "a\nb");
        assert_eq!(normalize_body("  kept  \n\n").unwrap(), "  kept");
        assert!(matches!(normalize_body(" \r\n "), Err(Fault::Usage(_))));
        assert!(matches!(normalize_body(""), Err(Fault::Usage(_))));
    }
}
