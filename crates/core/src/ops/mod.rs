//! The eleven base commands (spec §7).
//!
//! # Module boundaries (Application-owned contract)
//!
//! - [`context`]: the per-invocation [`Ctx`] (one clock, post lock budget,
//!   rendered argv, progress counters) plus the display/diag-context helpers
//!   every command reuses. Owned by the application lead.
//! - [`shared`]: the single implementation of state-file collection
//!   (§10.5.1 scope), conflict computation (§10.5) and the functional-lock
//!   verdict (§10.8). Consumed by `state`, `profile`, `brief` and `query`.
//! - [`state`]: the `state` command (declare / remove forms, §6.4).
//! - [`post`], [`subscribe`], [`profile`], [`brief`]: the single-file write
//!   commands. All writes go through `fsx::mutate` with `text::apply`
//!   splices; profile/brief additionally gate on `shared::functional_lock`.
//! - [`query`]: the read-only commands `check`, `list`, `lookfor-comment`,
//!   `read-brief`, `read-notification`, `check-state`.
//!
//! Commands never print and never touch clap types: the CLI validates every
//! argument (spec §7.1), builds a [`Request`] plus [`Ctx`], and calls
//! [`run`]. Business failures — including diagnostics-shaped failures — come
//! back as [`Report`]s; only usage and OS-level I/O conditions propagate as
//! `Fault` (mapped by the CLI to stderr exit 2 and the §8.3.5 form).
//!
//! # Entry point
//!
//! [`run`] dispatches one validated invocation.

pub mod brief;
pub mod context;
pub mod post;
pub mod profile;
pub mod query;
pub mod shared;
pub mod state;
pub mod subscribe;

pub use context::Ctx;

use std::path::PathBuf;

use crate::diag;
use crate::doc;
use crate::error::Fault;
use crate::fsx;
use crate::report::{Report, Status};

/// One validated command invocation (the CLI → core boundary).
///
/// All values are already validated and normalized by the CLI: nick / status
/// / section grammars, control-character and double-space checks, `-m`
/// normalization (CRLF folded, trailing whitespace removed, non-empty),
/// `--entry` title folding, `--since` / `--for` resolved to concrete values,
/// `--regex` compiled. Paths carry the appended suffix. Ops code may assume
/// well-formed input; it re-validates file content, not arguments.
pub enum Request {
    List(ListParams),
    Check(CheckParams),
    Profile(ProfileParams),
    Post(PostParams),
    Subscribe(SubscribeParams),
    Brief(BriefParams),
    State(StateParams),
    ReadNotification(ReadNotificationParams),
    LookforComment(LookforCommentParams),
    ReadBrief(ReadBriefParams),
    CheckState(CheckStateParams),
}

/// `list [DIR] [--type <type>]... [WALK]` — empty `types` = all four.
pub struct ListParams {
    pub dir: PathBuf,
    pub types: Vec<doc::Kind>,
    pub walk: fsx::walk::Options,
}

/// `check <PATH>`.
#[derive(Debug)]
pub struct CheckParams {
    pub path: PathBuf,
}

/// `profile <PATH> [--as <nick>] [--section <name> (-m <content> | --remove)]`.
#[derive(Debug)]
pub struct ProfileParams {
    pub path: PathBuf,
    /// Only used for the functional-lock verdict (§10.8); never written.
    pub as_nick: Option<String>,
    pub section: Option<String>,
    pub action: ProfileAction,
}

#[derive(Debug)]
pub enum ProfileAction {
    /// No `--section`: create the file if missing, else `unchanged`.
    None,
    SetBody(String),
    Remove,
}

/// `post <PATH> --as <nick> -m <content> [--edit <n>]`.
#[derive(Debug)]
pub struct PostParams {
    pub path: PathBuf,
    pub nick: String,
    /// Normalized `-m` body (LF-only, trailing whitespace stripped).
    pub body: String,
    pub edit: Option<u64>,
}

/// `subscribe <PATH> --as <nick> [--cancel]`.
#[derive(Debug)]
pub struct SubscribeParams {
    pub path: PathBuf,
    pub nick: String,
    pub cancel: bool,
}

/// `brief <PATH> [--as <nick>] <form>` — the six mutually exclusive forms of
/// spec §4.5 / §7.2.
#[derive(Debug)]
pub struct BriefParams {
    pub path: PathBuf,
    /// Only used for the functional-lock verdict (§10.8); never written.
    pub as_nick: Option<String>,
    pub form: BriefForm,
}

#[derive(Debug)]
pub enum BriefForm {
    /// No `-m` / `--target` / `--entry` / `--refresh`: create the skeleton
    /// when missing, else `unchanged` (§4.5.1–2).
    Bare,
    /// `-m` alone: create with a description fence, or replace it.
    SetDescription(String),
    /// `--target <path> [--entry <title>] [--regex <pattern>] [-m <note>]`.
    Target(BriefTarget),
    /// `--entry <title> -m <note>` (no `--target`): replace the note fence.
    NoteOnly { entry: String, note: String },
    /// `--entry <title> --remove`.
    RemoveEntry { entry: String },
    /// `--refresh [--entry <title>]`.
    Refresh { entry: Option<String> },
}

