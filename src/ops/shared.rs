//! The single implementation of state-file collection (§10.5.1 scope),
//! directory identity (§10.5.2), conflict computation (§10.5) and the
//! functional-lock verdict (§10.8).
//!
//! Consumed by `state`, `profile`, `brief` and `query`. Nothing here prints
//! or takes an exclusive lock: state files are only ever read through
//! [`fsx::read`] under a shared lock within `SCAN_READ_WAIT_MS`, including
//! the functional-lock check that runs while the caller holds the target
//! file's exclusive lock (§10.8.3).
//!
//! Scan incompleteness is data, never a swallowed error: a state file that
//! cannot be fully incorporated carries an [`ScanIssue`] and the consumer
//! renders the §10.5.6 `skipped` / `partial` rows.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::Time;
use crate::consts;
use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;

use super::context::{self, Ctx};

/// One parseable state declaration flattened from its section (spec §6.2).
///
/// `status` is the raw H2 status title. A section whose title equals
/// `"locked"` necessarily matches the STATUS grammar (§2.3), so filtering on
/// `status == "locked"` automatically implements "invalid status names never
/// count as locked" (§6.2.2) without an extra flag.
#[derive(Debug, Clone)]
pub struct StateEntryRec {
    pub status: String,
    pub nick: String,
    pub start: Time,
    pub until: Time,
}

impl StateEntryRec {
    /// Valid entry (spec §1): `start <= until` and `now < until`.
    pub fn valid(&self, now: Time) -> bool {
        self.start <= self.until && now < self.until
    }
}

/// Why a state file could not be fully incorporated into a scan (§10.5.6).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanIssue {
    /// Lock busy within `SCAN_READ_WAIT_MS` → `skipped <dir> busy`.
    Busy,
    /// A blocking diagnostic made the file unparseable → `skipped <dir>
    /// <E-ID>`; entries are empty. Also used with the non-diagnostic word
    /// `"unreadable"` for hard read failures (spec §1, §10.1.1), which
    /// renders as `skipped <dir> unreadable`.
    Blocking(&'static str),
    /// Parseable entries kept; omitted section or declaration text is
    /// reported → `partial <dir> <ID> L<n>`.
    Partial { id: &'static str, line: usize },
}

/// What a reader opens to see what an incomplete scan could not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Follow {
    /// The state file of this directory was skipped or partial:
    /// `check <dir>/state`.
    StateOf(PathBuf),
    /// This managed file was skipped or partial: `check <path>`.
    File(PathBuf),
    /// This directory could not be read or entered: `check-state <dir>` once
    /// it is readable and enterable.
    Unreadable(PathBuf),
    /// The state file of a directory the walk could not read either (its
    /// name was listed, the file could not be opened): `check <dir>/state`
    /// once the directory is readable.
    StateInUnreadable(PathBuf),
    /// Directory identity fell back to lexical paths (`canonicalize` rows)
    /// in a scan declared on this directory: `check-state <dir>` on that
    /// declared directory, once the path exists and every ancestor grants
    /// read and search permission.
    DeclaredDir(PathBuf),
    /// Directory identity fell back to lexical paths (`canonicalize` rows)
    /// in a write command's lock scan: `check-state <dir>` on the written
    /// file's parent directory, once the path exists and every ancestor
    /// grants read and search permission.
    WrittenFileDir(PathBuf),
    /// A `list` leaf of unknown type (`?` or `busy`): `check <path>`, which
    /// reads it again with the single-file lock wait and says why.
    Leaf(PathBuf),
    /// A directory `list` could not read or enter: `list <dir>` once it is
    /// readable and enterable.
    Listing(PathBuf),
}

/// One displayed `skipped` / `partial` row (§10.5.6).
#[derive(Debug, Clone)]
pub struct ScanRow {
    pub key: &'static str,
    /// Display path first, then the cause columns.
    pub values: Vec<String>,
    pub follow: Follow,
}

/// Row key and cause columns of one state-file issue. Every renderer uses
/// this, so one issue always has the same columns: `busy`, `<ID>`, or
/// `<ID>` and `L<n>` as separate values for a partial file.
pub fn issue_row(issue: &ScanIssue) -> (&'static str, Vec<String>) {
    match issue {
        ScanIssue::Busy => ("skipped", vec!["busy".to_string()]),
        ScanIssue::Blocking(id) => ("skipped", vec![(*id).to_string()]),
        ScanIssue::Partial { id, line } => ("partial", vec![(*id).to_string(), format!("L{line}")]),
    }
}

