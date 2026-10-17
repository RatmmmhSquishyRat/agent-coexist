//! The `post` command (spec §3): create a post file, append comment `#N`,
//! or splice-edit an existing comment (§3.5). Every write goes through
//! `fsx::mutate` + `text::apply`; unrelated bytes are never rewritten.
//!
//! This module also hosts the machinery shared by the four single-file
//! write commands (`post`, `subscribe`, `profile`, `brief`): the
//! `fsx::mutate` value/outcome plumbing, the check-shaped refusal report
//! (§2.7.4, §8.3.2), byte-geometry helpers for splices (§10.3), and the
//! functional-lock recipe (§10.8). `ops/mod.rs` is frozen and no extra
//! module file may be created, so those internals live here behind
//! `pub(super)`.

use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::Time;
use crate::consts;
use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx::{self, Mutation};
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::shared;
use crate::report::{Report, Row, Status};
use crate::text::{self, Splice};
use crate::time;

use super::{PostParams, reason_report};

pub fn run(ctx: &mut Ctx, p: &PostParams) -> Result<Report, Fault> {
    let edit = p.edit.is_some();
    let policy = fsx::WritePolicy {
        wait_ms: ctx.post_wait_ms,
        create: !edit,
        create_parents: !edit,
    };
    // `post` is never subject to the functional lock (spec §7.3, T5).
    let result = fsx::mutate(&p.path, policy, |bytes, created| {
        step(ctx, p, bytes, created)
    });
    finish(ctx, "post", &p.path, None, result)
}

/// One attempt under the exclusive lock: build the initial content when the
/// file was created (or an abandoned zero-byte file was taken over), else
/// parse the existing bytes and compute the splice.
fn step(
    ctx: &mut Ctx,
    p: &PostParams,
    bytes: &[u8],
    created: bool,
) -> Result<(Mutation, Step), Fault> {
    // The exclusive lock is already held. Sampling before the queue could
    // backdate this write behind an intervening successful shared read and
    // hide it from that reader's next notification window.
    let event_time = ctx.write_now();
    if created {
        return create_step(p, &time::format(event_time));
    }
    let doc = match checked_doc(bytes, &p.path, event_time, doc::Kind::Post) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let now = time::format(event_time);
    match p.edit {
        None => append_step(p, &doc, bytes, &now),
        Some(n) => edit_step(p, &doc, bytes, n, &now),
    }
}

/// §3.5.1 create form: `# <stem>.post` + blank + comment #1.
fn create_step(p: &PostParams, now: &str) -> Result<(Mutation, Step), Fault> {
    let stem = require_valid_stem(&p.path)?;
    let prefix = format!("# {stem}.post\n\n");
    let mut content = prefix.clone();
    content.push_str("## #1 ");
    content.push_str(&p.nick);
    content.push_str(" (");
    content.push_str(now);
    content.push_str(")\n\n");
    content.push_str(&fence_block(&p.body));
    let mut done = Done::empty();
    done.kv(
        "comment",
        vec![
            "#1".to_string(),
            format!("L{}", line_of(content.as_bytes(), prefix.len())),
        ],
    );
    add_reference_rows(&mut done, &p.body, (&p.nick, 1), [1]);
    Ok((Mutation::Write(content.into_bytes()), Step::Done(done)))
}

