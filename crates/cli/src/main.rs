//! `agent-coexist` entry point: argv capture, dispatch, fault mapping and
//! exit codes (spec §7, §8.3, §11; wait wiring per the extension plan).
//!
//! Flow:
//!
//! 1. clap parses the grammar; errors print clap's own multi-line
//!    stderr report (after [`args::grammar_error_message`] corrects the
//!    advice clap cannot give for a hyphen-leading value, and replaces
//!    the unusable rendering of a non-Unicode value with one custom
//!    line), while explicit help/version retain clap's stdout and
//!    exit 0.
//! 2. `guide` is an information command: emit the embedded Markdown page and
//!    return before argv recovery, progress, Ctx, environment validation, or
//!    business resolution. Write/flush failure is exit 1 with no status line.
//! 3. Business commands capture native `args_os()` into the POSIX-quoted
//!    `{argv}` string via `diag::Context::argv_from_args` (program name
//!    first) so the per-argument quoting exists only in the template layer.
//! 4. Build the one [`Ctx`] (env entry points `AGENT_COEXIST_NOW` /
//!    `AGENT_COEXIST_LOCK_WAIT_MS` surface here as usage-class faults,
//!    before any file access), with the progress sink for traversing
//!    commands only.
//! 5. [`args::resolve`] validates business values and produces the
//!    [`coexist_core::ops::Request`] (base commands) or the wait call.
//! 6. Base commands go through `ops::run`; the three wait commands are
//!    dispatched only here, never from a base arm.
//!
//! Business reports write stdout only through [`render::render`]. Explicit
//! help and version use clap's output; guide writes borrowed Markdown.
//! Usage failures exit 2 with stdout untouched: argument-parser failures
//! print the parser's native multi-line stderr report; the CLI's own
//! usage checks print exactly one escaped line. Every other business
//! failure is a rendered report: OS-level
//! I/O faults take the §8.3.5 reason/fix form, ignore/config source faults
//! the same form with a repair matched to the cause, and a write that left
//! its file changed or unconfirmed a form that never repeats the command;
//! `Reason`/`Diagnostics` faults that escape a command are rendered (never
//! discarded).

use std::cell::Cell;
use std::ffi::OsString;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;

mod args;
mod guide;
mod progress;
mod render;

use coexist_core::diag;
use coexist_core::error::{Diagnostic, Fault, SourceProblem, WriteLeft};
use coexist_core::ops::{self, context, Ctx};
use coexist_core::report::{Report, Row, Status};
use coexist_core::wait;

