//! Per-invocation context shared by every ops command.
//!
//! The CLI builds exactly one [`Ctx`] after argument validation and before
//! any file access; construction performs the two environment entry points
//! (`time::now`, `time::post_wait_ms`) so every later use sees one clock and
//! one lock budget (spec §1 `now`, §10.2.9). Ops code never reads
//! environment variables itself.

use std::path::Path;

use crate::consts;
use crate::diag;
use crate::error::Fault;
use crate::fsx;
use crate::names;
use crate::time;
use crate::Time;

/// Progress sink: `(directories entered, managed files accounted for)`
/// (spec §8.4). Supplied by the CLI; only ever fed through [`Ctx`], never
/// printed from core.
pub type ProgressFn = Box<dyn FnMut(usize, usize) + Send>;

pub struct Ctx {
    /// Resolved `now` (honors `AGENT_COEXIST_NOW`).
    pub now: Time,
    /// Exclusive-lock wait budget for post/subscribe writes (honors
    /// `AGENT_COEXIST_LOCK_WAIT_MS`), spec §10.2.
    pub post_wait_ms: u64,
    /// The original command line: every argument POSIX-quoted per spec
    /// §8.3.4, joined by single spaces (feeds `{argv}` in fix templates).
    pub argv: String,
    progress: Option<ProgressFn>,
    dirs: usize,
    files: usize,
}

impl Ctx {
    pub fn new(argv: String, progress: Option<ProgressFn>) -> Result<Ctx, Fault> {
        let now = time::now()?;
        let post_wait_ms = time::post_wait_ms()?;
        Ok(Ctx {
            now,
            post_wait_ms,
            argv,
            progress,
            dirs: 0,
            files: 0,
        })
    }

    /// Publish the current progress counters. Used directly by callers
    /// that walk or read on their own; the traversal phase goes through
    /// [`Ctx::walk_sink`], which never lets enumeration overwrite reads.
    pub fn progress(&mut self, dirs: usize, files: usize) {
        self.dirs = dirs;
        self.files = files;
        if let Some(sink) = self.progress.as_mut() {
            sink(dirs, files);
        }
    }

    /// One more managed file was read (or failed to read) — the counter must
    /// advance per file in the read loop, including failures (spec §8.4.1).
    pub fn file_read(&mut self) {
        self.files += 1;
        if let Some(sink) = self.progress.as_mut() {
            sink(self.dirs, self.files);
        }
    }

    /// Adapter for `fsx::walk::scan(&Path, &Options, &mut dyn FnMut(usize, usize))`.
    ///
    /// The walker's second counter is files *enumerated*, but §8.4.1's
    /// `<m>` is files actually *read* (failures included), advanced only by
    /// [`Ctx::file_read`]. Enumeration therefore advances the directory
    /// counter only: reads already accumulated (e.g. the ancestor-chain
    /// reads of a state scope, §10.5.2) survive the traversal phase instead
    /// of being overwritten by found names or reset to zero afterwards.
    pub fn walk_sink(&mut self) -> impl FnMut(usize, usize) + '_ {
        move |dirs, _enumerated| {
            let files = self.files;
            self.progress(dirs, files)
        }
    }
}

/// Deterministic construction for in-crate tests: one explicit clock, no
/// environment reads. Integration tests keep going through [`Ctx::new`].
#[cfg(test)]
pub(crate) fn test_ctx(now: Time) -> Ctx {
    Ctx {
        now,
        post_wait_ms: 0,
        argv: String::new(),
        progress: None,
        dirs: 0,
        files: 0,
    }
}

/// Display path for any managed file or directory: cwd-relative lexical
/// path, `/`-separated, whitespace folded so no value column can contain a
/// double space (spec §8.1.9, §10.9).
pub fn display_file(path: &Path) -> String {
    names::fold(&fsx::walk::display_path(path))
}

/// First-line object for PATH commands: the display path without the
/// `.agent-besties.md` suffix (spec §8.1.3; a PATH given with its suffix is
/// displayed without it).
pub fn object_of_path(path: &Path) -> String {
    let displayed = fsx::walk::display_path(path);
    let trimmed = displayed.strip_suffix(consts::SUFFIX).unwrap_or(&displayed);
    names::fold(trimmed)
}

