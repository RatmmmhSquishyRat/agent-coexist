//! The six read-only commands (spec §7.2 / §8.2): `check`, `list`,
//! `lookfor-comment`, `read-brief`, `read-notification`, `check-state`.
//!
//! Shared plumbing lives here so every query command treats the §10.2
//! read path and the §8.3 failure forms identically:
//!
//! - [`load_file`]: one shared-locked read plus parse; a missing path, a
//!   busy lock and read/parse refusals come back as data, OS-level
//!   conditions as `Fault::Io` (mapped by the CLI per §8.3.5).
//! - [`blocking_report`]: the §8.3.2 form — `file` row, `type` row when
//!   the H1 kind survived, diagnostic rows, first line `error`.
//! - [`dir_error`]: the §7.3.3 DIR preconditions.
//! - [`sort_diagnostics`]: §9 补充4 anchor ordering (line ascending,
//!   same line in §9 table order).
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

use std::path::Path;

use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::reason_report;
use crate::report::{Report, Status};
use crate::Time;

/// Outcome of the shared single-file load.
enum Loaded {
    /// Read and parsed; structure diagnostics are on the document.
    Doc(doc::Document),
    /// Path does not exist (§7.3.2: read commands never create).
    Missing,
    /// Lock wait expired — `E-LOCKED` / `E-INUSE`, exit 3 (§10.2, §11).
    Busy {
        id: &'static str,
        params: Vec<(&'static str, String)>,
    },
    /// Read or parse refused the bytes (`E-UTF8` / `E-TOO-LARGE`): there
    /// is no document, only diagnostics.
    Blocking(Vec<Diagnostic>),
}

/// One shared-locked read plus parse (§10.2 read path). A missing path
/// and a busy lock are business outcomes, not errors; OS-level
/// conditions propagate as `Fault::Io`.
fn load_file(path: &Path, now: Time, wait_ms: u64) -> Result<Loaded, Fault> {
    match std::fs::metadata(path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Loaded::Missing),
        Err(e) => return Err(Fault::io("read", path.to_path_buf(), e)),
    }
    let bytes = match fsx::read(path, wait_ms, None) {
        Ok(bytes) => bytes,
        Err(Fault::Reason { id, params }) => return Ok(Loaded::Busy { id, params }),
        Err(Fault::Diagnostics(diags)) => return Ok(Loaded::Blocking(diags)),
        Err(f) => return Err(f),
    };
    match doc::parse(bytes, path, now) {
        Ok(document) => Ok(Loaded::Doc(document)),
        Err(Fault::Diagnostics(diags)) => Ok(Loaded::Blocking(diags)),
        Err(f) => Err(f),
    }
}

/// §9 补充4: anchor line ascending; same line in §9 table order. Stable
/// for equal keys, so original parse order is kept.
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

/// Lexical parent of a display path: `a/b` → `a`, `x` → `.`.
fn parent_display(display: &str) -> String {
    match display.rfind('/') {
        Some(i) => display[..i].to_string(),
        None => ".".to_string(),
    }
}

/// §7.3.3 DIR preconditions for the DIR-taking commands (`list`,
/// `read-notification`, `check-state`). `Ok(None)` = usable;
/// `Ok(Some(report))` = the `E-DIR-MISSING` / `E-NOT-DIR` reason form.
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
                ("path", object.clone()),
                ("cmd", command.to_string()),
                ("parent", parent_display(&object)),
            ],
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Some(reason_report(
            &dctx,
            command,
            &object,
            "E-DIR-MISSING",
            vec![
                ("dir", object.clone()),
                ("parent", parent_display(&object)),
            ],
        ))),
        Err(e) => Err(Fault::io("read", dir.to_path_buf(), e)),
    }
}

/// `E-FILE-MISSING` reason form for the PATH-taking read commands
/// (§7.3.2). `{file}` carries the suffix — it is the file that does not
/// exist; the fix's `{dir}` is its lexical parent.
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
        vec![
            ("file", context::display_file(path)),
            ("dir", parent_display(object)),
        ],
    )
}

/// `E-TYPE` reason form (§9 补充1): `cmd_for_actual` mapping, with the
/// `actual = state` fix replacement available through `{dir}` (the
/// file's directory, same value as the diag context's).
fn type_report(
    dctx: &diag::Context,
    command: &'static str,
    object: &str,
    actual: doc::Kind,
    expected: &str,
) -> Report {
    let cmd_for_actual = match actual {
        doc::Kind::Post => "post",
        doc::Kind::Brief => "brief",
        doc::Kind::Profile => "profile",
        doc::Kind::State => "state",
    };
    reason_report(
        dctx,
        command,
        object,
        "E-TYPE",
        vec![
            ("actual", actual.as_str().to_string()),
            ("expected", expected.to_string()),
            ("cmd", cmd_for_actual.to_string()),
            ("dir", dctx.dir.clone()),
        ],
    )
}
