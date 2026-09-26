//! Fault and Diagnostic: the error currency of the whole core crate
//! (spec §1 "阻断诊断", §8.3, §9).
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
    /// Cooperative cancellation (wait commands): exit 130.
    Cancelled,
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
