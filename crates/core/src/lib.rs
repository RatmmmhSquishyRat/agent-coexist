//! `coexist-core` — parsing, types, filesystem access and command logic for
//! the `agent-coexist` CLI (v0.1 spec).
//!
//! Crate layout and ownership (single writer per file, see the outer work
//! order):
//!
//! - `consts`, `names`, `error`, `diag`, `text`, `doc`, `types` — raw-byte
//!   structure parsing, type views, name grammars and the diagnostic catalog
//!   (spec §9's only in-source copy).
//! - `time`, `fsx`, `hash` — time entry points, locked read/mutate lifecycle,
//!   directory walking and brief-target verification.
//! - `report` — the structured result every command returns; only the CLI
//!   crate renders it.
//! - `ops` — the eleven base commands; see `ops::` module docs for the
//!   internal boundaries.
//! - `wait` — the optional wait extensions (three extra commands). Base
//!   commands never depend on this module; only the CLI dispatches to it.
//!
//! Core never prints. Commands return pure data (`report::Report`) or a
//! `error::Fault` for usage / OS-level I/O conditions.

pub mod consts;
pub mod names;
pub mod error;
pub mod diag;
pub mod text;
pub mod doc;
pub mod types;
pub mod time;
pub mod fsx;
pub mod hash;
pub mod wait;
pub mod report;
pub mod ops;

/// All timestamps used across the CLI (spec §2.8): UTC, second precision in
/// serialized form.
pub type Time = chrono::DateTime<chrono::Utc>;