/// The `skipped` / `partial` rows of a scope's state-file issues and
/// unreadable directories. Rows keep native source identity (and
/// de-duplication) and are then ordered by displayed cwd-relative
/// directory path (§10.5.4).
pub fn scan_rows(issues: &[(PathBuf, ScanIssue)], unreadable_dirs: &[PathBuf]) -> Vec<ScanRow> {
    // (dir, key, cause, state file issue, directory unreadable)
    let mut tail: Vec<(&Path, &'static str, Vec<String>, bool, bool)> =
        Vec::with_capacity(issues.len() + unreadable_dirs.len());
    for (dir, issue) in issues {
        let (key, cause) = issue_row(issue);
        tail.push((dir.as_path(), key, cause, true, false));
    }
    for dir in unreadable_dirs {
        tail.push((
            dir.as_path(),
            "skipped",
            vec!["unreadable".to_string()],
            false,
            true,
        ));
    }
    tail.sort();
    // A state file that could not be opened (`skipped <dir> unreadable`) in a
    // directory the walk could not read either is one row; its next step
    // names the file and says the directory must be made readable first.
    tail.dedup_by(|later, earlier| {
        let same = later.0 == earlier.0 && later.1 == earlier.1 && later.2 == earlier.2;
        if same {
            earlier.3 |= later.3;
            earlier.4 |= later.4;
        }
        same
    });
    // Display paths may collide; rendering changes ordering, never identity.
    let mut rows: Vec<(String, ScanRow)> = tail
        .into_iter()
        .map(|(dir, key, cause, state, unreadable)| {
            let display = context::object_of_dir(dir);
            let mut values = Vec::with_capacity(1 + cause.len());
            values.push(display.clone());
            values.extend(cause);
            let dir = dir.to_path_buf();
            let follow = match (state, unreadable) {
                (true, true) => Follow::StateInUnreadable(dir),
                (false, _) => Follow::Unreadable(dir),
                (true, false) => Follow::StateOf(dir),
            };
            (
                display,
                ScanRow {
                    key,
                    values,
                    follow,
                },
            )
        })
        .collect();
    rows.sort_by(|a, b| (&a.0, a.1.key, &a.1.values[1..]).cmp(&(&b.0, b.1.key, &b.1.values[1..])));
    rows.into_iter().map(|(_, row)| row).collect()
}

/// Fill a scan follow-up fix (`{unreadable}`, `{commands}`): one command per
/// distinct thing the scan could not read, joined by `; `. When any path
/// cannot be written as a copyable operand, the `manual` template lists the
/// commands as descriptions instead. `{unreadable}` is empty, or
/// [`diag::SCAN_UNREADABLE_FIRST`] and/or [`diag::SCAN_CANONICALIZE_FIRST`].
pub fn scan_fix<'a>(
    follows: impl IntoIterator<Item = &'a Follow>,
    template: &str,
    manual: &str,
) -> String {
    let mut commands: Vec<String> = Vec::new();
    let mut copyable = true;
    let (mut state_of, mut file, mut unreadable) = (false, false, false);
    let (mut declared, mut written_file) = (false, false);
    let (mut leaf, mut listing) = (false, false);
    // `{unreadable}`: some directory must be made readable first, and/or a
    // canonicalize fallback must restore existence and ancestor search.
    let mut readable_first = false;
    for follow in follows {
        let command = match follow {
            Follow::StateOf(dir) => {
                state_of = true;
                shown_operand(dir, Some("state")).map(|op| format!("agent-coexist check {op}"))
            }
            Follow::StateInUnreadable(dir) => {
                state_of = true;
                readable_first = true;
                shown_operand(dir, Some("state")).map(|op| format!("agent-coexist check {op}"))
            }
            Follow::File(path) => {
                file = true;
                managed_operand(path).map(|op| format!("agent-coexist check {op}"))
            }
            Follow::Unreadable(dir) => {
                unreadable = true;
                readable_first = true;
                shown_operand(dir, None).map(|op| format!("agent-coexist check-state {op}"))
            }
            Follow::DeclaredDir(dir) => {
                declared = true;
                diag::path_operand(dir.as_os_str())
                    .map(|op| format!("agent-coexist check-state {op}"))
            }
            Follow::WrittenFileDir(dir) => {
                written_file = true;
                diag::path_operand(dir.as_os_str())
                    .map(|op| format!("agent-coexist check-state {op}"))
            }
            Follow::Leaf(path) => {
                leaf = true;
                managed_operand(path).map(|op| format!("agent-coexist check {op}"))
            }
            Follow::Listing(dir) => {
                listing = true;
                readable_first = true;
                shown_operand(dir, None).map(|op| format!("agent-coexist list {op}"))
            }
        };
        match command {
            Some(command) => {
                if !commands.contains(&command) {
                    commands.push(command);
                }
            }
            None => copyable = false,
        }
    }
    let mut extra = String::new();
    if readable_first {
        extra.push_str(diag::SCAN_UNREADABLE_FIRST);
    }
    if declared || written_file {
        extra.push_str(diag::SCAN_CANONICALIZE_FIRST);
    }
    // `{commands}` last: a path must never be read as a placeholder.
    if copyable {
        template
            .replace("{unreadable}", &extra)
            .replace("{commands}", &commands.join("; "))
    } else {
        let mut parts: Vec<&str> = Vec::with_capacity(4);
        if file {
            parts.push(diag::MANUAL_CHECK_SCAN_FILES);
        } else if state_of {
            parts.push(diag::MANUAL_CHECK_STATE_FILES);
        }
        if leaf {
            parts.push(diag::MANUAL_CHECK_LIST_LEAVES);
        }
        if unreadable {
            parts.push(diag::MANUAL_CHECK_STATE_UNREADABLE);
        }
        if listing {
            parts.push(diag::MANUAL_LIST_UNREADABLE);
        }
        if declared {
            parts.push(diag::MANUAL_CHECK_STATE_DECLARED);
        }
        if written_file {
            parts.push(diag::MANUAL_CHECK_STATE_WRITTEN_FILE);
        }
        manual
            .replace("{unreadable}", &extra)
            .replace("{commands}", &parts.join(", and "))
    }
}

/// Columns of one `canonicalize` row: display path, `failed`, then the
/// bounded OS reason when it is not empty.
pub fn canonicalize_values(dir: &Path, reason: &str) -> Vec<String> {
    let mut values = vec![context::object_of_dir(dir), "failed".to_string()];
    if !reason.is_empty() {
        values.push(reason.to_string());
    }
    values
}

/// Copyable operand for a scanned path as the reader sees it (cwd-relative
/// display, `/`-joined), optionally with one more component. `None` when
/// the display would not be the same path (non-UTF-8) or cannot be quoted.
fn shown_operand(path: &Path, tail: Option<&str>) -> Option<String> {
    let path = lexical_dir(path);
    path.to_str()?;
    let shown = fsx::walk::display_path(&path);
    let shown = match tail {
        Some(tail) if shown == "." => tail.to_string(),
        Some(tail) => format!("{shown}/{tail}"),
        None => shown,
    };
    diag::path_operand(OsStr::new(&shown))
}

/// `check` operand of a managed file: its display path without the exact
/// managed suffix. A differently-cased suffix cannot be addressed by a
/// PATH operand (§2.4.3), so it has no copyable command.
fn managed_operand(path: &Path) -> Option<String> {
    path.to_str()?;
    let shown = fsx::walk::display_path(path);
    let stem = shown
        .strip_suffix(consts::SUFFIX)
        .filter(|stem| !stem.is_empty())?;
    diag::path_operand(OsStr::new(stem))
}

/// Diagnostics that keep a state file out of a scan entirely (§10.6.4):
/// the structure is undecidable, so no declaration can be trusted.
const SKIP_CLASS: [&str; 4] = ["E-UTF8", "E-TOO-LARGE", "E-H1", "E-FENCE"];
/// Diagnostics whose parseable remainder still participates (§10.6.4):
/// entries are kept and the file is reported `partial`. `E-STATUS-DUP`
/// remains write-blocking (`diag::is_blocking` gates every write), but a
/// duplicate-section state file's declarations are exactly what a lock
/// scan must not lose; `W-STATE-ENTRY` marks in-section declaration text
/// that could not be incorporated, and `W-STRAY` marks non-comment stray
/// text that never became a section. Losing either silently would turn
/// omitted lock bytes into a clean `ok`. Reader comments are not stray.
/// Other formatting warnings carry no declaration loss and stay
/// non-partial.
const PARTIAL_CLASS: [&str; 5] = [
    "E-HEAD",
    "E-SEQ-DUP",
    "E-STATUS-DUP",
    "W-STATE-ENTRY",
    "W-STRAY",
];

/// One scanned state file: its identity and every parseable declaration.
#[derive(Debug, Clone)]
pub struct StateFileInfo {
    /// The actual state file path (on-disk casing preserved).
    pub file: PathBuf,
    /// Canonicalized directory (lexical fallback when canonicalize fails;
    /// the failure is recorded in [`StateScope::canonicalize_failed`]).
    pub dir: PathBuf,
    /// Folded cwd-relative display of the directory (§8.1.9, §10.5.5 —
    /// never the canonicalized absolute form).
    pub display_dir: String,
    /// All parseable declarations in file order, duplicates included.
    pub entries: Vec<StateEntryRec>,
    /// Selected raw positions, cached without discarding historical rows.
    effective_positions: Vec<usize>,
    /// Why the file could not be fully incorporated, if applicable.
    pub issue: Option<ScanIssue>,
}

