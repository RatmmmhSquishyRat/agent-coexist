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
use std::fs;
use std::path::{Path, PathBuf};

use crate::consts;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::Time;

use super::context::Ctx;

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
#[derive(Debug, Clone)]
pub enum ScanIssue {
    /// Lock busy within `SCAN_READ_WAIT_MS` → `skipped <dir> busy`.
    Busy,
    /// A blocking diagnostic made the file unparseable → `skipped <dir>
    /// <E-ID>`; entries are empty. Also used with the non-diagnostic word
    /// `"unreadable"` for hard read failures (spec §1, §10.1.1), which
    /// renders as `skipped <dir> unreadable`.
    Blocking(&'static str),
    /// Parseable entries kept, a section-level error skipped → `partial
    /// <dir> <E-ID> L<n>`.
    Partial { id: &'static str, line: usize },
}

/// Diagnostics that keep a state file out of a scan entirely (§10.6.4):
/// the structure is undecidable, so no declaration can be trusted.
const SKIP_CLASS: [&str; 4] = ["E-UTF8", "E-TOO-LARGE", "E-H1", "E-FENCE"];
/// Diagnostics whose parseable remainder still participates (§10.6.4):
/// entries are kept and the file is reported `partial`. `E-STATUS-DUP`
/// remains write-blocking (`diag::is_blocking` gates every write), but a
/// duplicate-section state file's declarations are exactly what a lock
/// scan must not lose; `W-STATE-ENTRY` marks declaration text that could
/// not be incorporated at all — losing it silently would turn an
/// unreadable lock into a clean `ok`. Other formatting warnings carry no
/// declaration loss and stay non-partial.
const PARTIAL_CLASS: [&str; 4] = ["E-HEAD", "E-SEQ-DUP", "E-STATUS-DUP", "W-STATE-ENTRY"];

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
    /// Why the file could not be fully incorporated, if applicable.
    pub issue: Option<ScanIssue>,
}

/// The §10.5.1 scope of a directory: itself, its ancestors and its subtree.
#[derive(Debug, Clone)]
pub struct StateScope {
    /// Canonicalized target directory (lexical fallback).
    pub root: PathBuf,
    /// Display dirs whose canonicalization failed (§10.5.2 warn rows).
    pub canonicalize_failed: Vec<String>,
    /// The target directory's own state file, when present and readable.
    pub self_file: Option<StateFileInfo>,
    /// Ancestor state files, near → far.
    pub ancestors: Vec<StateFileInfo>,
    /// Subtree state files, canonical directory ascending.
    pub descendants: Vec<StateFileInfo>,
    /// Display dir → issue, display-path ascending (§10.5.4 tail rows).
    pub issues: Vec<(String, ScanIssue)>,
    /// Display dirs that could not be entered/read.
    pub unreadable_dirs: Vec<String>,
}

/// One valid `locked` entry as reported by a conflict scan.
#[derive(Debug, Clone)]
pub struct ConflictRec {
    pub status: String,
    pub nick: String,
    pub start: Time,
    pub until: Time,
    /// Display of the state file's directory (the other party's dir).
    pub dir_display: String,
}

/// Functional-lock verdict (§10.8): holders plus the chain scope it used.
#[derive(Debug, Clone)]
pub struct FlockOutcome {
    /// Valid `locked` holders with `nick != as_nick` (all holders when
    /// `as_nick` is `None`), sorted `until` descending then nick ascending;
    /// the first row is the `E-FLOCKED` `{holder}`.
    pub holders: Vec<FlockHolder>,
    /// The chain scope (target dir + ancestors only; no descendants).
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
    let dir = parent_dir(file);
    let mut info = StateFileInfo {
        file: file.to_path_buf(),
        display_dir: display_of(&dir),
        dir,
        entries: Vec::new(),
        issue: None,
    };
    let bytes = match fsx::read(file, consts::SCAN_READ_WAIT_MS, None) {
        Ok(bytes) => bytes,
        Err(fault) => {
            ctx.file_read();
            match fault {
                Fault::Reason { id, .. } if id == "E-LOCKED" || id == "E-INUSE" => {
                    info.issue = Some(ScanIssue::Busy);
                }
                Fault::Diagnostics(ds) => {
                    info.issue = classify_diag_issue(&ds);
                }
                Fault::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound => {
                    // Vanished between listing and reading: no file, no entries.
                }
                Fault::Io { .. } => {
                    // Genuinely unreadable (permissions, ELOOP, …): the file
                    // is not incorporable; report it via the non-diagnostic
                    // "unreadable" reason (§1, §10.1.1).
                    info.issue = Some(ScanIssue::Blocking("unreadable"));
                }
                _ => {}
            }
            return info;
        }
    };
    ctx.file_read();
    match doc::parse(bytes, file, ctx.now) {
        Ok(d) => {
            if let Some(first) = d.diagnostics.iter().find(|x| SKIP_CLASS.contains(&x.id)) {
                // Undecidable structure: no declaration can be trusted.
                info.issue = Some(ScanIssue::Blocking(first.id));
            } else if d.kind != Some(doc::Kind::State) {
                // Reserved name, parseable, but not a state file: report
                // the wrong kind instead of pretending the directory has
                // no state file.
                info.issue = Some(ScanIssue::Blocking("E-TYPE"));
            } else {
                for sec in &d.state {
                    for e in &sec.entries {
                        info.entries.push(StateEntryRec {
                            status: sec.status.clone(),
                            nick: e.nick.clone(),
                            start: e.start,
                            until: e.until,
                        });
                    }
                }
                // Partial-class diagnostics keep the entries (§10.5.6,
                // §10.6.4); other warnings carry no declaration loss.
                if let Some(first) = d.diagnostics.iter().find(|x| PARTIAL_CLASS.contains(&x.id)) {
                    info.issue = Some(ScanIssue::Partial {
                        id: first.id,
                        line: first.line,
                    });
                }
            }
        }
        Err(Fault::Diagnostics(ds)) => {
            info.issue = classify_diag_issue(&ds);
        }
        Err(_) => {}
    }
    info
}

/// Collect the full §10.5.1 scope of `dir`: the dir itself, all its
/// ancestors (canonicalized basis, §10.5.2) and its subtree via
/// [`fsx::walk::scan`] with the caller's options.
///
/// Unreadable directories and scan failures surface in
/// [`StateScope::unreadable_dirs`] — never as an empty-looking success.
pub fn collect(ctx: &mut Ctx, dir: &Path, walk: &fsx::walk::Options) -> StateScope {
    let mut files: Vec<StateFileInfo> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();

    // Self + ancestors on the normalized path.
    let (root_norm, _root_ok, _failed) = normalize_dir(dir);
    for d in chain_dirs(&root_norm) {
        match fsx::walk::state_file(&d) {
            Ok(Some(f)) => files.push(read_state_file(ctx, &f)),
            Ok(None) => {}
            Err(_) => push_uniq(&mut unreadable, display_of(&d)),
        }
    }

    // Subtree walk. Files are lexically sorted, so same-directory entries
    // are adjacent; the scope root itself is handled by the chain above.
    let mut sink = ctx.walk_sink();
    let scanned = fsx::walk::scan(dir, walk, &mut sink);
    drop(sink);
    let walked = match scanned {
        Ok(r) => r,
        Err(_) => {
            push_uniq(&mut unreadable, display_of(dir));
            fsx::walk::WalkResult {
                files: Vec::new(),
                unreadable_dirs: Vec::new(),
                ignore_sources: Vec::new(),
                zero_includes: Vec::new(),
                dirs: 0,
            }
        }
    };
    let mut last_parent: Option<PathBuf> = None;
    for f in &walked.files {
        let parent = parent_dir(f);
        if last_parent.as_deref() == Some(parent.as_path()) {
            continue;
        }
        last_parent = Some(parent.clone());
        if parent.as_path() == dir {
            continue;
        }
        match fsx::walk::state_file(&parent) {
            Ok(Some(sf)) => files.push(read_state_file(ctx, &sf)),
            Ok(None) => {}
            Err(_) => push_uniq(&mut unreadable, display_of(&parent)),
        }
    }
    for ud in &walked.unreadable_dirs {
        push_uniq(&mut unreadable, display_of(ud));
    }

    assemble(dir, files, unreadable)
}

/// Assemble a scope from already-read state files (single pass): classify
/// each file against the root's canonicalized identity, order the groups
/// (§10.5.4) and collect the incompleteness markers.
pub fn assemble(dir: &Path, files: Vec<StateFileInfo>, unreadable_dirs: Vec<String>) -> StateScope {
    let (root, root_ok, root_failed) = normalize_dir(dir);
    let mut failed: Vec<String> = root_failed.into_iter().collect();

    let mut issues: Vec<(String, ScanIssue)> = files
        .iter()
        .filter_map(|f| f.issue.clone().map(|i| (f.display_dir.clone(), i)))
        .collect();
    issues.sort_by(|a, b| a.0.cmp(&b.0));

    let mut self_file: Option<StateFileInfo> = None;
    let mut ancestors: Vec<StateFileInfo> = Vec::new();
    let mut descendants: Vec<StateFileInfo> = Vec::new();

    for mut info in files {
        let lex = info.dir.clone();
        let canon = fs::canonicalize(&lex).ok();
        if canon.is_none() {
            push_uniq(&mut failed, info.display_dir.clone());
        }
        let mut rel = Rel::Other;
        if root_ok {
            if let Some(c) = canon.as_ref() {
                rel = rel_between(c, &root);
            }
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
    ancestors.sort_by(|a, b| b.dir.components().count().cmp(&a.dir.components().count()));
    ancestors.dedup_by(|a, b| a.dir == b.dir);
    descendants.sort_by(|a, b| a.dir.cmp(&b.dir));
    descendants.dedup_by(|a, b| a.dir == b.dir);

    let mut unreadable = unreadable_dirs;
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
    }
}

/// Functional-lock verdict (§10.8): the chain is the target file's directory
/// and its ancestors only — no subtree, no descendants (§10.8.4).
///
/// Runs while the caller holds the target's exclusive lock; only shared
/// reads happen here.
pub fn functional_lock(ctx: &mut Ctx, target_file: &Path, as_nick: Option<&str>) -> FlockOutcome {
    let dir = parent_dir(target_file);
    let (root_norm, _root_ok, _failed) = normalize_dir(&dir);
    let mut files: Vec<StateFileInfo> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    for d in chain_dirs(&root_norm) {
        match fsx::walk::state_file(&d) {
            Ok(Some(f)) => files.push(read_state_file(ctx, &f)),
            Ok(None) => {}
            Err(_) => push_uniq(&mut unreadable, display_of(&d)),
        }
    }
    let scope = assemble(&dir, files, unreadable);

    let mut holders: Vec<FlockHolder> = Vec::new();
    for info in scope.self_file.iter().chain(scope.ancestors.iter()) {
        for e in &info.entries {
            if e.status != "locked" || !e.valid(ctx.now) {
                continue;
            }
            if let Some(n) = as_nick {
                if e.nick == n {
                    continue;
                }
            }
            holders.push(FlockHolder {
                nick: e.nick.clone(),
                until: e.until,
                dir_display: info.display_dir.clone(),
            });
        }
    }
    holders.sort_by(|a, b| b.until.cmp(&a.until).then_with(|| a.nick.cmp(&b.nick)));
    FlockOutcome { holders, scope }
}

/// All valid `locked` entries in the scope, §10.5.4 order: self file →
/// ancestors near → far → descendants canonical-ascending; within one file
/// by `until` ascending, then nick.
pub fn valid_locked(scope: &StateScope, now: Time) -> Vec<ConflictRec> {
    locked_with_dirs(scope, now)
        .into_iter()
        .map(|(rec, _)| rec)
        .collect()
}

/// The other parties whose valid `locked` entries conflict with `nick`'s
/// valid `locked` entries in the scope (§10.5.3), §10.5.4 order. Empty when
/// `nick` holds no valid `locked` entry.
pub fn conflicts_with(scope: &StateScope, now: Time, nick: &str) -> Vec<ConflictRec> {
    let all = locked_with_dirs(scope, now);
    let mine: Vec<usize> = all
        .iter()
        .enumerate()
        .filter(|(_, (rec, _))| rec.nick == nick)
        .map(|(i, _)| i)
        .collect();
    let mut out = Vec::new();
    for (rec, dir) in all.iter() {
        if rec.nick == nick {
            continue;
        }
        if mine.iter().any(|&m| dirs_overlap(dir, all[m].1)) {
            out.push(rec.clone());
        }
    }
    out
}

/// Both sides of every conflicting pair in the scope (§10.5.3), deduped to
/// one row per entry, §10.5.4 order.
pub fn conflicts_all(scope: &StateScope, now: Time) -> Vec<ConflictRec> {
    let all = locked_with_dirs(scope, now);
    let n = all.len();
    let mut keep = vec![false; n];
    for i in 0..n {
        for j in (i + 1)..n {
            if all[i].0.nick != all[j].0.nick && dirs_overlap(all[i].1, all[j].1) {
                keep[i] = true;
                keep[j] = true;
            }
        }
    }
    all.into_iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|((rec, _), _)| rec)
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

/// All valid locked entries with their directory identities, §10.5.4 order.
fn locked_with_dirs<'a>(scope: &'a StateScope, now: Time) -> Vec<(ConflictRec, &'a Path)> {
    let mut out: Vec<(ConflictRec, &'a Path)> = Vec::new();
    for info in scope
        .self_file
        .iter()
        .chain(scope.ancestors.iter())
        .chain(scope.descendants.iter())
    {
        let mut recs: Vec<ConflictRec> = info
            .entries
            .iter()
            .filter(|e| e.status == "locked" && e.valid(now))
            .map(|e| ConflictRec {
                status: e.status.clone(),
                nick: e.nick.clone(),
                start: e.start,
                until: e.until,
                dir_display: info.display_dir.clone(),
            })
            .collect();
        recs.sort_by(|a, b| a.until.cmp(&b.until).then_with(|| a.nick.cmp(&b.nick)));
        let dir: &'a Path = info.dir.as_path();
        out.extend(recs.into_iter().map(|rec| (rec, dir)));
    }
    out
}

/// §10.5.3 directory relation: equal or ancestor/descendant
/// (`Path::starts_with` compares components).
fn dirs_overlap(a: &Path, b: &Path) -> bool {
    a == b || a.starts_with(b) || b.starts_with(a)
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

/// Canonicalized identity of `dir` (§10.5.2); on failure the lexical path
/// plus the display dir for the warn row.
fn normalize_dir(dir: &Path) -> (PathBuf, bool, Option<String>) {
    match fs::canonicalize(dir) {
        Ok(p) => (p, true, None),
        Err(_) => {
            let disp = display_of(dir);
            (dir.to_path_buf(), false, Some(disp))
        }
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
    names::fold(&fsx::walk::display_path(&lexical_display_path(p)))
}

/// Strip the Windows verbatim prefix `fs::canonicalize` adds (`\\?\C:\…` →
/// `C:\…`, `\\?\UNC\server\share` → `\\server\share`) so `display_path`
/// can relativize against the cwd. Only display is affected (§10.5.5);
/// identity comparisons keep the canonical form.
fn lexical_display_path(p: &Path) -> PathBuf {
    let s = p.as_os_str().to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        p.to_path_buf()
    }
}

fn push_uniq(v: &mut Vec<String>, s: String) {
    if !v.contains(&s) {
        v.push(s);
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
        dir
    }

    fn read_state(dir: &Path, content: &str) -> StateFileInfo {
        let file = dir.join("state.agent-besties.md");
        std::fs::write(&file, content).unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        read_state_file(&mut ctx, &file)
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
    fn busy_state_file_reports_busy() {
        let dir = scratch("busy");
        let file = dir.join("state.agent-besties.md");
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
}
