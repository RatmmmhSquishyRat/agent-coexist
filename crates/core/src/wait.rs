//! Optional wait extensions (three extra commands): in-memory snapshot,
//! semantic diff, projection filters and a bounded, cancellable one-shot
//! sampling loop over a single managed file.
//!
//! Behavior contract (self-contained): each wait command takes a baseline
//! snapshot of its single managed file, prints a `waiting  <command>  <object>`
//! line to stderr once the baseline is established, then samples that one file
//! once per second (`WAIT_POLL_MS`) until a change, a Ctrl-C cancellation
//! (exit 130) or a real failure. `wait <PATH>` watches any of the four types;
//! `wait-post <PATH> [--as <nick>]` watches a single post;
//! `wait-state [DIR] [--status <name>]` watches exactly
//! `DIR/state.agent-besties.md`. Without a projection filter any change ends
//! the wait, including byte-only format changes; `--as <nick>` /
//! `--status <name>` end the wait only on semantic changes qualifying for
//! that nick or status. The ending report carries `bytes: changed` and
//! `semantic: changed|unchanged` (plus `created` / `removed` where
//! applicable). This module reuses the
//! base parser (`doc`, `types`), the report rows, the time helpers and the
//! storage read entry `fsx::read_stable_optional`; it owns no renderer, no
//! watcher and no persistent state. Only adjacent admitted snapshots are
//! compared; nothing is synthesized from versions that were never observed.
//! The base eleven commands never depend on this module.
//!
//! Snapshots are compared through borrowed field views of the retained
//! `Document` (original bytes + existing raw-span parse): times are compared
//! as `Time` values and strings as slices; strings are allocated only for
//! values that are actually emitted. No new AST and no parallel document
//! model is built.
//!
//! Report row conventions (consumed by the single CLI renderer):
//!
//! - `bytes` / `semantic` / `created` / `removed` are ordinary key/value rows.
//! - Ordinary warn-level diagnostics of the current admitted snapshot (the
//!   before snapshot stands in when the file was just removed) are emitted as
//!   diagnostic rows with their real anchors and promote the first line to
//!   `warn` (still exit 0). They never invalidate a snapshot. Same-file
//!   `W-STATE-CONFLICT` rows are derived from that snapshot only after a
//!   qualifying hit, using the clock that admitted it.
//! - Entity changes are `Fields` rows with indent 2 and key width 6:
//!   `add` / `remove` carry the entity identity, `update` additionally names
//!   the changed field.
//! - Every field value is a `Text` row: for `add` / `remove` the key is the
//!   field name and the content is the one observed side; for `update` the
//!   keys are `before` / `after`. A missing side or missing field renders as
//!   the literal `none`.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::diag;
use crate::doc::{self, Document};
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::ops::context;
use crate::report::{Report, Status};
use crate::time;
use crate::types;
use crate::Time;

/// Fixed sampling interval for all three wait commands (defined by the wait
/// contract; deliberately not part of the base spec §12 table, which this
/// extension must not modify).
pub const WAIT_POLL_MS: u64 = 1000;

/// Key-column width shared by all entity record rows (`add` / `update` /
/// `remove` / `order`, the longest of which is six columns).
const ENTITY_KEY_WIDTH: usize = 6;

/// What a wait command observes: any of the four collaboration file types,
/// a single post, or a single state file. `Some(nick)` / `Some(status)`
/// restrict which semantic changes end the wait (projection filter); the
/// parse scope and error handling are identical for every filter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Filter {
    /// `agent-coexist wait <PATH>` — any of the four types, no filter.
    Any,
    /// `agent-coexist wait-post <PATH> [--as <nick>]`.
    Post(Option<String>),
    /// `agent-coexist wait-state [DIR] [--status <name>]`.
    State(Option<String>),
}

/// Cooperative cancellation shared between the Ctrl-C handler, the timed
/// wait between samples and the storage read's lock polling. Internal
/// layout is an implementation detail; all methods take `&self`.
pub struct Cancellation {
    flag: AtomicBool,
    signal: Mutex<bool>,
    wake: Condvar,
}

impl Cancellation {
    pub fn new() -> Self {
        Cancellation {
            flag: AtomicBool::new(false),
            signal: Mutex::new(false),
            wake: Condvar::new(),
        }
    }

    /// Request cancellation and wake a blocked timed wait.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.signal.lock() {
            *guard = true;
        }
        self.wake.notify_all();
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    fn flag(&self) -> &AtomicBool {
        &self.flag
    }

    /// Sleep up to `ms` on the monotonic clock, waking early on
    /// cancellation. Returns `true` when cancelled (never busy-polls).
    fn sleep_poll(&self, ms: u64) -> bool {
        let deadline = Instant::now() + Duration::from_millis(ms);
        let mut guard = match self.signal.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        loop {
            if *guard {
                return true;
            }
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            let (next, _) = match self.wake.wait_timeout(guard, deadline - now) {
                Ok(pair) => pair,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard = next;
        }
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

/// Command and display object derived from the filter plus the actual
/// managed file path (state shows its directory). The object comes from the
/// same folded display helpers as the base commands
/// (`ops::context::object_of_path` / `object_of_dir`, spec §8.1.3), so the
/// report first line, the stderr ready line and the diag context can never
/// disagree. Public so the CLI can print the identical
/// `waiting  <command>  <object>` stderr line when the ready callback
/// fires.
pub fn command_object(path: &Path, filter: &Filter) -> (&'static str, String) {
    match filter {
        Filter::Any => ("wait", context::object_of_path(path)),
        Filter::Post(_) => ("wait-post", context::object_of_path(path)),
        Filter::State(_) => ("wait-state", object_dir(path)),
    }
}

/// One-shot wait: sample `path` (always the actual managed file) until a
/// qualified transition is observed, then return the complete change report.
///
/// The parent directory must exist and be a directory. The first valid
/// snapshot is the baseline (an absent file is a legitimate `Absent`
/// baseline and is never created); `ready` is called exactly once after the
/// baseline is admitted. Invalid snapshots fail the wait instead of
/// replacing the baseline: present-file diagnostic refusals (storage
/// refusals before a Document exists, blocking parse, missing kind,
/// unreliable identities) return the check-shaped error report — file
/// row, type row when that same snapshot determines kind, then
/// diagnostics, exit 1 — never a hit or `Absent`; wrong type
/// for a specialized command stays `E-TYPE`. Unqualified but valid snapshots
/// advance the whole baseline. Cancellation returns `Fault::Cancelled`
/// (CLI exit 130). No lock is held while idle.
pub fn run(
    path: &Path,
    filter: Filter,
    cancel: &Cancellation,
    ready: impl FnOnce(),
) -> Result<Report, Fault> {
    let now = time::now()?;
    ensure_parent_dir(path, &filter)?;

    let mut baseline = match sample_raw(path, &filter, cancel)? {
        Ok(Raw::Absent) => None,
        Ok(Raw::Present(bytes)) => match admit_present(bytes, path, &filter, now)? {
            Ok(parsed) => Some(parsed),
            Err(report) => return Ok(report),
        },
        Err(report) => return Ok(report),
    };
    ready();

    loop {
        if cancel.sleep_poll(WAIT_POLL_MS) || cancel.is_cancelled() {
            return Err(Fault::Cancelled);
        }
        let raw = match sample_raw(path, &filter, cancel)? {
            Ok(raw) => raw,
            Err(report) => return Ok(report),
        };
        // Same bytes are never re-parsed (cheap pre-check against the
        // baseline's retained original bytes).
        let unchanged = match (&baseline, &raw) {
            (None, Raw::Absent) => true,
            (Some(before), Raw::Present(bytes)) => before.source.as_bytes() == bytes.as_slice(),
            _ => false,
        };
        if unchanged {
            continue;
        }
        let next = match raw {
            Raw::Absent => None,
            Raw::Present(bytes) => match admit_present(bytes, path, &filter, now)? {
                Ok(parsed) => Some(parsed),
                Err(report) => return Ok(report),
            },
        };
        // The transition is decided between the two admitted snapshots
        // (identical retained bytes ⇒ no transition at all).
        let changed = match (&baseline, &next) {
            (None, None) => false,
            (Some(before), Some(after)) => before.source != after.source,
            _ => true,
        };
        if changed {
            let before_model = baseline.as_ref().map(extract);
            let after_model = next.as_ref().map(extract);
            if let Some(report) = evaluate(&before_model, &after_model, &filter, path, now) {
                return Ok(report);
            }
        }
        baseline = next;
    }
}

// ---------------------------------------------------------------------------
// Sampling and snapshot validity
// ---------------------------------------------------------------------------

enum Raw {
    Absent,
    Present(Vec<u8>),
}

fn read_raw(path: &Path, cancel: &Cancellation) -> Result<Raw, Fault> {
    match fsx::read_stable_optional(path, Some(cancel.flag())) {
        Ok(Some(bytes)) => Ok(Raw::Present(bytes)),
        Ok(None) => Ok(Raw::Absent),
        Err(fault) => Err(if cancel.is_cancelled() {
            Fault::Cancelled
        } else {
            fault
        }),
    }
}

/// Common raw-read / admission boundary: storage-produced diagnostic
/// refusals become the check-shaped blocking-present report. Kind stays
/// unset because the rejected sample was never parsed. Absence, busy,
/// cancellation and I/O remain faults.
fn sample_raw(
    path: &Path,
    filter: &Filter,
    cancel: &Cancellation,
) -> Result<Result<Raw, Report>, Fault> {
    match read_raw(path, cancel) {
        Ok(raw) => Ok(Ok(raw)),
        Err(Fault::Diagnostics(diagnostics)) => Ok(Err(blocking_present_report(
            path,
            filter,
            None,
            diagnostics,
        ))),
        Err(fault) => Err(fault),
    }
}

fn ensure_parent_dir(path: &Path, filter: &Filter) -> Result<(), Fault> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    // Folded display params, the same convention the base DIR commands use
    // for these two templates (spec §8.1.3–§8.1.4): an unfolded value with
    // two consecutive spaces would break the two-space column split.
    let object = context::object_of_dir(parent);
    match std::fs::metadata(parent) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => match filter {
            // wait-state takes DIR: keep catalog E-NOT-DIR (re-run against a
            // directory). wait / wait-post take PATH: that recovery would
            // rewrite the object (e.g. `wait-post .`) and remain invalid.
            Filter::State(_) => {
                let (command, _) = command_object(path, filter);
                Err(Fault::reason("E-NOT-DIR")
                    .with("path", object)
                    .with("cmd", command)
                    .with(
                        "parent_arg",
                        context::parent_dir(parent).to_string_lossy().into_owned(),
                    ))
            }
            Filter::Any | Filter::Post(_) => Err(Fault::io(
                "read",
                parent.to_path_buf(),
                std::io::Error::new(std::io::ErrorKind::NotADirectory, "not a directory"),
            )),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(Fault::reason("E-DIR-MISSING").with("dir", object).with(
                "parent_arg",
                context::parent_dir(parent).to_string_lossy().into_owned(),
            ))
        }
        Err(error) => Err(Fault::io("read", parent.to_path_buf(), error)),
    }
}