fn main() -> ExitCode {
    let cli = match args::Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) if error.use_stderr() => {
            let words: Vec<OsString> = std::env::args_os().collect();
            // clap's own corrected report, multi-line as clap renders it;
            // its text already ends with the one trailing newline.
            eprint!("{}", args::grammar_error_message(error, &words));
            return ExitCode::from(2);
        }
        Err(error) => error.exit(),
    };
    let command = match cli.command {
        args::TopCommand::Guide { topic } => return guide::emit(topic),
        args::TopCommand::Business(command) => command,
    };

    let argv = diag::Context::argv_from_args(std::env::args_os());
    let (progress_fn, progress) = progress::make(command.traverses());
    let mut ctx = match Ctx::new(argv, progress_fn) {
        Ok(ctx) => ctx,
        Err(fault) => return usage_stderr(&args::usage_message(fault)),
    };

    match args::resolve(command, &ctx) {
        Ok(args::Invocation::Base {
            command,
            kind,
            request,
        }) => {
            let (object, dctx) = object_and_dctx(&ctx, command, &kind);
            let result = ops::run(&mut ctx, request);
            progress.finish();
            match result {
                Ok(report) => emit(&report, &dctx),
                Err(Fault::Usage(msg)) => usage_stderr(&msg),
                Err(Fault::Io {
                    action,
                    path,
                    source,
                }) => {
                    let report = io_report(command, &object, action, &path, &source, &ctx);
                    emit(&report, &dctx)
                }
                Err(Fault::WriteUncertain { path, cause, left }) => emit(
                    &write_uncertain_report(command, &object, &path, &cause, &left),
                    &dctx,
                ),
                Err(Fault::Source {
                    path,
                    problem,
                    cause,
                }) => emit(
                    &source_report(command, &object, &path, problem, &cause, &ctx),
                    &dctx,
                ),
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
            let cancel = Arc::new(wait::Cancellation::new());
            {
                let cancel = Arc::clone(&cancel);
                // If installation fails, the default interrupt disposition
                // still terminates the process (the shell reports 130) and
                // the wait loop is otherwise unaffected, so this is not
                // fatal.
                let _ = ctrlc::set_handler(move || cancel.cancel());
            }
            // Stack flag set by the ready callback itself (`wait::run`
            // calls it exactly once, after the baseline is admitted), so
            // a fault report below can tell a start-time path mistake
            // from a structural path loss the wait had already watched
            // past. The closure only borrows it; no Arc, no allocation.
            let ready_fired = Cell::new(false);
            let ready = || {
                // Recorded before the best-effort hint: the wait is
                // ready even when stderr cannot be written.
                ready_fired.set(true);
                // One readiness hint on stderr, best-effort: a failing
                // stderr must not cancel an otherwise valid wait result.
                // Empty object uses the same `/` column as stdout reports.
                let _ = writeln!(
                    std::io::stderr(),
                    "waiting  {command}  {}",
                    render::column(&object)
                );
            };
            let result = wait::run(&path, filter, &cancel, ready);
            // Bind recovery to the original inspected path after waiting: a
            // removed file or a changed reserved-state selection must not
            // inherit permission to emit directory-level mutation recipes.
            let dctx = match &kind {
                args::DiagKind::File(file) => context::diag_context_for_file(&ctx, command, file),
                args::DiagKind::Dir(dir) => context::diag_context_for_dir(&ctx, command, dir),
            };
            match result {
                Ok(report) => emit(&report, &dctx),
                Err(Fault::Cancelled) => ExitCode::from(130),
                Err(Fault::Usage(msg)) => usage_stderr(&msg),
                Err(Fault::Io {
                    action,
                    path: io_path,
                    source,
                }) => {
                    // A restarted wait takes a baseline containing changes
                    // made while it was stopped. Put the repair first, then
                    // restart, then re-read after its ready line; E-LOCKED
                    // and E-INUSE carry the same chronology. A path-class
                    // fault before the ready line is a start-time mistake
                    // and keeps corrected-path advice with no retry. After
                    // the ready line the same three kinds mean the path's
                    // surroundings changed while it was watched, so restore
                    // the intended path before restarting. Spelling-class
                    // faults (InvalidFilename, Windows raw 123/206) cannot
                    // follow a successful open and keep start-time advice.
                    let restore = if ready_fired.get() {
                        match source.kind() {
                            ErrorKind::NotFound => Some(diag::FIX_WAIT_RESTORE_DIR_GONE),
                            ErrorKind::NotADirectory => Some(diag::FIX_WAIT_RESTORE_DIR_REPLACED),
                            ErrorKind::IsADirectory => Some(diag::FIX_WAIT_RESTORE_NOT_REGULAR),
                            // On Unix, `Unsupported` here is only the
                            // storage read's own regular-file rejection
                            // (the watched path became a directory, FIFO
                            // or device); Windows also maps
                            // ERROR_NOT_SUPPORTED to it, so there it
                            // keeps the generic non-path repair.
                            ErrorKind::Unsupported if cfg!(unix) => {
                                Some(diag::FIX_WAIT_RESTORE_NOT_REGULAR)
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let reread = if command == "wait-state" {
                        diag::REREAD_STATE
                    } else {
                        diag::REREAD_FILE
                    };
                    let restart = if ctx.argv.is_some() {
                        diag::FIX_WAIT_RESTART
                    } else {
                        diag::FIX_WAIT_RESTART_MANUAL
                    }
                    .replace("{reread}", reread);
                    let recovery = WaitRecovery {
                        restart: &restart,
                        path_restore: restore,
                    };
                    let report = io_report_with_wait_recovery(
                        command,
                        &object,
                        action,
                        &io_path,
                        &source,
                        &ctx,
                        Some(&recovery),
                    );
                    emit(&report, &dctx)
                }
                Err(Fault::WriteUncertain { path, cause, left }) => emit(
                    &write_uncertain_report(command, &object, &path, &cause, &left),
                    &dctx,
                ),
                Err(Fault::Source {
                    path: source_path,
                    problem,
                    cause,
                }) => emit(
                    &source_report(command, &object, &source_path, problem, &cause, &ctx),
                    &dctx,
                ),
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

/// Render a business report to stdout and finish with its exit code.
/// Business reports write stdout only here (spec §8.1.1). Information
/// commands (guide, help, version) write through their own path; guide
/// checks write and flush, while this path still ignores flush failure.
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

/// One stderr line and exit 2, stdout untouched (spec §7.1.7, §11), for
/// the custom usage faults only: resolve and value-validation messages,
/// `Ctx` environment faults and late `Fault::Usage` escapes. CR/LF in
/// that custom usage text become spaces; stored body text is not
/// rewritten here, and clap grammar errors print clap's structured
/// report through their own emitter above.
fn usage_stderr(msg: &str) -> ExitCode {
    let line = msg.replace(['\n', '\r'], " ");
    eprintln!("error: {line}");
    ExitCode::from(2)
}

/// The wait dispatch's recovery context for a fault report: the
/// restart/read instruction ([`diag::FIX_WAIT_RESTART`] or its manual
/// variant, `{reread}` already substituted), plus an optional restore
/// template for a structural path fault after the wait's ready line
/// (a [`diag::FIX_WAIT_RESTORE_*`] constant, `{display}` still
/// unsubstituted). Every restore carries the restart/read instruction;
/// base commands pass no context.
struct WaitRecovery<'a> {
    restart: &'a str,
    path_restore: Option<&'static str>,
}

/// The §8.3.5 report for an OS-level I/O fault: `reason` =
/// `cannot <action> <display file>: <os error>`, `fix` chosen by error
/// nature. Path-class errors (NotFound / IsADirectory / NotADirectory /
/// InvalidFilename, or Windows raw 123 / 206) get `fix the path...`.
/// When that failing original path still contains a control character, the
/// fix names it with [`coexist_core::names::echo_raw`] so the character
/// folded out of `reason` stays visible; the folded reason path is
/// unchanged, the path is not rejected here, and Unix names that allow
/// the character keep the same OS class.
/// Every other error gets a repair for its kind (see [`io_repair`]),
/// followed by the retry with the POSIX-quoted argv, or a by-hand retry
/// when the command line cannot be written as one copyable command. Exit 1.
fn io_report(
    command: &'static str,
    object: &str,
    action: &'static str,
    path: &Path,
    source: &std::io::Error,
    ctx: &Ctx,
) -> Report {
    io_report_with_wait_recovery(command, object, action, path, source, ctx, None)
}

/// [`io_report`] with the wait dispatch's optional [`WaitRecovery`]
/// context: its restart/read instruction follows the repair and precedes
/// the original command. An optional restore template replaces both the
/// corrected-path advice and the generic I/O repair for a structural loss
/// of a path the command had already proven valid. Base commands pass no
/// context and get the plain [`io_report`] behavior.
fn io_report_with_wait_recovery(
    command: &'static str,
    object: &str,
    action: &'static str,
    path: &Path,
    source: &std::io::Error,
    ctx: &Ctx,
    wait_recovery: Option<&WaitRecovery<'_>>,
) -> Report {
    let mut report = Report::new(command, object);
    let display = context::display_file(path);
    // The reason sentence is one display column (§8.1.4–5): a raw cause
    // can embed two-space runs, so the composed human text is folded like
    // any other (§10.9.4) — never truncated, never re-quoted. The borrowed
    // `source` below still decides kind/raw-OS class, and the fix/argv
    // branch keeps native operands verbatim. A control character in the
    // original path is named only in the path-class fix, never by changing
    // this folded reason.
    report.kv(
        "reason",
        vec![coexist_core::names::display_text(&format!(
            "cannot {action} {display}: {source}"
        ))],
    );
    let restart = wait_recovery.map(|recovery| recovery.restart);
    let path_restore = wait_recovery.and_then(|recovery| recovery.path_restore);
    let path_class = matches!(
        source.kind(),
        ErrorKind::NotFound
            | ErrorKind::IsADirectory
            | ErrorKind::NotADirectory
            | ErrorKind::InvalidFilename
    ) || matches!(source.raw_os_error(), Some(123) | Some(206));
    // Repair and retry for every fault whose fix still offers the original
    // command. A restore template is decided by the wait dispatch for a
    // post-ready structural fault and takes this branch whenever it is
    // present — including the Unix regular-file rejection, which is
    // `Unsupported` and therefore not in `path_class` below, yet is the
    // same structural loss of a path the wait had proven valid. The
    // POSIX-quoted argv retry, the manual by-hand form for an
    // unrepresentable command line and the wait's restart step are shared.
    let repair_and_retry = |repair: String| match &ctx.argv {
        Some(argv) => match restart {
            Some(restart) => format!("{repair}; {restart}: {argv}"),
            None => format!("{repair}, then retry: {argv}"),
        },
        None => {
            let manual = diag::FIX_NATIVE_OPERAND;
            let why = manual.split_once("; ").map_or(manual, |(_, why)| why);
            match restart {
                Some(restart) => format!("{repair}; {restart}; {why}"),
                None => format!("{repair}, then retry the original command by hand; {why}"),
            }
        }
    };
    let fix = match path_restore {
        Some(template) => repair_and_retry(template.replace("{display}", &display)),
        None if path_class => {
            let unfolded = coexist_core::fsx::walk::display_path(path);
            if unfolded.chars().any(char::is_control) {
                format!(
                    "the path contains a line break or other control character (original: '{}'); fix the path, then run the corrected command",
                    coexist_core::names::echo_raw(&unfolded),
                )
            } else {
                "fix the path, then run the corrected command".to_string()
            }
        }
        None => repair_and_retry(io_repair(&display, source)),
    };
    report.kv("fix", vec![fix]);
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// The repair step for a non-path I/O error on `display`. The kinds name
/// what can still be done about them; none of them claims a single cause.
/// A denied access (Unix EACCES/EPERM, Windows error 5) can come from the
/// file's or a containing directory's permissions; on Windows error 5 also
/// means that another program is deleting or replacing the file (a delete
/// pending until its handle closes), so that remedy is named there too. A
/// Windows sharing violation at open is E-INUSE before it reaches this
/// report. Any other error keeps the OS text in `reason` as the thing to
/// resolve rather than guessing permissions or disk space.
fn io_repair(display: &str, source: &std::io::Error) -> String {
    match source.kind() {
        ErrorKind::PermissionDenied if cfg!(windows) => format!(
            "check the permissions of {display} and of the directories on its path, or wait until another program stops using, deleting or replacing it"
        ),
        ErrorKind::PermissionDenied => {
            format!("check the permissions of {display} and of the directories on its path")
        }
        ErrorKind::StorageFull | ErrorKind::QuotaExceeded => {
            format!("free disk space or quota where {display} is stored")
        }
        // EFBIG: the process file-size limit (RLIMIT_FSIZE) or the file
        // system's maximum file size, not free space.
        ErrorKind::FileTooLarge if cfg!(unix) => format!(
            "raise this shell's file-size limit (ulimit -f), or move {display} to a file system that allows larger files"
        ),
        ErrorKind::FileTooLarge => {
            format!("move {display} to a file system that allows larger files")
        }
        ErrorKind::ReadOnlyFilesystem => {
            format!("make the file system that holds {display} writable")
        }
        _ => "resolve the error shown in 'reason'".to_string(),
    }
}

/// The report for a write that stopped after changing its file, when the
/// file is not known to show its previous content (`Fault::WriteUncertain`).
/// Unlike [`io_report`] it never offers the command again: the change, or
/// part of it, may already be in the file, and repeating a post would add
/// it twice. It names the repair for the error that blocks recovery and
/// then `check` on the file. Exit 1.
fn write_uncertain_report(
    command: &'static str,
    object: &str,
    path: &Path,
    cause: &std::io::Error,
    left: &WriteLeft,
) -> Report {
    let mut report = Report::new(command, object);
    let display = context::display_file(path);
    let check = match diag::path_operand(path.as_os_str()) {
        Some(operand) => format!(": agent-coexist check {operand}"),
        None => {
            let manual = diag::FIX_NATIVE_OPERAND;
            let why = manual.split_once("; ").map_or(manual, |(_, why)| why);
            format!(" (run check on it by hand; {why})")
        }
    };
    let (reason, fix) = match left {
        WriteLeft::Unconfirmed => (
            diag::MESSAGE_WRITE_UNCONFIRMED
                .replace("{file}", &display)
                .replace("{cause}", &cause.to_string()),
            diag::FIX_WRITE_UNCONFIRMED
                .replace("{repair}", &io_repair(&display, cause))
                .replace("{check}", &check),
        ),
        WriteLeft::Mixed {
            from,
            len,
            rollback,
        } => (
            diag::MESSAGE_WRITE_MIXED
                .replace("{file}", &display)
                .replace("{cause}", &cause.to_string())
                .replace("{rollback}", &rollback.to_string()),
            diag::FIX_WRITE_MIXED
                .replace("{from}", &from.to_string())
                .replace("{len}", &len.to_string())
                .replace("{repair}", &io_repair(&display, rollback))
                .replace("{check}", &check),
        ),
    };
    // One display column, like the I/O reason (§8.1.4–5, §10.9.4).
    report.kv("reason", vec![coexist_core::names::display_text(&reason)]);
    report.kv("fix", vec![fix]);
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// The §8.3.5 report for an ignore or git config source that stopped a
/// directory scan: `reason` = `cannot scan <display source>: <cause>`, and a
/// `fix` that names the repair for this cause (a content error points at
/// the line to correct) followed by the original command. Exit 1.
fn source_report(
    command: &'static str,
    object: &str,
    path: &Path,
    problem: SourceProblem,
    cause: &str,
    ctx: &Ctx,
) -> Report {
    let mut report = Report::new(command, object);
    let display = context::display_file(path);
    // One display column (§8.1.4–5, §10.9.4), like the I/O reason above.
    report.kv(
        "reason",
        vec![coexist_core::names::display_text(&format!(
            "cannot scan {display}: {cause}"
        ))],
    );
    let fix = problem.fix(&display, "retry", ctx.argv.as_deref());
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
    params: Vec<(&'static str, OsString)>,
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
fn diagnostics_report(command: &'static str, object: &str, diagnostics: Vec<Diagnostic>) -> Report {
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

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStringExt;

    #[test]
    fn io_retry_with_unpaired_utf16_is_wholly_manual() {
        let native = OsString::from_wide(&[b'r' as u16, 0xd800]);
        let argv = diag::Context::argv_from_args([OsStr::new("agent-coexist"), native.as_os_str()]);
        assert!(argv.is_none());
        let ctx = Ctx::new(argv, None).expect("context");
        let report = io_report(
            "check",
            "room",
            "open",
            Path::new("room"),
            &std::io::Error::new(ErrorKind::PermissionDenied, "permission denied"),
            &ctx,
        );
        let fix = report
            .rows
            .iter()
            .find_map(|row| match row {
                Row::Fields { key, values, .. } if key == "fix" => Some(values),
                _ => None,
            })
            .expect("fix row");
        // No partial command may be printed; the repair for the error kind
        // stays, and the retry is by hand with the manual explanation.
        let manual = diag::FIX_NATIVE_OPERAND.split_once("; ").unwrap().1;
        assert_eq!(fix.len(), 1);
        assert!(!fix[0].contains("agent-coexist"), "{fix:?}");
        assert!(fix[0].contains("by hand"), "{fix:?}");
        assert!(fix[0].ends_with(manual), "{fix:?}");
    }
}
