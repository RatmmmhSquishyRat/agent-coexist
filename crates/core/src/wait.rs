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
//!   `warn` (still exit 0). They never invalidate a snapshot.
//! - Entity changes are `Fields` rows with indent 2 and key width 6:
//!   `add` / `remove` carry the entity identity, `update` additionally names
//!   the changed field.
//! - Every field value is a `Text` row: for `add` / `remove` the key is the
//!   field name and the content is the one observed side; for `update` the
//!   keys are `before` / `after`. A missing side or missing field renders as
//!   the literal `none`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::diag;
use crate::doc::{self, Document};
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::report::Report;
use crate::time;
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
/// managed file path (state shows its directory, spec §8.1.3 style). Public
/// so the CLI can print the identical `waiting  <command>  <object>` stderr
/// line when the ready callback fires.
pub fn command_object(path: &Path, filter: &Filter) -> (&'static str, String) {
    match filter {
        Filter::Any => ("wait", fsx::walk::object_path(path)),
        Filter::Post(_) => ("wait-post", fsx::walk::object_path(path)),
        Filter::State(_) => ("wait-state", object_dir(path)),
    }
}

/// One-shot wait: sample `path` (always the actual managed file) until a
/// qualified transition is observed, then return the complete change report.
///
/// The parent directory must exist and be a directory. The first valid
/// snapshot is the baseline (an absent file is a legitimate `Absent`
/// baseline and is never created); `ready` is called exactly once after the
/// baseline is admitted. Invalid snapshots (blocking diagnostics,
/// unreliable identities, wrong type for a specialized command) fail the
/// wait instead of replacing the baseline. Unqualified but valid snapshots
/// advance the whole baseline. Cancellation returns `Fault::Cancelled`
/// (CLI exit 130). No lock is held while idle.
pub fn run(
    path: &Path,
    filter: Filter,
    cancel: &Cancellation,
    ready: impl FnOnce(),
) -> Result<Report, Fault> {
    let now = time::now()?;
    let (command, _) = command_object(path, &filter);
    ensure_parent_dir(path, command)?;

    let mut baseline = match read_raw(path, cancel)? {
        Raw::Absent => None,
        Raw::Present(bytes) => {
            let parsed = doc::parse(bytes, path, now)?;
            validate(&parsed, &filter, path)?;
            Some(parsed)
        }
    };
    ready();

    loop {
        if cancel.sleep_poll(WAIT_POLL_MS) || cancel.is_cancelled() {
            return Err(Fault::Cancelled);
        }
        let raw = read_raw(path, cancel)?;
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
            Raw::Present(bytes) => {
                let parsed = doc::parse(bytes, path, now)?;
                validate(&parsed, &filter, path)?;
                Some(parsed)
            }
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
            if let Some(report) = evaluate(&before_model, &after_model, &filter, path) {
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

fn ensure_parent_dir(path: &Path, command: &str) -> Result<(), Fault> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let grandparent = parent.parent().unwrap_or_else(|| Path::new(""));
    let listed = if grandparent.as_os_str().is_empty() {
        ".".to_string()
    } else {
        fsx::walk::display_path(grandparent)
    };
    match std::fs::metadata(parent) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(Fault::reason("E-NOT-DIR")
            .with("path", fsx::walk::display_path(parent))
            .with("cmd", command)
            .with("parent", listed)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(Fault::reason("E-DIR-MISSING")
                .with("dir", fsx::walk::display_path(parent))
                .with("parent", listed))
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
        None => return Err(Fault::Diagnostics(clone_diagnostics(&document.diagnostics))),
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
                fault = fault.with("dir", object_dir(path));
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

/// Wait-specific reliability threshold: `(status, nick)` must be unique so
/// declarations have unambiguous identities. Base parsing tolerates
/// duplicates (latest `until` wins, `W-STATE-DUP`); wait rejects the
/// snapshot instead of guessing which declaration moved.
fn check_duplicate_state_identities(document: &Document, path: &Path) -> Result<(), Fault> {
    if document.kind.as_ref().map(|kind| kind.as_str()) != Some("state") {
        return Ok(());
    }
    for section in &document.state {
        let mut seen: Vec<&str> = Vec::new();
        for declaration in &section.entries {
            if seen.contains(&declaration.nick.as_str()) {
                let mut diagnostics: Vec<Diagnostic> = document
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.id == "W-STATE-DUP")
                    .map(clone_diagnostic)
                    .collect();
                if diagnostics.is_empty() {
                    diagnostics.push(
                        Diagnostic::new("W-STATE-DUP", declaration.line)
                            .with("nick", declaration.nick.clone())
                            .with("status", section.status.clone())
                            .with("dir", object_dir(path)),
                    );
                }
                return Err(Fault::Diagnostics(diagnostics));
            }
            seen.push(declaration.nick.as_str());
        }
    }
    Ok(())
}

fn clone_diagnostic(source: &Diagnostic) -> Diagnostic {
    let mut copy = Diagnostic::new(source.id, source.line);
    for (key, value) in &source.params {
        copy = copy.with(key, value.clone());
    }
    copy
}

fn clone_diagnostics(source: &[Diagnostic]) -> Vec<Diagnostic> {
    source.iter().map(clone_diagnostic).collect()
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
    authors: HashMap<u64, &'a str>,
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
}

struct SectionView<'a> {
    name: &'a str,
    body: Option<&'a str>,
}

struct ProfileView<'a> {
    sections: Vec<SectionView<'a>>,
}

struct DeclarationView<'a> {
    nick: &'a str,
    start: Time,
    until: Time,
}