/// A snapshot becomes a baseline (or a compared sample) only when it is
/// reliably comparable: no blocking diagnostics, a known type, the type the
/// command requires, and unique identities. Ordinary warnings are retained
/// (and reported on a hit) but never invalidate a snapshot.
fn validate(document: &Document, filter: &Filter, path: &Path) -> Result<(), Fault> {
    let blocking = document.blocking();
    if !blocking.is_empty() {
        return Err(Fault::Diagnostics(blocking));
    }
    let kind = match document.kind.as_ref().map(|kind| kind.as_str()) {
        Some(kind) => kind,
        None => return Err(Fault::Diagnostics(document.diagnostics.clone())),
    };
    let expected = match filter {
        Filter::Any => None,
        Filter::Post(_) => Some("post"),
        Filter::State(_) => Some("state"),
    };
    if let Some(expected) = expected {
        if kind != expected {
            let mut fault = Fault::reason("E-TYPE")
                .with("actual", kind)
                .with("expected", expected);
            if kind == "state" {
                fault = fault.with(
                    "dir_arg",
                    context::parent_dir(path).to_string_lossy().into_owned(),
                );
            } else {
                let cmd = match kind {
                    "post" => "post",
                    "brief" => "brief",
                    _ => "profile",
                };
                fault = fault.with("cmd_for_actual", cmd);
            }
            return Err(fault);
        }
    }
    check_duplicate_state_identities(document, path)
}

/// Parse and validate one present sample. Diagnostic refusals become the
/// check-shaped error report from this snapshot (no extra read to recover
/// kind). `E-TYPE` and I/O stay `Fault`.
fn admit_present(
    bytes: Vec<u8>,
    path: &Path,
    filter: &Filter,
    now: Time,
) -> Result<Result<Document, Report>, Fault> {
    let document = match doc::parse(bytes, path, now) {
        Ok(document) => document,
        Err(Fault::Diagnostics(diagnostics)) => {
            return Ok(Err(blocking_present_report(
                path,
                filter,
                None,
                diagnostics,
            )));
        }
        Err(fault) => return Err(fault),
    };
    match validate(&document, filter, path) {
        Ok(()) => Ok(Ok(document)),
        Err(Fault::Diagnostics(diagnostics)) => Ok(Err(blocking_present_report(
            path,
            filter,
            document.kind,
            diagnostics,
        ))),
        Err(fault) => Err(fault),
    }
}

/// Spec §8.3.2 form reused by wait: file row always, type row when `kind`
/// came from the same snapshot, then diagnostics, first line error, exit 1.
fn blocking_present_report(
    path: &Path,
    filter: &Filter,
    kind: Option<doc::Kind>,
    mut diagnostics: Vec<Diagnostic>,
) -> Report {
    let (command, object) = command_object(path, filter);
    let mut report = Report::new(command, object);
    report.kv("file", vec![context::display_file(path)]);
    if let Some(kind) = kind {
        report.kv("type", vec![kind.as_str().to_string()]);
    }
    diagnostics.sort_by_key(|diagnostic| (diagnostic.line, diag::order(diagnostic.id)));
    for diagnostic in diagnostics {
        report.diagnostic(diagnostic);
    }
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// Wait-specific reliability threshold: `(status, nick)` must be unique so
/// declarations have unambiguous identities. Base parsing tolerates
/// duplicates (latest `until` wins, `W-STATE-DUP`); wait rejects the
/// snapshot instead of guessing which declaration moved.
fn check_duplicate_state_identities(document: &Document, path: &Path) -> Result<(), Fault> {
    if document.kind.as_ref().map(|kind| kind.as_str()) != Some("state") {
        return Ok(());
    }
    for section in &document.state {
        let mut seen = HashSet::with_capacity(section.entries.len());
        for declaration in &section.entries {
            if !seen.insert(declaration.nick.as_str()) {
                let mut diagnostics: Vec<Diagnostic> = document
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.id == "W-STATE-DUP")
                    .cloned()
                    .collect();
                if diagnostics.is_empty() {
                    diagnostics.push(
                        Diagnostic::new("W-STATE-DUP", declaration.line)
                            .with("nick", declaration.nick.clone())
                            .with("status", section.status.clone())
                            .with(
                                "dir_arg",
                                context::parent_dir(path).to_string_lossy().into_owned(),
                            ),
                    );
                }
                return Err(Fault::Diagnostics(diagnostics));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Borrowed field views over one admitted parse
// ---------------------------------------------------------------------------

struct CommentView<'a> {
    seq: u64,
    nick: &'a str,
    created: Time,
    edited: Option<Time>,
    body: Option<&'a str>,
    mentions: &'a [String],
    replies: &'a [u64],
}

struct PostView<'a> {
    comments: Vec<CommentView<'a>>,
    subscribers: &'a [String],
    comments_by_seq: HashMap<u64, usize>,
    subscriber_nicks: HashSet<&'a str>,
}

struct EntryView<'a> {
    title: &'a str,
    path: Option<&'a str>,
    hash: Option<&'a str>,
    regex: Option<&'a str>,
    note: Option<&'a str>,
}

struct BriefView<'a> {
    description: Option<&'a str>,
    entries: Vec<EntryView<'a>>,
    entries_by_title: HashMap<&'a str, usize>,
}

struct SectionView<'a> {
    name: &'a str,
    body: Option<&'a str>,
}

struct ProfileView<'a> {
    sections: Vec<SectionView<'a>>,
    sections_by_name: HashMap<&'a str, usize>,
}

struct DeclarationView<'a> {
    nick: &'a str,
    start: Time,
    until: Time,
}

struct StatusView<'a> {
    status: &'a str,
    declarations: Vec<DeclarationView<'a>>,
    declarations_by_nick: HashMap<&'a str, usize>,
}

struct StateView<'a> {
    sections: Vec<StatusView<'a>>,
    sections_by_status: HashMap<&'a str, usize>,
}

enum View<'a> {
    Post(PostView<'a>),
    Brief(BriefView<'a>),
    Profile(ProfileView<'a>),
    State(StateView<'a>),
}

struct Model<'a> {
    name: Option<&'a str>,
    kind: &'static str,
    view: View<'a>,
    document: &'a Document,
}

impl Model<'_> {
    fn as_post(&self) -> Option<&PostView<'_>> {
        match &self.view {
            View::Post(post) => Some(post),
            _ => None,
        }
    }

    fn as_state(&self) -> Option<&StateView<'_>> {
        match &self.view {
            View::State(state) => Some(state),
            _ => None,
        }
    }
}

impl PostView<'_> {
    fn comment(&self, seq: u64) -> Option<&CommentView<'_>> {
        self.comments_by_seq
            .get(&seq)
            .map(|&index| &self.comments[index])
    }

    fn has_subscriber(&self, nick: &str) -> bool {
        self.subscriber_nicks.contains(nick)
    }
}

impl BriefView<'_> {
    fn entry(&self, title: &str) -> Option<&EntryView<'_>> {
        self.entries_by_title
            .get(title)
            .map(|&index| &self.entries[index])
    }
}

impl ProfileView<'_> {
    fn section(&self, name: &str) -> Option<&SectionView<'_>> {
        self.sections_by_name
            .get(name)
            .map(|&index| &self.sections[index])
    }
}

impl StatusView<'_> {
    fn declaration(&self, nick: &str) -> Option<&DeclarationView<'_>> {
        self.declarations_by_nick
            .get(nick)
            .map(|&index| &self.declarations[index])
    }
}

impl StateView<'_> {
    fn find(&self, status: &str) -> Option<&StatusView<'_>> {
        self.sections_by_status
            .get(status)
            .map(|&index| &self.sections[index])
    }
}

/// First-occurrence indexes matching `.find()` on the ordered vectors that
/// remain the emission and order authority. Duplicate keys cannot appear in
/// an admitted wait snapshot (blocking `E-*-DUP` plus wait's unique
/// `(status, nick)` check).
fn first_positions<K: Eq + Hash>(
    len: usize,
    keys: impl IntoIterator<Item = K>,
) -> HashMap<K, usize> {
    let mut map = HashMap::with_capacity(len);
    for (index, key) in keys.into_iter().enumerate() {
        map.entry(key).or_insert(index);
    }
    map
}

/// Borrow the compared field values of one admitted parse. The retained
/// `Document` is borrowed for field views and, after a qualifying hit, for
/// ordinary plus on-demand conflict diagnostics. Nothing is copied and no
/// time is formatted here.
fn extract(document: &Document) -> Model<'_> {
    let kind = document
        .kind
        .as_ref()
        .map(|kind| kind.as_str())
        .unwrap_or("");
    let view = match kind {
        "post" => View::Post(extract_post(document)),
        "brief" => View::Brief(extract_brief(document)),
        "agent-profile" => View::Profile(extract_profile(document)),
        "state" => View::State(extract_state(document)),
        other => unreachable!("snapshot validated before extraction, got kind {:?}", other),
    };
    Model {
        name: document.name.as_deref(),
        kind,
        view,
        document,
    }
}

