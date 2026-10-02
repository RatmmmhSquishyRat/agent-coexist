//! Raw-byte text primitives: spans, splices, fences, line index.
//!
//! Everything here works on the ORIGINAL bytes of a managed file (spec
//! §2.1.2, §10.3): all structural positions are byte offsets into the
//! source, so a splice write never disturbs unrelated bytes. Line
//! terminators are `\n` and `\r\n` (mixed accepted); an isolated `\r` is
//! line content and never a terminator.

use crate::consts;
use crate::error::{Diagnostic, Fault};

/// A half-open byte range `[start, end)` into a document's original bytes.
pub type Span = std::ops::Range<usize>;

/// One bounded replacement over original bytes (spec §10.3.1). An empty
/// `range` is an insertion. Splices covering the same offset keep the order
/// they appear in (impl_plan M3: same-point additions are merged by the
/// caller before they get here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Splice {
    pub range: Span,
    pub replacement: Vec<u8>,
}

/// Apply `splices` to `bytes` in a single copy (spec §10.3.1).
///
/// Validation first: ranges must be in bounds and mutually non-overlapping
/// (insertions at the same offset are ordered by (start, end), i.e. all
/// same-point insertions keep list order). The final length must stay within
/// [`consts::MAX_MANAGED_FILE_BYTES`] — a longer result is refused with an
/// `E-TOO-LARGE` diagnostic and nothing is written. Untouched bytes are
/// preserved verbatim.
///
/// Overlapping or out-of-bounds ranges cannot be derived from any parsed
/// document; they would be an internal geometry bug, so this function
/// panics loudly instead of silently dropping bytes.
pub fn apply(bytes: &[u8], splices: Vec<Splice>) -> Result<Vec<u8>, Fault> {
    let mut sorted = splices;
    sorted.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
    });

    let mut covered_end = 0usize;
    for s in &sorted {
        let r = &s.range;
        assert!(r.start <= r.end, "splice with inverted range {:?}", r);
        assert!(r.end <= bytes.len(), "splice past end of file: {:?}", r);
        assert!(
            r.start >= covered_end,
            "overlapping splices: {:?} starts inside previous range ending at {}",
            r,
            covered_end
        );
        covered_end = r.end;
    }

    let removed: usize = sorted.iter().map(|s| s.range.end - s.range.start).sum();
    let added: usize = sorted.iter().map(|s| s.replacement.len()).sum();
    let final_len = bytes.len() - removed + added;
    if final_len > consts::MAX_MANAGED_FILE_BYTES {
        // The file itself is within the limit; only this result is not.
        return Err(Fault::Diagnostics(vec![Diagnostic::new("E-TOO-LARGE", 1)
            .with(
                "limit",
                (consts::MAX_MANAGED_FILE_BYTES / (1024 * 1024)).to_string(),
            )
            .with("size", bytes.len().to_string())
            .with("result", final_len.to_string())]));
    }

    let mut out = Vec::with_capacity(final_len);
    let mut pos = 0usize;
    for s in &sorted {
        out.extend_from_slice(&bytes[pos..s.range.start]);
        out.extend_from_slice(&s.replacement);
        pos = s.range.end;
    }
    out.extend_from_slice(&bytes[pos..]);
    debug_assert_eq!(out.len(), final_len);
    Ok(out)
}

/// The fenced block the CLI writes for `content` (spec §2.5.4): info string
/// `md`, fence length `max(3, longest backtick run in content + 1)`. The
/// block has NO trailing newline — callers add terminators per the splice
/// geometry. `content` must already be normalized ([`crate::names::normalize_body`]).
pub fn fenced(content: &str) -> String {
    let mut longest = 2usize; // so the result is always at least 3
    let mut run = 0usize;
    for &b in content.as_bytes() {
        if b == b'`' {
            run += 1;
            if run > longest {
                longest = run;
            }
        } else {
            run = 0;
        }
    }
    let fence = "`".repeat(longest + 1);
    format!("{fence}md\n{content}\n{fence}")
}