#[derive(Debug)]
pub struct BriefTarget {
    /// Target path exactly as the user gave it, separators already
    /// normalized to `/` (stored verbatim otherwise, §4.3.3); resolved
    /// against the brief file's directory via `hash::resolve_target`.
    pub target: String,
    pub entry: Option<String>,
    pub regex: RegexOpt,
    pub note: Option<String>,
}

/// `--regex` tri-state (§4.5.4).
#[derive(Debug)]
pub enum RegexOpt {
    /// Flag not given: keep any existing `regex` line.
    Keep,
    /// `--regex ""`: delete the `regex` line.
    Delete,
    /// `--regex <pattern>`: add or replace (already compiled once by the CLI).
    Set(String),
}

/// `state [DIR] --as <nick> --status <name> [--for <duration>]` or
/// `state [DIR] --remove <nick> [--as <nick>] [--status <name>]`.
#[derive(Debug)]
pub enum StateParams {
    Declare(StateDeclare),
    Remove(StateRemove),
}

#[derive(Debug)]
pub struct StateDeclare {
    pub dir: PathBuf,
    pub nick: String,
    pub status: String,
    /// `--for` in seconds; defaults per §6.3.1 (locked 2h, others 8h).
    pub for_secs: Option<i64>,
}

#[derive(Debug)]
pub struct StateRemove {
    pub dir: PathBuf,
    /// The nick whose entries are removed.
    pub nick: String,
    /// `--as` when given; defaults to `nick` (self-removal), §6.4.2.
    pub as_nick: Option<String>,
    pub status: Option<String>,
}

/// `read-notification [DIR] --as <nick> --since <time|duration> [WALK]`.
pub struct ReadNotificationParams {
    pub dir: PathBuf,
    pub nick: String,
    /// `--since` already resolved to a UTC whole-second time (DURATION
    /// converted with the invocation's `now`).
    pub since: crate::Time,
    pub walk: fsx::walk::Options,
}

/// `lookfor-comment <PATH> [filters]` — no filters = all comments; several
/// filters intersect (§10.7).
#[derive(Debug)]
pub struct LookforCommentParams {
    pub path: PathBuf,
    pub filters: Filters,
}

#[derive(Debug, Default)]
pub struct Filters {
    /// Original value, never folded (§7.1.6); case-insensitive substring.
    pub keyword: Option<String>,
    pub by: Option<String>,
    pub mention: Option<String>,
    pub reply_to: Option<u64>,
    pub comment: Option<u64>,
}

/// `read-brief <PATH> [--entry <title>]`.
#[derive(Debug)]
pub struct ReadBriefParams {
    pub path: PathBuf,
    pub entry: Option<String>,
}

/// `check-state [DIR] [WALK]`.
pub struct CheckStateParams {
    pub dir: PathBuf,
    pub walk: fsx::walk::Options,
}

/// Dispatch one validated invocation.
pub fn run(ctx: &mut Ctx, req: Request) -> Result<Report, Fault> {
    match req {
        Request::List(p) => query::list(ctx, &p),
        Request::Check(p) => query::check(ctx, &p),
        Request::Profile(p) => profile::run(ctx, &p),
        Request::Post(p) => post::run(ctx, &p),
        Request::Subscribe(p) => subscribe::run(ctx, &p),
        Request::Brief(p) => brief::run(ctx, &p),
        Request::State(p) => state::run(ctx, &p),
        Request::ReadNotification(p) => query::read_notification(ctx, &p),
        Request::LookforComment(p) => query::lookfor_comment(ctx, &p),
        Request::ReadBrief(p) => query::read_brief(ctx, &p),
        Request::CheckState(p) => query::check_state(ctx, &p),
    }
}

/// Build a reason-form failure report (spec §8.3.1): `reason` and `fix`
/// rows, first line `error`. Exit code is 3 for the retryable lock failures
/// (`E-LOCKED`, `E-INUSE`, `E-FLOCKED`), otherwise 1 (spec §11).
///
/// Extra rows that follow the fix line (`also-locked`, `skipped`, `partial`)
/// are appended by the caller.
pub fn reason_report(
    dctx: &diag::Context,
    command: &'static str,
    object: &str,
    id: &'static str,
    params: Vec<(&'static str, String)>,
) -> Report {
    let rendered = diag::reason(id, &params, dctx);
    let mut report = Report::new(command, object);
    report.kv("reason", vec![rendered.message]);
    report.kv("fix", vec![rendered.fix]);
    report.status = Status::Error;
    report.exit = match id {
        "E-LOCKED" | "E-INUSE" | "E-FLOCKED" => 3,
        _ => 1,
    };
    report
}