/// First-line object for DIR commands (omitted DIR displays as `.`).
pub fn object_of_dir(dir: &Path) -> String {
    names::fold(&fsx::walk::display_path(dir))
}

/// `diag::Context` for a command whose object is a managed file PATH.
/// `file` is the actual filesystem path (suffix appended by the CLI).
/// Resolve its reserved-state selection once, not for every diagnostic.
/// An unresolved selection permits only manual actual-file recovery.
pub fn diag_context_for_file(ctx: &Ctx, command: &'static str, file: &Path) -> diag::Context {
    let object = object_of_path(file);
    let (dir, stem) = split_object(&object);
    let state_file_selected = super::shared::is_reserved_state_file(file).unwrap_or(false);
    diag::Context {
        path: object,
        dir,
        path_arg: file.to_path_buf(),
        dir_arg: parent_dir(file).to_path_buf(),
        state_file_selected,
        stem,
        argv: ctx.argv.clone(),
        command: command.to_string(),
        now: time::format(ctx.now),
    }
}

/// `diag::Context` for DIR-based commands (state, list, check-state,
/// read-notification). `dir` is the directory the user gave (or `.`).
pub fn diag_context_for_dir(ctx: &Ctx, command: &'static str, dir: &Path) -> diag::Context {
    let dir_display = object_of_dir(dir);
    let path = if dir_display == "." {
        "state".to_string()
    } else {
        format!("{dir_display}/state")
    };
    let path_arg = dir.join(format!("state{}", consts::SUFFIX));
    // wait-state inspects this exact native path. Other DIR commands do
    // not identify one inspected file for these recovery diagnostics.
    let state_file_selected = command == "wait-state"
        && super::shared::is_reserved_state_file(&path_arg).unwrap_or(false);
    diag::Context {
        path,
        dir: dir_display,
        stem: "state".to_string(),
        path_arg,
        dir_arg: dir.to_path_buf(),
        state_file_selected,
        argv: ctx.argv.clone(),
        command: command.to_string(),
        now: time::format(ctx.now),
    }
}

/// Native lexical parent, with `.` only for an absent or empty parent.
/// Unlike display helpers this must not fold whitespace or rewrite prefixes.
pub(crate) fn parent_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn split_object(object: &str) -> (String, String) {
    match object.rfind('/') {
        Some(i) => (object[..i].to_string(), object[i + 1..].to_string()),
        None => (".".to_string(), object.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// The sink plus its observed `(dirs, files)` calls, in order.
    type RecordedProgress = (ProgressFn, Arc<Mutex<Vec<(usize, usize)>>>);

    /// Recording progress sink (`ProgressFn` must be `Send`).
    fn recording() -> RecordedProgress {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink_seen = Arc::clone(&seen);
        let sink: ProgressFn = Box::new(move |dirs, files| {
            sink_seen.lock().expect("sink state").push((dirs, files));
        });
        (sink, seen)
    }

    /// §8.4.1 consumer contract: `<m>` counts files read (failures
    /// included), never files merely enumerated, and reads from before a
    /// traversal phase survive it (regression: PROGRESS-M-DURING-SCAN).
    #[test]
    fn walk_sink_preserves_reads_across_enumeration() {
        let (sink, seen) = recording();
        let mut ctx = Ctx {
            now: time::parse("2026-09-17T10:00:00Z").expect("fixed clock"),
            post_wait_ms: 0,
            argv: String::new(),
            progress: Some(sink),
            dirs: 0,
            files: 0,
        };
        // Two files read before the traversal phase (a state scope's
        // ancestor chain is read before its subtree scan, §10.5.2).
        ctx.file_read();
        ctx.file_read();
        {
            let mut walk = ctx.walk_sink();
            // The walker reports directories entered and files *enumerated*;
            // enumeration must neither overwrite nor reset the read counter.
            walk(3, 17);
            walk(4, 42);
        }
        ctx.file_read();
        assert_eq!(
            *seen.lock().expect("sink state"),
            vec![(0, 1), (0, 2), (3, 2), (4, 2), (4, 3)]
        );
    }
}