/// §3.5.2 append form: pad to one blank line, then the new section.
fn append_step(
    p: &PostParams,
    doc: &doc::Document,
    bytes: &[u8],
    now: &str,
) -> Result<(Mutation, Step), Fault> {
    let max_seq = doc.post.comments.iter().map(|c| c.seq).max().unwrap_or(0);
    if max_seq == u64::MAX - 1 {
        // §3.3.1: the next number would not be a legal comment header.
        return Err(Fault::reason("E-SEQ-EXHAUSTED")
            .with("max", max_seq.to_string())
            .with("nick", p.nick.clone()));
    }
    let seq = max_seq + 1;
    let gap = text::append_gap(&doc.source);
    let header_off = doc.source.len() + gap.len();
    let mut replacement = String::with_capacity(gap.len() + 64 + p.nick.len() + p.body.len());
    replacement.push_str(gap);
    replacement.push_str("## #");
    replacement.push_str(&seq.to_string());
    replacement.push(' ');
    replacement.push_str(&p.nick);
    replacement.push_str(" (");
    replacement.push_str(now);
    replacement.push_str(")\n\n");
    replacement.push_str(&fence_block(&p.body));
    let splices = vec![Splice {
        range: doc.source.len()..doc.source.len(),
        replacement: replacement.into_bytes(),
    }];
    match guarded_apply(bytes, splices, kind_str(doc))? {
        Applied::Bytes(new_bytes) => {
            let mut done = Done::empty();
            done.kv(
                "comment",
                vec![
                    format!("#{seq}"),
                    format!("L{}", line_of(&new_bytes, header_off)),
                ],
            );
            add_reference_rows(
                &mut done,
                &p.body,
                (&p.nick, seq),
                doc.post.comments.iter().map(|c| c.seq).chain([seq]),
            );
            Ok((Mutation::Write(new_bytes), Step::Done(done)))
        }
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// §3.5.3 edit form: replace the body fence (or insert one for a W-BODY
/// comment) and append/replace `, edited <now>` in the header.
fn edit_step(
    p: &PostParams,
    doc: &doc::Document,
    bytes: &[u8],
    n: u64,
    now: &str,
) -> Result<(Mutation, Step), Fault> {
    let max_seq = doc.post.comments.iter().map(|c| c.seq).max().unwrap_or(0);
    let idx = match doc.post.comments.iter().position(|c| c.seq == n) {
        Some(idx) => idx,
        None => {
            return Err(Fault::reason("E-COMMENT-NOTFOUND")
                .with("n", n.to_string())
                .with("max", max_seq.to_string())
                .with("nick", p.nick.clone())
                .with("path_arg", p.path.as_os_str()));
        }
    };
    let comment = &doc.post.comments[idx];
    if comment.nick != p.nick {
        return Err(Fault::reason("E-EDIT-AUTHOR")
            .with("n", n.to_string())
            .with("author", comment.nick.clone())
            .with("nick", p.nick.clone())
            .with("path_arg", p.path.as_os_str()));
    }
    let section = &doc.sections[comment.section];
    let mut splices = Vec::new();
    // Header: rewrite from the `##` through the `)` closing the time group;
    // anything after it on the line (trailing spaces) stays byte-for-byte.
    let head_text = doc.text(&section.head);
    if let Some(rparen) = strip_term(head_text).rfind(')') {
        let header_end = section.head.start + rparen + 1;
        let mut header = format!(
            "## #{} {} ({}",
            comment.seq,
            comment.nick,
            time::format(comment.created)
        );
        header.push_str(", edited ");
        header.push_str(now);
        header.push(')');
        splices.push(Splice {
            range: section.head.start..header_end,
            replacement: header.into_bytes(),
        });
    }
    // Body fence.
    match &section.body {
        Some(body) => {
            let span = body.span.clone();
            splices.push(Splice {
                range: span,
                replacement: text::fenced(&p.body).into_bytes(),
            });
        }
        None => {
            let (pos, inserted) =
                after_line_and_blank(&doc.source, section.head.start, &fence_block(&p.body));
            splices.push(Splice {
                range: pos..pos,
                replacement: separated(&doc.source, pos, inserted).into_bytes(),
            });
        }
    }
    let line = line_of(doc.source.as_bytes(), section.head.start);
    match guarded_apply(bytes, splices, kind_str(doc))? {
        Applied::Bytes(new_bytes) => {
            let mut done = Done::empty();
            done.kv(
                "comment",
                vec![
                    format!("#{}", comment.seq),
                    format!("L{line}"),
                    "edited".to_string(),
                ],
            );
            add_reference_rows(
                &mut done,
                &p.body,
                (&p.nick, comment.seq),
                doc.post.comments.iter().map(|c| c.seq),
            );
            Ok((Mutation::Write(new_bytes), Step::Done(done)))
        }
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// `mentions` / `reply-to` rows derived from the freshly written body
/// (§3.4.2, §3.5.4, §8.1.14). `existing` is the seqs present after the
/// write and is consumed only when the body contains replies; a missing
/// target still gets written but downgrades to `warn`, recording `own`
/// (the author and number of the comment this write announces) so the
/// report can name the author's own repair as a `fix` row.
fn add_reference_rows(
    done: &mut Done,
    body: &str,
    own: (&str, u64),
    existing: impl IntoIterator<Item = u64>,
) {
    let (mentions, replies) = crate::types::post::references(body);
    if !mentions.is_empty() {
        done.kv("mentions", vec![mentions.join(", ")]);
    }
    if replies.is_empty() {
        return;
    }
    let existing: HashSet<u64> = existing.into_iter().collect();
    for m in replies {
        if existing.contains(&m) {
            done.kv("reply-to", vec![format!("#{m}")]);
        } else {
            done.kv("reply-to", vec![format!("#{m}"), "(not found)".to_string()]);
            done.warn = true;
            done.reply_fix = Some((own.0.to_string(), own.1));
        }
    }
}

// ---------------------------------------------------------------------------
// Shared write-command machinery (see module docs).
// ---------------------------------------------------------------------------

/// Rows plus a warn flag computed inside the `fsx::mutate` closure for a
/// completed step (successful write or no-op outcome); a post write that
/// replied to a missing number also records its own repair.
pub(super) struct Done {
    pub rows: Vec<Row>,
    pub warn: bool,
    /// A reply in the written body named a number no comment in this
    /// post carries: the author's own repair (nick and number of the
    /// comment this report announced), rendered as a `fix` row by
    /// [`finish`].
    pub reply_fix: Option<(String, u64)>,
}

impl Done {
    pub fn empty() -> Done {
        Done {
            rows: Vec::new(),
            warn: false,
            reply_fix: None,
        }
    }

    /// Key/value row with the same layout as `Report::kv`.
    pub fn kv(&mut self, key: &str, values: Vec<String>) {
        self.rows.push(Row::Fields {
            key: key.to_string(),
            values,
            indent: 2,
            width: consts::KEY_WIDTH,
        });
    }
}

/// The value channel of `fsx::mutate` for every write command.
pub(super) enum Step {
    Done(Done),
    /// The target carries blocking diagnostics (spec §1): refuse to write
    /// and emit check-shaped rows (§2.7.4, §8.3.2).
    Refused {
        kind: Option<&'static str>,
        diags: Vec<Diagnostic>,
    },
}

/// Early exits from the shared existing-file gates.
pub(super) enum Early {
    Step(Step),
    Fault(Fault),
}

/// Result of computing a splice set: the new bytes, or a refusal.
pub(super) enum Applied {
    Bytes(Vec<u8>),
    Refused(Step),
}

/// Stashed functional-lock verdict (spec §10.8), collected inside the
/// `fsx::mutate` closure and turned into the `E-FLOCKED` report after it.
pub(super) struct FlockStash {
    holders: Vec<shared::FlockHolder>,
    issues: Vec<(PathBuf, shared::ScanIssue)>,
    unreadable_dirs: Vec<PathBuf>,
    canonicalize_failed: Vec<shared::CanonicalizeFail>,
}

/// Parse existing bytes; byte-level faults (`E-UTF8` / `E-TOO-LARGE`) and
/// blocking diagnostics become check-shaped refusals. The stem is the
/// listed spelling ([`fsx::listed_path`]), so a refusal's `W-NAME` row
/// matches `check`.
fn parse_existing(bytes: &[u8], path: &Path, now: Time) -> Result<doc::Document, Early> {
    match doc::parse(bytes.to_vec(), &fsx::listed_path(path), now) {
        Ok(doc) => {
            if !doc.blocking().is_empty() {
                return Err(Early::Step(Step::Refused {
                    kind: kind_str(&doc),
                    diags: doc.diagnostics.clone(),
                }));
            }
            Ok(doc)
        }
        Err(Fault::Diagnostics(diags)) => Err(Early::Step(Step::Refused { kind: None, diags })),
        Err(fault) => Err(Early::Fault(fault)),
    }
}

/// Parse + blocking gate + `E-TYPE` gate (§7.3) for an existing target.
pub(super) fn checked_doc(
    bytes: &[u8],
    path: &Path,
    now: Time,
    expected: doc::Kind,
) -> Result<doc::Document, Early> {
    let doc = parse_existing(bytes, path, now)?;
    if doc.kind != Some(expected) {
        let actual = doc.kind.unwrap_or(expected);
        return Err(Early::Fault(type_fault(actual, expected, path)));
    }
    Ok(doc)
}

fn kind_str(doc: &doc::Document) -> Option<&'static str> {
    doc.kind.as_ref().map(doc::Kind::as_str)
}

/// Apply splices with the §10.2.2 result-size guard: a prospective result
/// above `MAX_MANAGED_FILE_BYTES` refuses without touching the file.
pub(super) fn guarded_apply(
    bytes: &[u8],
    splices: Vec<Splice>,
    kind: Option<&'static str>,
) -> Result<Applied, Fault> {
    let delta: i64 = splices
        .iter()
        .map(|s| s.replacement.len() as i64 - s.range.len() as i64)
        .sum();
    let result = bytes.len() as i64 + delta;
    if result > consts::MAX_MANAGED_FILE_BYTES as i64 {
        // The file itself is within the limit; only this result is not.
        return Ok(Applied::Refused(Step::Refused {
            kind,
            diags: vec![
                Diagnostic::new("E-TOO-LARGE", 1)
                    .with(
                        "limit",
                        (consts::MAX_MANAGED_FILE_BYTES / (1024 * 1024)).to_string(),
                    )
                    .with("size", bytes.len().to_string())
                    .with("result", result.to_string()),
            ],
        }));
    }
    Ok(Applied::Bytes(text::apply(bytes, splices)?))
}

/// The fix row after a write whose functional-lock check was incomplete:
/// the write already happened, so the recovery is to open exactly what the
/// check could not read (the state file behind each skipped/partial row,
/// each unlisted directory, or this directory when identity fell back to
/// lexical paths) and agree with any holder named there, never to rerun
/// the write.
fn lock_scan_fix(follows: &[shared::Follow]) -> String {
    shared::scan_fix(
        follows,
        diag::FIX_LOCK_SCAN_INCOMPLETE,
        diag::FIX_LOCK_SCAN_INCOMPLETE_MANUAL,
    )
}

/// Read-only pointer at `check` on this file after remaining errors or a
/// brief `--refresh` skip. Check owns the diagnostic text; this never
/// rewrites and never substitutes for the lock-scan fix.
pub(super) fn file_check_fix(path: &Path) -> String {
    match diag::path_operand(path.as_os_str()) {
        Some(op) => diag::FIX_FILE_CHECK.replace("{path_arg}", &op),
        None => diag::FIX_FILE_CHECK_MANUAL.to_string(),
    }
}

/// Run the functional-lock verdict (§10.8) inside the closure of a
/// byte-writing `profile` / `brief` form. The outcome data is always
/// stashed — a later successful write must still surface an incomplete
/// scan (`skipped` / `partial` / unreadable-directory / `canonicalize`
/// rows with a `warn` first line, §10.5.6). Returns `true` when an
/// actual other-party lock means the write must not proceed; the stashed
/// verdict becomes the `E-FLOCKED` report in [`finish`].
pub(super) fn stash_flock(
    stash: &mut Option<FlockStash>,
    ctx: &mut Ctx,
    path: &Path,
    as_nick: Option<&str>,
) -> bool {
    let outcome = shared::functional_lock(ctx, path, as_nick);
    let blocked = !outcome.holders.is_empty();
    *stash = Some(FlockStash {
        holders: outcome.holders,
        issues: outcome.scope.issues,
        unreadable_dirs: outcome.scope.unreadable_dirs,
        canonicalize_failed: outcome.scope.canonicalize_failed,
    });
    blocked
}

/// Exit for a failed functional-lock verdict inside the closure: always
/// `Err(E-FLOCKED)`, never a successful mutation. The refusal therefore
/// runs fsx's plan-failure cleanup (§10.8.3): a file this invocation just
/// `create_new`d is deleted (still empty) together with the directories
/// this invocation created (still empty), while a pre-existing empty file
/// taken over at the deadline survives — initial-content mode is never
/// confused with ownership of the file.
pub(super) fn flock_blocked() -> Result<(Mutation, Step), Fault> {
    Err(Fault::reason("E-FLOCKED"))
}

/// Common report assembly after `fsx::mutate`: success rows (with `created`
/// rows first), check-shaped refusals, and `reason`-form translation of
/// `Fault::Reason` failures (including fsx's `E-FILE-MISSING` / `E-LOCKED`
/// / `E-INUSE` / `E-FLOCKED`).
///
/// A completed write whose functional-lock scan was incomplete keeps the
/// scan visible: `skipped` / `partial` / unreadable-directory /
/// `canonicalize` rows are appended and the report is promoted to `warn`
/// (best-effort semantics — no recognizable holder may write, the unread
/// portion is never hidden). `remaining` rows stay the final group after
/// those scan rows (§2.7.4). Remaining errors get one read-only `check`
/// pointer ([`file_check_fix`]) after any reply-target `fix` and before
/// lock-scan rows; it never replaces the lock-scan fix. A post write that
/// replied to a number no comment carries keeps its `fix` row after the
/// `reply-to` rows.
pub(super) fn finish(
    ctx: &mut Ctx,
    command: &'static str,
    path: &Path,
    stash: Option<FlockStash>,
    result: Result<fsx::WriteResult<Step>, Fault>,
) -> Result<Report, Fault> {
    let object = context::object_of_path(path);
    match result {
        Ok(wr) => match wr.value {
            Step::Done(done) => {
                let mut report = Report::new(command, &object);
                if wr.created {
                    created_rows(&mut report, &wr.created_dirs, path);
                }
                let (body, remaining): (Vec<Row>, Vec<Row>) = done
                    .rows
                    .into_iter()
                    .partition(|row| !matches!(row, Row::Fields { key, .. } if key == "remaining"));
                report.rows.extend(body);
                if done.warn {
                    report.warn();
                }
                if let Some((nick, seq)) = done.reply_fix {
                    // The reply-to rows above end in `(not found)`; the
                    // fix names the writer's own repair while the comment
                    // is still the newest thing on their mind.
                    let dctx = context::diag_context_for_file(ctx, command, path);
                    report.kv(
                        "fix",
                        vec![diag::reply_target_missing_fix(&dctx, &nick, seq)],
                    );
                }
                if !remaining.is_empty() {
                    report.kv("fix", vec![file_check_fix(path)]);
                }
                if let Some(stash) = &stash {
                    let follows = append_scan_rows(&mut report, stash, path);
                    if !follows.is_empty() {
                        report.kv("fix", vec![lock_scan_fix(&follows)]);
                        report.warn();
                    }
                }
                report.rows.extend(remaining);
                Ok(report)
            }
            Step::Refused { kind, diags } => Ok(refuse_report(command, &object, path, kind, diags)),
        },
        Err(Fault::Reason { id, params }) => {
            if id == "E-FLOCKED"
                && let Some(stash) = stash
            {
                return Ok(flocked_report(ctx, command, &object, path, stash));
            }
            let params = if id == "E-FILE-MISSING" {
                file_missing_params(path)
            } else {
                params
            };
            let dctx = context::diag_context_for_file(ctx, command, path);
            Ok(reason_report(&dctx, command, &object, id, params))
        }
        Err(Fault::Diagnostics(diags)) => Ok(refuse_report(command, &object, path, None, diags)),
        Err(other) => Err(other),
    }
}

/// `E-FLOCKED` report (§8.3.1): `reason` / `fix` from the latest-until
/// holder, then `also-locked` (until descending, nick ascending — the
/// order `shared::functional_lock` already returns), then the skipped /
/// partial / unreadable-directory rows for state the verdict could not
/// read (§10.5.6), then canonicalize-failure rows.
fn flocked_report(
    ctx: &mut Ctx,
    command: &'static str,
    object: &str,
    path: &Path,
    stash: FlockStash,
) -> Report {
    let primary = &stash.holders[0];
    let dctx = context::diag_context_for_file(ctx, command, path);
    let mut report = reason_report(
        &dctx,
        command,
        object,
        "E-FLOCKED",
        vec![
            ("dir", primary.dir_display.clone().into()),
            ("holder", primary.nick.clone().into()),
            ("until", time::format(primary.until).into()),
        ],
    );
    for holder in &stash.holders[1..] {
        report.kv(
            "also-locked",
            vec![
                holder.nick.clone(),
                format!("until {}", time::format(holder.until)),
            ],
        );
    }
    append_scan_rows(&mut report, &stash, path);
    report
}

/// Append the stashed scan-incompleteness rows (§10.5.6):
/// `skipped` / `partial` for every state file the verdict could not
/// incorporate and `skipped <dir> unreadable` for directories that could
/// not be listed ([`shared::scan_rows`]), then `canonicalize <dir> failed`
/// rows with the bounded OS reason. Returns what a reader must open to
/// see what was not read (empty when nothing was appended), so a
/// successful report can promote itself to `warn` and name those files.
fn append_scan_rows(report: &mut Report, stash: &FlockStash, path: &Path) -> Vec<shared::Follow> {
    let mut follows = Vec::new();
    for row in shared::scan_rows(&stash.issues, &stash.unreadable_dirs) {
        report.kv(row.key, row.values);
        follows.push(row.follow);
    }
    for fail in &stash.canonicalize_failed {
        report.kv(
            "canonicalize",
            shared::canonicalize_values(&fail.dir, &fail.reason),
        );
    }
    if !stash.canonicalize_failed.is_empty() {
        // Identity fell back to lexical paths: the whole chain is re-checked
        // from the written file's parent directory, not from the working
        // directory the scan happened to run in.
        follows.push(shared::Follow::WrittenFileDir(
            context::parent_dir(path).to_path_buf(),
        ));
    }
    follows
}

/// `created  <dir>/` rows (fsx reports them outer→inner), then
/// `created  <file>` (§7.3.2).
fn created_rows(report: &mut Report, created_dirs: &[PathBuf], file: &Path) {
    for dir in created_dirs {
        report.kv(
            "created",
            vec![format!("{}/", names::fold(&fsx::walk::display_path(dir)))],
        );
    }
    report.kv("created", vec![context::display_file(file)]);
}

/// Check-shaped refusal (§8.3.2): `file`, `type` when determinable, then
/// every diagnostic in §9 supplement 4 order (anchor line, catalog order).
pub(super) fn refuse_report(
    command: &'static str,
    object: &str,
    file: &Path,
    kind: Option<&'static str>,
    mut diags: Vec<Diagnostic>,
) -> Report {
    let mut report = Report::new(command, object);
    report.kv("file", vec![context::display_file(file)]);
    if let Some(kind) = kind {
        report.kv("type", vec![kind.to_string()]);
    }
    diags.sort_by_key(|d| (d.line, diag::order(d.id)));
    let refused = diag::write_refusal(&diags);
    for d in diags {
        report.diagnostic(d);
    }
    if let Some(text) = refused {
        report.kv("refused", vec![text]);
    }
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// `E-FILE-MISSING` params for any PATH command (write forms here, read
/// commands in `query`). Called only after the file was found missing, so
/// the one extra `metadata` never touches a successful run. When the
/// PATH without its managed suffix names an existing directory (`.`,
/// `docs`, `docs/`), the reader most likely passed a directory where one
/// collaboration file is required: `given_dir` selects the catalog variant
/// that says so, and `dir_arg` lists that directory instead of its parent.
/// Suffix resolution is unchanged; non-UTF-8 paths keep the base text.
/// When the file's own directory is missing too, `dir_arg` is the nearest
/// existing lexical ancestor (`missing_dir` selects that variant), so the
/// listing recipe never names a directory that does not exist.
pub(super) fn file_missing_params(path: &Path) -> Vec<(&'static str, OsString)> {
    let given_dir = path
        .to_str()
        .and_then(|p| p.strip_suffix(consts::SUFFIX))
        .filter(|p| !p.is_empty())
        .map(Path::new)
        .filter(|p| p.is_dir());
    if let Some(dir) = given_dir {
        return vec![
            ("file", context::display_file(path).into()),
            ("given_dir", context::object_of_dir(dir).into()),
            ("dir_arg", dir.as_os_str().to_os_string()),
        ];
    }
    let (dir_arg, missing_dir) = context::nearest_existing_ancestor(path);
    if missing_dir {
        vec![
            ("file", context::display_file(path).into()),
            ("missing_dir", "1".into()),
            ("dir_arg", dir_arg.as_os_str().to_os_string()),
        ]
    } else {
        vec![
            ("file", context::display_file(path).into()),
            ("dir_arg", dir_arg.as_os_str().to_os_string()),
        ]
    }
}

fn type_fault(actual: doc::Kind, expected: doc::Kind, path: &Path) -> Fault {
    Fault::reason("E-TYPE")
        .with("actual", actual.as_str())
        .with("expected", expected.as_str())
        .with("dir_arg", context::parent_dir(path).as_os_str())
}

/// `unchanged  <file>` rows for the no-write forms.
pub(super) fn unchanged_done(path: &Path) -> Done {
    let mut done = Done::empty();
    done.kv("unchanged", vec![context::display_file(path)]);
    done
}

// ---------------------------------------------------------------------------
// Byte geometry (spec §10.3).
// ---------------------------------------------------------------------------

/// End offset (exclusive) of the line that contains `start`, including its
/// line terminator (`\n` or `\r\n`) when present.
pub(super) fn line_end(src: &str, start: usize) -> usize {
    match src[start..].find('\n') {
        Some(i) => start + i + 1,
        None => src.len(),
    }
}

/// 1-based line number of the byte at `off`. Every line terminator
/// (`\n` or `\r\n`) contains exactly one `\n`; lone `\r` is not one.
fn line_of(src: &[u8], off: usize) -> usize {
    1 + src[..off].iter().filter(|&&b| b == b'\n').count()
}

/// Line text without its terminator.
pub(super) fn strip_term(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .unwrap_or(line)
}

/// Blank line per spec §1 (empty, or only spaces and tabs).
pub(super) fn is_blank(line: &str) -> bool {
    line.bytes().all(|b| b == b' ' || b == b'\t')
}

/// Insertion point and bytes for "after the line starting at `line_start`
/// and its immediately following blank line, adding the blank when absent"
/// (the §3.5.3 / §5.3.2 geometry). `block` must end with a terminator.
///
/// An unterminated whitespace-only following line is not an insertion
/// boundary: its original bytes stay put, a terminator is added, then the
/// block. A following isolated CR is not blank, so this never concatenates
/// new syntax onto a CR-tainted line.
pub(super) fn after_line_and_blank(src: &str, line_start: usize, block: &str) -> (usize, String) {
    let end = line_end(src, line_start);
    let terminated = end > line_start && src.as_bytes()[end - 1] == b'\n';
    if !terminated {
        // Last line without a terminator: terminate it, add the blank, then
        // the block. A terminal isolated CR stays line content via append_gap.
        let pad = if end > 0 && src.as_bytes()[end - 1] == b'\r' {
            text::append_gap(&src[..end])
        } else {
            "\n\n"
        };
        return (end, format!("{pad}{block}"));
    }
    if end < src.len() {
        let next_end = line_end(src, end);
        if is_blank(strip_term(&src[end..next_end])) {
            if next_end > end && src.as_bytes()[next_end - 1] == b'\n' {
                return (next_end, block.to_string());
            }
            return (next_end, format!("\n{block}"));
        }
    }
    (end, format!("\n{block}"))
}

/// `block` (ending with a terminator) as inserted at `pos`, plus one blank
/// line when the line already at `pos` has content. An inserted fence or
/// property line then never touches the heading, fence or text after it,
/// matching the layout of files the CLI creates. Existing bytes are kept.
pub(super) fn separated(src: &str, pos: usize, mut block: String) -> String {
    if pos < src.len() && !is_blank(strip_term(&src[pos..line_end(src, pos)])) {
        block.push('\n');
    }
    block
}

/// Fenced body block ending with exactly one terminator — for insertions
/// and freshly built file content (§2.5.4 via `text::fenced`).
pub(super) fn fence_block(body: &str) -> String {
    let mut s = text::fenced(body);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Whole-section deletion (§10.3.5): `Section.span` runs from the H2 line
/// through the byte before the next H2 (or EOF) with terminators included,
/// which is exactly the deletion range; blank lines before the H2 survive
/// and one blank line is inserted when the surviving neighbours would
/// otherwise touch without reaching EOF.
pub(super) fn section_delete_splice(src: &str, section: &doc::Section) -> Splice {
    let mut replacement = Vec::new();
    if section.span.end < src.len() && !ends_with_blank_line(&src[..section.span.start]) {
        replacement.push(b'\n');
    }
    Splice {
        range: section.span.clone(),
        replacement,
    }
}

fn ends_with_blank_line(prefix: &str) -> bool {
    let b = prefix.as_bytes();
    if b.is_empty() || b[b.len() - 1] != b'\n' {
        return false;
    }
    let mut i = b.len() - 1;
    while i > 0 && b[i - 1] != b'\n' {
        i -= 1;
    }
    is_blank(strip_term(&prefix[i..]))
}

/// `removed-lines  L<a>..L<b>` value for a deleted byte range (1-based
/// lines of the original file).
pub(super) fn removed_lines_label(src: &str, start: usize, end: usize) -> String {
    let last = if end > start { end - 1 } else { end };
    format!(
        "L{}..L{}",
        line_of(src.as_bytes(), start),
        line_of(src.as_bytes(), last)
    )
}

/// Start offset of the anchor line for preamble insertions: the last
/// comment line of the H1's leading comment block, or the H1 itself
/// (§2.6.2, §10.3.2).
pub(super) fn preamble_anchor(doc: &doc::Document) -> usize {
    let h1 = doc.h1.clone().expect("no blocking diagnostics imply an H1");
    let mut anchor = h1.start;
    let mut pos = line_end(&doc.source, h1.start);
    let end = doc.preamble.end.min(doc.source.len());
    while pos < end {
        let line_start = pos;
        let next = line_end(&doc.source, pos);
        let line = strip_term(&doc.source[line_start..next]);
        if crate::types::is_reader_comment(line.as_bytes()) {
            anchor = line_start;
        } else if !is_blank(line) {
            break;
        }
        pos = next;
    }
    anchor
}

/// Creation gate shared by every H1-building form (§2.2.2): the H1's name
/// is the target's file-name stem, so a stem outside the NAME grammar
/// cannot produce valid content. Fails `E-NAME` (param `name`) before any
/// content byte is built; fsx's plan-failure cleanup then removes what this
/// invocation created. Existing files are never judged here — a valid file
/// whose H1 name differs from the stem is check's `W-NAME`, not a new
/// write refusal.
pub(super) fn require_valid_stem(path: &Path) -> Result<String, Fault> {
    let stem = stem_of(path);
    if names::valid_name(&stem) {
        Ok(stem)
    } else {
        Err(Fault::reason("E-NAME").with("name", names::display_text(&stem)))
    }
}

/// Stem of a managed file path (file name without the
/// `.agent-coexist.md` suffix) for H1 creation.
pub(super) fn stem_of(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    match name.strip_suffix(consts::SUFFIX) {
        Some(stem) => stem.to_string(),
        None => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::context::test_ctx;
    use crate::time;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-post-tests")
            .join(format!(
                "{}-{}-{}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn invalid_stem_fails_ename_with_the_actual_name() {
        let target = PathBuf::from("has space.agent-coexist.md");
        match require_valid_stem(&target) {
            Ok(stem) => panic!("invalid stem accepted: {stem}"),
            Err(Fault::Reason { id, params }) => {
                assert_eq!(id, "E-NAME");
                assert_eq!(params, vec![("name", "has space".into())]);
            }
            Err(other) => panic!("unexpected fault: {other:?}"),
        }
        // The raw stem is judged, not a folded or truncated surrogate.
        assert!(require_valid_stem(&PathBuf::from("ok.agent-coexist.md")).is_ok());
    }

    #[test]
    fn create_rejects_invalid_stem_without_leaving_a_file() {
        let dir = scratch("ename");
        let target = dir.join("bad stem.agent-coexist.md");
        let p = PostParams {
            path: target.clone(),
            nick: "me".to_string(),
            body: "hello".to_string(),
            edit: None,
        };
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let report = run(&mut ctx, &p).unwrap();
        assert_eq!(report.status, crate::report::Status::Error);
        assert_eq!(report.exit, 1);
        assert!(!target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn field_keys(report: &Report) -> Vec<&str> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key, .. } => Some(key.as_str()),
                _ => None,
            })
            .collect()
    }

    fn field_lists(rows: &[Row], key: &str) -> Vec<Vec<String>> {
        rows.iter()
            .filter_map(|row| match row {
                Row::Fields { key: k, values, .. } if k == key => Some(values.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn remaining_rows_follow_incomplete_scan_rows() {
        let path = PathBuf::from("notes.agent-coexist.md");
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut done = Done::empty();
        done.kv("description", vec!["set".to_string()]);
        done.kv(
            "remaining",
            vec!["E-ENTRY-HASH".to_string(), "L6".to_string()],
        );
        done.warn = true;
        let stash = FlockStash {
            holders: Vec::new(),
            issues: vec![(
                PathBuf::from("remaining-order"),
                shared::ScanIssue::Blocking("E-H1"),
            )],
            unreadable_dirs: Vec::new(),
            canonicalize_failed: Vec::new(),
        };
        let wr = fsx::WriteResult {
            value: Step::Done(done),
            created: false,
            created_dirs: Vec::new(),
            deleted: false,
        };
        let report = finish(&mut ctx, "brief", &path, Some(stash), Ok(wr)).unwrap();
        assert_eq!(report.status, Status::Warn);
        // the write already happened: the fix opens the state file the lock
        // check could not read, never reruns the write
        assert_eq!(
            field_keys(&report),
            ["description", "fix", "skipped", "fix", "remaining"]
        );
        let fixes = field_lists(&report.rows, "fix");
        assert_eq!(fixes.len(), 2);
        assert!(
            fixes[1][0].ends_with(": agent-coexist check remaining-order/state"),
            "{}",
            fixes[1][0]
        );
    }

    #[test]
    fn skipped_scan_rows_follow_displayed_dir_order() {
        let cwd = std::env::current_dir().expect("cwd available");
        let parent = cwd.parent().expect("cwd has a parent").to_path_buf();
        let path = PathBuf::from("notes.agent-coexist.md");
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut done = Done::empty();
        done.kv("description", vec!["set".to_string()]);
        done.kv(
            "remaining",
            vec!["E-ENTRY-HASH".to_string(), "L6".to_string()],
        );
        let stash = FlockStash {
            holders: Vec::new(),
            // Native parent-before-child; displayed order is '.' then '..'.
            // Folded-equal relatives must both remain as separate rows.
            issues: vec![
                (parent, shared::ScanIssue::Blocking("E-H1")),
                (cwd, shared::ScanIssue::Blocking("E-H1")),
                (
                    PathBuf::from("alias  dir"),
                    shared::ScanIssue::Blocking("E-H1"),
                ),
                (
                    PathBuf::from("alias dir"),
                    shared::ScanIssue::Blocking("E-H1"),
                ),
            ],
            unreadable_dirs: Vec::new(),
            canonicalize_failed: Vec::new(),
        };
        let wr = fsx::WriteResult {
            value: Step::Done(done),
            created: false,
            created_dirs: Vec::new(),
            deleted: false,
        };
        let report = finish(&mut ctx, "profile", &path, Some(stash), Ok(wr)).unwrap();
        let skipped: Vec<&[String]> = report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key, values, .. } if key == "skipped" => Some(values.as_slice()),
                _ => None,
            })
            .collect();
        let dots: Vec<[&str; 2]> = skipped
            .iter()
            .filter_map(|row| match row.first().map(String::as_str) {
                Some(".") | Some("..") => Some([row[0].as_str(), row[1].as_str()]),
                _ => None,
            })
            .collect();
        assert_eq!(dots, [[".", "E-H1"], ["..", "E-H1"]]);
        let aliases: Vec<&&[String]> = skipped
            .iter()
            .filter(|row| row.first().map(String::as_str) == Some("alias dir"))
            .collect();
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0], aliases[1]);
        assert_eq!(field_keys(&report).last().copied(), Some("remaining"));
    }

    #[test]
    fn subscribe_inserts_before_cr_tainted_gt_line() {
        let dir = scratch("preamble-cr");
        let target = dir.join("foo.agent-coexist.md");
        std::fs::write(&target, b"# foo.post\n\n> hello\rworld\n").unwrap();
        let p = crate::ops::SubscribeParams {
            path: target.clone(),
            nick: "alice".to_string(),
            cancel: false,
        };
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let report = crate::ops::subscribe::run(&mut ctx, &p).unwrap();
        assert_ne!(report.status, Status::Error);
        let bytes = std::fs::read(&target).unwrap();
        assert_eq!(
            bytes.as_slice(),
            b"# foo.post\n\n- subscribers: alice\n\n> hello\rworld\n"
        );

        let target = dir.join("noted.agent-coexist.md");
        std::fs::write(&target, b"# noted.post\n> note\n> also\n").unwrap();
        let p = crate::ops::SubscribeParams {
            path: target.clone(),
            nick: "alice".to_string(),
            cancel: false,
        };
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        crate::ops::subscribe::run(&mut ctx, &p).unwrap();
        let bytes = std::fs::read(&target).unwrap();
        assert_eq!(
            bytes.as_slice(),
            b"# noted.post\n> note\n> also\n\n- subscribers: alice\n"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn append_does_not_activate_terminal_cr_comment_header() {
        let dir = scratch("append-cr-h2");
        let target = dir.join("foo.agent-coexist.md");
        // R3 witness: opaque `## #2 …\r` is not a comment; appending LF
        // against that CR used to form CRLF and mint a duplicate #2.
        std::fs::write(
            &target,
            b"# foo.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```md\nhi\n```\n## #2 bob (2026-09-25T11:00:00Z)\r",
        )
        .unwrap();
        let p = PostParams {
            path: target.clone(),
            nick: "bob".to_string(),
            body: "x".to_string(),
            edit: None,
        };
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = run(&mut ctx, &p).unwrap();
        assert_ne!(report.status, Status::Error);
        let bytes = std::fs::read(&target).unwrap();
        assert_eq!(
            bytes.as_slice(),
            b"# foo.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```md\nhi\n```\n## #2 bob (2026-09-25T11:00:00Z)\r \n\n## #2 bob (2026-09-25T12:00:00Z)\n\n```md\nx\n```\n",
        );
        let parsed = doc::parse(bytes, &target, now).unwrap();
        assert_eq!(
            parsed
                .post
                .comments
                .iter()
                .map(|c| (c.seq, c.nick.as_str(), time::format(c.created)))
                .collect::<Vec<_>>(),
            vec![
                (1, "alice", "2026-09-25T10:00:00Z".to_string()),
                (2, "bob", "2026-09-25T12:00:00Z".to_string()),
            ]
        );
        assert!(parsed.diagnostics.iter().all(|d| d.id != "E-SEQ-DUP"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn apply_after(src: &str, line_start: usize, block: &str) -> String {
        let (pos, inserted) = after_line_and_blank(src, line_start, block);
        format!("{}{}{}", &src[..pos], inserted, &src[pos..])
    }

    #[test]
    fn after_line_and_blank_does_not_glue_onto_unterminated_whitespace() {
        let block = "- subscribers: alice\n";
        assert_eq!(
            apply_after("# foo.post\n ", 0, block),
            "# foo.post\n \n- subscribers: alice\n"
        );
        let profile = "# a.agent-profile\n\n## role\n    ";
        let head = profile.find("## role").expect("role heading");
        assert_eq!(
            apply_after(profile, head, &fence_block("x")),
            "# a.agent-profile\n\n## role\n    \n```md\nx\n```\n"
        );
        assert_eq!(
            apply_after("# foo.post\n\n", 0, block),
            "# foo.post\n\n- subscribers: alice\n"
        );
        // following isolated CR is not blank; original CR bytes survive
        assert_eq!(
            apply_after("# foo.post\n\r", 0, block),
            "# foo.post\n\n- subscribers: alice\n\r"
        );
    }

    #[test]
    fn subscribe_keeps_unterminated_whitespace_and_effective_nick() {
        let dir = scratch("eof-blank-sub");
        let target = dir.join("foo.agent-coexist.md");
        std::fs::write(&target, b"# foo.post\n ").unwrap();
        let p = crate::ops::SubscribeParams {
            path: target.clone(),
            nick: "alice".to_string(),
            cancel: false,
        };
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = crate::ops::subscribe::run(&mut ctx, &p).unwrap();
        assert_ne!(report.status, Status::Error);
        let bytes = std::fs::read(&target).unwrap();
        assert_eq!(bytes.as_slice(), b"# foo.post\n \n- subscribers: alice\n");
        let parsed = doc::parse(bytes, &target, now).unwrap();
        assert_eq!(parsed.post.subscribers, vec!["alice".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_keeps_unterminated_whitespace_and_valid_fence() {
        let dir = scratch("eof-blank-profile");
        let target = dir.join("a.agent-coexist.md");
        std::fs::write(&target, b"# a.agent-profile\n\n## role\n    ").unwrap();
        let p = crate::ops::ProfileParams {
            path: target.clone(),
            as_nick: None,
            section: Some("role".to_string()),
            action: crate::ops::ProfileAction::SetBody("x".to_string()),
        };
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = crate::ops::profile::run(&mut ctx, &p).unwrap();
        assert_ne!(report.status, Status::Error);
        let bytes = std::fs::read(&target).unwrap();
        assert_eq!(
            bytes.as_slice(),
            b"# a.agent-profile\n\n## role\n    \n```md\nx\n```\n"
        );
        let parsed = doc::parse(bytes, &target, now).unwrap();
        assert!(parsed.diagnostics.iter().all(|d| d.id != "E-FENCE"));
        let role = parsed
            .profile
            .iter()
            .find(|s| s.name == "role")
            .expect("role section");
        assert!(parsed.sections[role.section].body.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_reference_rows_keep_first_reply_order_and_missing_targets() {
        let mut done = Done::empty();
        add_reference_rows(&mut done, "@carol @#5 @#1 @#5 @#9", ("bob", 2), [1, 5]);
        assert_eq!(
            field_lists(&done.rows, "mentions"),
            vec![vec!["carol".to_string()]]
        );
        assert_eq!(
            field_lists(&done.rows, "reply-to"),
            vec![
                vec!["#5".to_string()],
                vec!["#1".to_string()],
                vec!["#9".to_string(), "(not found)".to_string()],
            ]
        );
        assert!(done.warn);
    }

    #[test]
    fn append_reply_includes_new_sequence_and_warns_on_missing() {
        let dir = scratch("reply-seq");
        let target = dir.join("foo.agent-coexist.md");
        std::fs::write(
            &target,
            b"# foo.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```md\nhi\n```\n",
        )
        .unwrap();
        let p = PostParams {
            path: target.clone(),
            nick: "bob".to_string(),
            body: "@#1 @#2 @#9 @#2".to_string(),
            edit: None,
        };
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = run(&mut ctx, &p).unwrap();
        assert_eq!(report.status, Status::Warn);
        assert_eq!(
            field_lists(&report.rows, "reply-to"),
            vec![
                vec!["#1".to_string()],
                vec!["#2".to_string()],
                vec!["#9".to_string(), "(not found)".to_string()],
            ]
        );
        let bytes = std::fs::read(&target).unwrap();
        let parsed = doc::parse(bytes, &target, now).unwrap();
        assert_eq!(
            parsed
                .post
                .comments
                .iter()
                .map(|c| c.seq)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn edit_comment_replaces_body_and_updates_header_in_place() {
        let dir = scratch("edit-comment-in-place");
        let target = dir.join("thread.agent-coexist.md");
        std::fs::write(
            &target,
            b"# thread.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```md\noriginal body\n```\n\n## #2 bob (2026-09-25T11:00:00Z)\n\n```md\nreply from bob\n```\n",
        )
        .unwrap();

        let p = PostParams {
            path: target.clone(),
            nick: "alice".to_string(),
            body: "revised body mentioning @bob and @#2".to_string(),
            edit: Some(1),
        };
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = run(&mut ctx, &p).unwrap();
        assert_ne!(report.status, Status::Error);

        assert_eq!(
            field_lists(&report.rows, "comment"),
            vec![vec![
                "#1".to_string(),
                "L3".to_string(),
                "edited".to_string()
            ]]
        );
        assert_eq!(
            field_lists(&report.rows, "mentions"),
            vec![vec!["bob".to_string()]]
        );
        assert_eq!(
            field_lists(&report.rows, "reply-to"),
            vec![vec!["#2".to_string()]]
        );

        let bytes = std::fs::read(&target).unwrap();
        let parsed = doc::parse(bytes, &target, now).unwrap();
        assert_eq!(parsed.post.comments.len(), 2);
        let c1 = &parsed.post.comments[0];
        assert_eq!(c1.seq, 1);
        assert_eq!(c1.nick, "alice");
        assert_eq!(time::format(c1.created), "2026-09-25T10:00:00Z");
        assert_eq!(
            c1.edited.map(time::format),
            Some("2026-09-25T12:00:00Z".to_string())
        );

        let body1 = parsed.sections[c1.section].body.as_ref().unwrap();
        assert_eq!(
            &parsed.source[body1.content.clone()],
            "revised body mentioning @bob and @#2"
        );

        let c2 = &parsed.post.comments[1];
        assert_eq!(c2.seq, 2);
        assert_eq!(c2.nick, "bob");
        assert_eq!(c2.edited, None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn edit_comment_without_prior_body_inserts_fence() {
        let dir = scratch("edit-empty-body");
        let target = dir.join("thread.agent-coexist.md");
        std::fs::write(
            &target,
            b"# thread.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n",
        )
        .unwrap();

        let p = PostParams {
            path: target.clone(),
            nick: "alice".to_string(),
            body: "newly added body".to_string(),
            edit: Some(1),
        };
        let now = time::parse("2026-09-25T13:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = run(&mut ctx, &p).unwrap();
        assert_ne!(report.status, Status::Error);

        let bytes = std::fs::read(&target).unwrap();
        let parsed = doc::parse(bytes, &target, now).unwrap();
        assert_eq!(parsed.post.comments.len(), 1);
        let c1 = &parsed.post.comments[0];
        assert_eq!(
            c1.edited.map(time::format),
            Some("2026-09-25T13:00:00Z".to_string())
        );
        let body1 = parsed.sections[c1.section].body.as_ref().unwrap();
        assert_eq!(&parsed.source[body1.content.clone()], "newly added body");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn edit_comment_replaces_prior_edit_header_and_body() {
        let dir = scratch("edit-reedit");
        let target = dir.join("thread.agent-coexist.md");
        std::fs::write(
            &target,
            b"# thread.post\n\n## #1 alice (2026-09-25T10:00:00Z, edited 2026-09-25T11:00:00Z)\n\n```md\nfirst revised body\n```\n",
        )
        .unwrap();

        let p = PostParams {
            path: target.clone(),
            nick: "alice".to_string(),
            body: "second revised body".to_string(),
            edit: Some(1),
        };
        let now = time::parse("2026-09-25T14:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = run(&mut ctx, &p).unwrap();
        assert_ne!(report.status, Status::Error);

        let bytes = std::fs::read(&target).unwrap();
        let parsed = doc::parse(bytes, &target, now).unwrap();
        assert_eq!(parsed.post.comments.len(), 1);
        let c1 = &parsed.post.comments[0];
        assert_eq!(c1.seq, 1);
        assert_eq!(c1.nick, "alice");
        assert_eq!(time::format(c1.created), "2026-09-25T10:00:00Z");
        assert_eq!(
            c1.edited.map(time::format),
            Some("2026-09-25T14:00:00Z".to_string())
        );

        let head_line = &parsed.source[parsed.sections[c1.section].head.clone()];
        assert_eq!(
            head_line.trim_end(),
            "## #1 alice (2026-09-25T10:00:00Z, edited 2026-09-25T14:00:00Z)"
        );

        let body1 = parsed.sections[c1.section].body.as_ref().unwrap();
        assert_eq!(&parsed.source[body1.content.clone()], "second revised body");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn edit_rejects_unowned_comment_and_reports_edit_author() {
        let dir = scratch("edit-unowned");
        let target = dir.join("thread.agent-coexist.md");
        std::fs::write(
            &target,
            b"# thread.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```md\nalice's body\n```\n",
        )
        .unwrap();

        let p = PostParams {
            path: target.clone(),
            nick: "bob".to_string(),
            body: "bob trying to edit alice".to_string(),
            edit: Some(1),
        };
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = run(&mut ctx, &p).unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.command, "post");
        let reasons = field_lists(&report.rows, "reason");
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0][0].contains("comment #1 was written by alice, not bob"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn edit_rejects_nonexistent_comment_and_reports_comment_notfound() {
        let dir = scratch("edit-notfound");
        let target = dir.join("thread.agent-coexist.md");
        std::fs::write(
            &target,
            b"# thread.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```md\nbody\n```\n",
        )
        .unwrap();

        let p = PostParams {
            path: target.clone(),
            nick: "alice".to_string(),
            body: "editing comment 42".to_string(),
            edit: Some(42),
        };
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut ctx = test_ctx(now);
        let report = run(&mut ctx, &p).unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.command, "post");
        let reasons = field_lists(&report.rows, "reason");
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0][0].contains("comment #42 does not exist"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