impl StateFileInfo {
    /// Latest-until declaration per (status, nick), in source order.
    pub fn effective_entries(&self) -> impl Iterator<Item = &StateEntryRec> {
        self.effective_positions.iter().map(|&i| &self.entries[i])
    }
}

/// One directory whose canonical identity could not be resolved (§10.5.2).
#[derive(Debug, Clone)]
pub struct CanonicalizeFail {
    /// Native lexical source spelling used as the warning key.
    pub dir: PathBuf,
    /// Bounded OS reason (`names::display_text` of `io::Error::to_string()`).
    pub reason: String,
}

/// The §10.5.1 scope of a directory: itself, its ancestors and its subtree.
#[derive(Debug, Clone)]
pub struct StateScope {
    /// Canonicalized target directory (lexical fallback).
    pub root: PathBuf,
    /// Directories whose canonicalization failed (§10.5.2 warn rows), each
    /// with its bounded OS reason.
    pub canonicalize_failed: Vec<CanonicalizeFail>,
    /// The target directory's own state file, when present and readable.
    pub self_file: Option<StateFileInfo>,
    /// Ancestor state files, near → far.
    pub ancestors: Vec<StateFileInfo>,
    /// Subtree state files, canonical directory ascending.
    pub descendants: Vec<StateFileInfo>,
    /// Native lexical source dir → issue; consumers order the displayed rows.
    pub issues: Vec<(PathBuf, ScanIssue)>,
    /// Native lexical dirs that could not be entered/read.
    pub unreadable_dirs: Vec<PathBuf>,
    /// Unmatched includes from the subtree walk; hints are not records.
    pub zero_includes: Vec<String>,
    /// The subtree walk loaded at least one ignore source.
    pub ignore_in_play: bool,
}

/// One valid `locked` entry as reported by a conflict scan.
#[derive(Debug, Clone)]
pub struct ConflictRec<'a> {
    pub status: &'a str,
    pub nick: &'a str,
    pub start: Time,
    pub until: Time,
    /// Native directory identity borrowed from the scope, never a display key.
    pub dir: &'a Path,
    /// Display of the state file's directory (the other party's dir).
    pub dir_display: &'a str,
    /// Selected declaration's original position, before conflict sorting.
    pub source_order: usize,
}

/// Functional-lock verdict (§10.8): holders plus directory scan metadata.
#[derive(Debug, Clone)]
pub struct FlockOutcome {
    /// Valid `locked` holders with `nick != as_nick` (all holders when
    /// `as_nick` is `None`), sorted `until` descending then nick ascending;
    /// the first row is the `E-FLOCKED` `{holder}`.
    pub holders: Vec<FlockHolder>,
    /// Supplied-directory scope plus incompleteness from both chains.
    pub scope: StateScope,
}

/// One functional-lock holder row (`also-locked`, §10.8.2).
#[derive(Debug, Clone)]
pub struct FlockHolder {
    pub nick: String,
    pub until: Time,
    pub dir_display: String,
}

/// Read one state file under a shared lock (`SCAN_READ_WAIT_MS`) and flatten
/// its parseable declarations (spec §10.5.6).
///
/// `ctx.file_read()` advances the managed-file counter whether the read
/// succeeds or fails (§8.4.1). Classification follows §10.6.4: skip-class
/// diagnostics (undecidable structure) leave `entries` empty; partial-class
/// errors — notably `E-STATUS-DUP`, still write-blocking — keep the
/// parseable entries. A reserved state-file name whose parseable content is
/// another kind is never "no state file": it is skipped as `E-TYPE` so its
/// directory stays visible in the scan instead of silently losing every
/// declaration it might have carried.
pub fn read_state_file(ctx: &mut Ctx, file: &Path) -> StateFileInfo {
    let bytes = match fsx::read(file, consts::SCAN_READ_WAIT_MS, None) {
        Ok(bytes) => bytes,
        Err(fault) => {
            ctx.file_read();
            let mut info = empty_state_info(file);
            match fault {
                Fault::Reason { id, .. } if id == "E-LOCKED" || id == "E-INUSE" => {
                    info.issue = Some(ScanIssue::Busy);
                }
                Fault::Diagnostics(ds) => {
                    info.issue = classify_diag_issue(&ds);
                }
                Fault::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound => {
                    // Following a dangling symlink also returns NotFound. Only
                    // an absent directory entry means the metadata vanished.
                    if !matches!(fs::symlink_metadata(file), Err(e) if e.kind() == std::io::ErrorKind::NotFound)
                    {
                        info.issue = Some(ScanIssue::Blocking("unreadable"));
                    }
                }
                Fault::Io { .. } => {
                    info.issue = Some(ScanIssue::Blocking("unreadable"));
                }
                _ => {}
            }
            return info;
        }
    };
    ctx.file_read();
    match doc::parse(bytes, file, ctx.now) {
        Ok(d) => state_file_info(file, &d),
        Err(Fault::Diagnostics(ds)) => {
            let mut info = empty_state_info(file);
            info.issue = classify_diag_issue(&ds);
            info
        }
        Err(_) => empty_state_info(file),
    }
}

/// Build both raw history and its effective view from a document already
/// read by a notification scan. Metadata classification is identical to
/// the direct root/ancestor reader, without reading the bytes twice.
pub(crate) fn state_file_info(file: &Path, document: &doc::Document) -> StateFileInfo {
    let mut info = empty_state_info(file);
    if let Some(first) = document
        .diagnostics
        .iter()
        .find(|x| SKIP_CLASS.contains(&x.id))
    {
        info.issue = Some(ScanIssue::Blocking(first.id));
    } else if document.kind != Some(doc::Kind::State) {
        info.issue = Some(ScanIssue::Blocking("E-TYPE"));
    } else {
        info.entries = document
            .state
            .iter()
            .flat_map(|section| {
                section.entries.iter().map(move |entry| StateEntryRec {
                    status: section.status.clone(),
                    nick: entry.nick.clone(),
                    start: entry.start,
                    until: entry.until,
                })
            })
            .collect();
        info.effective_positions = crate::types::state::select_latest(
            info.entries
                .iter()
                .enumerate()
                .map(|(i, entry)| (entry.status.as_str(), entry.nick.as_str(), entry.until, i)),
        );
        if let Some(first) = document
            .diagnostics
            .iter()
            .find(|x| PARTIAL_CLASS.contains(&x.id))
        {
            info.issue = Some(ScanIssue::Partial {
                id: first.id,
                line: first.line,
            });
        }
    }
    info
}

fn empty_state_info(file: &Path) -> StateFileInfo {
    let dir = parent_dir(file);
    StateFileInfo {
        file: file.to_path_buf(),
        display_dir: display_of(&dir),
        dir,
        entries: Vec::new(),
        effective_positions: Vec::new(),
        issue: None,
    }
}