fn extract_post(document: &Document) -> PostView<'_> {
    let comments: Vec<CommentView<'_>> = document
        .post
        .comments
        .iter()
        .map(|comment| CommentView {
            seq: comment.seq,
            nick: comment.nick.as_str(),
            created: comment.created,
            edited: comment.edited,
            body: document.body(comment.section),
            mentions: comment.mentions.as_slice(),
            replies: comment.replies.as_slice(),
        })
        .collect();
    PostView {
        comments_by_seq: first_positions(
            comments.len(),
            comments.iter().map(|comment| comment.seq),
        ),
        subscriber_nicks: document
            .post
            .subscribers
            .iter()
            .map(|nick| nick.as_str())
            .collect(),
        subscribers: document.post.subscribers.as_slice(),
        comments,
    }
}

fn extract_brief(document: &Document) -> BriefView<'_> {
    let entries: Vec<EntryView<'_>> = document
        .brief
        .entries
        .iter()
        .map(|entry| EntryView {
            title: entry.title.as_str(),
            path: entry.path.as_ref().map(|property| property.value.as_str()),
            hash: entry.hash.as_ref().map(|property| property.value.as_str()),
            regex: entry.regex.as_ref().map(|property| property.value.as_str()),
            note: document.body(entry.section),
        })
        .collect();
    BriefView {
        description: document
            .brief
            .description
            .as_ref()
            .map(|body| document.text(&body.content)),
        entries_by_title: first_positions(entries.len(), entries.iter().map(|entry| entry.title)),
        entries,
    }
}

fn extract_profile(document: &Document) -> ProfileView<'_> {
    let sections: Vec<SectionView<'_>> = document
        .profile
        .iter()
        .map(|section| SectionView {
            name: section.name.as_str(),
            body: document.body(section.section),
        })
        .collect();
    ProfileView {
        sections_by_name: first_positions(
            sections.len(),
            sections.iter().map(|section| section.name),
        ),
        sections,
    }
}

fn extract_state(document: &Document) -> StateView<'_> {
    let sections: Vec<StatusView<'_>> = document
        .state
        .iter()
        .map(|section| {
            let declarations: Vec<DeclarationView<'_>> = section
                .entries
                .iter()
                .map(|declaration| DeclarationView {
                    nick: declaration.nick.as_str(),
                    start: declaration.start,
                    until: declaration.until,
                })
                .collect();
            StatusView {
                status: section.status.as_str(),
                declarations_by_nick: first_positions(
                    declarations.len(),
                    declarations.iter().map(|declaration| declaration.nick),
                ),
                declarations,
            }
        })
        .collect();
    StateView {
        sections_by_status: first_positions(
            sections.len(),
            sections.iter().map(|section| section.status),
        ),
        sections,
    }
}

// ---------------------------------------------------------------------------
// Transition evaluation and hit decision
// ---------------------------------------------------------------------------

fn evaluate(
    before: &Option<Model<'_>>,
    after: &Option<Model<'_>>,
    filter: &Filter,
    path: &Path,
    now: Time,
) -> Option<Report> {
    let hit = match filter {
        Filter::Any | Filter::Post(None) | Filter::State(None) => true,
        Filter::Post(Some(nick)) => post_hit(before, after, nick),
        Filter::State(Some(status)) => state_hit(before, after, status),
    };
    if hit {
        Some(build_report(path, filter, before, after, now))
    } else {
        None
    }
}

/// Post projection: a message qualifies when it is relevant in the after
/// snapshot (direct `@nick`, or `@#n` whose author *in that snapshot* is
/// nick — self-mentions and self-replies included) and it is new, has
/// edited parsed fields, or became relevant since the before snapshot.
/// Deletions, subscriber changes and pure reorders never qualify.
fn post_hit(before: &Option<Model<'_>>, after: &Option<Model<'_>>, nick: &str) -> bool {
    let after_post = match after.as_ref().and_then(Model::as_post) {
        Some(post) => post,
        None => return false,
    };
    let before_post = before.as_ref().and_then(Model::as_post);
    for comment in &after_post.comments {
        if !relevant(comment, after_post, nick) {
            continue;
        }
        let previous = before_post.and_then(|post| post.comment(comment.seq));
        match previous {
            None => return true,
            Some(previous) => {
                if comment_fields_differ(previous, comment) {
                    return true;
                }
                let was_relevant = before_post
                    .map(|post| relevant(previous, post, nick))
                    .unwrap_or(false);
                if !was_relevant {
                    return true;
                }
            }
        }
    }
    false
}

/// Whether `comment` is relevant to `nick` within `post`'s own snapshot:
/// a direct mention, or a reply whose target author is nick. Purely derived
/// from the base-parsed mentions/replies; allocation-free.
fn relevant(comment: &CommentView<'_>, post: &PostView<'_>, nick: &str) -> bool {
    if comment
        .mentions
        .iter()
        .any(|mention| mention.as_str() == nick)
    {
        return true;
    }
    comment.replies.iter().any(|seq| {
        post.comment(*seq)
            .map(|target| target.nick == nick)
            .unwrap_or(false)
    })
}

/// The reference that makes `comment` relevant (for emission only): the
/// direct mention, else the first reply whose target author is nick.
fn relevance_source(comment: &CommentView<'_>, post: &PostView<'_>, nick: &str) -> Option<String> {
    if comment
        .mentions
        .iter()
        .any(|mention| mention.as_str() == nick)
    {
        return Some(format!("@{}", nick));
    }
    comment
        .replies
        .iter()
        .find(|seq| {
            post.comment(**seq)
                .map(|target| target.nick == nick)
                .unwrap_or(false)
        })
        .map(|seq| format!("@#{}", seq))
}

fn comment_fields_differ(before: &CommentView<'_>, after: &CommentView<'_>) -> bool {
    before.nick != after.nick
        || before.created != after.created
        || before.edited != after.edited
        || before.body != after.body
}

