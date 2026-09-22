//! Target hashing and brief-entry verification (spec §4.4, §10.4).
//!
//! [`read_target`] streams a file through SHA-256 in fixed 64 KiB chunks
//! (paperwork `hash_file` lineage) and buffers at most `MAX_TARGET_BYTES
//! + 1` bytes in memory — the one extra byte is what distinguishes
//! "within the limit" from "over the limit" using the bytes actually read
//! (§10.4.2), never a racy `stat`. [`verify`] walks the §4.4 nine-step
//!   table strictly top to bottom; the state is derived from that order
//!   alone.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use regex::Regex;
use sha2::{Digest, Sha256};

use crate::consts::MAX_TARGET_BYTES;
use crate::error::Diagnostic;
use crate::names;
use crate::text::Span;
use crate::types::brief::Entry;

/// Streaming read chunk: hashing proceeds in fixed-size increments so
/// peak memory stays constant regardless of file size.
const HASH_CHUNK_SIZE: usize = 64 * 1024;

/// Upper bound for one buffer-reservation step (see [`read_target`]):
/// keeps the peak overshoot of the in-memory target buffer bounded while
/// avoiding per-chunk reallocation.
const RESERVE_STEP_CAP: usize = 4 * 1024 * 1024;

/// SHA-256 of a target's raw bytes, plus its lossy-decoded content when
/// the target fits in memory (for regex evaluation and rendering).
#[derive(Debug, Clone)]
pub struct Target {
    pub sha256: String,
    pub content: Option<String>,
    pub bytes: u64,
}

/// Single-pass lowercase hex encoding (one preallocated `String`, nibble
/// lookup table) — bit-identical to `format!`-per-byte output.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX_DIGITS[(byte >> 4) as usize] as char);
        out.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Read a target file exactly once (§4.4.2): the same bytes feed the
/// hasher and, when the target is within `MAX_TARGET_BYTES`, the in-memory
/// content. Over the limit, the remaining bytes are streamed to the hasher
/// with no size cap and `content` is `None`. Read errors propagate as the
/// raw [`io::Error`] — callers embed `{err}` in `E-ENTRY-TARGET`.
pub fn read_target(path: &Path) -> Result<Target, io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut chunk = vec![0u8; HASH_CHUNK_SIZE];
    // In-memory cap: MAX_TARGET_BYTES + 1. Reading the extra byte is how
    // "over the limit" is detected from the actual read (§10.4.2).
    let mut data: Vec<u8> = Vec::new();
    let mut total: u64 = 0;
    let cap = MAX_TARGET_BYTES as u64 + 1;
    let mut next_reserve = HASH_CHUNK_SIZE;
    loop {
        if total < cap {
            let want = (cap - total).min(HASH_CHUNK_SIZE as u64) as usize;
            let n = file.read(&mut chunk[..want])?;
            if n == 0 {
                break;
            }
            hasher.update(&chunk[..n]);
            // Bounded reservation: grow by doubling steps capped at
            // 4 MiB — capacity never exceeds the data by more than one
            // step (no ~2x cap overshoot from geometric Vec growth, no
            // 64 MiB preallocation for small targets), while keeping the
            // realloc count logarithmic instead of one per chunk.
            let reserve = next_reserve.min((cap - total) as usize).max(n);
            data.reserve_exact(reserve);
            data.extend_from_slice(&chunk[..n]);
            next_reserve = (next_reserve * 2).min(RESERVE_STEP_CAP);
            total += n as u64;
        } else {
            // Over the limit: keep feeding the hasher to EOF, unbounded.
            let n = file.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            hasher.update(&chunk[..n]);
            total += n as u64;
        }
    }
    let sha256 = hex_encode(&hasher.finalize());
    if total <= MAX_TARGET_BYTES as u64 {
        // Consume the buffer: valid UTF-8 needs no duplicate; lossy
        // decoding (U+FFFD replacements) happens only for invalid input.
        let content = match String::from_utf8(data) {
            Ok(text) => text,
            Err(utf8_err) => String::from_utf8_lossy(utf8_err.as_bytes()).into_owned(),
        };
        Ok(Target {
            sha256,
            content: Some(content),
            bytes: total,
        })
    } else {
        drop(data);
        Ok(Target {
            sha256,
            content: None,
            bytes: total,
        })
    }
}