/// Match the reserved name only after filtering, and retain state_file's
/// first-on-disk-name authority when case variants coexist. A surviving
/// unrelated file can never trigger an unfiltered metadata lookup.
pub(crate) fn is_reserved_state_file(file: &Path) -> Result<bool, Fault> {
    let Some(name) = file
        .file_name()
        .filter(|name| fsx::walk::is_state_file_name(name))
    else {
        return Ok(false);
    };
    // Both names belong to the queried parent. Compare native names so a
    // bare filename and `./filename` agree without changing case authority.
    Ok(fsx::walk::state_file(&parent_dir(file))?
        .is_some_and(|selected| selected.file_name() == Some(name)))
}

/// Collect the full §10.5.1 scope of `dir`: the dir itself, all its
/// ancestors (canonicalized basis, §10.5.2) and its subtree via
/// [`fsx::walk::scan`] with the caller's options.
///
/// Ordinary unreadable directories stay in
/// [`StateScope::unreadable_dirs`]. A fatal ignore/config source error
/// keeps its typed [`Fault::Source`] (source path, cause and repair kind)
/// instead of being rewritten as an unreadable root.
pub fn collect(ctx: &mut Ctx, dir: &Path, walk: &fsx::walk::Options) -> Result<StateScope, Fault> {
    let mut files: Vec<StateFileInfo> = Vec::new();
    let mut unreadable: Vec<PathBuf> = Vec::new();

    // Self + ancestors on the normalized path.
    let (root_norm, _root_ok, _failed) = normalize_dir(dir);
    for d in chain_dirs(&root_norm) {
        match fsx::walk::state_file(&d) {
            Ok(Some(f)) => files.push(read_state_file(ctx, &f)),
            Ok(None) => {}
            Err(_) => push_uniq(&mut unreadable, lexical_dir(&d)),
        }
    }

    // Subtree metadata participates only when the reserved file itself
    // survives the walk filters. The root is handled by the chain above.
    let mut sink = ctx.walk_sink();
    let scanned = fsx::walk::scan(dir, walk, &mut sink);
    drop(sink);
    let walked = scanned?;
    for f in &walked.files {
        if !f.file_name().is_some_and(fsx::walk::is_state_file_name) {
            continue;
        }
        let parent = parent_dir(f);
        if normalize_dir(&parent).0 == root_norm {
            continue;
        }
        match is_reserved_state_file(f) {
            Ok(true) => files.push(read_state_file(ctx, f)),
            Ok(false) => {}
            Err(_) => push_uniq(&mut unreadable, lexical_dir(&parent)),
        }
    }
    for ud in &walked.unreadable_dirs {
        push_uniq(&mut unreadable, lexical_dir(ud));
    }

    let mut scope = assemble(dir, files, unreadable);
    scope.zero_includes = walked.zero_includes;
    scope.ignore_in_play = ignore_in_play(&walked.ignore_sources);
    Ok(scope)
}

/// Whether a walk loaded any ignore source: `WalkResult::ignore_sources`
/// is `none` or `--no-ignore` otherwise.
pub fn ignore_in_play(sources: &[String]) -> bool {
    sources.iter().any(|s| s != "none" && s != "--no-ignore")
}

/// Assemble a scope from already-read state files (single pass): classify
/// each file against the root's canonicalized identity, order the groups
/// (§10.5.4) and collect the incompleteness markers.
pub fn assemble(
    dir: &Path,
    mut files: Vec<StateFileInfo>,
    unreadable_dirs: Vec<PathBuf>,
) -> StateScope {
    // Misnamed .state documents remain notification history, not directory
    // metadata. They must never shadow or supply a functional lock.
    files.retain(|info| {
        info.file
            .file_name()
            .is_some_and(fsx::walk::is_state_file_name)
    });
    let (root, root_ok, root_failed) = normalize_dir(dir);
    // Repeated encounters may use canonical, absolute or relative paths.
    // Give each native directory one lexical source spelling across all
    // incompleteness classes; display strings never participate in this map.
    let mut source_dirs: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut source_dir = |path: &Path, identity: PathBuf| {
        source_dirs
            .entry(identity)
            .or_insert_with(|| lexical_dir(path))
            .clone()
    };
    let mut failed: Vec<CanonicalizeFail> = Vec::new();
    if let Some((path, reason)) = root_failed {
        push_failed(
            &mut failed,
            source_dir(&path, lexical_identity(&root)),
            reason,
        );
    }
    let mut issues: Vec<(PathBuf, ScanIssue)> = Vec::new();

    let mut self_file: Option<StateFileInfo> = None;
    let mut ancestors: Vec<StateFileInfo> = Vec::new();
    let mut descendants: Vec<StateFileInfo> = Vec::new();

    for mut info in files {
        let lex = info.dir.clone();
        let (canon, canon_err) = match fs::canonicalize(&lex) {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(names::display_text(&e.to_string()))),
        };
        if info.issue.is_some() || canon.is_none() {
            let source = source_dir(
                &parent_dir(&info.file),
                lexical_identity(canon.as_deref().unwrap_or(&lex)),
            );
            if let Some(issue) = &info.issue {
                issues.push((source.clone(), issue.clone()));
            }
            if let Some(reason) = canon_err {
                push_failed(&mut failed, source, reason);
            }
        }
        let mut rel = Rel::Other;
        if root_ok && let Some(c) = canon.as_ref() {
            rel = rel_between(c, &root);
        }
        if rel == Rel::Other {
            // Canonical basis unavailable (root or dir failed to
            // canonicalize): compare the lexical forms instead (§13 Q4).
            rel = rel_between(&lex, dir);
        }
        info.dir = canon.unwrap_or(lex);
        match rel {
            Rel::SelfDir => {
                if self_file.is_none() {
                    self_file = Some(info);
                }
            }
            Rel::Ancestor => ancestors.push(info),
            Rel::Descendant => descendants.push(info),
            Rel::Other => {}
        }
    }

    // Ancestors near → far (deepest first); descendants canonical-ascending.
    ancestors.sort_by_key(|a| std::cmp::Reverse(a.dir.components().count()));
    ancestors.dedup_by(|a, b| a.dir == b.dir);
    descendants.sort_by(|a, b| a.dir.cmp(&b.dir));
    descendants.dedup_by(|a, b| a.dir == b.dir);

    issues.sort();
    issues.dedup();
    let mut unreadable: Vec<PathBuf> = unreadable_dirs
        .into_iter()
        .map(|path| {
            let identity = directory_identity(&path);
            source_dir(&path, identity)
        })
        .collect();
    unreadable.sort();
    unreadable.dedup();

    StateScope {
        root,
        canonicalize_failed: failed,
        self_file,
        ancestors,
        descendants,
        issues,
        unreadable_dirs: unreadable,
        zero_includes: Vec::new(),
        ignore_in_play: false,
    }
}