/// State projection: additions, removals and `start`/`until` updates of
/// declarations under the selected status qualify (renewal included).
/// Expired-but-stored declarations are compared like any other; clock
/// expiry alone never matches. File deletion hits exactly when the before
/// snapshot had declarations under the selected status.
fn state_hit(before: &Option<Model<'_>>, after: &Option<Model<'_>>, status: &str) -> bool {
    let before_section = before
        .as_ref()
        .and_then(Model::as_state)
        .and_then(|state| state.find(status));
    let after_section = after
        .as_ref()
        .and_then(Model::as_state)
        .and_then(|state| state.find(status));
    let before_declarations = before_section
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    let after_declarations = after_section
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    for declaration in after_declarations {
        match before_section.and_then(|section| section.declaration(declaration.nick)) {
            None => return true,
            Some(previous) => {
                if previous.start != declaration.start || previous.until != declaration.until {
                    return true;
                }
            }
        }
    }
    for declaration in before_declarations {
        if after_section
            .and_then(|section| section.declaration(declaration.nick))
            .is_none()
        {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Report construction
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Side {
    Before,
    After,
}

impl Side {
    fn verb(self) -> &'static str {
        match self {
            Side::After => "add",
            Side::Before => "remove",
        }
    }
}

struct Block {
    verb: &'static str,
    entity: String,
    identity: Vec<String>,
    field: Option<&'static str>,
    field_values: Vec<(&'static str, String)>,
    before: Option<String>,
    after: Option<String>,
}

struct OrderRow {
    list: String,
    before: String,
    after: String,
}

fn none_str(value: Option<&str>) -> String {
    value.unwrap_or("none").to_string()
}

fn none_time(value: Option<Time>) -> String {
    value
        .map(time::format)
        .unwrap_or_else(|| "none".to_string())
}

fn update_block(
    entity: &str,
    identity: &[String],
    field: Option<&'static str>,
    before: String,
    after: String,
) -> Block {
    Block {
        verb: "update",
        entity: entity.to_string(),
        identity: identity.to_vec(),
        field,
        field_values: Vec::new(),
        before: Some(before),
        after: Some(after),
    }
}

fn comment_block(side: Side, comment: &CommentView<'_>) -> Block {
    Block {
        verb: side.verb(),
        entity: "comment".to_string(),
        identity: vec![format!("#{}", comment.seq)],
        field: None,
        field_values: vec![
            ("author", comment.nick.to_string()),
            ("created", time::format(comment.created)),
            ("edited", none_time(comment.edited)),
            ("body", none_str(comment.body)),
        ],
        before: None,
        after: None,
    }
}

fn subscriber_block(side: Side, nick: &str) -> Block {
    Block {
        verb: side.verb(),
        entity: "subscriber".to_string(),
        identity: vec![nick.to_string()],
        field: None,
        field_values: Vec::new(),
        before: None,
        after: None,
    }
}

fn entry_block(side: Side, entry: &EntryView<'_>) -> Block {
    Block {
        verb: side.verb(),
        entity: "entry".to_string(),
        identity: vec![names::display_text(entry.title)],
        field: None,
        field_values: vec![
            ("path", none_str(entry.path)),
            ("hash", none_str(entry.hash)),
            ("regex", none_str(entry.regex)),
            ("note", none_str(entry.note)),
        ],
        before: None,
        after: None,
    }
}

fn section_block(side: Side, section: &SectionView<'_>) -> Block {
    Block {
        verb: side.verb(),
        entity: "section".to_string(),
        identity: vec![names::display_text(section.name)],
        field: None,
        field_values: vec![("body", none_str(section.body))],
        before: None,
        after: None,
    }
}

fn status_title(status: &str) -> String {
    names::display_text(status)
}

fn status_block(side: Side, status: &str) -> Block {
    Block {
        verb: side.verb(),
        entity: "status".to_string(),
        identity: vec![status_title(status)],
        field: None,
        field_values: Vec::new(),
        before: None,
        after: None,
    }
}

fn declaration_block(side: Side, status: &str, declaration: &DeclarationView<'_>) -> Block {
    Block {
        verb: side.verb(),
        entity: status_title(status),
        identity: vec![declaration.nick.to_string()],
        field: None,
        field_values: vec![
            ("start", time::format(declaration.start)),
            ("until", time::format(declaration.until)),
        ],
        before: None,
        after: None,
    }
}

fn emit_block(report: &mut Report, block: Block) {
    let mut values = vec![block.entity];
    values.extend(block.identity);
    if let Some(field) = block.field {
        values.push(field.to_string());
    }
    report.record(block.verb, values, 2, ENTITY_KEY_WIDTH);
    for (key, value) in block.field_values {
        report.text(key, value);
    }
    if let Some(before) = block.before {
        report.text("before", before);
    }
    if let Some(after) = block.after {
        report.text("after", after);
    }
}

fn emit_order_row(report: &mut Report, row: OrderRow) {
    report.record("order", vec![row.list], 2, ENTITY_KEY_WIDTH);
    report.text("before", row.before);
    report.text("after", row.after);
}

/// Warn-level diagnostics of one admitted snapshot, including same-file
/// locked conflicts derived only here (after a qualifying hit). `now` is
/// the clock that admitted this snapshot, not a later emission clock.
fn snapshot_warnings(model: &Model<'_>, now: Time) -> Vec<Diagnostic> {
    let mut diagnostics = model.document.diagnostics.clone();
    if matches!(model.view, View::State(_)) {
        let selected = types::state::selected(&model.document.state);
        types::state::append_conflict_diagnostics(&selected, now, &mut diagnostics);
        diagnostics.sort_by_key(|diagnostic| (diagnostic.line, diag::order(diagnostic.id)));
    }
    diagnostics
}

fn build_report(
    path: &Path,
    filter: &Filter,
    before: &Option<Model<'_>>,
    after: &Option<Model<'_>>,
    now: Time,
) -> Report {
    let created = before.is_none() && after.is_some();
    let removed = before.is_some() && after.is_none();
    let semantic = match (before.as_ref(), after.as_ref()) {
        (Some(before_model), Some(after_model)) => semantic_changed(before_model, after_model),
        _ => true,
    };
    let (command, object) = command_object(path, filter);
    let mut report = Report::new(command, object);
    report.kv("bytes", vec!["changed".to_string()]);
    let semantic_value = if semantic { "changed" } else { "unchanged" };
    report.kv("semantic", vec![semantic_value.to_string()]);
    if created {
        report.kv("created", vec![context::display_file(path)]);
    }
    if removed {
        report.kv("removed", vec![context::display_file(path)]);
    }
    // Ordinary warn-level diagnostics of the current admitted snapshot (the
    // before snapshot stands in when the file was just removed), with their
    // real anchors. Error-level diagnostics of an admitted snapshot do not
    // exist by construction (blocking ones failed validation; the remaining
    // non-blocking E-* belong to `check`, not to a change report).
    let warnings = match after.as_ref() {
        Some(model) => snapshot_warnings(model, now),
        None => before
            .as_ref()
            .map(|model| snapshot_warnings(model, now))
            .unwrap_or_default(),
    };
    let mut has_warnings = false;
    for diagnostic in warnings
        .into_iter()
        .filter(|diagnostic| !diag::is_error(diagnostic.id))
    {
        report.diagnostic(diagnostic);
        has_warnings = true;
    }
    if has_warnings {
        report.warn();
    }
    match filter {
        Filter::Post(Some(nick)) => {
            let before_post = before.as_ref().and_then(Model::as_post);
            if let Some(after_post) = after.as_ref().and_then(Model::as_post) {
                for block in post_qualified_blocks(before_post, after_post, nick) {
                    emit_block(&mut report, block);
                }
            }
        }
        Filter::State(Some(status)) => {
            let before_state = before.as_ref().and_then(Model::as_state);
            let after_state = after.as_ref().and_then(Model::as_state);
            for block in state_qualified_blocks(before_state, after_state, status) {
                emit_block(&mut report, block);
            }
        }
        _ => {
            for block in doc_blocks(before, after) {
                emit_block(&mut report, block);
            }
            for block in entity_blocks(before, after) {
                emit_block(&mut report, block);
            }
            for row in order_rows(before, after) {
                emit_order_row(&mut report, row);
            }
        }
    }
    report
}

/// Whether the two admitted snapshots differ in any parsed field, identity,
/// or observable order. Allocation-free: this only decides the `semantic`
/// row; strings are allocated later for the rows that are actually emitted.
fn semantic_changed(before: &Model<'_>, after: &Model<'_>) -> bool {
    if before.name != after.name || before.kind != after.kind {
        return true;
    }
    match (&before.view, &after.view) {
        (View::Post(before_post), View::Post(after_post)) => post_changed(before_post, after_post),
        (View::Brief(before_brief), View::Brief(after_brief)) => {
            brief_changed(before_brief, after_brief)
        }
        (View::Profile(before_profile), View::Profile(after_profile)) => {
            profile_changed(before_profile, after_profile)
        }
        (View::State(before_state), View::State(after_state)) => {
            state_changed(before_state, after_state)
        }
        _ => true,
    }
}

fn post_changed(before: &PostView<'_>, after: &PostView<'_>) -> bool {
    if before.comments.len() != after.comments.len() || before.subscribers != after.subscribers {
        return true;
    }
    for (previous, current) in before.comments.iter().zip(after.comments.iter()) {
        if previous.seq != current.seq || comment_fields_differ(previous, current) {
            return true;
        }
    }
    false
}

fn brief_changed(before: &BriefView<'_>, after: &BriefView<'_>) -> bool {
    if before.description != after.description || before.entries.len() != after.entries.len() {
        return true;
    }
    for (previous, current) in before.entries.iter().zip(after.entries.iter()) {
        if previous.title != current.title
            || previous.path != current.path
            || previous.hash != current.hash
            || previous.regex != current.regex
            || previous.note != current.note
        {
            return true;
        }
    }
    false
}

fn profile_changed(before: &ProfileView<'_>, after: &ProfileView<'_>) -> bool {
    if before.sections.len() != after.sections.len() {
        return true;
    }
    for (previous, current) in before.sections.iter().zip(after.sections.iter()) {
        if previous.name != current.name || previous.body != current.body {
            return true;
        }
    }
    false
}

fn state_changed(before: &StateView<'_>, after: &StateView<'_>) -> bool {
    if before.sections.len() != after.sections.len() {
        return true;
    }
    for (previous, current) in before.sections.iter().zip(after.sections.iter()) {
        if previous.status != current.status
            || previous.declarations.len() != current.declarations.len()
        {
            return true;
        }
        for (old, new) in previous
            .declarations
            .iter()
            .zip(current.declarations.iter())
        {
            if old.nick != new.nick || old.start != new.start || old.until != new.until {
                return true;
            }
        }
    }
    false
}

/// Brief description of one snapshot side. A missing snapshot or a
/// non-brief document contributes no description, so the existing
/// `none` comparison covers creation, deletion and legal type switches
/// without a second description model.
fn brief_description<'a>(model: Option<&Model<'a>>) -> Option<&'a str> {
    match model.map(|model| &model.view) {
        Some(View::Brief(brief)) => brief.description,
        _ => None,
    }
}

/// Document-level field updates (H1 name/type and the brief description).
/// Name and type compare two present snapshots. Description compares each
/// side's brief description with the same `none` field values already used
/// for brief→brief updates.
fn doc_blocks(before: &Option<Model<'_>>, after: &Option<Model<'_>>) -> Vec<Block> {
    let mut blocks = Vec::new();
    if let (Some(before_model), Some(after_model)) = (before.as_ref(), after.as_ref()) {
        if before_model.name != after_model.name {
            blocks.push(update_block(
                "document",
                &[],
                Some("name"),
                none_str(before_model.name),
                none_str(after_model.name),
            ));
        }
        if before_model.kind != after_model.kind {
            blocks.push(update_block(
                "document",
                &[],
                Some("type"),
                before_model.kind.to_string(),
                after_model.kind.to_string(),
            ));
        }
    }
    let before_description = brief_description(before.as_ref());
    let after_description = brief_description(after.as_ref());
    if before_description != after_description {
        blocks.push(update_block(
            "description",
            &[],
            None,
            none_str(before_description),
            none_str(after_description),
        ));
    }
    blocks
}

/// Entity blocks in report order: additions and updates follow the after
/// snapshot, removals the before snapshot. A legal type switch (generic
/// `wait` only) is reported as new-type additions plus old-type removals
/// with no cross-type pairing.
fn entity_blocks(before: &Option<Model<'_>>, after: &Option<Model<'_>>) -> Vec<Block> {
    match (before.as_ref(), after.as_ref()) {
        (None, None) => Vec::new(),
        (None, Some(after_model)) => all_blocks(after_model, Side::After),
        (Some(before_model), None) => all_blocks(before_model, Side::Before),
        (Some(before_model), Some(after_model)) => {
            if before_model.kind != after_model.kind {
                let mut blocks = all_blocks(after_model, Side::After);
                blocks.extend(all_blocks(before_model, Side::Before));
                blocks
            } else {
                match (&after_model.view, &before_model.view) {
                    (View::Post(after_post), View::Post(before_post)) => {
                        post_blocks(Some(before_post), after_post)
                    }
                    (View::Brief(after_brief), View::Brief(before_brief)) => {
                        brief_blocks(Some(before_brief), after_brief)
                    }
                    (View::Profile(after_profile), View::Profile(before_profile)) => {
                        profile_blocks(Some(before_profile), after_profile)
                    }
                    (View::State(after_state), View::State(before_state)) => {
                        state_blocks(Some(before_state), after_state)
                    }
                    _ => Vec::new(),
                }
            }
        }
    }
}

/// Meaningful order changes, last in the report: only for pure permutations
/// of the same identity set (never fabricated alongside add/remove).
fn order_rows(before: &Option<Model<'_>>, after: &Option<Model<'_>>) -> Vec<OrderRow> {
    match (before.as_ref(), after.as_ref()) {
        (Some(before_model), Some(after_model)) if before_model.kind == after_model.kind => {
            match (&after_model.view, &before_model.view) {
                (View::Post(after_post), View::Post(before_post)) => {
                    post_order_rows(Some(before_post), after_post)
                }
                (View::Brief(after_brief), View::Brief(before_brief)) => {
                    brief_order_rows(Some(before_brief), after_brief)
                }
                (View::Profile(after_profile), View::Profile(before_profile)) => {
                    profile_order_rows(Some(before_profile), after_profile)
                }
                (View::State(after_state), View::State(before_state)) => {
                    state_order_rows(Some(before_state), after_state)
                }
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

fn all_blocks(model: &Model<'_>, side: Side) -> Vec<Block> {
    match &model.view {
        View::Post(post) => {
            let mut blocks: Vec<Block> = post
                .comments
                .iter()
                .map(|comment| comment_block(side, comment))
                .collect();
            blocks.extend(
                post.subscribers
                    .iter()
                    .map(|nick| subscriber_block(side, nick)),
            );
            blocks
        }
        View::Brief(brief) => brief_all_blocks(brief, side),
        View::Profile(profile) => profile_all_blocks(profile, side),
        View::State(state) => state_all_blocks(state, side),
    }
}

fn brief_all_blocks(brief: &BriefView<'_>, side: Side) -> Vec<Block> {
    brief
        .entries
        .iter()
        .map(|entry| entry_block(side, entry))
        .collect()
}

fn profile_all_blocks(profile: &ProfileView<'_>, side: Side) -> Vec<Block> {
    profile
        .sections
        .iter()
        .map(|section| section_block(side, section))
        .collect()
}

fn state_all_blocks(state: &StateView<'_>, side: Side) -> Vec<Block> {
    let mut blocks = Vec::new();
    for section in &state.sections {
        blocks.push(status_block(side, section.status));
        blocks.extend(
            section
                .declarations
                .iter()
                .map(|declaration| declaration_block(side, section.status, declaration)),
        );
    }
    blocks
}

fn post_blocks(before: Option<&PostView<'_>>, after: &PostView<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for comment in &after.comments {
        match before.and_then(|post| post.comment(comment.seq)) {
            None => blocks.push(comment_block(Side::After, comment)),
            Some(previous) => {
                let identity = vec![format!("#{}", comment.seq)];
                if previous.nick != comment.nick {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("author"),
                        previous.nick.to_string(),
                        comment.nick.to_string(),
                    ));
                }
                if previous.created != comment.created {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("created"),
                        time::format(previous.created),
                        time::format(comment.created),
                    ));
                }
                if previous.edited != comment.edited {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("edited"),
                        none_time(previous.edited),
                        none_time(comment.edited),
                    ));
                }
                if previous.body != comment.body {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("body"),
                        none_str(previous.body),
                        none_str(comment.body),
                    ));
                }
            }
        }
    }
    for nick in after.subscribers {
        if !before
            .map(|post| post.has_subscriber(nick))
            .unwrap_or(false)
        {
            blocks.push(subscriber_block(Side::After, nick));
        }
    }
    if let Some(before) = before {
        for comment in &before.comments {
            if after.comment(comment.seq).is_none() {
                blocks.push(comment_block(Side::Before, comment));
            }
        }
        for nick in before.subscribers {
            if !after.has_subscriber(nick) {
                blocks.push(subscriber_block(Side::Before, nick));
            }
        }
    }
    blocks
}