/// Spec §10.4.3: every `\` becomes `/` before any path reasoning;
/// nothing else changes.
pub fn normalized_target(target: &str) -> String {
    target.replace('\\', "/")
}

/// `true` when `s` starts with a drive-letter prefix (`X:`) — including
/// the drive-relative forms (`X:`, `X:foo`) that `Path::is_absolute`
/// rejects. §10.4.3 treats any `X:`-prefixed target as absolute on
/// every platform: it is opened as written, never joined under the
/// brief's directory.
fn has_drive_prefix(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Resolve a brief entry's `path` (§10.4.3): normalize separators; a
/// leading `/` (covering `//server/share` and `//?/`) is absolute, as is
/// any drive-prefixed form; on Windows `Path::is_absolute` additionally
/// covers `X:/...`. Everything else joins the brief file's directory.
pub fn resolve_target(brief_path: &Path, target: &str) -> PathBuf {
    let normalized = normalized_target(target);
    let absolute = normalized.starts_with('/')
        || has_drive_prefix(&normalized)
        || (cfg!(windows) && Path::new(&normalized).is_absolute());
    if absolute {
        PathBuf::from(normalized)
    } else {
        brief_path
            .parent()
            .unwrap_or(Path::new(""))
            .join(normalized)
    }
}

/// §4.4 step 7 / §4.3.4: exactly 64 characters of lowercase hex.
fn is_valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// 1-based line number of the byte at `index` (a `\n` belongs to the line
/// it terminates). Lines are counted by `\n`.
fn line_of_byte(content: &str, index: usize) -> usize {
    1 + content.as_bytes()[..index]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
}

/// Number of lines in `content`: zero only when the content is empty; a
/// final line without a trailing `\n` still counts as a line.
fn line_count(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        line_of_byte(content, content.len() - 1)
    }
}

/// Expand a match byte span to whole-line boundaries: the 1-based
/// inclusive line range it covers, and the byte span of those same lines
/// (each line including its `\n` terminator).
fn expand_to_lines(content: &str, start: usize, end: usize) -> ((usize, usize), Span) {
    let bytes = content.as_bytes();
    let start_line = line_of_byte(content, start);
    let end_line = if start == end {
        start_line
    } else {
        line_of_byte(content, end - 1)
    };
    let line_start = bytes[..start]
        .iter()
        .rposition(|&b| b == b'\n')
        .map_or(0, |i| i + 1);
    // The first `\n` at or after the last matched byte terminates the
    // last covered line; a match ending on a `\n` includes it.
    let probe = if start == end { start } else { end - 1 };
    let line_end = bytes[probe..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |i| probe + i + 1);
    (
        (start_line, end_line),
        Span {
            start: line_start,
            end: line_end,
        },
    )
}