/// Functional-lock verdict (§10.8): the supplied directory and its ancestors,
/// plus the resolved file's directory chain when different. Neither chain
/// scans a subtree (§10.8.4).
///
/// Runs while the caller holds the target's exclusive lock; only shared
/// reads happen here.
pub fn functional_lock(ctx: &mut Ctx, target_file: &Path, as_nick: Option<&str>) -> FlockOutcome {
    let dir = parent_dir(target_file);
    let (root_norm, _root_ok, _failed) = normalize_dir(&dir);
    let mut dirs = chain_dirs(&root_norm);
    // A file symlink adds the referent chain; it never replaces the supplied
    // chain. Resolve best-effort and read shared native directories only once.
    if let Ok(referent) = fs::canonicalize(target_file)
        && let Some(referent_dir) = referent
            .parent()
            .filter(|parent| *parent != root_norm.as_path())
    {
        for d in chain_dirs(referent_dir) {
            push_uniq(&mut dirs, d);
        }
    }
    let mut files: Vec<StateFileInfo> = Vec::new();
    let mut unreadable: Vec<PathBuf> = Vec::new();
    for d in dirs {
        match fsx::walk::state_file(&d) {
            Ok(Some(f)) => files.push(read_state_file(ctx, &f)),
            Ok(None) => {}
            Err(_) => push_uniq(&mut unreadable, lexical_dir(&d)),
        }
    }

    let mut holders: Vec<FlockHolder> = Vec::new();
    // These records all came from one of the two chains. A referent outside
    // the supplied scope must still contribute holders before assembly.
    for info in &files {
        for e in info.effective_entries() {
            if e.status != "locked" || !e.valid(ctx.now) {
                continue;
            }
            if let Some(n) = as_nick
                && e.nick == n
            {
                continue;
            }
            holders.push(FlockHolder {
                nick: e.nick.clone(),
                until: e.until,
                dir_display: info.display_dir.clone(),
            });
        }
    }
    holders.sort_by(|a, b| b.until.cmp(&a.until).then_with(|| a.nick.cmp(&b.nick)));
    let scope = assemble(&dir, files, unreadable);
    FlockOutcome { holders, scope }
}

/// All valid `locked` entries in the scope, §10.5.4 order: self file →
/// ancestors near → far → descendants canonical-ascending; within one file
/// by `until` ascending, then nick.
pub fn valid_locked(scope: &StateScope, now: Time) -> Vec<ConflictRec<'_>> {
    let mut out = Vec::new();
    for info in scope
        .self_file
        .iter()
        .chain(scope.ancestors.iter())
        .chain(scope.descendants.iter())
    {
        let mut recs: Vec<ConflictRec<'_>> = info
            .effective_positions
            .iter()
            .map(|&i| (i, &info.entries[i]))
            .filter(|(_, e)| e.status == "locked" && e.valid(now))
            .map(|(source_order, e)| ConflictRec {
                status: &e.status,
                nick: &e.nick,
                start: e.start,
                until: e.until,
                dir: &info.dir,
                dir_display: &info.display_dir,
                source_order,
            })
            .collect();
        recs.sort_by(|a, b| a.until.cmp(&b.until).then_with(|| a.nick.cmp(b.nick)));
        out.extend(recs);
    }
    out
}

/// The other parties whose valid `locked` entries conflict with `nick`'s
/// valid `locked` entries in the scope (§10.5.3), §10.5.4 order. Each row
/// includes the display of the deepest overlapping own lock; equal depths
/// select the last file in scope order. Empty when `nick` has no valid lock.
pub fn conflicts_with<'a>(
    scope: &'a StateScope,
    now: Time,
    nick: &str,
) -> Vec<(ConflictRec<'a>, &'a str)> {
    let all = valid_locked(scope, now);
    if !all.iter().any(|rec| rec.nick == nick) {
        return Vec::new();
    }
    let index = LockIndex::new(&all, Some(nick));
    all.into_iter()
        .enumerate()
        .filter_map(|(position, rec)| {
            if rec.nick == nick {
                return None;
            }
            index
                .overlap(position)
                .own
                .map(|(_, _, display)| (rec, display))
        })
        .collect()
}

/// Both sides of every conflicting pair in the scope (§10.5.3), deduped to
/// one row per entry, §10.5.4 order.
pub fn conflicts_all(scope: &StateScope, now: Time) -> Vec<ConflictRec<'_>> {
    let all = valid_locked(scope, now);
    let index = LockIndex::new(&all, None);
    all.into_iter()
        .enumerate()
        .filter(|(position, rec)| index.overlap(*position).has_other(rec.nick))
        .map(|(_, rec)| rec)
        .collect()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn classify_diag_issue(ds: &[Diagnostic]) -> Option<ScanIssue> {
    let first = ds.first()?;
    Some(if SKIP_CLASS.contains(&first.id) {
        ScanIssue::Blocking(first.id)
    } else {
        ScanIssue::Partial {
            id: first.id,
            line: first.line,
        }
    })
}

/// Only the facts needed for one conflict row, not the underlying pairs.
#[derive(Clone, Copy, Default)]
struct LockSummary<'a> {
    nick: Option<&'a str>,
    mixed: bool,
    /// Depth then original record order: maximum preserves last-on-depth-tie.
    own: Option<(usize, usize, &'a str)>,
}

impl<'a> LockSummary<'a> {
    fn merge(&mut self, other: Self) {
        if let Some(nick) = other.nick {
            if let Some(first) = self.nick {
                self.mixed |= first != nick;
            } else {
                self.nick = Some(nick);
            }
        }
        self.mixed |= other.mixed;
        self.own = self.own.max(other.own);
    }

    fn has_other(&self, nick: &str) -> bool {
        self.nick.is_some_and(|first| self.mixed || first != nick)
    }
}

#[derive(Default)]
struct LockDir<'a> {
    parent: usize,
    depth: usize,
    below: LockSummary<'a>,
    above: LockSummary<'a>,
}

/// A scan-local index of the existing native directory components. Its
/// prefixes have exactly `Path::starts_with` semantics, including roots and
/// lexical fallbacks. No path is rendered, canonicalized again, or cloned.
/// With C input components and R records, construction takes expected O(C + R)
/// component lookups and O(C + R) space; each overlap summary is O(1).
struct LockIndex<'a> {
    dirs: Vec<LockDir<'a>>,
    record_dirs: Vec<usize>,
}