struct StatusView<'a> {
    status: &'a str,
    declarations: Vec<DeclarationView<'a>>,
}

struct StateView<'a> {
    sections: Vec<StatusView<'a>>,
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
    warnings: &'a [Diagnostic],
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

impl StateView<'_> {
    fn find(&self, status: &str) -> Option<&StatusView<'_>> {
        self.sections
            .iter()
            .find(|section| section.status == status)
    }
}

/// Borrow the compared field values of one admitted parse. The retained
/// `Document` keeps the original bytes and raw spans; nothing is copied and
/// no time is formatted here.
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
        warnings: document.diagnostics.as_slice(),
    }
}

fn extract_post(document: &Document) -> PostView<'_> {
    PostView {
        comments: document
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
            .collect(),
        subscribers: document.post.subscribers.as_slice(),
        authors: document
            .post
            .comments
            .iter()
            .map(|comment| (comment.seq, comment.nick.as_str()))
            .collect(),
    }
}

fn extract_brief(document: &Document) -> BriefView<'_> {
    BriefView {
        description: document
            .brief
            .description
            .as_ref()
            .map(|body| document.text(&body.content)),
        entries: document
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
            .collect(),
    }
}

fn extract_profile(document: &Document) -> ProfileView<'_> {
    ProfileView {
        sections: document
            .profile
            .iter()
            .map(|section| SectionView {
                name: section.name.as_str(),
                body: document.body(section.section),
            })
            .collect(),
    }
}