/// Outcome of checking one brief entry against its target (§4.4).
pub struct Verification {
    /// "fresh" | "shifted" | "stale" | "unchecked"
    pub state: &'static str,
    /// "no path" | "target missing" | "target unreadable" | "regex invalid"
    /// | "target too large" | "regex not found" | "no hash"
    pub reason: Option<&'static str>,
    pub target: Option<Target>,
    /// `L<a>..L<b>`, 1-based inclusive line numbers.
    pub range: Option<(usize, usize)>,
    /// Byte span in `content` covering those same whole lines.
    pub content_span: Option<Span>,
    /// Named capture groups of the first match, values folded (§10.9.2).
    pub groups: Vec<(String, String)>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Verification {
    fn empty(state: &'static str, reason: Option<&'static str>) -> Verification {
        Verification {
            state,
            reason,
            target: None,
            range: None,
            content_span: None,
            groups: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

/// Verify one entry per the §4.4 table, top to bottom, first hit wins:
///
/// 1. no `path` property → `unchecked  no path` (no diagnostics here; the
///    parser's `E-ENTRY-PATH` already reports the same fact);
/// 2. target not found → `stale  target missing` + `W-ENTRY-STALE`;
/// 3. any other read error → `unchecked  target unreadable` +
///    `E-ENTRY-TARGET` at the path line;
/// 4. regex present but uncompilable → `stale  regex invalid` (no
///    `W-ENTRY-STALE`; `E-ENTRY-REGEX` already reports it);
/// 5. regex present and the read went over `MAX_TARGET_BYTES` →
///    `unchecked  target too large` (hash alone must not promote a regex
///    entry to `fresh`);
/// 6. regex present, compiles, no match → `stale  regex not found` +
///    `W-ENTRY-STALE`;
/// 7. no `hash` property or not 64 lowercase hex digits →
///    `unchecked  no hash` (no diagnostics; content stays renderable,
///    §10.10.2);
/// 8. hash equal → `fresh`;
/// 9. hash differs → `shifted` + `W-ENTRY-SHIFTED`.
pub fn verify(brief_path: &Path, entry: &Entry) -> Verification {
    // Step 1.
    let path_prop = match entry.path.as_ref() {
        None => return Verification::empty("unchecked", Some("no path")),
        Some(prop) => prop,
    };
    let normalized = normalized_target(&path_prop.value);
    let resolved = resolve_target(brief_path, &path_prop.value);

    // Steps 2-3: one read feeds hash and regex alike (§4.4.2).
    let target = match read_target(&resolved) {
        Ok(target) => target,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Verification {
                diagnostics: vec![Diagnostic::new("W-ENTRY-STALE", entry.head_line)
                    .with("title", names::display_text(&entry.title))
                    .with("reason", "target missing")],
                ..Verification::empty("stale", Some("target missing"))
            };
        }
        Err(err) => {
            return Verification {
                diagnostics: vec![Diagnostic::new("E-ENTRY-TARGET", path_prop.line)
                    .with("title", names::display_text(&entry.title))
                    .with("target", names::display_text(&normalized))
                    .with("err", err.to_string())],
                ..Verification::empty("unchecked", Some("target unreadable"))
            };
        }
    };

    let compiled = entry.regex.as_ref().map(|prop| Regex::new(&prop.value));

    // Step 4.
    if let Some(Err(_)) = &compiled {
        return Verification {
            target: Some(target),
            ..Verification::empty("stale", Some("regex invalid"))
        };
    }

    // Step 5: judged on the bytes actually read, never on `stat`.
    if compiled.is_some() && target.bytes > MAX_TARGET_BYTES as u64 {
        return Verification {
            target: Some(target),
            ..Verification::empty("unchecked", Some("target too large"))
        };
    }

    // Step 6 / match extraction (§10.10): first match via `captures` on
    // the lossy string, expanded to whole lines.
    let mut range: Option<(usize, usize)> = None;
    let mut content_span: Option<Span> = None;
    let mut groups: Vec<(String, String)> = Vec::new();
    let mut regex_matched = true;
    if let (Some(Ok(re)), Some(content)) = (&compiled, target.content.as_deref()) {
        match re.captures(content) {
            Some(caps) => {
                if let Some(whole) = caps.get(0) {
                    let (lines, span) = expand_to_lines(content, whole.start(), whole.end());
                    range = Some(lines);
                    content_span = Some(span);
                    for name in re.capture_names().flatten() {
                        if let Some(captured) = caps.name(name) {
                            groups.push((name.to_string(), names::fold(captured.as_str())));
                        }
                    }
                }
            }
            None => regex_matched = false,
        }
    }
    if !regex_matched {
        return Verification {
            target: Some(target),
            diagnostics: vec![Diagnostic::new("W-ENTRY-STALE", entry.head_line)
                .with("title", names::display_text(&entry.title))
                .with("reason", "regex not found")],
            ..Verification::empty("stale", Some("regex not found"))
        };
    }

    // No regex: the whole content renders (empty content has no lines,
    // so `range` stays `None`); over the limit all three fields stay
    // empty while the hash is still judged (§10.4.2).
    if compiled.is_none() {
        if let Some(content) = target.content.as_deref() {
            let lines = line_count(content);
            range = if lines == 0 { None } else { Some((1, lines)) };
            content_span = Some(Span {
                start: 0,
                end: content.len(),
            });
        }
    }

    // Steps 7-9.
    match entry.hash.as_ref().map(|prop| prop.value.as_str()) {
        Some(recorded) if is_valid_hash(recorded) => {
            if target.sha256 == recorded {
                Verification {
                    state: "fresh",
                    reason: None,
                    target: Some(target),
                    range,
                    content_span,
                    groups,
                    diagnostics: Vec::new(),
                }
            } else {
                Verification {
                    state: "shifted",
                    reason: None,
                    target: Some(target),
                    range,
                    content_span,
                    groups,
                    diagnostics: vec![Diagnostic::new("W-ENTRY-SHIFTED", entry.head_line)
                        .with("title", names::display_text(&entry.title))
                        .with("target", names::display_text(&normalized))],
                }
            }
        }
        _ => Verification {
            state: "unchecked",
            reason: Some("no hash"),
            target: Some(target),
            range,
            content_span,
            groups,
            diagnostics: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::brief::Property;
    use std::fs;
    use std::io::Write;

    /// SHA-256 of the empty input.
    const EMPTY_SHA: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    /// Scratch file with a process-unique name in the temp dir; the
    /// "brief" lives in the same directory so relative targets resolve
    /// onto it. Cleanup by the caller is best-effort.
    fn scratch(name: &str, bytes: &[u8]) -> (PathBuf, String, PathBuf) {
        let unique = format!("coexist-hash-{}-{}.txt", std::process::id(), name);
        let full = std::env::temp_dir().join(&unique);
        fs::write(&full, bytes).unwrap();
        let brief = std::env::temp_dir().join("coexist-hash.brief");
        (brief, unique, full)
    }

    fn prop(key: &str, value: &str, line: usize) -> Property {
        Property {
            key: key.to_string(),
            value: value.to_string(),
            span: 0..0,
            value_span: 0..0,
            line,
        }
    }

    fn entry(path: Option<&str>, hash: Option<&str>, regex: Option<&str>) -> Entry {
        Entry {
            title: "sample entry".to_string(),
            section: 0,
            path: path.map(|v| prop("path", v, 3)),
            hash: hash.map(|v| prop("hash", v, 4)),
            regex: regex.map(|v| prop("regex", v, 5)),
            attributes: Vec::new(),
            attrs_end: 0,
            head_line: 2,
        }
    }

    #[test]
    fn read_target_small_files() {
        let (_, _, full) = scratch("empty", b"");
        let t = read_target(&full).unwrap();
        assert_eq!(t.bytes, 0);
        assert_eq!(t.content.as_deref(), Some(""));
        assert_eq!(t.sha256, EMPTY_SHA);
        let _ = fs::remove_file(&full);

        let (_, _, full) = scratch("abc", b"abc");
        let t = read_target(&full).unwrap();
        assert_eq!(t.bytes, 3);
        assert_eq!(t.content.as_deref(), Some("abc"));
        assert_eq!(
            t.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let _ = fs::remove_file(&full);
    }

    #[test]
    fn read_target_utf8_content_round_trips() {
        // Valid multibyte UTF-8 decodes exactly, no replacement pass.
        let (_, _, full) = scratch("utf8", "héllo→世界\n".as_bytes());
        let t = read_target(&full).unwrap();
        assert_eq!(t.content.as_deref(), Some("héllo→世界\n"));
        let _ = fs::remove_file(&full);

        // Invalid UTF-8 is lossy-decoded: the bad byte becomes U+FFFD.
        let (_, _, full) = scratch("badutf8", b"foo\xffbar");
        let t = read_target(&full).unwrap();
        assert_eq!(t.bytes, 7);
        assert_eq!(t.content.as_deref(), Some("foo\u{fffd}bar"));
        let _ = fs::remove_file(&full);
    }

    #[test]
    fn resolves_relative_and_absolute_targets() {
        assert_eq!(normalized_target("a\\b\\c.rs"), "a/b/c.rs");
        assert_eq!(normalized_target("keep  as-is.txt"), "keep  as-is.txt");

        let brief = Path::new("docs/onboarding.brief");
        assert_eq!(
            resolve_target(brief, "src\\parser\\mod.rs"),
            PathBuf::from("docs").join("src/parser/mod.rs")
        );
        assert_eq!(
            resolve_target(brief, "/abs/target.rs"),
            PathBuf::from("/abs/target.rs")
        );
        // Parent of a bare file name is the empty path: the join yields
        // the bare relative target.
        assert_eq!(
            resolve_target(Path::new("top.brief"), "leaf.rs"),
            PathBuf::from("leaf.rs")
        );
        // Drive-prefixed targets are absolute on every platform
        // (§10.4.3): opened as written, never joined under the brief's
        // directory.
        assert_eq!(resolve_target(brief, "C:/x.rs"), PathBuf::from("C:/x.rs"));
        assert_eq!(
            resolve_target(brief, "X:whatever.rs"),
            PathBuf::from("X:whatever.rs")
        );
    }

    #[test]
    fn hash_value_shape_gates_step_seven() {
        let (brief, name, full) = scratch("gate", b"stable bytes\n");
        let t = read_target(&full).unwrap();
        let upper = t.sha256.to_uppercase();
        let short = t.sha256[..63].to_string();
        let non_hex = "z".repeat(64);
        for bad in [upper.as_str(), short.as_str(), non_hex.as_str()] {
            let v = verify(&brief, &entry(Some(&name), Some(bad), None));
            assert_eq!(
                (v.state, v.reason),
                ("unchecked", Some("no hash")),
                "{:?}",
                bad
            );
            assert!(v.diagnostics.is_empty());
            assert!(v.target.as_ref().unwrap().content.is_some());
            assert_eq!(v.range, Some((1, 1)));
            assert!(v.content_span.is_some());
        }
        let v = verify(&brief, &entry(Some(&name), None, None));
        assert_eq!((v.state, v.reason), ("unchecked", Some("no hash")));
        assert!(v.diagnostics.is_empty());
        let _ = fs::remove_file(&full);
    }

    #[test]
    fn target_missing_beats_invalid_regex() {
        let brief = std::env::temp_dir().join("coexist-hash.brief");
        let v = verify(
            &brief,
            &entry(
                Some("definitely-not-here-7f3a9.rs"),
                Some(EMPTY_SHA),
                Some("(unclosed"),
            ),
        );
        assert_eq!((v.state, v.reason), ("stale", Some("target missing")));
        assert!(v.target.is_none());
        assert_eq!(v.diagnostics.len(), 1);
        assert_eq!(v.diagnostics[0].id, "W-ENTRY-STALE");
        assert_eq!(v.diagnostics[0].line, 2);
        assert!(v.diagnostics[0]
            .params
            .iter()
            .any(|(k, val)| *k == "reason" && val == "target missing"));

        // Step 4 on its own: unreadable regex, readable target — no
        // W-ENTRY-STALE (E-ENTRY-REGEX already reports the fact).
        let (brief, name, full) = scratch("reinvalid", b"content\n");
        let v = verify(
            &brief,
            &entry(Some(&name), Some(EMPTY_SHA), Some("(unclosed")),
        );
        assert_eq!((v.state, v.reason), ("stale", Some("regex invalid")));
        assert!(v.diagnostics.is_empty());
        assert!(v.target.is_some());
        let _ = fs::remove_file(&full);
    }

    #[test]
    fn regex_hit_with_bad_hash_keeps_match_range() {
        let (brief, name, full) = scratch("rehit", b"one\nfind me here\nthree\n");
        // "find me" matches bytes 4..11 on line 2; the whole line spans
        // 4..17 ("find me here\n").
        let v = verify(
            &brief,
            &entry(Some(&name), Some("not-a-hash"), Some("find me")),
        );
        assert_eq!((v.state, v.reason), ("unchecked", Some("no hash")));
        assert!(v.diagnostics.is_empty());
        assert!(v.target.as_ref().unwrap().content.is_some());
        assert_eq!(v.range, Some((2, 2)));
        assert_eq!(v.content_span, Some(Span { start: 4, end: 17 }));
        assert!(v.groups.is_empty());

        let v = verify(
            &brief,
            &entry(Some(&name), Some("not-a-hash"), Some("(?P<word>find) me")),
        );
        assert_eq!(v.groups, vec![("word".to_string(), "find".to_string())]);
        let _ = fs::remove_file(&full);
    }

    #[test]
    fn regex_match_expands_to_whole_line_range() {
        // "a\nbb\nccc\ndd\n": the match "bb\nccc" covers bytes 2..8 on
        // lines 2..3; those whole lines span 2..9.
        let (brief, name, full) = scratch("range", b"a\nbb\nccc\ndd\n");
        let v = verify(
            &brief,
            &entry(Some(&name), Some(EMPTY_SHA), Some("bb\nccc")),
        );
        assert_eq!(v.state, "shifted");
        assert_eq!(v.range, Some((2, 3)));
        assert_eq!(v.content_span, Some(Span { start: 2, end: 9 }));
        assert_eq!(v.diagnostics.len(), 1);
        assert_eq!(v.diagnostics[0].id, "W-ENTRY-SHIFTED");
        let _ = fs::remove_file(&full);
    }

    #[test]
    fn regex_not_found_is_stale_with_warning() {
        let (brief, name, full) = scratch("notfound", b"nothing to see\n");
        let v = verify(
            &brief,
            &entry(Some(&name), Some(EMPTY_SHA), Some("absent-token")),
        );
        assert_eq!((v.state, v.reason), ("stale", Some("regex not found")));
        assert!(v.range.is_none() && v.content_span.is_none() && v.groups.is_empty());
        assert!(v.target.is_some());
        assert_eq!(v.diagnostics.len(), 1);
        assert_eq!(v.diagnostics[0].id, "W-ENTRY-STALE");
        assert!(v.diagnostics[0]
            .params
            .iter()
            .any(|(k, val)| *k == "reason" && val == "regex not found"));
        let _ = fs::remove_file(&full);
    }

    #[test]
    fn no_regex_over_limit_streams_hash_without_content() {
        // One file of MAX_TARGET_BYTES + 1 bytes: the +1 read proves the
        // over-limit branch from bytes actually read (§10.4.2); the rest
        // is streamed. Writing also goes through a repeating 64 KiB
        // pattern so the test never holds the file in memory either.
        let name = format!("coexist-hash-{}-big.bin", std::process::id());
        let full = std::env::temp_dir().join(&name);
        let mut pattern = [0u8; HASH_CHUNK_SIZE];
        for (i, byte) in pattern.iter_mut().enumerate() {
            *byte = b'a' + (i % 26) as u8;
        }
        let mut hasher = Sha256::new();
        {
            let mut file = fs::File::create(&full).unwrap();
            let limit = MAX_TARGET_BYTES as u64;
            let mut written: u64 = 0;
            while written < limit + 1 {
                let take = HASH_CHUNK_SIZE.min((limit + 1 - written) as usize);
                file.write_all(&pattern[..take]).unwrap();
                hasher.update(&pattern[..take]);
                written += take as u64;
            }
        }
        let expected = hex_encode(&hasher.finalize());
        let brief = std::env::temp_dir().join("coexist-hash.brief");

        let t = read_target(&full).unwrap();
        assert_eq!(t.bytes, MAX_TARGET_BYTES as u64 + 1);
        assert!(t.content.is_none());

        let v = verify(&brief, &entry(Some(&name), Some(expected.as_str()), None));
        assert_eq!(v.state, "fresh");
        assert!(v.range.is_none() && v.content_span.is_none() && v.groups.is_empty());

        let mut wrong = expected.clone();
        let last = wrong.pop().unwrap();
        wrong.push(if last == '0' { '1' } else { '0' });
        let v = verify(&brief, &entry(Some(&name), Some(wrong.as_str()), None));
        assert_eq!(v.state, "shifted");
        assert!(v.content_span.is_none() && v.range.is_none());
        assert!(v.target.as_ref().unwrap().content.is_none());
        assert_eq!(v.diagnostics.len(), 1);
        assert_eq!(v.diagnostics[0].id, "W-ENTRY-SHIFTED");

        let _ = fs::remove_file(&full);
    }
}
