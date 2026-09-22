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

    /// Publish the current progress counters. Used as the walker callback
    /// target during traversal (`fsx::walk::scan`) and directly by callers
    /// that walk or read on their own.
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
    pub fn walk_sink(&mut self) -> impl FnMut(usize, usize) + '_ {
        move |dirs, files| self.progress(dirs, files)
    }
}

/// Display path for any managed file or directory: cwd-relative lexical
/// path, `/`-separated, whitespace folded so no value column can contain a
/// double space (spec §8.1.9, §10.9).
pub fn display_file(path: &Path) -> String {
    names::fold(&fsx::display_path(path))
}

/// First-line object for PATH commands: the display path without the
/// `.agent-besties.md` suffix (spec §8.1.3; a PATH given with its suffix is
/// displayed without it).
pub fn object_of_path(path: &Path) -> String {
    let displayed = fsx::display_path(path);
    let trimmed = displayed.strip_suffix(consts::SUFFIX).unwrap_or(&displayed);
    names::fold(trimmed)
}

/// First-line object for DIR commands (omitted DIR displays as `.`).
pub fn object_of_dir(dir: &Path) -> String {
    names::fold(&fsx::display_path(dir))
}

/// `diag::Context` for a command whose object is a managed file PATH.
/// `file` is the actual filesystem path (suffix appended by the CLI).
pub fn diag_context_for_file(ctx: &Ctx, command: &'static str, file: &Path) -> diag::Context {
    let object = object_of_path(file);
    let (dir, stem) = split_object(&object);
    diag::Context {
        path: object,
        dir,
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
    diag::Context {
        path,
        dir: dir_display,
        stem: "state".to_string(),
        argv: ctx.argv.clone(),
        command: command.to_string(),
        now: time::format(ctx.now),
    }
}

fn split_object(object: &str) -> (String, String) {
    match object.rfind('/') {
        Some(i) => (object[..i].to_string(), object[i + 1..].to_string()),
        None => (".".to_string(), object.to_string()),
    }
}