impl<'a> LockIndex<'a> {
    fn new(records: &[ConflictRec<'a>], own_nick: Option<&str>) -> Self {
        let mut dirs = vec![LockDir::default()];
        let mut children: HashMap<(usize, Component<'a>), usize> = HashMap::new();
        let mut record_dirs = Vec::with_capacity(records.len());
        for (order, rec) in records.iter().enumerate() {
            let mut dir = 0;
            for component in rec.dir.components() {
                dir = *children.entry((dir, component)).or_insert_with(|| {
                    let child = dirs.len();
                    dirs.push(LockDir {
                        parent: dir,
                        depth: dirs[dir].depth + 1,
                        ..Default::default()
                    });
                    child
                });
            }
            let own =
                (own_nick == Some(rec.nick)).then_some((dirs[dir].depth, order, rec.dir_display));
            dirs[dir].below.merge(LockSummary {
                nick: Some(rec.nick),
                mixed: false,
                own,
            });
            record_dirs.push(dir);
        }
        // Parents are created before children. Capture strict ancestors
        // while `below` still contains only the declarations at that node.
        for dir in 1..dirs.len() {
            let parent = dirs[dir].parent;
            let mut above = dirs[parent].above;
            above.merge(dirs[parent].below);
            dirs[dir].above = above;
        }
        // Fold descendants upward once; sibling subtrees never enter a
        // record's own ancestor/subtree union.
        for dir in (1..dirs.len()).rev() {
            let parent = dirs[dir].parent;
            let below = dirs[dir].below;
            dirs[parent].below.merge(below);
        }
        Self { dirs, record_dirs }
    }

    fn overlap(&self, position: usize) -> LockSummary<'a> {
        let dir = &self.dirs[self.record_dirs[position]];
        let mut summary = dir.below;
        summary.merge(dir.above);
        summary
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rel {
    SelfDir,
    Ancestor,
    Descendant,
    Other,
}

fn rel_between(dir: &Path, root: &Path) -> Rel {
    if dir == root {
        Rel::SelfDir
    } else if root.starts_with(dir) {
        Rel::Ancestor
    } else if dir.starts_with(root) {
        Rel::Descendant
    } else {
        Rel::Other
    }
}

/// Canonicalized identity of `dir` (§10.5.2); on failure retain the native
/// lexical source for the warning, independently of its eventual display,
/// together with a bounded OS reason.
fn normalize_dir(dir: &Path) -> (PathBuf, bool, Option<(PathBuf, String)>) {
    match fs::canonicalize(dir) {
        Ok(p) => (p, true, None),
        Err(e) => (
            dir.to_path_buf(),
            false,
            Some((lexical_dir(dir), names::display_text(&e.to_string()))),
        ),
    }
}

/// Native directory key for repeated scan encounters. Canonicalization is
/// best-effort; its warning metadata is still collected by the scope builder.
/// The lexical fallback uses an absolute basis without passing through text.
pub(crate) fn directory_identity(dir: &Path) -> PathBuf {
    match fs::canonicalize(dir) {
        Ok(path) => lexical_dir(&path),
        Err(_) => lexical_identity(dir),
    }
}

fn lexical_identity(dir: &Path) -> PathBuf {
    match std::path::absolute(dir) {
        Ok(path) => lexical_dir(&path),
        Err(_) => lexical_dir(dir),
    }
}

/// The scope root first, then its ancestors near → far. Robust to any
/// ordering or self-inclusion of [`fsx::walk::ancestors`].
fn chain_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    for a in fsx::walk::ancestors(root) {
        if a.as_os_str().is_empty() || a == *root {
            continue;
        }
        if !out.contains(&a) {
            out.push(a);
        }
    }
    let depth = |p: &Path| p.components().count();
    let mut rest = out.split_off(1);
    rest.sort_by_key(|p| Reverse(depth(p)));
    rest.dedup();
    out.extend(rest);
    out
}

fn parent_dir(p: &Path) -> PathBuf {
    match p.parent() {
        Some(par) if !par.as_os_str().is_empty() => par.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn display_of(p: &Path) -> String {
    context::object_of_dir(&lexical_dir(p))
}

/// Strip the Windows verbatim prefix `fs::canonicalize` adds without
/// converting native path components to text. Warning keys retain this
/// lexical spelling; directory-relation comparisons keep the canonical form.
pub(super) fn lexical_dir(p: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::path::{Component, Prefix};

        let mut components = p.components();
        if let Some(Component::Prefix(prefix)) = components.next() {
            let mut native = match prefix.kind() {
                Prefix::VerbatimDisk(drive) => OsString::from(format!("{}:", char::from(drive))),
                Prefix::VerbatimUNC(server, share) => {
                    let mut native = OsString::from(r"\\");
                    native.push(server);
                    native.push(r"\");
                    native.push(share);
                    native
                }
                Prefix::Verbatim(name) => OsString::from(name),
                _ => return p.to_path_buf(),
            };
            native.push(components.as_path().as_os_str());
            return PathBuf::from(native);
        }
    }
    p.to_path_buf()
}

fn push_uniq(v: &mut Vec<PathBuf>, s: PathBuf) {
    if !v.contains(&s) {
        v.push(s);
    }
}

fn push_failed(failed: &mut Vec<CanonicalizeFail>, dir: PathBuf, reason: String) {
    if !failed.iter().any(|f| f.dir == dir) {
        failed.push(CanonicalizeFail { dir, reason });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::context::test_ctx;
    use crate::time;
    use std::fs::OpenOptions;
    use std::path::PathBuf;

    /// `now` sits inside both fixture declarations' windows.
    const NOW: &str = "2026-09-25T12:00:00Z";

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-shared-tests")
            .join(format!(
                "{}-{}-{}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
        std::fs::create_dir_all(&dir).unwrap();
        lexical_dir(&std::fs::canonicalize(dir).unwrap())
    }

    fn read_state(dir: &Path, content: &str) -> StateFileInfo {
        let file = dir.join("state.agent-coexist.md");
        std::fs::write(&file, content).unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        read_state_file(&mut ctx, &file)
    }

    #[test]
    fn reserved_state_selection_accepts_bare_filename_without_case_folding() {
        const CHILD: &str = "COEXIST_SHARED_RESERVED_STATE_CHILD";
        if std::env::var_os(CHILD).is_some() {
            assert!(is_reserved_state_file(Path::new("state.agent-coexist.md")).unwrap());
            assert!(is_reserved_state_file(Path::new("./state.agent-coexist.md")).unwrap());
            assert!(!is_reserved_state_file(Path::new("STATE.agent-coexist.md")).unwrap());
            return;
        }
        let dir = scratch("reserved-state-bare-name");
        std::fs::write(dir.join("state.agent-coexist.md"), b"# .state\n").unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "ops::shared::tests::reserved_state_selection_accepts_bare_filename_without_case_folding",
            ])
            .current_dir(&dir)
            .env(CHILD, "1")
            .status()
            .expect("spawn isolated cwd child");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.success(), "bare reserved state selection failed");
    }

    #[test]
    fn conflict_selectors_keep_native_siblings_and_invalid_locks_separate() {
        let dir = scratch("conflict-components");
        let mut files = Vec::new();
        for (name, declarations) in [
            (
                "branch",
                concat!(
                    "alice 2026-09-25T11:00:00Z..2026-09-25T15:00:00Z\n",
                    "zoe 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                    "bob 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                ),
            ),
            (
                "branch/child",
                "carol 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
            (
                "branch-more",
                "dave 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
            ("same", "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"),
            (
                "same/child",
                "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
            (
                "expired",
                concat!(
                    "alice 2026-09-25T11:00:00Z..2026-09-25T12:00:00Z\n",
                    "dave 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
                ),
            ),
            (
                "invalid",
                concat!(
                    "alice 2026-09-25T15:00:00Z..2026-09-25T14:00:00Z\n",
                    "dave 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
                ),
            ),
            (
                "observing",
                concat!(
                    "dave 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
                    "\n## observing\n",
                    "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
                ),
            ),
        ] {
            let path = dir.join(name);
            std::fs::create_dir_all(&path).unwrap();
            files.push(read_state(
                &path,
                &format!("# .state\n\n## locked\n{declarations}"),
            ));
        }
        let scope = assemble(&dir, files, Vec::new());
        let now = time::parse(NOW).unwrap();
        let branch = std::fs::canonicalize(dir.join("branch")).unwrap();
        let child = std::fs::canonicalize(dir.join("branch/child")).unwrap();
        assert_eq!(
            conflicts_with(&scope, now, "alice")
                .iter()
                .map(|(rec, _)| (rec.dir, rec.nick))
                .collect::<Vec<_>>(),
            vec![
                (branch.as_path(), "bob"),
                (branch.as_path(), "zoe"),
                (child.as_path(), "carol"),
            ]
        );
        assert_eq!(
            conflicts_all(&scope, now)
                .iter()
                .map(|rec| (rec.dir, rec.nick))
                .collect::<Vec<_>>(),
            vec![
                (branch.as_path(), "bob"),
                (branch.as_path(), "zoe"),
                (branch.as_path(), "alice"),
                (child.as_path(), "carol"),
            ]
        );
        assert!(conflicts_with(&scope, now, "absent").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn conflict_selectors_keep_self_ancestor_and_descendant_order() {
        let dir = scratch("conflict-scope-order");
        let near = dir.join("near");
        let root = near.join("root");
        let first = root.join("a");
        let last = root.join("z");
        let mut files = Vec::new();
        for (path, declarations) in [
            (&last, "frank 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"),
            (&dir, "erin 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"),
            (&first, "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"),
            (&near, "dan 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"),
            (
                &root,
                concat!(
                    "bob 2026-09-25T11:00:00Z..2026-09-25T15:00:00Z\n",
                    "carol 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                ),
            ),
        ] {
            std::fs::create_dir_all(path).unwrap();
            files.push(read_state(
                path,
                &format!("# .state\n\n## locked\n{declarations}"),
            ));
        }
        let scope = assemble(&root, files, Vec::new());
        let now = time::parse(NOW).unwrap();
        assert_eq!(
            conflicts_with(&scope, now, "alice")
                .iter()
                .map(|(rec, _)| rec.nick)
                .collect::<Vec<_>>(),
            vec!["carol", "bob", "dan", "erin"]
        );
        assert_eq!(
            conflicts_all(&scope, now)
                .iter()
                .map(|rec| rec.nick)
                .collect::<Vec<_>>(),
            vec!["carol", "bob", "dan", "erin", "alice", "frank"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn valid_state_file_yields_entries_without_issue() {
        let dir = scratch("valid");
        let info = read_state(
            &dir,
            "# .state\n\n## locked\n\nalice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
        );
        assert_eq!(info.entries.len(), 1);
        assert_eq!(info.entries[0].nick, "alice");
        assert_eq!(info.entries[0].status, "locked");
        assert!(info.entries[0].valid(time::parse(NOW).unwrap()));
        assert!(info.issue.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn duplicate_status_sections_keep_entries_and_report_partial() {
        let dir = scratch("dup-sections");
        let info = read_state(
            &dir,
            "# .state\n\n## locked\n\nother 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n\n## locked\n\nfriend 2026-09-25T11:30:00Z..2026-09-25T13:30:00Z\n",
        );
        // Both sections' declarations participate (file order, duplicates
        // included): a duplicate-section file still exposes every
        // parseable lock.
        assert_eq!(info.entries.len(), 2);
        assert_eq!(info.entries[0].nick, "other");
        assert_eq!(info.entries[1].nick, "friend");
        match info.issue {
            Some(ScanIssue::Partial { id, line }) => {
                assert_eq!(id, "E-STATUS-DUP");
                assert_eq!(line, 7);
            }
            other => panic!("expected E-STATUS-DUP partial, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_declaration_reports_partial_not_clean_ok() {
        let dir = scratch("w-state-entry");
        let info = read_state(
            &dir,
            "# .state\n\n## locked\n\nalice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z, bob not-a-time\n",
        );
        // The parseable declaration still participates; the lost fragment
        // must downgrade the scan to partial instead of a clean ok.
        assert_eq!(info.entries.len(), 1);
        assert_eq!(info.entries[0].nick, "alice");
        match info.issue {
            Some(ScanIssue::Partial { id, line }) => {
                assert_eq!(id, "W-STATE-ENTRY");
                assert_eq!(line, 5);
            }
            other => panic!("expected W-STATE-ENTRY partial, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stray_preamble_is_partial_without_inventing_locks() {
        let dir = scratch("w-stray");
        let info = read_state(
            &dir,
            "# .state\n\n##locked\nalice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
        );
        // Malformed heading never becomes a section; omitted bytes are
        // partial, not a clean empty scan and not a guessed lock.
        assert!(info.entries.is_empty());
        match &info.issue {
            Some(ScanIssue::Partial { id, line }) => {
                assert_eq!(*id, "W-STRAY");
                assert_eq!(*line, 3);
            }
            other => panic!("expected W-STRAY partial, got {other:?}"),
        }
        let scope = assemble(&dir, vec![info], Vec::new());
        assert!(scope.issues.iter().any(|(_, issue)| matches!(
            issue,
            ScanIssue::Partial {
                id: "W-STRAY",
                line: 3
            }
        )));
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let verdict = functional_lock(&mut ctx, &dir.join("note.agent-coexist.md"), Some("bob"));
        assert!(verdict.holders.is_empty());
        assert!(verdict.scope.issues.iter().any(|(_, issue)| matches!(
            issue,
            ScanIssue::Partial {
                id: "W-STRAY",
                line: 3
            }
        )));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parseable_entries_survive_stray_preamble() {
        let dir = scratch("w-stray-kept");
        let info = read_state(
            &dir,
            "# .state\n\n##locked\nalice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n\n## locked\nbob 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
        );
        assert_eq!(info.entries.len(), 1);
        assert_eq!(info.entries[0].nick, "bob");
        assert_eq!(info.entries[0].status, "locked");
        match info.issue {
            Some(ScanIssue::Partial { id, line }) => {
                assert_eq!(id, "W-STRAY");
                assert_eq!(line, 3);
            }
            other => panic!("expected W-STRAY partial, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reader_comment_preamble_is_not_partial() {
        let dir = scratch("reader-comment");
        let info = read_state(
            &dir,
            "# .state\n\n> locked = please do not change this directory.\n\n## locked\nalice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
        );
        assert_eq!(info.entries.len(), 1);
        assert_eq!(info.entries[0].nick, "alice");
        assert!(info.issue.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_kind_reserved_state_file_is_skipped_as_etype() {
        let dir = scratch("wrong-kind");
        let info = read_state(
            &dir,
            "# state.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```\nhello\n```\n",
        );
        // A parseable non-state file at the reserved name is not "no state
        // file": its directory must stay visible in the scan.
        assert!(info.entries.is_empty());
        match info.issue {
            Some(ScanIssue::Blocking(id)) => assert_eq!(id, "E-TYPE"),
            other => panic!("expected E-TYPE blocking, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn undecidable_structure_still_skips_whole_file() {
        let dir = scratch("skip-class");
        let info = read_state(
            &dir,
            "# .state\n\n## locked\n\n```\nalice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
        );
        assert!(info.entries.is_empty());
        match info.issue {
            Some(ScanIssue::Blocking(id)) => assert_eq!(id, "E-FENCE"),
            other => panic!("expected E-FENCE blocking, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_state_keeps_history_without_reviving_superseded_locks() {
        let dir = scratch("effective-state");
        let info = read_state(
            &dir,
            concat!(
                "# .state\n\n## locked\n",
                "alice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                "\n## locked\n",
                "alice 2026-09-25T15:00:00Z..2026-09-25T14:00:00Z\n",
                "bob 2026-09-25T11:00:00Z..2026-09-25T16:00:00Z\n",
            ),
        );
        let now = time::parse(NOW).unwrap();
        assert_eq!(
            info.entries.len(),
            3,
            "raw notification history is retained"
        );
        let effective: Vec<_> = info
            .effective_entries()
            .map(|e| (e.nick.as_str(), e.valid(now)))
            .collect();
        assert_eq!(effective, vec![("alice", false), ("bob", true)]);
        let scope = assemble(&dir, vec![info], Vec::new());
        assert!(conflicts_with(&scope, now, "bob").is_empty());
        assert_eq!(
            valid_locked(&scope, now)
                .iter()
                .map(|e| e.nick)
                .collect::<Vec<_>>(),
            vec!["bob"]
        );
        let mut ctx = test_ctx(now);
        let verdict = functional_lock(&mut ctx, &dir.join("note.agent-coexist.md"), Some("bob"));
        assert!(!verdict.holders.iter().any(|holder| holder.nick == "alice"));
        assert!(verdict.scope.issues.iter().any(|(_, issue)| matches!(
            issue,
            ScanIssue::Partial {
                id: "E-STATUS-DUP",
                ..
            }
        )));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn descendant_filters_cannot_be_bypassed_by_an_unrelated_post() {
        let dir = scratch("filtered-descendant");
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();
        read_state(
            &sub,
            "# .state\n\n## locked\nbob 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
        );
        std::fs::write(sub.join("note.agent-coexist.md"), "# note.post\n").unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let excluded = fsx::walk::Options {
            exclude: vec!["state.agent-coexist.md".to_string()],
            ..Default::default()
        };
        assert!(
            collect(&mut ctx, &dir, &excluded)
                .expect("valid exclude walk")
                .descendants
                .is_empty()
        );
        let included = fsx::walk::Options {
            include: vec!["note.agent-coexist.md".to_string()],
            ..Default::default()
        };
        assert!(
            collect(&mut ctx, &dir, &included)
                .expect("valid include walk")
                .descendants
                .is_empty()
        );
        std::fs::write(dir.join(".ignore"), "state.agent-coexist.md\n").unwrap();
        assert!(
            collect(&mut ctx, &dir, &fsx::walk::Options::default())
                .expect("valid ignore walk")
                .descendants
                .is_empty()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn native_aliases_do_not_duplicate_colliding_source_issues() {
        let dir = scratch("native-source-aliases");
        let mut files = Vec::new();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        for name in ["a  b", "a b"] {
            let source = dir.join(name);
            let hop = source.join("hop");
            std::fs::create_dir_all(&hop).unwrap();
            files.push(read_state(&source, "# invalid\n"));
            // A second native spelling of the same on-disk state file.
            assert_ne!(
                source,
                hop.join(".."),
                "fixture must retain an alternate native spelling"
            );
            files.push(read_state_file(
                &mut ctx,
                &hop.join("..").join("state.agent-coexist.md"),
            ));
        }
        let scope = assemble(&dir, files, Vec::new());
        assert_eq!(scope.issues.len(), 2);
        assert!(
            scope
                .issues
                .iter()
                .all(|(_, issue)| matches!(issue, ScanIssue::Blocking("E-H1")))
        );
        assert_ne!(scope.issues[0].0, scope.issues[1].0);
        assert_eq!(
            context::object_of_dir(&scope.issues[0].0),
            context::object_of_dir(&scope.issues[1].0)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn dangling_state_is_unreadable_but_a_vanished_entry_is_absent() {
        let dir = scratch("dangling-state");
        let file = dir.join("state.agent-coexist.md");
        std::os::unix::fs::symlink("missing.agent-coexist.md", &file).unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let verdict = functional_lock(&mut ctx, &dir.join("note.agent-coexist.md"), Some("bob"));
        assert!(verdict.scope.issues.iter().any(|(path, issue)| {
            path == &dir && matches!(issue, ScanIssue::Blocking("unreadable"))
        }));
        assert!(
            std::fs::symlink_metadata(&file)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        std::fs::remove_file(&file).unwrap();
        assert!(read_state_file(&mut ctx, &file).issue.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn busy_state_file_reports_busy() {
        let dir = scratch("busy");
        let file = dir.join("state.agent-coexist.md");
        std::fs::write(
            &file,
            "# .state\n\n## locked\n\nalice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
        )
        .unwrap();
        // A second handle's exclusive lock blocks the shared scan read.
        let holder = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file)
            .unwrap();
        holder.try_lock().unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let info = read_state_file(&mut ctx, &file);
        holder.unlock().unwrap();
        assert!(matches!(info.issue, Some(ScanIssue::Busy)));
        assert!(info.entries.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_ignore_is_a_typed_scan_fault_not_an_unreadable_root() {
        let dir = scratch("malformed-ignore");
        std::fs::write(dir.join(".ignore"), "{broken\n").unwrap();
        let child = dir.join("child");
        std::fs::create_dir(&child).unwrap();
        read_state(
            &child,
            "# .state\n\n## locked\nalice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
        );
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        match collect(&mut ctx, &dir, &fsx::walk::Options::default()) {
            Ok(scope) => panic!(
                "fatal ignore source must not succeed as an unreadable root: {:?}",
                scope.unreadable_dirs
            ),
            Err(Fault::Source {
                path,
                problem,
                cause,
            }) => {
                assert_eq!(path, dir.join(".ignore"));
                assert_eq!(
                    problem,
                    crate::error::SourceProblem::Content { line: Some(1) }
                );
                assert!(cause.starts_with("line 1: "), "{cause}");
            }
            Err(other) => panic!("expected source fault, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
