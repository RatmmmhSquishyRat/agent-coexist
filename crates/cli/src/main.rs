//! `agent-coexist` entry point: argv capture, dispatch, fault mapping and
//! exit codes (spec §7, §8.3, §11; wait wiring per the extension plan).
//!
//! Flow:
//!
//! 1. Build the POSIX-quoted `argv` string (every `args_os()` entry,
//!    program name first) via `diag::Context::argv_from_args`, so the
//!    per-argument quoting exists only in the template layer — it feeds
//!    `{argv}` in §8.3 fix templates.
//! 2. clap parses the grammar (its own stderr + exit 2 for grammar errors).
//! 3. Build the one [`Ctx`] (env entry points `AGENT_COEXIST_NOW` /
//!    `AGENT_COEXIST_LOCK_WAIT_MS` surface here as usage-class faults,
//!    before any file access), with the progress sink for traversing
//!    commands only.
//! 4. [`args::resolve`] validates values and produces the
//!    [`coexist_core::ops::Request`] (base commands) or the wait call.
//! 5. Base commands go through `ops::run`; the three wait commands are
//!    dispatched only here, never from a base arm.
//!
//! stdout is written only by [`render::render`]. Usage failures print
//! exactly one stderr line and exit 2 with stdout untouched. Every other
//! failure is a rendered report: OS-level I/O faults take the §8.3.5
//! reason/fix form; `Reason`/`Diagnostics` faults that escape a command are
//! rendered (never discarded).

use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;

mod args;
mod progress;
mod render;

use coexist_core::diag;
use coexist_core::error::{Diagnostic, Fault};
use coexist_core::ops::{self, context, Ctx};
use coexist_core::report::{Report, Row, Status};
use coexist_core::wait;

fn main() -> ExitCode {
    let argv = diag::Context::argv_from_args(
        &std::env::args_os()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
    );
    let cli = args::Cli::parse();
    let (progress_fn, progress) = progress::make(cli.command.traverses());
    let mut ctx = match Ctx::new(argv, progress_fn) {
        Ok(ctx) => ctx,
        Err(fault) => {
            eprintln!(
                "error: {}",
                args::fault_usage_message(
                    &fault,
                    "AGENT_COEXIST_NOW must be a strict TIME like 2026-09-17T09:12:00Z and \
                     AGENT_COEXIST_LOCK_WAIT_MS a non-negative decimal integer"
                )
            );
            return ExitCode::from(2);
        }
    };

    match args::resolve(cli, &ctx) {
        Ok(args::Invocation::Base { command, kind, request }) => {
            let (object, dctx) = object_and_dctx(&ctx, command, &kind);
            let result = ops::run(&mut ctx, request);
            progress.finish();
            match result {
                Ok(report) => emit(&report, &dctx),
                Err(Fault::Usage(msg)) => usage_stderr(&msg),
                Err(Fault::Io { action, path, source }) => {
                    let report = io_report(command, &object, action, &path, &source, &ctx);
                    emit(&report, &dctx)
                }
                // Base commands never install a cancellation source, so this
                // is an internal inconsistency; report it as an error rather
                // than pretending a result exists.
                Err(Fault::Cancelled) => emit(&cancelled_report(command, &object), &dctx),
                Err(Fault::Reason { id, params }) => {
                    let report = reason_from_fault(&dctx, command, &object, id, params);
                    emit(&report, &dctx)
                }
                Err(Fault::Diagnostics(list)) => {
                    let report = diagnostics_report(command, &object, list);
                    emit(&report, &dctx)
                }
            }
        }
        Ok(args::Invocation::Wait { kind, path, filter }) => {
            // Command and object come from the wait module so the ready
            // line, any fault report and wait's own reports always agree
            // (state object = dir; file objects = suffix-stripped display).
            let (command, object) = wait::command_object(&path, &filter);
            let dctx = match &kind {
                args::DiagKind::File(file) => {
                    context::diag_context_for_file(&ctx, command, file)
                }
                args::DiagKind::Dir(dir) => context::diag_context_for_dir(&ctx, command, dir),
            };
            let cancel = Arc::new(wait::Cancellation::new());
            {
                let cancel = Arc::clone(&cancel);
                // If installation fails, the default interrupt disposition
                // still terminates the process (the shell reports 130) and
                // the wait loop is otherwise unaffected, so this is not
                // fatal.
                let _ = ctrlc::set_handler(move || cancel.cancel());
            }
            let ready = move || {
                // One readiness hint on stderr, best-effort: a failing
                // stderr must not cancel an otherwise valid wait result.
                let _ = writeln!(std::io::stderr(), "waiting  {command}  {object}");
            };
            match wait::run(&path, filter, &cancel, ready) {
                Ok(report) => emit(&report, &dctx),
                Err(Fault::Cancelled) => ExitCode::from(130),
                Err(Fault::Usage(msg)) => usage_stderr(&msg),
                Err(Fault::Io { action, path: io_path, source }) => {
                    let report = io_report(command, &object, action, &io_path, &source, &ctx);
                    emit(&report, &dctx)
                }
                Err(Fault::Reason { id, params }) => {
                    let report = reason_from_fault(&dctx, command, &object, id, params);
                    emit(&report, &dctx)
                }
                Err(Fault::Diagnostics(list)) => {
                    let report = diagnostics_report(command, &object, list);
                    emit(&report, &dctx)
                }
            }
        }
        Err(msg) => {
            progress.finish();
            usage_stderr(&msg)
        }
    }
}

