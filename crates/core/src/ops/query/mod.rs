//! The six read-only commands (spec §7.2 / §8.2): `check`, `list`,
//! `lookfor-comment`, `read-brief`, `read-notification`, `check-state`.
//!
//! Shared plumbing lives here so every query command treats the §10.2
//! read path and the §8.3 failure forms identically:
//!
//! - [`load_file`]: one shared-locked read plus parse; a missing path, a
//!   busy lock, read/parse refusals and OS-level conditions all come
//!   back as `LoadError` data, the OS case as `Fault::Io` (mapped by
//!   the CLI per §8.3.5).
//! - [`blocking_report`]: the §8.3.2 form — `file` row, `type` row when
//!   the H1 kind survived, diagnostic rows, first line `error`.
//! - [`dir_error`]: the §7.3.3 DIR preconditions.
//! - [`sort_diagnostics`]: §9 supplement 4 anchor ordering (line
//!   ascending, same line in §9 table order).
//!
//! Command row logic lives in the per-command files below.

mod check;
mod check_state;
mod list;
mod lookfor;
mod read_brief;
mod read_notification;

pub use check::check;
pub use check_state::check_state;
pub use list::list;
pub use lookfor::lookfor_comment;
pub use read_brief::read_brief;
pub use read_notification::read_notification;

use std::ffi::OsString;
use std::path::Path;

use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::reason_report;
use crate::ops::shared;
use crate::report::{Report, Status};
use crate::Time;

/// `include` hint rows when `--include` was given and no file survived the
/// filters (`zero_includes`, one row per supplied glob), with or without
/// `--no-ignore`: a glob that is not relative to DIR matches nothing either
/// way. The walk does not record which filter removed the files, so the
/// text states the empty final result (`empty`, worded for the command's
/// filtered scope) and names only the causes this invocation makes
/// possible: a glob not relative to DIR, any `--exclude` that was given,
/// and ignore rules only when a source was in play, so `--no-ignore` is
/// never recommended when it is already given. The glob column is
/// human-display only (`names::display_text`); matching identity stays the
/// original admitted string.
fn include_hints(
    report: &mut Report,
    zero_includes: &[String],
    walk: &fsx::walk::Options,
    ignore_active: bool,
    empty: &str,
) -> bool {
    if zero_includes.is_empty() {
        return false;
    }
    let mut text = String::from(empty);
    if !walk.exclude.is_empty() {
        text.push_str(INCLUDE_HINT_EXCLUDE);
    }
    if ignore_active {
        text.push_str(INCLUDE_HINT_IGNORED);
    }
    for glob in zero_includes {
        report.record(
            "include",
            vec![names::display_text(glob), text.clone()],
            2,
            crate::consts::KEY_WIDTH,
        );
    }
    true
}

/// `canonicalize  <dir>  failed  <reason>` rows (§10.5.2), then a `fix`
/// that names path existence and ancestor search permission before retrying
/// this command. Not counted in `total`. Returns whether any row was added.
fn canonicalize_notes(
    report: &mut Report,
    failed: &[shared::CanonicalizeFail],
    width: usize,
) -> bool {
    if failed.is_empty() {
        return false;
    }
    for fail in failed {
        report.record(
            "canonicalize",
            shared::canonicalize_values(&fail.dir, &fail.reason),
            2,
            width,
        );
    }
    report.kv("fix", vec![diag::FIX_CANONICALIZE.to_string()]);
    true
}

/// `include` hint of `list`, whose every file comes from the filtered
/// walk: what happened, and the first cause to check.
const INCLUDE_HINT_EMPTY: &str =
    "no file is left after the filters; check that this glob is relative to DIR";
/// `include` hint of `check-state` and `read-notification`: the filters
/// apply only to the walk under DIR, while the state files of DIR and
/// its ancestors are always read (the notification ancestor pass also
/// picks up DIR's own state file when the filtered walk skipped it), so
/// rows from them may stand next to this hint.
const INCLUDE_HINT_EMPTY_STATE: &str = "no file under DIR is left after the filters (the state files of DIR and its ancestors are read regardless); check that this glob is relative to DIR";
/// Appended when `--exclude` was given: exclude wins over include.
const INCLUDE_HINT_EXCLUDE: &str = ", and that --exclude does not remove the files it matches";
/// Appended when an ignore source was in play: `--include` cannot reopen a
/// directory those rules pruned.
const INCLUDE_HINT_IGNORED: &str =
    "; if the files are in a directory that ignore rules skip, --include cannot reach them: add --no-ignore";