fn post_order_rows(before: Option<&PostView<'_>>, after: &PostView<'_>) -> Vec<OrderRow> {
    let mut rows = Vec::new();
    if let Some(before) = before {
        let before_seqs: Vec<u64> = before.comments.iter().map(|comment| comment.seq).collect();
        let after_seqs: Vec<u64> = after.comments.iter().map(|comment| comment.seq).collect();
        if same_set_different_order(&before_seqs, &after_seqs) {
            rows.push(OrderRow {
                list: "comments".to_string(),
                before: join_seqs(&before_seqs),
                after: join_seqs(&after_seqs),
            });
        }
        if same_set_different_order(before.subscribers, after.subscribers) {
            rows.push(OrderRow {
                list: "subscribers".to_string(),
                before: before.subscribers.join(", "),
                after: after.subscribers.join(", "),
            });
        }
    }
    rows
}

fn brief_blocks(before: Option<&BriefView<'_>>, after: &BriefView<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for entry in &after.entries {
        match before.and_then(|brief| brief.entry(entry.title)) {
            None => blocks.push(entry_block(Side::After, entry)),
            Some(previous) => {
                let identity = vec![names::display_text(entry.title)];
                if previous.path != entry.path {
                    blocks.push(update_block(
                        "entry",
                        &identity,
                        Some("path"),
                        none_str(previous.path),
                        none_str(entry.path),
                    ));
                }
                if previous.hash != entry.hash {
                    blocks.push(update_block(
                        "entry",
                        &identity,
                        Some("hash"),
                        none_str(previous.hash),
                        none_str(entry.hash),
                    ));
                }
                if previous.regex != entry.regex {
                    blocks.push(update_block(
                        "entry",
                        &identity,
                        Some("regex"),
                        none_str(previous.regex),
                        none_str(entry.regex),
                    ));
                }
                if previous.note != entry.note {
                    blocks.push(update_block(
                        "entry",
                        &identity,
                        Some("note"),
                        none_str(previous.note),
                        none_str(entry.note),
                    ));
                }
            }
        }
    }
    if let Some(before) = before {
        for entry in &before.entries {
            if after.entry(entry.title).is_none() {
                blocks.push(entry_block(Side::Before, entry));
            }
        }
    }
    blocks
}