/// What must be inserted before content appended at end-of-file so old text
/// and the new section are separated by at least one blank line (spec
/// §10.3.3). Trailing blank lines already present satisfy the rule and are
/// never removed; a trailing CRLF counts like LF.
///
/// A terminal isolated CR is not a terminator (§2.1.2). Inserting LF
/// against it would form CRLF and activate a previously opaque heading or
/// property; keep that CR in line content by placing a neutral ASCII space
/// before the LF gap.
pub fn append_gap(file: &str) -> &'static str {
    let bytes = file.as_bytes();
    let mut terminators = 0usize;
    let mut i = bytes.len();
    while i > 0 {
        if bytes[i - 1] == b'\n' {
            i -= 1;
            if i > 0 && bytes[i - 1] == b'\r' {
                i -= 1;
            }
            terminators += 1;
        } else {
            break;
        }
    }
    if terminators == 0 && i > 0 && bytes[i - 1] == b'\r' {
        return " \n\n";
    }
    match terminators {
        0 => "\n\n",
        1 => "\n",
        _ => "",
    }
}

/// One line of the source: content `[start, end)` plus terminator
/// `[end, next)` (empty when the file does not end with one).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Line {
    pub start: usize,
    pub end: usize,
    pub next: usize,
}

/// Byte-offset line index over a source string. `starts[0]` is always 0; a
/// file ending with a terminator has a final phantom empty line, which every
/// scan treats as blank. Terminator lengths (LF / CRLF / none) are captured
/// at construction so the index is self-contained and can be stored next to
/// an owned `String` without borrowing it.
pub(crate) struct LineIndex {
    starts: Vec<usize>,
    terms: Vec<u8>,
    len: usize,
}

impl LineIndex {
    pub(crate) fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut starts = vec![0usize];
        let mut terms = Vec::new();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
                // CRLF when the preceding byte is a lone `\r`
                let crlf = i > 0 && bytes[i - 1] == b'\r';
                let mut len = 1usize;
                if crlf {
                    len += 1;
                }
                terms.push(len as u8);
            }
        }
        // the last (possibly phantom) line has no terminator
        terms.push(0);
        debug_assert_eq!(starts.len(), terms.len());
        LineIndex {
            starts,
            terms,
            len: source.len(),
        }
    }

    /// Number of line starts (includes the phantom trailing line).
    pub(crate) fn count(&self) -> usize {
        self.starts.len()
    }

    pub(crate) fn line(&self, idx: usize) -> Line {
        let start = self.starts[idx];
        let next = self.starts.get(idx + 1).copied().unwrap_or(self.len);
        let end = next - self.terms[idx] as usize;
        Line { start, end, next }
    }

    /// 0-based line index containing byte `offset` (clamped into the file).
    pub(crate) fn idx_at(&self, offset: usize) -> usize {
        let off = offset.min(self.len);
        self.starts.partition_point(|&s| s <= off).saturating_sub(1)
    }

    /// 1-based line number of byte `offset`.
    pub(crate) fn line_at(&self, offset: usize) -> usize {
        self.idx_at(offset) + 1
    }

    /// Exclusive 0-based line index for a half-open byte offset: the first
    /// line whose start is not strictly before `offset`. Keep [`Self::idx_at`]
    /// for point lookups: at EOF without a terminator it names the last
    /// real line, which must not be used as an exclusive range end.
    /// A phantom trailing-newline line starts at EOF, so it becomes this
    /// exclusive end and is not treated as content.
    pub(crate) fn idx_end(&self, offset: usize) -> usize {
        let off = offset.min(self.len);
        self.starts.partition_point(|&s| s < off)
    }

    /// 0-based line indices whose bytes intersect the half-open `span`.
    /// Empty spans yield an empty range. A line whose start equals
    /// `span.end` is excluded. An unterminated last line is included when
    /// `span` runs to EOF; a phantom trailing-newline line is not.
    pub(crate) fn lines_in(&self, span: Span) -> std::ops::Range<usize> {
        if span.start >= span.end {
            let i = self.idx_at(span.start);
            return i..i;
        }
        let start = self.idx_at(span.start);
        let end = self.idx_end(span.end);
        start..end.max(start)
    }
}
/// Closed backtick fence block (spec §2.5).
#[derive(Debug, Clone)]
pub(crate) struct FenceBlock {
    /// 0-based open line index.
    pub open: usize,
    /// Whole block: start of the open line through the end of the close
    /// line's content (no trailing terminator).
    pub block: Span,
    /// Inner bytes: after the open line's terminator through the start of
    /// the close line (fence lines excluded).
    pub content: Span,
}