/// First-line object plus diag context for a command, from the validated
/// target: PATH commands render a suffix-stripped display path, DIR
/// commands the display directory (`.` when omitted).
fn object_and_dctx(
    ctx: &Ctx,
    command: &'static str,
    kind: &args::DiagKind,
) -> (String, diag::Context) {
    match kind {
        args::DiagKind::File(path) => (
            context::object_of_path(path),
            context::diag_context_for_file(ctx, command, path),
        ),
        args::DiagKind::Dir(dir) => (
            context::object_of_dir(dir),
            context::diag_context_for_dir(ctx, command, dir),
        ),
    }
}

/// Render a report to stdout and finish with its exit code. stdout is
/// written only here (spec §8.1.1).
fn emit(report: &Report, dctx: &diag::Context) -> ExitCode {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    match render::render(report, dctx, &mut out) {
        Ok(()) => {
            let _ = out.flush();
            ExitCode::from(report.exit)
        }
        // stdout itself failed (e.g. closed pipe): the report cannot be
        // delivered and nothing further may be written to it.
        Err(_) => ExitCode::from(1),
    }
}

/// One stderr line and exit 2, stdout untouched (spec §7.1.7, §11).
fn usage_stderr(msg: &str) -> ExitCode {
    eprintln!("error: {msg}");
    ExitCode::from(2)
}

/// The §8.3.5 report for an OS-level I/O fault: `reason` =
/// `cannot <action> <display file>: <os error>`, `fix` chosen by error
/// nature — path-class errors (NotFound / IsADirectory / NotADirectory /
/// InvalidFilename, or Windows raw 123 / 206) get `fix the path...`, all
/// others (disk full, permission, transient) get the retry form with the
/// POSIX-quoted argv. Exit 1.
fn io_report(
    command: &'static str,
    object: &str,
    action: &'static str,
    path: &Path,
    source: &std::io::Error,
    ctx: &Ctx,
) -> Report {
    let mut report = Report::new(command, object);
    let display = context::display_file(path);
    report.kv("reason", vec![format!("cannot {action} {display}: {source}")]);
    let path_class = matches!(
        source.kind(),
        ErrorKind::NotFound | ErrorKind::IsADirectory | ErrorKind::NotADirectory | ErrorKind::InvalidFilename
    ) || matches!(source.raw_os_error(), Some(123) | Some(206));
    let fix = if path_class {
        "fix the path, then run the corrected command".to_string()
    } else {
        format!("check the file's permissions and free space, then retry: {}", ctx.argv)
    };
    report.kv("fix", vec![fix]);
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// Minimal reason-form report for a `Fault::Reason` that escaped a command:
/// reason/fix through the diag catalog (exit 3 for the retryable lock ids,
/// else 1). If the catalog rendered no message for the id, the raw id is
/// kept visible instead of an empty value.
fn reason_from_fault(
    dctx: &diag::Context,
    command: &'static str,
    object: &str,
    id: &'static str,
    params: Vec<(&'static str, String)>,
) -> Report {
    let mut report = ops::reason_report(dctx, command, object, id, params);
    if let Some(Row::Fields { key, values, .. }) = report.rows.first_mut() {
        if key == "reason" && values.iter().all(|value| value.is_empty()) {
            *values = vec![id.to_string()];
        }
    }
    report
}

/// Report carrying raw diagnostics for a `Fault::Diagnostics` that escaped
/// a command (rendered like any diagnostic rows, first line `error`,
/// exit 1).
fn diagnostics_report(
    command: &'static str,
    object: &str,
    diagnostics: Vec<Diagnostic>,
) -> Report {
    let mut report = Report::new(command, object);
    for diagnostic in diagnostics {
        report.diagnostic(diagnostic);
    }
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// Internal-error report for an unreachable `Fault::Cancelled` in a base
/// command: nothing fake is written to stdout beyond this error form.
fn cancelled_report(command: &'static str, object: &str) -> Report {
    let mut report = Report::new(command, object);
    report.kv("reason", vec!["cancelled".to_string()]);
    report.status = Status::Error;
    report.exit = 1;
    report
}