fn brief_order_rows(before: Option<&BriefView<'_>>, after: &BriefView<'_>) -> Vec<OrderRow> {
    let mut rows = Vec::new();
    if let Some(before) = before {
        let before_titles: Vec<&str> = before.entries.iter().map(|entry| entry.title).collect();
        let after_titles: Vec<&str> = after.entries.iter().map(|entry| entry.title).collect();
        if same_set_different_order(&before_titles, &after_titles) {
            rows.push(OrderRow {
                list: "entries".to_string(),
                before: before_titles
                    .iter()
                    .map(|title| names::display_text(title))
                    .collect::<Vec<_>>()
                    .join(", "),
                after: after_titles
                    .iter()
                    .map(|title| names::display_text(title))
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
    }
    rows
}

fn profile_blocks(before: Option<&ProfileView<'_>>, after: &ProfileView<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for section in &after.sections {
        match before.and_then(|profile| profile.section(section.name)) {
            None => blocks.push(section_block(Side::After, section)),
            Some(previous) => {
                if previous.body != section.body {
                    blocks.push(update_block(
                        "section",
                        &[names::display_text(section.name)],
                        Some("body"),
                        none_str(previous.body),
                        none_str(section.body),
                    ));
                }
            }
        }
    }
    if let Some(before) = before {
        for section in &before.sections {
            if after.section(section.name).is_none() {
                blocks.push(section_block(Side::Before, section));
            }
        }
    }
    blocks
}

fn profile_order_rows(before: Option<&ProfileView<'_>>, after: &ProfileView<'_>) -> Vec<OrderRow> {
    let mut rows = Vec::new();
    if let Some(before) = before {
        let before_names: Vec<&str> = before.sections.iter().map(|section| section.name).collect();
        let after_names: Vec<&str> = after.sections.iter().map(|section| section.name).collect();
        if same_set_different_order(&before_names, &after_names) {
            rows.push(OrderRow {
                list: "sections".to_string(),
                before: before_names
                    .iter()
                    .map(|name| names::display_text(name))
                    .collect::<Vec<_>>()
                    .join(", "),
                after: after_names
                    .iter()
                    .map(|name| names::display_text(name))
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
    }
    rows
}

fn state_blocks(before: Option<&StateView<'_>>, after: &StateView<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for section in &after.sections {
        let previous = before.and_then(|state| state.find(section.status));
        if previous.is_none() {
            blocks.push(status_block(Side::After, section.status));
        }
        for declaration in &section.declarations {
            match previous.and_then(|previous| previous.declaration(declaration.nick)) {
                None => blocks.push(declaration_block(Side::After, section.status, declaration)),
                Some(previous_declaration) => {
                    if previous_declaration.start != declaration.start {
                        blocks.push(update_block(
                            &status_title(section.status),
                            &[declaration.nick.to_string()],
                            Some("start"),
                            time::format(previous_declaration.start),
                            time::format(declaration.start),
                        ));
                    }
                    if previous_declaration.until != declaration.until {
                        blocks.push(update_block(
                            &status_title(section.status),
                            &[declaration.nick.to_string()],
                            Some("until"),
                            time::format(previous_declaration.until),
                            time::format(declaration.until),
                        ));
                    }
                }
            }
        }
    }
    if let Some(before) = before {
        for section in &before.sections {
            let after_section = after.find(section.status);
            for declaration in &section.declarations {
                if after_section
                    .and_then(|found| found.declaration(declaration.nick))
                    .is_none()
                {
                    blocks.push(declaration_block(Side::Before, section.status, declaration));
                }
            }
            if after_section.is_none() {
                blocks.push(status_block(Side::Before, section.status));
            }
        }
    }
    blocks
}

fn state_order_rows(before: Option<&StateView<'_>>, after: &StateView<'_>) -> Vec<OrderRow> {
    let mut rows = Vec::new();
    if let Some(before) = before {
        for section in &before.sections {
            if let Some(after_section) = after.find(section.status) {
                let before_nicks: Vec<&str> = section
                    .declarations
                    .iter()
                    .map(|declaration| declaration.nick)
                    .collect();
                let after_nicks: Vec<&str> = after_section
                    .declarations
                    .iter()
                    .map(|declaration| declaration.nick)
                    .collect();
                if same_set_different_order(&before_nicks, &after_nicks) {
                    rows.push(OrderRow {
                        list: status_title(section.status),
                        before: before_nicks.join(", "),
                        after: after_nicks.join(", "),
                    });
                }
            }
        }
        let before_statuses: Vec<&str> = before
            .sections
            .iter()
            .map(|section| section.status)
            .collect();
        let after_statuses: Vec<&str> = after
            .sections
            .iter()
            .map(|section| section.status)
            .collect();
        if same_set_different_order(&before_statuses, &after_statuses) {
            rows.push(OrderRow {
                list: "sections".to_string(),
                before: before_statuses
                    .iter()
                    .map(|status| status_title(status))
                    .collect::<Vec<_>>()
                    .join(", "),
                after: after_statuses
                    .iter()
                    .map(|status| status_title(status))
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
    }
    rows
}

/// Blocks for the qualified messages of a post-filtered hit: every after
/// comment that is relevant to `nick` and new, edited, or newly relevant.
/// A became-relevant message with unchanged parsed fields is presented as a
/// `relevance` update whose after value is the reference that made it
/// relevant; nothing else about that message changed.
fn post_qualified_blocks(
    before: Option<&PostView<'_>>,
    after: &PostView<'_>,
    nick: &str,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    for comment in &after.comments {
        if !relevant(comment, after, nick) {
            continue;
        }
        match before.and_then(|post| post.comment(comment.seq)) {
            None => blocks.push(comment_block(Side::After, comment)),
            Some(previous) => {
                let identity = vec![format!("#{}", comment.seq)];
                let mut changed = false;
                if previous.nick != comment.nick {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("author"),
                        previous.nick.to_string(),
                        comment.nick.to_string(),
                    ));
                    changed = true;
                }
                if previous.created != comment.created {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("created"),
                        time::format(previous.created),
                        time::format(comment.created),
                    ));
                    changed = true;
                }
                if previous.edited != comment.edited {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("edited"),
                        none_time(previous.edited),
                        none_time(comment.edited),
                    ));
                    changed = true;
                }
                if previous.body != comment.body {
                    blocks.push(update_block(
                        "comment",
                        &identity,
                        Some("body"),
                        none_str(previous.body),
                        none_str(comment.body),
                    ));
                    changed = true;
                }
                if !changed {
                    let was_relevant = before
                        .map(|post| relevant(previous, post, nick))
                        .unwrap_or(false);
                    if !was_relevant {
                        blocks.push(update_block(
                            "comment",
                            &identity,
                            Some("relevance"),
                            "none".to_string(),
                            relevance_source(comment, after, nick)
                                .expect("comment checked relevant above"),
                        ));
                    }
                }
            }
        }
    }
    blocks
}

/// Blocks for the qualified declaration changes of a state-filtered hit
/// (the selected status only; empty-section changes never qualify).
fn state_qualified_blocks(
    before: Option<&StateView<'_>>,
    after: Option<&StateView<'_>>,
    status: &str,
) -> Vec<Block> {
    let before_section = before.and_then(|state| state.find(status));
    let after_section = after.and_then(|state| state.find(status));
    let before_declarations = before_section
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    let after_declarations = after_section
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    let mut blocks = Vec::new();
    for declaration in after_declarations {
        match before_section.and_then(|section| section.declaration(declaration.nick)) {
            None => blocks.push(declaration_block(Side::After, status, declaration)),
            Some(previous) => {
                if previous.start != declaration.start {
                    blocks.push(update_block(
                        &status_title(status),
                        &[declaration.nick.to_string()],
                        Some("start"),
                        time::format(previous.start),
                        time::format(declaration.start),
                    ));
                }
                if previous.until != declaration.until {
                    blocks.push(update_block(
                        &status_title(status),
                        &[declaration.nick.to_string()],
                        Some("until"),
                        time::format(previous.until),
                        time::format(declaration.until),
                    ));
                }
            }
        }
    }
    for declaration in before_declarations {
        if after_section
            .and_then(|section| section.declaration(declaration.nick))
            .is_none()
        {
            blocks.push(declaration_block(Side::Before, status, declaration));
        }
    }
    blocks
}

fn same_set_different_order<T: Eq + Hash>(before: &[T], after: &[T]) -> bool {
    if before == after || before.len() != after.len() {
        return false;
    }
    let after_set: HashSet<&T> = after.iter().collect();
    before.iter().all(|item| after_set.contains(item))
}

fn join_seqs(seqs: &[u64]) -> String {
    seqs.iter()
        .map(|seq| format!("#{}", seq))
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// Display objects
// ---------------------------------------------------------------------------

fn object_dir(path: &Path) -> String {
    // An empty parent displays as `.` (spec §8.1.3).
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    context::object_of_dir(parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Row;

    /// Minimal valid one-comment post as stored bytes.
    fn post_bytes() -> Vec<u8> {
        let mut text = String::new();
        for line in [
            "# standup.post",
            "",
            "## #1 alice (2026-09-17T09:00:00Z)",
            "",
            "```md",
            "hello",
            "```",
        ] {
            text.push_str(line);
            text.push('\n');
        }
        text.into_bytes()
    }

    fn fixed_now() -> Time {
        time::parse("2026-09-17T10:00:00Z").expect("fixed clock")
    }

    fn field_values<'a>(report: &'a Report, wanted: &str) -> Option<&'a [String]> {
        report.rows.iter().find_map(|row| match row {
            Row::Fields { key, values, .. } if key == wanted => Some(values.as_slice()),
            _ => None,
        })
    }

    fn diagnostic_ids(report: &Report) -> Vec<&'static str> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Diagnostic(diagnostic) => Some(diagnostic.id),
                _ => None,
            })
            .collect()
    }

    fn blocked(admission: Result<Document, Report>) -> Report {
        match admission {
            Err(report) => report,
            Ok(_) => panic!("present file was admitted"),
        }
    }

    fn record_values<'a>(report: &'a Report, wanted: &str) -> Vec<&'a [String]> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key, values, .. } if key == wanted => Some(values.as_slice()),
                _ => None,
            })
            .collect()
    }

    /// The wait object is the same folded display the base commands produce
    /// (spec §8.1.3–§8.1.4; regression: WAIT-OBJECT-UNFOLDED).
    #[test]
    fn command_object_folds_like_base_commands() {
        let path = Path::new("docs/foo  bar.agent-besties.md");
        let (command, object) = command_object(path, &Filter::Any);
        assert_eq!(command, "wait");
        assert_eq!(object, "docs/foo bar");
        assert!(
            !object.contains("  "),
            "object must stay two-space-split safe: {object:?}"
        );
        // The CLI's diag context for the same target embeds this value.
        assert_eq!(object, context::object_of_path(path));

        let (command, object) = command_object(path, &Filter::Post(None));
        assert_eq!(command, "wait-post");
        assert_eq!(object, context::object_of_path(path));

        let state_path = Path::new("foo  bar").join(format!("state{}", crate::consts::SUFFIX));
        let (command, object) = command_object(&state_path, &Filter::State(None));
        assert_eq!(command, "wait-state");
        assert_eq!(object, "foo bar");
        assert_eq!(object, context::object_of_dir(Path::new("foo  bar")));
    }

    /// PATH wait must not reuse DIR-command E-NOT-DIR recovery when the
    /// parent of the target file exists as a regular file
    /// (regression: WAIT-PARENT-ENOTDIR-FIX).
    #[test]
    fn file_target_parent_file_is_path_class_not_dir_command() {
        let root = tempfile::tempdir().expect("tempdir");
        let parent = root.path().join("notdir");
        std::fs::write(&parent, b"regular file").expect("write parent file");
        let path = parent.join(format!("notes{}", crate::consts::SUFFIX));
        let cancel = Cancellation::new();
        for filter in [Filter::Any, Filter::Post(None)] {
            let err = run(&path, filter, &cancel, || {}).expect_err("parent is a file");
            match err {
                Fault::Io {
                    path: io_path,
                    source,
                    ..
                } => {
                    assert_eq!(source.kind(), std::io::ErrorKind::NotADirectory);
                    assert_eq!(io_path, parent);
                }
                Fault::Reason { id, params } => panic!(
                    "file-target wait must not use catalog {id} (DIR re-run); params={params:?}"
                ),
                other => panic!("expected path-class Io, got {other:?}"),
            }
        }
    }

    /// wait-state is a DIR command: a file in the DIR slot stays E-NOT-DIR.
    #[test]
    fn wait_state_file_dir_keeps_not_dir_catalog() {
        let root = tempfile::tempdir().expect("tempdir");
        let dir = root.path().join("notdir");
        std::fs::write(&dir, b"regular file").expect("write file as dir");
        let path = dir.join(format!("state{}", crate::consts::SUFFIX));
        let cancel = Cancellation::new();
        let err = run(&path, Filter::State(None), &cancel, || {}).expect_err("dir is a file");
        match err {
            Fault::Reason { id, params } => {
                assert_eq!(id, "E-NOT-DIR");
                assert!(
                    params
                        .iter()
                        .any(|(key, value)| *key == "cmd" && value == "wait-state"),
                    "wait-state keeps DIR-command recovery params: {params:?}"
                );
            }
            other => panic!("wait-state must keep E-NOT-DIR, got {other:?}"),
        }
    }

    /// `created` / `removed` rows carry the folded display path, the base
    /// convention for `created` rows (regression: WAIT-OBJECT-UNFOLDED).
    #[test]
    fn created_and_removed_rows_use_folded_display() {
        let path = Path::new("docs/foo  bar.agent-besties.md");
        let parsed = doc::parse(post_bytes(), path, fixed_now()).expect("valid post");
        let created = build_report(
            path,
            &Filter::Any,
            &None,
            &Some(extract(&parsed)),
            fixed_now(),
        );
        assert_eq!(
            field_values(&created, "created").expect("created row"),
            &["docs/foo bar.agent-besties.md"]
        );
        assert_eq!(created.object, "docs/foo bar");
        let removed = build_report(
            path,
            &Filter::Any,
            &Some(extract(&parsed)),
            &None,
            fixed_now(),
        );
        assert_eq!(
            field_values(&removed, "removed").expect("removed row"),
            &["docs/foo bar.agent-besties.md"]
        );
    }

    /// Empty present bytes fail with the check-shaped report: file row, no
    /// type (kind is none), blocking diagnostics, exit 1. Not Absent.
    #[test]
    fn empty_present_file_reports_file_row_without_type() {
        let path = Path::new("notes.agent-besties.md");
        let report =
            blocked(admit_present(Vec::new(), path, &Filter::Any, fixed_now()).expect("admit"));
        assert_eq!(report.command, "wait");
        assert_eq!(report.object, "notes");
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 1);
        assert_eq!(
            field_values(&report, "file").expect("file row"),
            &["notes.agent-besties.md"]
        );
        assert!(
            field_values(&report, "type").is_none(),
            "empty file has no H1 kind: {:?}",
            field_values(&report, "type")
        );
        assert!(
            diagnostic_ids(&report).contains(&"E-H1"),
            "empty file is E-H1, got {:?}",
            diagnostic_ids(&report)
        );
        assert!(
            field_values(&report, "bytes").is_none(),
            "blocking present file is not a change hit"
        );
    }

    /// Unclosed first fence: same snapshot supplies post kind, so the error
    /// report has file + type + E-FENCE.
    #[test]
    fn unclosed_post_fence_reports_file_type_and_fence() {
        let path = Path::new("standup.agent-besties.md");
        let bytes =
            b"# standup.post\n\n## #1 alice (2026-09-17T09:12:00Z)\n\n```md\nnever closed\n"
                .to_vec();
        let report =
            blocked(admit_present(bytes, path, &Filter::Post(None), fixed_now()).expect("admit"));
        assert_eq!(report.command, "wait-post");
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 1);
        assert_eq!(
            field_values(&report, "file").expect("file row"),
            &["standup.agent-besties.md"]
        );
        assert_eq!(field_values(&report, "type").expect("type row"), &["post"]);
        assert!(
            diagnostic_ids(&report).contains(&"E-FENCE"),
            "unclosed fence is E-FENCE, got {:?}",
            diagnostic_ids(&report)
        );
    }

    /// wait-state object stays the directory; the file row is still the
    /// managed state path.
    #[test]
    fn wait_state_empty_file_keeps_dir_object_and_file_row() {
        let path = Path::new("room/state.agent-besties.md");
        let report = blocked(
            admit_present(Vec::new(), path, &Filter::State(None), fixed_now()).expect("admit"),
        );
        assert_eq!(report.command, "wait-state");
        assert_eq!(report.object, "room");
        assert_eq!(
            field_values(&report, "file").expect("file row"),
            &["room/state.agent-besties.md"]
        );
        assert!(field_values(&report, "type").is_none());
        assert_eq!(report.exit, 1);
    }

    /// Invalid UTF-8 is a parse refusal with no snapshot kind: file row,
    /// no type, E-UTF8.
    #[test]
    fn invalid_utf8_present_file_reports_file_row_without_type() {
        let path = Path::new("notes.agent-besties.md");
        let report =
            blocked(admit_present(vec![0xff], path, &Filter::Any, fixed_now()).expect("admit"));
        assert_eq!(
            field_values(&report, "file").expect("file row"),
            &["notes.agent-besties.md"]
        );
        assert!(field_values(&report, "type").is_none());
        assert!(
            diagnostic_ids(&report).contains(&"E-UTF8"),
            "got {:?}",
            diagnostic_ids(&report)
        );
        assert_eq!(report.exit, 1);
        assert_eq!(report.status, Status::Error);
    }

    /// Status headings are display_text'd at wait output only: ESC becomes
    /// U+FFFD in title labels, nick identity stays raw.
    #[test]
    fn wait_state_entity_rows_sanitize_status_titles() {
        let path = Path::new("room/state.agent-besties.md");
        let bytes =
            b"# .state\n\n## custom\x1b[2J\nbob 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"
                .to_vec();
        let parsed = doc::parse(bytes, path, fixed_now()).expect("parseable state");
        let report = build_report(
            path,
            &Filter::Any,
            &None,
            &Some(extract(&parsed)),
            fixed_now(),
        );
        let adds = record_values(&report, "add");
        assert!(!adds.is_empty(), "created state must emit entity rows");
        for values in &adds {
            for value in *values {
                assert!(
                    !value.contains('\u{1b}'),
                    "status title must not leak ESC: {values:?}"
                );
            }
        }
        let titles = adds
            .iter()
            .flat_map(|values| values.iter())
            .filter(|value| value.contains('\u{FFFD}'))
            .count();
        assert!(
            titles > 0,
            "displayed status title replaces ESC with U+FFFD, rows: {adds:?}"
        );
        assert!(
            adds.iter()
                .any(|values| values.iter().any(|value| value == "bob")),
            "nick identity stays raw: {adds:?}"
        );
    }

    /// Brief entry identities are display_text'd at wait output only: ESC
    /// becomes U+FFFD, and two raw-distinct titles that render the same
    /// stay two entities.
    #[test]
    fn wait_brief_entity_rows_sanitize_entry_titles_without_merging() {
        let path = Path::new("notes.agent-besties.md");
        let bytes = concat!(
            "# notes.brief\n\n",
            "## t\u{1b}itle\n",
            "- path: a.txt\n",
            "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
            "## t\u{07}itle\n",
            "- path: b.txt\n",
            "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
        )
        .as_bytes()
        .to_vec();
        let parsed = doc::parse(bytes, path, fixed_now()).expect("parseable brief");
        assert_eq!(parsed.brief.entries.len(), 2);
        assert_ne!(parsed.brief.entries[0].title, parsed.brief.entries[1].title);
        let report = build_report(
            path,
            &Filter::Any,
            &None,
            &Some(extract(&parsed)),
            fixed_now(),
        );
        let adds: Vec<&[String]> = record_values(&report, "add")
            .into_iter()
            .filter(|values| values.first().map(|value| value.as_str()) == Some("entry"))
            .collect();
        assert_eq!(
            adds.len(),
            2,
            "raw-distinct titles stay two add rows: {adds:?}"
        );
        for values in &adds {
            for value in *values {
                assert!(
                    !value.contains('\u{1b}') && !value.contains('\u{07}'),
                    "entry title must not leak Cc: {values:?}"
                );
            }
            assert_eq!(
                values.get(1).map(|value| value.as_str()),
                Some("t\u{FFFD}itle")
            );
        }
        assert!(record_values(&report, "update").is_empty());
    }

    /// Profile section identities are display_text'd at wait output only.
    #[test]
    fn wait_profile_entity_rows_sanitize_section_titles_without_merging() {
        let path = Path::new("a.agent-besties.md");
        let bytes = concat!(
            "# a.agent-profile\n\n",
            "## role\u{1b}\n\n",
            "```md\n",
            "dev\n",
            "```\n",
            "## role\u{07}\n\n",
            "```md\n",
            "ops\n",
            "```\n",
        )
        .as_bytes()
        .to_vec();
        let parsed = doc::parse(bytes, path, fixed_now()).expect("parseable profile");
        assert_eq!(parsed.profile.len(), 2);
        assert_ne!(parsed.profile[0].name, parsed.profile[1].name);
        let report = build_report(
            path,
            &Filter::Any,
            &None,
            &Some(extract(&parsed)),
            fixed_now(),
        );
        let adds: Vec<&[String]> = record_values(&report, "add")
            .into_iter()
            .filter(|values| values.first().map(|value| value.as_str()) == Some("section"))
            .collect();
        assert_eq!(
            adds.len(),
            2,
            "raw-distinct section titles stay two add rows: {adds:?}"
        );
        for values in &adds {
            for value in *values {
                assert!(
                    !value.contains('\u{1b}') && !value.contains('\u{07}'),
                    "section title must not leak Cc: {values:?}"
                );
            }
            assert_eq!(
                values.get(1).map(|value| value.as_str()),
                Some("role\u{FFFD}")
            );
        }
        assert!(record_values(&report, "update").is_empty());
    }

    fn doc_bytes(lines: &[&str]) -> Vec<u8> {
        let mut text = String::new();
        for line in lines {
            text.push_str(line);
            text.push('\n');
        }
        text.into_bytes()
    }

    fn admitted(bytes: Vec<u8>, path: &Path) -> Document {
        let parsed = doc::parse(bytes, path, fixed_now()).expect("parse");
        validate(&parsed, &Filter::Any, path).expect("admitted snapshot");
        parsed
    }

    fn transition(
        path: &Path,
        filter: &Filter,
        before: &Document,
        after: &Document,
    ) -> Option<Report> {
        evaluate(
            &Some(extract(before)),
            &Some(extract(after)),
            filter,
            path,
            fixed_now(),
        )
    }

    fn state_two_observing(first: &str, second: &str) -> Vec<u8> {
        format!(
            "# .state\n\n## observing\n{first} 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z\n{second} 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z\n"
        )
        .into_bytes()
    }

    /// Filtered wait-state ignores an unrelated empty status and a pure
    /// reorder of the selected declarations; add/remove/renewal still hit.
    #[test]
    fn filtered_state_pairs_by_nick_and_ignores_unrelated_or_order_only() {
        let path = Path::new("room/state.agent-besties.md");
        let filter = Filter::State(Some("observing".to_string()));
        let baseline = admitted(state_two_observing("alice", "bob"), path);

        let unrelated = admitted(
            doc_bytes(&[
                "# .state",
                "",
                "## observing",
                "alice 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
                "bob 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
                "",
                "## custom",
            ]),
            path,
        );
        assert!(
            transition(path, &filter, &baseline, &unrelated).is_none(),
            "empty custom section must not qualify"
        );

        let reordered = admitted(state_two_observing("bob", "alice"), path);
        assert!(
            transition(path, &filter, &baseline, &reordered).is_none(),
            "pure reorder of the same nicks and times must not qualify"
        );

        let added = admitted(
            doc_bytes(&[
                "# .state",
                "",
                "## observing",
                "alice 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
                "bob 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
                "carol 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
            ]),
            path,
        );
        let added_report = transition(path, &filter, &baseline, &added).expect("add hits");
        let adds = record_values(&added_report, "add");
        assert!(
            adds.iter().any(|values| {
                values.first().map(String::as_str) == Some("observing")
                    && values.get(1).map(String::as_str) == Some("carol")
            }),
            "selected-status add must keep after-file order identity: {adds:?}"
        );
        assert!(record_values(&added_report, "remove").is_empty());
        assert!(record_values(&added_report, "order").is_empty());

        let removed = admitted(
            doc_bytes(&[
                "# .state",
                "",
                "## observing",
                "alice 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
            ]),
            path,
        );
        let removed_report = transition(path, &filter, &baseline, &removed).expect("remove hits");
        let removes = record_values(&removed_report, "remove");
        assert!(
            removes
                .iter()
                .any(|values| values.get(1).map(String::as_str) == Some("bob")),
            "selected-status remove must keep before-file order identity: {removes:?}"
        );

        let renewed = admitted(
            doc_bytes(&[
                "# .state",
                "",
                "## observing",
                "alice 2026-09-17T09:00:00Z..2031-01-01T00:00:00Z",
                "bob 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
            ]),
            path,
        );
        let renewed_report = transition(path, &filter, &baseline, &renewed).expect("renewal hits");
        let updates = record_values(&renewed_report, "update");
        assert!(
            updates.iter().any(|values| {
                values.get(1).map(String::as_str) == Some("alice")
                    && values.get(2).map(String::as_str) == Some("until")
            }),
            "stored renewal must update until: {updates:?}"
        );
        assert!(record_values(&renewed_report, "add").is_empty());
        assert!(record_values(&renewed_report, "remove").is_empty());
    }

    /// Unfiltered wait reports a pure permutation as order-only, and a
    /// same-length identity replacement as add/remove without fabricating
    /// an order row.
    #[test]
    fn unfiltered_state_order_only_and_replacement_keep_file_order() {
        let path = Path::new("room/state.agent-besties.md");
        let filter = Filter::Any;
        let baseline = admitted(state_two_observing("alice", "bob"), path);
        let reordered = admitted(state_two_observing("bob", "alice"), path);
        let report = transition(path, &filter, &baseline, &reordered).expect("order-only hits");
        assert!(record_values(&report, "add").is_empty());
        assert!(record_values(&report, "remove").is_empty());
        assert!(record_values(&report, "update").is_empty());
        let orders = record_values(&report, "order");
        assert_eq!(
            orders.len(),
            1,
            "same identity set in a new file order is one order row: {orders:?}"
        );
        assert_eq!(orders[0], ["observing".to_string()].as_slice());
        let texts: Vec<(&str, &str)> = report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Text { key, content } => Some((key.as_str(), content.as_str())),
                _ => None,
            })
            .collect();
        assert!(
            texts.contains(&("before", "alice, bob")) && texts.contains(&("after", "bob, alice")),
            "order row keeps each snapshot's file order: {texts:?}"
        );

        let replaced = admitted(state_two_observing("alice", "carol"), path);
        let replaced_report =
            transition(path, &filter, &baseline, &replaced).expect("replacement hits");
        let adds = record_values(&replaced_report, "add");
        let removes = record_values(&replaced_report, "remove");
        assert!(
            adds.iter()
                .any(|values| values.get(1).map(String::as_str) == Some("carol")),
            "new nick is an add: {adds:?}"
        );
        assert!(
            removes
                .iter()
                .any(|values| values.get(1).map(String::as_str) == Some("bob")),
            "missing nick is a remove: {removes:?}"
        );
        assert!(
            record_values(&replaced_report, "order").is_empty(),
            "add/remove must not invent an order row: {:?}",
            record_values(&replaced_report, "order")
        );
    }

    /// Post comments pair by seq across reorder; subscriber-only changes
    /// never qualify a nick filter.
    #[test]
    fn post_pairs_by_seq_and_subscriber_only_is_not_a_filtered_hit() {
        let path = Path::new("standup.agent-besties.md");
        let before = admitted(
            doc_bytes(&[
                "# standup.post",
                "",
                "- subscribers: alice",
                "",
                "## #1 alice (2026-09-17T09:00:00Z)",
                "",
                "```md",
                "@alice hello",
                "```",
                "",
                "## #2 bob (2026-09-17T09:05:00Z)",
                "",
                "```md",
                "@alice ping",
                "```",
            ]),
            path,
        );
        let reordered = admitted(
            doc_bytes(&[
                "# standup.post",
                "",
                "- subscribers: alice",
                "",
                "## #2 bob (2026-09-17T09:05:00Z)",
                "",
                "```md",
                "@alice ping",
                "```",
                "",
                "## #1 alice (2026-09-17T09:00:00Z)",
                "",
                "```md",
                "@alice hello edited",
                "```",
            ]),
            path,
        );
        let filter = Filter::Post(Some("alice".to_string()));
        let filtered = transition(path, &filter, &before, &reordered).expect("body change hits");
        let updates = record_values(&filtered, "update");
        assert!(
            updates.iter().any(|values| {
                values.get(1).map(String::as_str) == Some("#1")
                    && values.get(2).map(String::as_str) == Some("body")
            }),
            "seq #1 must update body after reorder: {updates:?}"
        );
        assert!(
            !updates
                .iter()
                .any(|values| values.get(1).map(String::as_str) == Some("#2")),
            "unchanged #2 must not be reported: {updates:?}"
        );
        assert!(record_values(&filtered, "add").is_empty());
        assert!(record_values(&filtered, "remove").is_empty());
        assert!(
            record_values(&filtered, "order").is_empty(),
            "filtered post output does not include order rows"
        );

        let any_report = transition(path, &Filter::Any, &before, &reordered).expect("any hits");
        assert!(record_values(&any_report, "add").is_empty());
        assert!(record_values(&any_report, "remove").is_empty());
        let orders = record_values(&any_report, "order");
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0], ["comments".to_string()].as_slice());

        let subscribers_only = admitted(
            doc_bytes(&[
                "# standup.post",
                "",
                "- subscribers: alice, carol",
                "",
                "## #1 alice (2026-09-17T09:00:00Z)",
                "",
                "```md",
                "@alice hello",
                "```",
                "",
                "## #2 bob (2026-09-17T09:05:00Z)",
                "",
                "```md",
                "@alice ping",
                "```",
            ]),
            path,
        );
        assert!(
            transition(path, &filter, &before, &subscribers_only).is_none(),
            "subscriber-only change must not qualify"
        );
        let subscriber_report =
            transition(path, &Filter::Any, &before, &subscribers_only).expect("any subscriber hit");
        let adds = record_values(&subscriber_report, "add");
        assert!(
            adds.iter().any(|values| {
                values.first().map(String::as_str) == Some("subscriber")
                    && values.get(1).map(String::as_str) == Some("carol")
            }),
            "unfiltered subscriber add follows after-file order: {adds:?}"
        );
        assert!(record_values(&subscriber_report, "remove").is_empty());
        assert!(record_values(&subscriber_report, "order").is_empty());
    }

    /// Brief entries and profile sections pair by raw identity, not file
    /// position; a permutation of the same set is order-only.
    #[test]
    fn brief_and_profile_pair_by_raw_identity_across_reorder() {
        let brief_path = Path::new("notes.agent-besties.md");
        let before_brief = admitted(
            doc_bytes(&[
                "# notes.brief",
                "",
                "## alpha",
                "- path: a.txt",
                "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "## beta",
                "- path: b.txt",
                "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ]),
            brief_path,
        );
        let after_brief = admitted(
            doc_bytes(&[
                "# notes.brief",
                "",
                "## beta",
                "- path: b.txt",
                "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "## alpha",
                "- path: a2.txt",
                "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ]),
            brief_path,
        );
        let brief_report =
            transition(brief_path, &Filter::Any, &before_brief, &after_brief).expect("brief hits");
        assert!(record_values(&brief_report, "add").is_empty());
        assert!(record_values(&brief_report, "remove").is_empty());
        let updates = record_values(&brief_report, "update");
        assert!(
            updates.iter().any(|values| {
                values.first().map(String::as_str) == Some("entry")
                    && values.get(1).map(String::as_str) == Some("alpha")
                    && values.get(2).map(String::as_str) == Some("path")
            }),
            "alpha must update path after reorder: {updates:?}"
        );
        assert!(
            !updates
                .iter()
                .any(|values| values.get(1).map(String::as_str) == Some("beta")),
            "unchanged beta must not update: {updates:?}"
        );
        let brief_orders = record_values(&brief_report, "order");
        assert_eq!(brief_orders.len(), 1);
        assert_eq!(brief_orders[0], ["entries".to_string()].as_slice());

        let profile_path = Path::new("a.agent-besties.md");
        let before_profile = admitted(
            doc_bytes(&[
                "# a.agent-profile",
                "",
                "## role",
                "",
                "```md",
                "dev",
                "```",
                "",
                "## scope",
                "",
                "```md",
                "core",
                "```",
            ]),
            profile_path,
        );
        let after_profile = admitted(
            doc_bytes(&[
                "# a.agent-profile",
                "",
                "## scope",
                "",
                "```md",
                "core",
                "```",
                "",
                "## role",
                "",
                "```md",
                "ops",
                "```",
            ]),
            profile_path,
        );
        let profile_report =
            transition(profile_path, &Filter::Any, &before_profile, &after_profile)
                .expect("profile hits");
        assert!(record_values(&profile_report, "add").is_empty());
        assert!(record_values(&profile_report, "remove").is_empty());
        let profile_updates = record_values(&profile_report, "update");
        assert!(
            profile_updates.iter().any(|values| {
                values.first().map(String::as_str) == Some("section")
                    && values.get(1).map(String::as_str) == Some("role")
                    && values.get(2).map(String::as_str) == Some("body")
            }),
            "role must update body after reorder: {profile_updates:?}"
        );
        assert!(
            !profile_updates
                .iter()
                .any(|values| values.get(1).map(String::as_str) == Some("scope")),
            "unchanged scope must not update: {profile_updates:?}"
        );
        let profile_orders = record_values(&profile_report, "order");
        assert_eq!(profile_orders.len(), 1);
        assert_eq!(profile_orders[0], ["sections".to_string()].as_slice());
    }
}