/// Global fence scan (spec §2.5, impl_plan M2). Fences are recognized
/// everywhere; everything between an open and its close is opaque. An
/// unclosed fence marks every later line as fenced so no structure is
/// derived after it (spec §2.5.3).
pub(crate) struct FenceScan {
    pub blocks: Vec<FenceBlock>,
    /// Per line: inside a fence block (open/close/content lines included).
    pub inside: Vec<bool>,
    /// Unclosed fence: 0-based open line and its backtick count.
    pub unclosed: Option<(usize, usize)>,
}

impl FenceScan {
    pub(crate) fn scan(source: &str, lines: &LineIndex) -> Self {
        let n = lines.count();
        let mut inside = vec![false; n];
        let mut blocks = Vec::new();
        let mut unclosed = None;
        let mut i = 0usize;
        while i < n {
            let l = lines.line(i);
            let content = &source[l.start..l.end];
            if content.contains('\r') {
                i += 1;
                continue;
            }
            if let Some(ticks) = open_fence(content) {
                let mut close = None;
                let mut j = i + 1;
                while j < n {
                    let lj = lines.line(j);
                    let cj = &source[lj.start..lj.end];
                    if !cj.contains('\r') && close_fence(cj, ticks) {
                        close = Some(j);
                        break;
                    }
                    j += 1;
                }
                match close {
                    Some(j) => {
                        let lj = lines.line(j);
                        blocks.push(FenceBlock {
                            open: i,
                            block: l.start..lj.end,
                            content: l.next..lj.start,
                        });
                        inside[i..=j].fill(true);
                        i = j + 1;
                    }
                    None => {
                        inside[i..n].fill(true);
                        unclosed = Some((i, ticks));
                        break;
                    }
                }
            } else {
                i += 1;
            }
        }
        FenceScan {
            blocks,
            inside,
            unclosed,
        }
    }
}

/// Open fence `^ {0,3}(`{3,})(?P<info>[^`]*)$` — returns the backtick count.
/// The line must not contain `\r` (checked by callers, spec §2.1.2).
fn open_fence(line: &str) -> Option<usize> {
    let b = line.as_bytes();
    let mut k = 0usize;
    while k < b.len() && k < 3 && b[k] == b' ' {
        k += 1;
    }
    let mut ticks = 0usize;
    while k < b.len() && b[k] == b'`' {
        ticks += 1;
        k += 1;
    }
    if ticks < 3 {
        return None;
    }
    if b[k..].contains(&b'`') {
        return None; // info string may not contain backticks
    }
    Some(ticks)
}

