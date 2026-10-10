//! Internal library of the `agent-coexist` CLI: parsing, types, filesystem
//! access and command logic for the product binary (v0.1 spec). It is an
//! implementation detail of the CLI — unstable and unsupported as a public
//! API.
//!
//! Module layout and ownership (single writer per file, see the outer work
//! order):
//!
//! - `consts`, `names`, `error`, `diag`, `text`, `doc`, `types` — raw-byte
//!   structure parsing, type views, name grammars and the diagnostic catalog
//!   (spec §9's only in-source copy).
//! - `time`, `fsx`, `hash` — time entry points, locked read/mutate lifecycle,
//!   directory walking and brief-target verification.
//! - `report` — the structured result every command returns; only the CLI
//!   binary renders it.
//! - `ops` — the eleven base commands; see `ops::` module docs for the
//!   internal boundaries.
//! - `wait` — the optional wait extensions (three extra commands). Base
//!   commands never depend on this module; only the CLI dispatches to it.
//! - `vendor_ignore` — the vendored `ignore` fork (private; see
//!   CONTRIBUTING.md for provenance and the update rules).
//!
//! The library never prints. Commands return pure data (`report::Report`) or
//! an `error::Fault` for usage / OS-level I/O conditions.

#[allow(clippy::all)]
#[allow(dead_code)]
#[allow(unused_imports)]
#[rustfmt::skip]
mod vendor_ignore;

pub mod consts;
pub mod diag;
pub mod doc;
pub mod error;
pub mod fsx;
pub mod hash;
pub mod names;
pub mod ops;
pub mod report;
pub mod text;
pub mod time;
pub mod types;
pub mod wait;

/// All timestamps used across the CLI (spec §2.8): UTC, second precision in
/// serialized form.
pub type Time = chrono::DateTime<chrono::Utc>;