fn extract_state(document: &Document) -> StateView<'_> {
    StateView {
        sections: document
            .state
            .iter()
            .map(|section| StatusView {
                status: section.status.as_str(),
                declarations: section
                    .entries
                    .iter()
                    .map(|declaration| DeclarationView {
                        nick: declaration.nick.as_str(),
                        start: declaration.start,
                        until: declaration.until,
                    })
                    .collect(),
            })
            .collect(),
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
) -> Option<Report> {
    let hit = match filter {
        Filter::Any | Filter::Post(None) | Filter::State(None) => true,
        Filter::Post(Some(nick)) => post_hit(before, after, nick),
        Filter::State(Some(status)) => state_hit(before, after, status),
    };
    if hit {
        Some(build_report(path, filter, before, after))
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
        let previous = before_post
            .and_then(|post| post.comments.iter().find(|other| other.seq == comment.seq));
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
        post.authors
            .get(seq)
            .map(|author| *author == nick)
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
            post.authors
                .get(seq)
                .map(|author| *author == nick)
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
    let before_declarations = before
        .as_ref()
        .and_then(Model::as_state)
        .and_then(|state| state.find(status))
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    let after_declarations = after
        .as_ref()
        .and_then(Model::as_state)
        .and_then(|state| state.find(status))
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    for declaration in after_declarations {
        match before_declarations
            .iter()
            .find(|other| other.nick == declaration.nick)
        {
            None => return true,
            Some(previous) => {
                if previous.start != declaration.start || previous.until != declaration.until {
                    return true;
                }
            }
        }
    }
    for declaration in before_declarations {
        if !after_declarations
            .iter()
            .any(|other| other.nick == declaration.nick)
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
        identity: vec![entry.title.to_string()],
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
        identity: vec![names::fold(section.name)],
        field: None,
        field_values: vec![("body", none_str(section.body))],
        before: None,
        after: None,
    }
}

fn status_block(side: Side, status: &str) -> Block {
    Block {
        verb: side.verb(),
        entity: "status".to_string(),
        identity: vec![names::fold(status)],
        field: None,
        field_values: Vec::new(),
        before: None,
        after: None,
    }
}

fn declaration_block(side: Side, status: &str, declaration: &DeclarationView<'_>) -> Block {
    Block {
        verb: side.verb(),
        entity: status.to_string(),
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

fn emit_block(report: &mut Report, block: &Block) {
    let mut values = vec![block.entity.clone()];
    values.extend(block.identity.iter().cloned());
    if let Some(field) = block.field {
        values.push(field.to_string());
    }
    report.record(block.verb, values, 2, ENTITY_KEY_WIDTH);
    for (key, value) in &block.field_values {
        report.text(*key, value.clone());
    }
    if let Some(before) = &block.before {
        report.text("before", before.clone());
    }
    if let Some(after) = &block.after {
        report.text("after", after.clone());
    }
}

fn emit_order_row(report: &mut Report, row: &OrderRow) {
    report.record("order", vec![row.list.clone()], 2, ENTITY_KEY_WIDTH);
    report.text("before", row.before.clone());
    report.text("after", row.after.clone());
}

fn build_report(
    path: &Path,
    filter: &Filter,
    before: &Option<Model<'_>>,
    after: &Option<Model<'_>>,
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
        report.kv("created", vec![fsx::walk::display_path(path)]);
    }
    if removed {
        report.kv("removed", vec![fsx::walk::display_path(path)]);
    }
    // Ordinary warn-level diagnostics of the current admitted snapshot (the
    // before snapshot stands in when the file was just removed), with their
    // real anchors. Error-level diagnostics of an admitted snapshot do not
    // exist by construction (blocking ones failed validation; the remaining
    // non-blocking E-* belong to `check`, not to a change report).
    let warnings = match after.as_ref() {
        Some(model) => model.warnings,
        None => before.as_ref().map(|model| model.warnings).unwrap_or(&[]),
    };
    let mut has_warnings = false;
    for diagnostic in warnings
        .iter()
        .filter(|diagnostic| !diag::is_error(diagnostic.id))
    {
        report.diagnostic(clone_diagnostic(diagnostic));
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
                    emit_block(&mut report, &block);
                }
            }
        }
        Filter::State(Some(status)) => {
            let before_state = before.as_ref().and_then(Model::as_state);
            let after_state = after.as_ref().and_then(Model::as_state);
            for block in state_qualified_blocks(before_state, after_state, status) {
                emit_block(&mut report, &block);
            }
        }
        _ => {
            for block in doc_blocks(before, after) {
                emit_block(&mut report, &block);
            }
            for block in entity_blocks(before, after) {
                emit_block(&mut report, &block);
            }
            for row in order_rows(before, after) {
                emit_order_row(&mut report, &row);
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

/// Document-level field updates (H1 name/type and the brief description).
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
        if let (View::Brief(before_brief), View::Brief(after_brief)) =
            (&before_model.view, &after_model.view)
        {
            if before_brief.description != after_brief.description {
                blocks.push(update_block(
                    "description",
                    &[],
                    None,
                    none_str(before_brief.description),
                    none_str(after_brief.description),
                ));
            }
        }
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
        let previous =
            before.and_then(|post| post.comments.iter().find(|other| other.seq == comment.seq));
        match previous {
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
            .map(|post| post.subscribers.iter().any(|other| other == nick))
            .unwrap_or(false)
        {
            blocks.push(subscriber_block(Side::After, nick));
        }
    }
    if let Some(before) = before {
        for comment in &before.comments {
            if !after.comments.iter().any(|other| other.seq == comment.seq) {
                blocks.push(comment_block(Side::Before, comment));
            }
        }
        for nick in before.subscribers {
            if !after.subscribers.iter().any(|other| other == nick) {
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
        let previous = before.and_then(|brief| {
            brief
                .entries
                .iter()
                .find(|other| other.title == entry.title)
        });
        match previous {
            None => blocks.push(entry_block(Side::After, entry)),
            Some(previous) => {
                let identity = vec![entry.title.to_string()];
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
            if !after.entries.iter().any(|other| other.title == entry.title) {
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
                before: before_titles.join(", "),
                after: after_titles.join(", "),
            });
        }
    }
    rows
}

fn profile_blocks(before: Option<&ProfileView<'_>>, after: &ProfileView<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for section in &after.sections {
        let previous = before.and_then(|profile| {
            profile
                .sections
                .iter()
                .find(|other| other.name == section.name)
        });
        match previous {
            None => blocks.push(section_block(Side::After, section)),
            Some(previous) => {
                if previous.body != section.body {
                    blocks.push(update_block(
                        "section",
                        &[names::fold(section.name)],
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
            if !after
                .sections
                .iter()
                .any(|other| other.name == section.name)
            {
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
                    .map(|name| names::fold(name))
                    .collect::<Vec<_>>()
                    .join(", "),
                after: after_names
                    .iter()
                    .map(|name| names::fold(name))
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
            let previous_declaration = previous.and_then(|previous| {
                previous
                    .declarations
                    .iter()
                    .find(|other| other.nick == declaration.nick)
            });
            match previous_declaration {
                None => blocks.push(declaration_block(Side::After, section.status, declaration)),
                Some(previous_declaration) => {
                    if previous_declaration.start != declaration.start {
                        blocks.push(update_block(
                            section.status,
                            &[declaration.nick.to_string()],
                            Some("start"),
                            time::format(previous_declaration.start),
                            time::format(declaration.start),
                        ));
                    }
                    if previous_declaration.until != declaration.until {
                        blocks.push(update_block(
                            section.status,
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
                if !after_section
                    .map(|found| {
                        found
                            .declarations
                            .iter()
                            .any(|other| other.nick == declaration.nick)
                    })
                    .unwrap_or(false)
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
                        list: names::fold(section.status),
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
                    .map(|status| names::fold(status))
                    .collect::<Vec<_>>()
                    .join(", "),
                after: after_statuses
                    .iter()
                    .map(|status| names::fold(status))
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
        let previous =
            before.and_then(|post| post.comments.iter().find(|other| other.seq == comment.seq));
        match previous {
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
    let before_declarations = before
        .and_then(|state| state.find(status))
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    let after_declarations = after
        .and_then(|state| state.find(status))
        .map(|section| section.declarations.as_slice())
        .unwrap_or(&[]);
    let mut blocks = Vec::new();
    for declaration in after_declarations {
        match before_declarations
            .iter()
            .find(|other| other.nick == declaration.nick)
        {
            None => blocks.push(declaration_block(Side::After, status, declaration)),
            Some(previous) => {
                if previous.start != declaration.start {
                    blocks.push(update_block(
                        status,
                        &[declaration.nick.to_string()],
                        Some("start"),
                        time::format(previous.start),
                        time::format(declaration.start),
                    ));
                }
                if previous.until != declaration.until {
                    blocks.push(update_block(
                        status,
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
        if !after_declarations
            .iter()
            .any(|other| other.nick == declaration.nick)
        {
            blocks.push(declaration_block(Side::Before, status, declaration));
        }
    }
    blocks
}

fn same_set_different_order<T: PartialEq>(before: &[T], after: &[T]) -> bool {
    before != after && before.len() == after.len() && before.iter().all(|item| after.contains(item))
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
    // `display_path` renders an empty parent as `.` (spec §8.1.3).
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    fsx::walk::display_path(parent)
}