/// Close fence `^ {0,3}`{N,} *$` — at least `n` backticks, then spaces
/// (U+0020 only; tabs rejected), nothing else.
fn close_fence(line: &str, n: usize) -> bool {
    let b = line.as_bytes();
    let mut k = 0usize;
    while k < b.len() && k < 3 && b[k] == b' ' {
        k += 1;
    }
    let mut ticks = 0usize;
    while k < b.len() && b[k] == b'`' {
        ticks += 1;
        k += 1;
    }
    if ticks < n {
        return false;
    }
    b[k..].iter().all(|&c| c == b' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_preserves_untouched_bytes() {
        let orig = b"BOM\xff\xfe# head\r\n\r\n## s\r\nold\r\nTAIL".to_vec();
        // Byte math on `orig` (30 bytes): `BOM\xff\xfe# head` = 0..11,
        // CRLF = 11..13, CRLF = 13..15, `## s` = 15..19, `\r\n` = 19..21,
        // `old` = 21..24. An insertion at the H2's line start points at 15
        // — AFTER the complete CRLF pair — never at 14, which would splice
        // between the `\r` and its `\n`. apply() is a raw-byte splice
        // (§10.3.1): it honors any offset it is given verbatim, so the
        // geometry is derived from the bytes instead of hand-counted.
        let h2 = orig.windows(4).position(|w| w == b"## s").unwrap();
        let old = orig.windows(3).position(|w| w == b"old").unwrap();
        let splices = vec![
            Splice {
                range: old..old + 3,
                replacement: b"NEW".to_vec(),
            },
            Splice {
                range: h2..h2,
                replacement: b"insert".to_vec(),
            },
        ];
        let out = apply(&orig, splices).unwrap();
        assert_eq!(
            out,
            b"BOM\xff\xfe# head\r\n\r\ninsert## s\r\nNEW\r\nTAIL".to_vec()
        );
    }

    #[test]
    fn apply_same_point_inserts_keep_list_order() {
        let out = apply(
            b"ab".as_slice(),
            vec![
                Splice {
                    range: 1..1,
                    replacement: b"1".to_vec(),
                },
                Splice {
                    range: 1..1,
                    replacement: b"2".to_vec(),
                },
            ],
        )
        .unwrap();
        assert_eq!(out, b"a12b".to_vec());
    }

    #[test]
    fn apply_adjacent_ranges_allowed_overlap_panics() {
        let out = apply(
            b"abcd".as_slice(),
            vec![
                Splice {
                    range: 0..2,
                    replacement: b"X".to_vec(),
                },
                Splice {
                    range: 2..4,
                    replacement: b"Y".to_vec(),
                },
            ],
        )
        .unwrap();
        assert_eq!(out, b"XY".to_vec());
        let result = std::panic::catch_unwind(|| {
            let _ = apply(
                b"abcd".as_slice(),
                vec![
                    Splice {
                        range: 0..3,
                        replacement: b"X".to_vec(),
                    },
                    Splice {
                        range: 2..4,
                        replacement: b"Y".to_vec(),
                    },
                ],
            );
        });
        assert!(result.is_err(), "overlap must panic loudly");
    }

    #[test]
    fn apply_refuses_oversize_result() {
        let one_mib = vec![b'x'; 1024 * 1024];
        let mut splices = Vec::new();
        for _ in 0..17usize {
            splices.push(Splice {
                range: 0..0,
                replacement: one_mib.clone(),
            });
        }
        let err = apply(b"".as_slice(), splices).unwrap_err();
        match err {
            Fault::Diagnostics(d) => {
                assert_eq!(d.len(), 1);
                assert_eq!(d[0].id, "E-TOO-LARGE");
                assert_eq!(d[0].line, 1);
                assert_eq!(d[0].params[0].1, "16");
            }
            other => panic!("unexpected fault: {other:?}"),
        }
    }

    #[test]
    fn fenced_uses_safe_fence_length() {
        // §2.5.4: info string `md`, fence length = max(3, longest backtick
        // run in the content + 1). Min-fence boundary: a run of 2 still
        // rides the minimum 3-fence, and growth starts exactly at a run
        // of 3, which gets a 4-fence. The exact-block form also pins the
        // roundtrip — content verbatim between equal fences, and the
        // fence itself can never occur inside the content it delimits.
        for (body, fence_len) in [("x `` y", 3usize), ("run ``` mid", 4)] {
            let fence = "`".repeat(fence_len);
            assert_eq!(
                fenced(body),
                format!("{fence}md\n{body}\n{fence}"),
                "fence length for {body:?}"
            );
        }
    }

    #[test]
    fn append_gap_equivalence_classes() {
        assert_eq!(append_gap("abc"), "\n\n");
        assert_eq!(append_gap("abc\n"), "\n");
        assert_eq!(append_gap("abc\r\n"), "\n");
        assert_eq!(append_gap("abc\n\n"), "");
        assert_eq!(append_gap("abc\r\n\r\n"), "");
        assert_eq!(append_gap("abc\n\n\n\n"), "");
        assert_eq!(append_gap(""), "\n\n");
        assert_eq!(append_gap("abc\r"), " \n\n");
        assert_eq!(append_gap("abc\n\r"), " \n\n");
    }

    #[test]
    fn line_index_terminators_and_phantom() {
        let idx = LineIndex::new("a\r\nb\nc");
        assert_eq!(idx.count(), 3);
        let l0 = idx.line(0);
        assert_eq!((l0.start, l0.end, l0.next), (0, 1, 3));
        let l1 = idx.line(1);
        assert_eq!((l1.start, l1.end, l1.next), (3, 4, 5));
        assert_eq!(idx.line_at(0), 1);
        assert_eq!(idx.line_at(3), 2);
        assert_eq!(idx.line_at(4), 2);
        assert_eq!(idx.line_at(6), 3);
        // half-open EOF includes the unterminated last line; empty-at-EOF
        // does not turn that line into a range
        assert_eq!(idx.idx_end(6), 3);
        assert_eq!(idx.lines_in(0..6), 0..3);
        assert_eq!(idx.lines_in(5..6), 2..3);
        assert_eq!(idx.lines_in(6..6), 2..2);

        let idx = LineIndex::new("a\r\nb\nc\n");
        assert_eq!(idx.count(), 4);
        assert_eq!(idx.idx_at(7), 3);
        assert_eq!(idx.idx_end(7), 3);
        assert_eq!(idx.lines_in(0..7), 0..3);
        assert_eq!(idx.lines_in(7..7), 3..3);
    }
}
