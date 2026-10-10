//! Fault and Diagnostic: the error currency of the whole core crate
//! (spec §1 "blocking diagnostics", §8.3, §9).
//!
//! `Diagnostic` is one entry of the frozen §9 catalog, anchored at a 1-based
//! line of the source file, carrying the substitution params for the catalog
//! template. `Fault` is the single error enum returned by core entry points;
//! ids outside the §9 table are never invented.

use std::ffi::OsString;
use std::path::PathBuf;

/// One catalog diagnostic anchored at a 1-based line of the parsed file.
///
/// `params` keys are the `{placeholder}` names of the §9 templates. Heading
/// identity values retain their parsed UTF-8 form; the renderer sanitizes
/// only human display. Native command operands use `*_arg` keys and retain
/// their exact platform encoding, never reconstructed from display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub id: &'static str,
    pub line: usize,
    pub params: Vec<(&'static str, OsString)>,
}

impl Diagnostic {
    pub fn new(id: &'static str, line: usize) -> Self {
        Diagnostic {
            id,
            line,
            params: Vec::new(),
        }
    }

    /// Append a substitution param. For duplicate keys the first occurrence
    /// wins at render time.
    pub fn with(mut self, key: &'static str, value: impl Into<OsString>) -> Self {
        self.params.push((key, value.into()));
        self
    }
}

/// The single core error type (spec §8.3 failure forms, §11 exit codes).
#[derive(Debug)]
pub enum Fault {
    /// Usage error: one `error: ...` line on stderr, exit 2 (spec §7.1.7).
    Usage(String),
    /// File-level diagnostics that abort the operation (e.g. `E-UTF8`,
    /// `E-TOO-LARGE`); rendered like `check` diagnostic rows, exit 1.
    Diagnostics(Vec<Diagnostic>),
    /// Reason-form failure: the §9 catalog row for `id` supplies the reason
    /// and fix text; params carry the runtime values (spec §8.3.1).
    Reason {
        id: &'static str,
        params: Vec<(&'static str, OsString)>,
    },
    /// Unclassified OS-level I/O failure (spec §8.3.5): the action verb,
    /// the path involved and the original error are all preserved.
    Io {
        action: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    /// An ignore or git config file that a directory walk depends on could
    /// not be used (spec §8.3.5, §10.1.2). `list` / `read-notification` /
    /// `check-state` fail with it; `state` reports it after a committed
    /// write (§10.5.4). Ordinary unreadable directories are not this.
    Source {
        /// The failing source file (a `.git` file for a bad gitdir
        /// reference); only a rule failure that names no file gives the
        /// directory whose rules were being loaded. Caller's lexical form.
        path: PathBuf,
        /// What is wrong with it; selects the recovery advice.
        problem: SourceProblem,
        /// The walker's error text without the path, e.g. `line 1: error
        /// parsing glob ...`; display folding happens where it is shown.
        cause: String,
    },
    /// A managed-file write stopped after it had begun changing the file,
    /// and the file is not known to show its previous content (spec
    /// §10.2.2). Unlike [`Fault::Io`], running the command again is not a
    /// safe recovery: the change may already be (partly) in the file.
    WriteUncertain {
        path: PathBuf,
        /// The error that stopped the write.
        cause: std::io::Error,
        /// What the file holds now.
        left: WriteLeft,
    },
    /// Cooperative cancellation (wait commands): exit 130.
    Cancelled,
}

/// Why an ignore or git config source failed (spec §8.3.5). Content errors
/// are fixed by editing the file; the others are about the file itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceProblem {
    /// The file was read but its content is invalid: an unparsable glob or
    /// non-UTF-8 text at `line`, or a malformed git reference (no line).
    Content { line: Option<u64> },
    /// The path exists but is not a regular file (directory, FIFO, device).
    NotRegularFile,
    /// The OS refused or failed the open or read (permissions, I/O error).
    Access,
    /// Every line parsed, but the file's rules could not be compiled
    /// together: too many or too long patterns (size limit), or `{…}`
    /// groups nested too deeply (nesting limit).
    Rules,
}

/// The state a [`Fault::WriteUncertain`] write left its file in.
#[derive(Debug)]
pub enum WriteLeft {
    /// Every new byte is in place; only the final `sync_data` failed, so
    /// the content is visible but not confirmed on disk. Nothing was undone.
    Unconfirmed,
    /// Putting the previous content back failed with `rollback`. Bytes
    /// before offset `from` are the previous content; from `from` on the
    /// file may mix old and new bytes. `len` is the previous length.
    Mixed {
        from: u64,
        len: u64,
        rollback: std::io::Error,
    },
}

impl SourceProblem {
    /// The repair step for `source` (a display path).
    fn repair(self, source: &str) -> String {
        match self {
            SourceProblem::Content { line: Some(line) } => {
                format!("correct line {line} of {source}")
            }
            SourceProblem::Content { line: None } => format!("correct the contents of {source}"),
            SourceProblem::NotRegularFile => {
                format!("replace {source} with a regular file or remove it")
            }
            SourceProblem::Access => format!("check that {source} can be read"),
            SourceProblem::Rules => {
                format!(
                    "shorten, simplify or remove rules in {source} until they fit the pattern matcher's size and nesting limits"
                )
            }
        }
    }

    /// The whole `fix` sentence: the repair, then `next` followed by the
    /// runnable `command` (already §8.3.4-quoted). Without a representable
    /// command the sentence asks for `next` by hand, with the existing
    /// manual-recovery explanation, never a lossy look-alike (§8.3.4).
    pub fn fix(self, source: &str, next: &str, command: Option<&str>) -> String {
        let repair = self.repair(source);
        match command {
            Some(command) => format!("{repair}, then {next}: {command}"),
            None => {
                let manual = crate::diag::FIX_NATIVE_OPERAND;
                let why = manual.split_once("; ").map_or(manual, |(_, why)| why);
                format!("{repair}, then {next} by hand; {why}")
            }
        }
    }
}

impl Fault {
    /// Start a reason-form fault for a §9 catalog id.
    pub fn reason(id: &'static str) -> Self {
        Fault::Reason {
            id,
            params: Vec::new(),
        }
    }

    /// Append a substitution param to a reason-form fault. On other variants
    /// this is a documented no-op (they carry no param list).
    pub fn with(mut self, key: &'static str, value: impl Into<OsString>) -> Self {
        if let Fault::Reason { params, .. } = &mut self {
            params.push((key, value.into()));
        }
        self
    }

    /// Wrap an OS I/O error with the action verb and the path it happened on.
    pub fn io(action: &'static str, path: PathBuf, error: std::io::Error) -> Self {
        Fault::Io {
            action,
            path,
            source: error,
        }
    }
}