/// Failure half of the shared single-file load: every outcome that
/// yields no document, business results and faults alike.
enum LoadError {
    /// Path does not exist (§7.3.2: read commands never create).
    Missing,
    /// Lock wait expired — `E-LOCKED` / `E-INUSE`, exit 3 (§10.2, §11).
    Busy {
        id: &'static str,
        params: Vec<(&'static str, OsString)>,
    },
    /// Read or parse refused the bytes (`E-UTF8` / `E-TOO-LARGE`): there
    /// is no document, only diagnostics.
    Blocking(Vec<Diagnostic>),
    /// A fault to propagate (`Fault::Io` for OS-level conditions,
    /// §8.3.5).
    Fault(Fault),
}

/// One shared-locked read plus parse (§10.2 read path). Success is the
/// parsed document; a missing path, a busy lock and read/parse
/// refusals are business outcomes folded into [`LoadError`], while
/// OS-level conditions stay `Fault::Io` (mapped by the CLI per §8.3.5).
/// The document's stem is the listed spelling ([`fsx::listed_path`]);
/// report rows keep `path` as typed.
fn load_file(path: &Path, now: Time, wait_ms: u64) -> Result<doc::Document, LoadError> {
    match std::fs::metadata(path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(LoadError::Missing),
        Err(e) => return Err(LoadError::Fault(Fault::io("read", path.to_path_buf(), e))),
    }
    let bytes = match fsx::read(path, wait_ms, None) {
        Ok(bytes) => bytes,
        Err(Fault::Reason { id, params }) => return Err(LoadError::Busy { id, params }),
        Err(Fault::Diagnostics(diags)) => return Err(LoadError::Blocking(diags)),
        Err(f) => return Err(LoadError::Fault(f)),
    };
    match doc::parse(bytes, &fsx::listed_path(path), now) {
        Ok(document) => Ok(document),
        Err(Fault::Diagnostics(diags)) => Err(LoadError::Blocking(diags)),
        Err(f) => Err(LoadError::Fault(f)),
    }
}

/// §9 supplement 4: anchor line ascending; same line in §9 table order.
/// Stable for equal keys, so original parse order is kept.
fn sort_diagnostics(mut diags: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diags.sort_by_key(|d| (d.line, diag::order(d.id)));
    diags
}

/// The §8.3.2 failure form: `file` row, `type` row when the kind is
/// determinable, all diagnostics in anchor order, first line `error`,
/// exit 1.
fn blocking_report(
    command: &'static str,
    object: &str,
    file_display: &str,
    kind: Option<doc::Kind>,
    diagnostics: Vec<Diagnostic>,
) -> Report {
    let mut report = Report::new(command, object);
    report.kv("file", vec![file_display.to_string()]);
    if let Some(kind) = kind {
        report.kv("type", vec![kind.as_str().to_string()]);
    }
    for diagnostic in sort_diagnostics(diagnostics) {
        report.diagnostic(diagnostic);
    }
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// §8.1.8 excerpt of a section's first fenced body, `None` when the
/// body is missing or folds to nothing (`W-BODY` and blank bodies: the
/// excerpt column is simply omitted, §8.2 check).
fn body_excerpt(document: &doc::Document, section: usize) -> Option<String> {
    let excerpt = names::excerpt(document.body(section)?);
    if excerpt.is_empty() {
        None
    } else {
        Some(excerpt)
    }
}

/// §7.3.3 DIR preconditions for the DIR-taking commands (`list`,
/// `read-notification`, `check-state`). `Ok(None)` = usable;
/// `Ok(Some(report))` = the `E-DIR-MISSING` / `E-NOT-DIR` reason form.
/// `E-DIR-MISSING` `parent_arg` is the nearest existing lexical ancestor
/// (same walk as `file_missing_params`); `missing_dir` is set when that
/// ancestor is not the immediate parent, so the catalog can say so.
/// `E-NOT-DIR` still names the immediate parent. State and wait keep
/// their own DIR checks.
fn dir_error(ctx: &Ctx, command: &'static str, dir: &Path) -> Result<Option<Report>, Fault> {
    let object = context::object_of_dir(dir);
    let dctx = context::diag_context_for_dir(ctx, command, dir);
    match std::fs::metadata(dir) {
        Ok(meta) if meta.is_dir() => Ok(None),
        Ok(_) => Ok(Some(reason_report(
            &dctx,
            command,
            &object,
            "E-NOT-DIR",
            vec![
                ("path", object.clone().into()),
                ("cmd", command.into()),
                (
                    "parent_arg",
                    context::parent_dir(dir).as_os_str().to_os_string(),
                ),
            ],
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let (existing, missing_parent) = context::nearest_existing_ancestor(dir);
            let parent_arg = existing.as_os_str().to_os_string();
            let params = if missing_parent {
                vec![
                    ("dir", object.clone().into()),
                    ("parent_arg", parent_arg),
                    ("missing_dir", "1".into()),
                ]
            } else {
                vec![("dir", object.clone().into()), ("parent_arg", parent_arg)]
            };
            Ok(Some(reason_report(
                &dctx,
                command,
                &object,
                "E-DIR-MISSING",
                params,
            )))
        }
        Err(e) => Err(Fault::io("read", dir.to_path_buf(), e)),
    }
}

/// `E-FILE-MISSING` reason form for the PATH-taking read commands
/// (§7.3.2). `{file}` carries the suffix — it is the file that does not
/// exist; params (including the directory-given variant) are shared with
/// the write forms.
fn file_missing_report(
    dctx: &diag::Context,
    command: &'static str,
    object: &str,
    path: &Path,
) -> Report {
    reason_report(
        dctx,
        command,
        object,
        "E-FILE-MISSING",
        crate::ops::post::file_missing_params(path),
    )
}

/// `E-TYPE` reason form: the fix asks for a path of the `expected` type in
/// the inspected file's directory (`dctx.dir_arg`), never for a command
/// that acts on the actual type.
fn type_report(
    dctx: &diag::Context,
    command: &'static str,
    object: &str,
    actual: doc::Kind,
    expected: &str,
) -> Report {
    reason_report(
        dctx,
        command,
        object,
        "E-TYPE",
        vec![
            ("actual", actual.as_str().into()),
            ("expected", expected.into()),
        ],
    )
}
