//! `read-notification [DIR] --as <nick> --since <time|duration> [WALK]`
//! (spec §7.2, §8.2, §10.6).
//!
//! One subtree walk reads every managed file once; post files yield
//! mention / reply / subscribed records, state files (plus the ancestor
//! state files, §10.6.2) yield `state` records from all parseable
//! declarations of others. Raw declarations also retain own expiry history;
//! current expiring rows use each file's selected view, and conflicts use
//! only reserved directory metadata. The tree is never walked twice.
//! Records sort per §10.6.5; `skipped` /
//! `partial` rows follow them; scope warnings precede `total`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::{
    INCLUDE_HINT_EMPTY_STATE, canonicalize_notes, dir_error, include_hints, sort_diagnostics,
};
use crate::Time;
use crate::consts;
use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::ops::ReadNotificationParams;
use crate::ops::context::{self, Ctx};
use crate::ops::shared::{self, Follow, ScanIssue, StateFileInfo};
use crate::report::Report;
use crate::time;

/// Diagnostics that keep a file out of the result entirely (§10.6.4).
const SKIP_CLASS: [&str; 4] = ["E-UTF8", "E-TOO-LARGE", "E-H1", "E-FENCE"];
/// Diagnostics whose parseable remainder still participates (§10.6.4),
/// mirroring `shared`'s classes so the walk and the ancestor pass (which
/// reads through `shared::read_state_file`) classify one file the same
/// way: `E-STATUS-DUP` sections still parse, and `W-STATE-ENTRY` marks
/// declaration text that could not be incorporated — losing it silently
/// would turn an unreadable lock into a clean `ok`. Reserved state metadata
/// is classified through `shared` during this same walk, independently of
/// real post events carried by a wrong-kind reserved file.
const PARTIAL_CLASS: [&str; 4] = ["E-HEAD", "E-SEQ-DUP", "E-STATUS-DUP", "W-STATE-ENTRY"];

/// One notification record before ordering.
struct Event {
    when: Time,
    /// Source path for the §10.6.5 ordering (post display path without
    /// suffix / state file's directory).
    path_key: String,
    kind_rank: u8,
    /// seq for comment records, file-order index otherwise (§10.6.5).
    tie: u64,
    key: &'static str,
    values: Vec<String>,
}

/// Scan incompleteness keeps its native source independently of its display
/// object: both different directories and different post files can display alike.
struct AuxRecord {
    source: PathBuf,
    path: String,
    key: &'static str,
    /// Cause columns: `busy` / `unreadable` / `<ID>`, or `<ID>` and `L<n>`.
    causes: Vec<String>,
    /// What the reader opens to see what was not read.
    follow: Follow,
}

impl AuxRecord {
    fn new(
        source: &Path,
        path: &str,
        key: &'static str,
        causes: Vec<String>,
        follow: Follow,
    ) -> Self {
        Self {
            source: source.to_path_buf(),
            path: path.to_string(),
            key,
            causes,
            follow,
        }
    }

    /// `skipped <dir> unreadable`: a directory that could not be read or entered.
    fn unreadable_dir(source: &Path, dir: &Path) -> Self {
        Self::new(
            source,
            &context::object_of_dir(dir),
            "skipped",
            vec!["unreadable".to_string()],
            Follow::Unreadable(dir.to_path_buf()),
        )
    }
}

pub fn read_notification(ctx: &mut Ctx, p: &ReadNotificationParams) -> Result<Report, Fault> {
    fsx::walk::validate(&p.walk)?;
    let object = context::object_of_dir(&p.dir);
    if let Some(rep) = dir_error(ctx, "read-notification", &p.dir)? {
        return Ok(rep);
    }
    let scanned = {
        let mut sink = ctx.walk_sink();
        fsx::walk::scan(&p.dir, &p.walk, &mut sink)?
    };

    let mut report = Report::new("read-notification", object.clone());
    report.kv("as", vec![p.nick.clone()]);
    report.kv("since", vec![time::format(p.since)]);

    let now = ctx.now;
    let since = p.since;
    let mut events: Vec<Event> = Vec::new();
    let mut aux: Vec<AuxRecord> = Vec::new();
    let mut tie: u64 = 0;
    let mut state_infos: Vec<StateFileInfo> = Vec::new();
    let mut seen_state: HashSet<PathBuf> = HashSet::new();
    let root = std::fs::canonicalize(&p.dir).unwrap_or_else(|_| p.dir.clone());

    for file in &scanned.files {
        ctx.file_read();
        let path_key = context::object_of_path(file);
        let parent = file.parent().unwrap_or_else(|| Path::new("."));
        let is_reserved = match shared::is_reserved_state_file(file) {
            Ok(reserved) => reserved,
            Err(_) => {
                let source = shared::directory_identity(parent);
                aux.push(AuxRecord::unreadable_dir(&source, parent));
                false
            }
        };
        let (issue_source, issue_path, follow) = if is_reserved {
            // Root, ancestor and descendant metadata share a native source
            // identity, even when H1 names another kind.
            let source = state_source_path(file);
            seen_state.insert(source.clone());
            (
                source,
                context::object_of_dir(parent),
                Follow::StateOf(parent.to_path_buf()),
            )
        } else {
            (file.clone(), path_key.clone(), Follow::File(file.clone()))
        };
        let skipped = |cause: &str| {
            AuxRecord::new(
                &issue_source,
                &issue_path,
                "skipped",
                vec![cause.to_string()],
                follow.clone(),
            )
        };
        match fsx::read(file, consts::SCAN_READ_WAIT_MS, None) {
            Err(Fault::Reason { .. }) => aux.push(skipped("busy")),
            Err(Fault::Diagnostics(diags)) => {
                aux.push(skipped(blocking_id(&diags).unwrap_or("E-TOO-LARGE")));
            }
            Err(_) => aux.push(skipped("unreadable")),
            Ok(bytes) => match doc::parse(bytes, file, now) {
                Err(Fault::Diagnostics(diags)) => {
                    aux.push(skipped(blocking_id(&diags).unwrap_or("E-UTF8")));
                }
                Err(f) => return Err(f),
                Ok(parsed) => {
                    if is_reserved || parsed.kind == Some(doc::Kind::State) {
                        let info = shared::state_file_info(file, &parsed);
                        append_state_issue(
                            &mut aux,
                            &issue_source,
                            &issue_path,
                            &follow,
                            &info.issue,
                        );
                        if parsed.kind == Some(doc::Kind::State) {
                            // Misnamed .state files still provide their real
                            // history and parse diagnostics, never directory locks.
                            state_events(&info, &p.nick, since, now, &mut events, &mut tie);
                        }
                        if is_reserved {
                            state_infos.push(info);
                        }
                    }
                    match parsed.kind {
                        None if !is_reserved => {
                            aux.push(skipped(blocking_id(&parsed.diagnostics).unwrap_or("E-H1")));
                        }
                        Some(doc::Kind::Post) => {
                            if let Some(d) = first_diag(&parsed.diagnostics, &SKIP_CLASS) {
                                aux.push(skipped(d.id));
                            } else {
                                if let Some(d) = first_diag(&parsed.diagnostics, &PARTIAL_CLASS) {
                                    let (key, causes) = shared::issue_row(&ScanIssue::Partial {
                                        id: d.id,
                                        line: d.line,
                                    });
                                    aux.push(AuxRecord::new(
                                        &issue_source,
                                        &issue_path,
                                        key,
                                        causes,
                                        follow.clone(),
                                    ));
                                }
                                post_events(&parsed, &p.nick, since, now, &path_key, &mut events);
                            }
                        }
                        _ => {}
                    }
                }
            },
        }
    }

    // Reserved metadata already attempted by the walk is authoritative,
    // including wrong-kind and unreadable outcomes. Do not reread it.
    for ancestor in fsx::walk::ancestors(&root) {
        let state_file = match fsx::walk::state_file(&ancestor) {
            Ok(Some(file)) => file,
            Ok(None) => continue,
            Err(Fault::Io { .. }) => {
                // §10.1.6: an unreadable ancestor is reported, not fatal.
                let source = shared::directory_identity(&ancestor);
                aux.push(AuxRecord::unreadable_dir(&source, &ancestor));
                continue;
            }
            Err(f) => return Err(f),
        };
        let canonical = state_source_path(&state_file);
        if seen_state.contains(&canonical) {
            continue;
        }
        let info = shared::read_state_file(ctx, &state_file);
        append_state_issue(
            &mut aux,
            &canonical,
            &info.display_dir,
            &Follow::StateOf(ancestor.clone()),
            &info.issue,
        );
        seen_state.insert(canonical);
        state_events(&info, &p.nick, since, now, &mut events, &mut tie);
        state_infos.push(info);
    }

    // §10.1.5: unreadable directories of the subtree walk.
    let unreadable = scanned.unreadable_dirs;
    for dir in &unreadable {
        let source = shared::directory_identity(dir);
        aux.push(AuxRecord::unreadable_dir(&source, dir));
    }

    // Conflicts and own-entry records on the assembled scope.
    let scope = shared::assemble(&root, state_infos, unreadable);
    for (rec, own_dir) in shared::conflicts_with(&scope, now, &p.nick) {
        events.push(Event {
            when: rec.start,
            path_key: rec.dir_display.to_string(),
            kind_rank: 4,
            tie: rec.source_order as u64,
            key: "conflict",
            values: vec![
                time::format(rec.start),
                rec.dir_display.to_string(),
                names::display_text(rec.status),
                rec.nick.to_string(),
                format!("overlaps your lock on {own_dir}"),
            ],
        });
    }

    // §10.6.5 ordering; auxiliary scope warnings follow the records.
    events.sort_by(|a, b| {
        a.when
            .cmp(&b.when)
            .then_with(|| a.path_key.cmp(&b.path_key))
            .then_with(|| a.kind_rank.cmp(&b.kind_rank))
            .then_with(|| a.tie.cmp(&b.tie))
    });
    for event in &events {
        report.record(event.key, event.values.clone(), 2, consts::KIND_WIDTH);
    }
    let record_count = events.len();

    dedup_aux(&mut aux);
    let aux_count = aux.len();
    let mut unread: Vec<Follow> = Vec::new();
    for row in aux {
        let mut values = Vec::with_capacity(1 + row.causes.len());
        values.push(row.path);
        values.extend(row.causes);
        report.record(row.key, values, 2, consts::KIND_WIDTH);
        if !matches!(row.follow, Follow::Unreadable(_)) {
            unread.push(row.follow);
        }
    }
    // Skipped or partial files may hold records this report does not show:
    // name each one to open (one in an unreadable directory says to make the
    // directory readable first). Unreadable directories with no known file
    // inside already say what blocks them.
    if !unread.is_empty() {
        report.kv(
            "fix",
            vec![shared::scan_fix(
                &unread,
                diag::FIX_NOTIFY_UNREAD,
                diag::FIX_NOTIFY_UNREAD_MANUAL,
            )],
        );
    }

    // §10.5.2: lexical identity fallback warns, but is metadata rather
    // than a notification or skipped/partial record (§8.1.10). The fix
    // names existence and ancestor search permission before retrying.
    let canonicalize_warn =
        canonicalize_notes(&mut report, &scope.canonicalize_failed, consts::KIND_WIDTH);
    let include_warn = include_hints(
        &mut report,
        &scanned.zero_includes,
        &p.walk,
        shared::ignore_in_play(&scanned.ignore_sources),
        INCLUDE_HINT_EMPTY_STATE,
    );

    report.total(record_count + aux_count);
    // §8.1.11: conflict rows (row-keyed), skipped / partial rows, scope
    // warnings, and the include hint all warn.
    if events.iter().any(|event| event.key == "conflict")
        || aux_count > 0
        || canonicalize_warn
        || include_warn
    {
        report.warn();
    }
    Ok(report)
}

/// mention / reply / subscribed records of one post (§10.6.3).
fn post_events(
    parsed: &doc::Document,
    nick: &str,
    since: Time,
    now: Time,
    path_key: &str,
    events: &mut Vec<Event>,
) {
    let post = &parsed.post;
    // First author per seq decides who `@#m` refers to.
    let mut author_of: HashMap<u64, &str> = HashMap::new();
    for comment in &post.comments {
        author_of
            .entry(comment.seq)
            .or_insert(comment.nick.as_str());
    }
    let subscribed = post.subscribers.iter().any(|s| s == nick);
    for comment in &post.comments {
        let when = comment.activity();
        if !(when >= since && when < now) || comment.nick == nick {
            continue;
        }
        let is_reply = comment
            .replies
            .iter()
            .any(|target| author_of.get(target).is_some_and(|author| *author == nick));
        let is_mention = comment.mentions.iter().any(|m| m == nick);
        let (kind_rank, key) = if is_reply {
            (1u8, "reply")
        } else if is_mention {
            (0u8, "mention")
        } else if subscribed {
            (2u8, "subscribed")
        } else {
            continue;
        };
        let mut values = vec![
            time::format(when),
            format!("{path_key} #{}", comment.seq),
            comment.nick.clone(),
        ];
        if let Some(body) = parsed.body(comment.section) {
            let excerpt = names::excerpt(body);
            if !excerpt.is_empty() {
                values.push(excerpt);
            }
        }
        events.push(Event {
            when,
            path_key: path_key.to_string(),
            kind_rank,
            tie: comment.seq,
            key,
            values,
        });
    }
}

/// Historical state/expired rows use every parseable declaration. Only
/// current expiring rows use the effective (latest-until) view (§10.6.3).
fn state_events(
    info: &StateFileInfo,
    nick: &str,
    since: Time,
    now: Time,
    events: &mut Vec<Event>,
    tie: &mut u64,
) {
    for entry in &info.entries {
        if entry.nick == nick {
            if entry.until >= since && entry.until < now {
                // The end time is the first column; per §10.6.3 the
                // qualifier column stays additive (like `until <t>` /
                // `ends in <m>`), so an expired row simply has none.
                events.push(Event {
                    when: entry.until,
                    path_key: info.display_dir.clone(),
                    kind_rank: 6,
                    tie: *tie,
                    key: "expired",
                    values: vec![
                        time::format(entry.until),
                        info.display_dir.clone(),
                        names::display_text(&entry.status),
                        entry.nick.clone(),
                    ],
                });
                *tie += 1;
            }
            continue;
        }
        if !(entry.start >= since && entry.start < now) {
            continue;
        }
        let mut values = vec![
            time::format(entry.start),
            info.display_dir.clone(),
            names::display_text(&entry.status),
            entry.nick.clone(),
            format!("until {}", time::format(entry.until)),
        ];
        if now >= entry.until {
            values.push("expired".to_string());
        }
        events.push(Event {
            when: entry.start,
            path_key: info.display_dir.clone(),
            kind_rank: 3,
            tie: *tie,
            key: "state",
            values,
        });
        *tie += 1;
    }
    for entry in info.effective_entries() {
        if entry.nick == nick
            && entry.valid(now)
            && (entry.until - now).num_seconds() <= consts::EXPIRING_WINDOW
        {
            let minutes = (entry.until - now).num_seconds().max(0) / 60;
            events.push(Event {
                when: entry.until,
                path_key: info.display_dir.clone(),
                kind_rank: 5,
                tie: *tie,
                key: "expiring",
                values: vec![
                    time::format(entry.until),
                    info.display_dir.clone(),
                    names::display_text(&entry.status),
                    entry.nick.clone(),
                    format!("ends in {minutes}m"),
                ],
            });
            *tie += 1;
        }
    }
}

fn append_state_issue(
    aux: &mut Vec<AuxRecord>,
    source: &Path,
    path: &str,
    follow: &Follow,
    issue: &Option<ScanIssue>,
) {
    if let Some(issue) = issue {
        let (key, causes) = shared::issue_row(issue);
        aux.push(AuxRecord::new(source, path, key, causes, follow.clone()));
    }
}

fn state_source_path(file: &Path) -> PathBuf {
    // A symlink's directory owns its state declaration, not its referent's
    // directory. Canonicalize the parent only when deduplicating reads.
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    shared::directory_identity(parent).join(file.file_name().unwrap_or_default())
}

fn first_diag(diags: &[Diagnostic], class: &[&str]) -> Option<Diagnostic> {
    sort_diagnostics(diags.to_vec())
        .into_iter()
        .find(|d| class.contains(&d.id))
}

fn blocking_id(diags: &[Diagnostic]) -> Option<&'static str> {
    sort_diagnostics(diags.to_vec())
        .into_iter()
        .find(|d| SKIP_CLASS.contains(&d.id))
        .map(|d| d.id)
}

/// Deduplicate only repeat encounters of the same native source and cause,
/// then order the displayed rows (§10.6.5). Distinct sources remain distinct
/// records even when their display paths, row kinds and causes all match.
///
/// One merge, matching `shared::scan_rows`: a state file that could not be
/// opened (`skipped <dir> unreadable`) in a directory the walk could not read
/// either displays exactly like that directory's row. It is one row whose
/// next step names the state file and says the directory must be made
/// readable first.
fn dedup_aux(aux: &mut Vec<AuxRecord>) {
    aux.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then_with(|| a.key.cmp(b.key))
            .then_with(|| a.causes.cmp(&b.causes))
    });
    aux.dedup_by(|a, b| a.source == b.source && a.key == b.key && a.causes == b.causes);
    let unreadable_state = |row: &AuxRecord| {
        matches!(row.follow, Follow::StateOf(_))
            && row.key == "skipped"
            && row.causes.len() == 1
            && row.causes[0] == "unreadable"
    };
    let merged: HashMap<PathBuf, PathBuf> = aux
        .iter()
        .filter(|row| unreadable_state(row))
        .filter_map(|row| match &row.follow {
            Follow::StateOf(dir) => Some((row.source.parent()?.to_path_buf(), dir.clone())),
            _ => None,
        })
        .filter(|(identity, _)| {
            aux.iter()
                .any(|row| matches!(row.follow, Follow::Unreadable(_)) && row.source == *identity)
        })
        .collect();
    if !merged.is_empty() {
        aux.retain(|row| {
            !(unreadable_state(row)
                && row
                    .source
                    .parent()
                    .is_some_and(|dir| merged.contains_key(dir)))
        });
        for row in aux.iter_mut() {
            if matches!(row.follow, Follow::Unreadable(_))
                && let Some(dir) = merged.get(&row.source)
            {
                row.follow = Follow::StateInUnreadable(dir.clone());
            }
        }
    }
    aux.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.key.cmp(b.key))
            .then_with(|| a.causes.cmp(&b.causes))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::ReadNotificationParams;
    use crate::ops::context::{Ctx, test_ctx};
    use crate::report::{Row, Status};

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-read-notification-tests")
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
        // Scope identities are canonicalized (§10.5.2) while temp_dir may be
        // a lexical alias of the native directory; return the created
        // directory's canonical disk spelling so fixtures compare like with
        // like. lexical_dir keeps that spelling but drops the Windows
        // verbatim prefix, matching the ordinary native representation the
        // compared display paths use.
        shared::lexical_dir(&std::fs::canonicalize(&dir).unwrap())
    }

    fn params(dir: &Path, ctx: &Ctx) -> ReadNotificationParams {
        ReadNotificationParams {
            dir: dir.to_path_buf(),
            nick: "alice".to_string(),
            since: ctx.now - chrono::Duration::hours(1),
            walk: fsx::walk::Options::default(),
        }
    }

    fn value_rows(report: &Report, key: &str) -> Vec<Vec<String>> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key: k, values, .. } if k == key => Some(values.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn root_state_file_is_reported_skipped_once() {
        // A root state file with an undecidable H1 is skipped by the
        // subtree walk; the ancestor pass (which starts at DIR itself)
        // used to re-read it and print a second skipped row under the
        // directory's display name.
        let dir = scratch("root-state-skip-once");
        std::fs::write(dir.join("state.agent-besties.md"), b"# not a managed H1\n").unwrap();
        let mut ctx = Ctx::new(Some("agent-coexist read-notification".to_string()), None).unwrap();
        let p = params(&dir, &ctx);
        let report = read_notification(&mut ctx, &p).unwrap();
        let skipped = value_rows(&report, "skipped");
        assert_eq!(skipped.len(), 1, "rows: {skipped:?}");
        assert_eq!(skipped[0][1], "E-H1");
        assert_eq!(value_rows(&report, "total")[0][0], "1");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn root_state_file_entries_are_counted_once() {
        // The suppression must not over-reach: a readable root state
        // file still contributes its declarations exactly once.
        let dir = scratch("root-state-once");
        let now = crate::time::now().unwrap();
        let start = crate::time::format(now - chrono::Duration::minutes(30));
        let until = crate::time::format(now + chrono::Duration::hours(2));
        std::fs::write(
            dir.join("state.agent-besties.md"),
            format!("# .state\n\n## locked\nbob {start}..{until}\n").as_bytes(),
        )
        .unwrap();
        let mut ctx = Ctx::new(Some("agent-coexist read-notification".to_string()), None).unwrap();
        let p = params(&dir, &ctx);
        let report = read_notification(&mut ctx, &p).unwrap();
        let state_rows = value_rows(&report, "state");
        assert_eq!(state_rows.len(), 1, "rows: {state_rows:?}");
        assert_eq!(state_rows[0][3], "bob");
        assert!(value_rows(&report, "skipped").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reserved_name_post_is_diagnosed_from_subtree_root_and_ancestor() {
        let dir = scratch("reserved-name-post-etype");
        let sub = dir.join("sub");
        let child = sub.join("child");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(
            sub.join("state.agent-besties.md"),
            concat!(
                "# state.post\n\n## #1 bob (2026-09-25T11:30:00Z)\n\n```md\n@alice hello\n```\n",
                "\n## #malformed\n\n```md\nunparsed comment\n```\n",
            ),
        )
        .unwrap();
        let now = time::parse("2026-09-25T12:00:00Z").unwrap();
        for (root, post_visible) in [(&dir, true), (&sub, true), (&child, false)] {
            let mut ctx = test_ctx(now);
            let p = params(root, &ctx);
            let report = read_notification(&mut ctx, &p).unwrap();
            let skipped = value_rows(&report, "skipped");
            assert_eq!(skipped.len(), 1, "skipped: {skipped:?}");
            assert_eq!(
                skipped[0],
                vec![context::object_of_dir(&sub), "E-TYPE".to_string()]
            );
            let mentions = value_rows(&report, "mention");
            let partial = value_rows(&report, "partial");
            if post_visible {
                assert_eq!(mentions.len(), 1);
                assert_eq!(mentions[0][2], "bob");
                // The post's malformed remainder is a different cause, not
                // a duplicate of the reserved-name metadata type failure.
                assert_eq!(partial.len(), 1);
                assert_eq!(partial[0][1], "E-HEAD");
                assert!(partial[0][2].starts_with('L'));
            } else {
                assert!(mentions.is_empty());
                assert!(partial.is_empty());
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn latest_state_does_not_erase_history_or_revive_superseded_locks() {
        let dir = scratch("effective-history");
        std::fs::write(
            dir.join("state.agent-besties.md"),
            concat!(
                "# .state\n\n## locked\n",
                "alice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                "alice 2026-09-25T15:00:00Z..2026-09-25T14:00:00Z\n",
                "bob 2026-09-25T10:00:00Z..2026-09-25T11:30:00Z\n",
                "bob 2026-09-25T11:00:00Z..2026-09-25T12:10:00Z\n",
                "bob 2026-09-25T11:00:00Z..2026-09-25T16:00:00Z\n",
            ),
        )
        .unwrap();
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut p = params(&dir, &ctx);
        p.nick = "bob".to_string();
        let report = read_notification(&mut ctx, &p).unwrap();
        let history = value_rows(&report, "state");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0][3], "alice");
        let expired = value_rows(&report, "expired");
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0][0], "2026-09-25T11:30:00Z");
        assert!(value_rows(&report, "conflict").is_empty());
        assert!(value_rows(&report, "expiring").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn misnamed_history_cannot_shadow_metadata_or_reorder_conflicts() {
        let dir = scratch("misnamed-state-history");
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(
            sub.join("a.agent-besties.md"),
            concat!(
                "# .state\n\n## observing\n",
                "alice 2026-09-25T10:00:00Z..2026-09-25T11:30:00Z\n",
                "bob 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                "unparseable declaration\n",
            ),
        )
        .unwrap();
        std::fs::write(
            sub.join("state.agent-besties.md"),
            concat!(
                "# .state\n\n## locked\n",
                "alice 2026-09-25T10:00:00Z..2026-09-25T16:00:00Z\n",
                "bob 2026-09-25T11:00:00Z..2026-09-25T15:00:00Z\n",
                "carol 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
            ),
        )
        .unwrap();
        for root in [&dir, &sub] {
            let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
            let p = params(root, &ctx);
            let report = read_notification(&mut ctx, &p).unwrap();
            let conflicts = value_rows(&report, "conflict");
            assert_eq!(
                conflicts
                    .iter()
                    .map(|row| row[3].as_str())
                    .collect::<Vec<_>>(),
                vec!["bob", "carol"]
            );
            let history = value_rows(&report, "state");
            assert!(
                history
                    .iter()
                    .any(|row| row[2] == "observing" && row[3] == "bob")
            );
            let expired = value_rows(&report, "expired");
            assert_eq!(expired.len(), 1);
            assert_eq!(expired[0][2], "observing");
            let partial = value_rows(&report, "partial");
            assert_eq!(partial.len(), 1);
            assert_eq!(
                partial[0][0],
                context::object_of_path(&sub.join("a.agent-besties.md"))
            );
            assert_eq!(partial[0][1], "W-STATE-ENTRY");
            assert!(partial[0][2].starts_with('L'));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn conflicts_choose_deepest_last_own_lock_without_changing_report_order() {
        let dir = scratch("deepest-own-lock");
        let left = dir.join("left");
        let left_own = left.join("deep");
        let left_child = left_own.join("child");
        let lonely = dir.join("lonely");
        let right_own = dir.join("right/deep");
        for (path, declarations) in [
            (
                &dir,
                concat!(
                    "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
                    "bob 2026-09-25T11:00:00Z..2026-09-25T15:00:00Z\n",
                    "carol 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                ),
            ),
            (&left, "dave 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"),
            (
                &left_own,
                "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
            (
                &left_child,
                "erin 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
            (
                &lonely,
                "grace 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
            (
                &right_own,
                concat!(
                    "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
                    "frank 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
                ),
            ),
        ] {
            std::fs::create_dir_all(path).unwrap();
            std::fs::write(
                path.join("state.agent-besties.md"),
                format!("# .state\n\n## locked\n{declarations}"),
            )
            .unwrap();
        }
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut p = params(&dir, &ctx);
        p.walk.no_ignore = true;
        let report = read_notification(&mut ctx, &p).unwrap();
        let expected: Vec<_> = [
            (&dir, "bob", &right_own),
            (&dir, "carol", &right_own),
            (&left, "dave", &left_own),
            (&left_child, "erin", &left_own),
            (&lonely, "grace", &dir),
            (&right_own, "frank", &right_own),
        ]
        .into_iter()
        .map(|(path, nick, own)| {
            vec![
                "2026-09-25T11:00:00Z".to_string(),
                context::object_of_dir(path),
                "locked".to_string(),
                nick.to_string(),
                format!("overlaps your lock on {}", context::object_of_dir(own)),
            ]
        })
        .collect();
        assert_eq!(value_rows(&report, "conflict"), expected);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn folded_sibling_paths_keep_the_actual_own_lock_location() {
        let dir = scratch("folded-own-lock-path");
        let unrelated = dir.join("a  b");
        let conflicting = dir.join("a b");
        let own = conflicting.join("deep");
        for (path, nick) in [
            (&unrelated, "charlie"),
            (&conflicting, "bob"),
            (&own, "alice"),
        ] {
            std::fs::create_dir_all(path).unwrap();
            std::fs::write(
                path.join("state.agent-besties.md"),
                format!(
                    "# .state\n\n## locked\n{nick} 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"
                ),
            )
            .unwrap();
        }
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut p = params(&dir, &ctx);
        p.walk.no_ignore = true;
        let report = read_notification(&mut ctx, &p).unwrap();
        let conflicts = value_rows(&report, "conflict");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0][1], context::object_of_dir(&conflicting));
        assert_eq!(conflicts[0][3], "bob");
        assert_eq!(
            conflicts[0][4],
            format!("overlaps your lock on {}", context::object_of_dir(&own))
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn colliding_canonicalize_warnings_preserve_history_without_inflating_total() {
        let dir = scratch("canonicalize-after-read");
        let a = dir.join("a  b");
        let b = dir.join("a b");
        let last = dir.join("z");
        for source in [&a, &b] {
            std::fs::create_dir(source).unwrap();
            std::fs::write(
                source.join("state.agent-besties.md"),
                "# .state\n\n## observing\nbob 2026-09-25T11:30:00Z..2026-09-25T14:00:00Z\n",
            )
            .unwrap();
        }
        std::fs::create_dir(&last).unwrap();
        std::fs::write(last.join("notes.agent-besties.md"), "# notes.post\n").unwrap();
        let removed = [a.clone(), b.clone()];
        let mut ctx = Ctx::new(
            Some(String::new()),
            Some(Box::new(move |_, files| {
                // Both native sources have already contributed history;
                // invalidate only their later directory-identity lookup.
                if files == 3 {
                    for source in &removed {
                        std::fs::remove_dir_all(source).unwrap();
                    }
                }
            })),
        )
        .unwrap();
        ctx.now = time::parse("2026-09-25T12:00:00Z").unwrap();
        let mut p = params(&dir, &ctx);
        p.walk.no_ignore = true;
        let report = read_notification(&mut ctx, &p).unwrap();
        assert!(!a.exists() && !b.exists());
        assert_eq!(report.status, Status::Warn);
        let warnings = value_rows(&report, "canonicalize");
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0], warnings[1]);
        let history = value_rows(&report, "state");
        assert_eq!(history.len(), 2);
        assert!(history.iter().all(|row| row[3] == "bob"));
        assert_eq!(value_rows(&report, "total")[0][0], "2");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_history_and_expiry_titles_are_sanitized_only_for_output() {
        let dir = scratch("status-title-controls");
        let file = dir.join("state.agent-besties.md");
        let source = concat!(
            "# .state\n\n## custom\u{1b}[2J\n",
            "bob 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            "alice 2026-09-25T10:00:00Z..2026-09-25T11:30:00Z\n",
            "alice 2026-09-25T11:00:00Z..2026-09-25T12:10:00Z\n",
        );
        std::fs::write(&file, source).unwrap();
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut p = params(&dir, &ctx);
        p.walk.no_ignore = true;
        let report = read_notification(&mut ctx, &p).unwrap();
        for kind in ["state", "expired", "expiring"] {
            let rows = value_rows(&report, kind);
            assert_eq!(rows.len(), 1, "{kind}");
            assert_eq!(rows[0][2], "custom\u{FFFD}[2J", "{kind}");
        }
        assert_eq!(std::fs::read(&file).unwrap(), source.as_bytes());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn colliding_post_and_state_failures_remain_separate_records() {
        let dir = scratch("colliding-source-failures");
        for name in ["a  b", "a b"] {
            let child = dir.join(name);
            std::fs::create_dir(&child).unwrap();
            std::fs::write(child.join("state.agent-besties.md"), b"# invalid\n").unwrap();
            std::fs::write(dir.join(format!("{name}.agent-besties.md")), b"# invalid\n").unwrap();
        }
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut p = params(&dir, &ctx);
        p.walk.no_ignore = true;
        let report = read_notification(&mut ctx, &p).unwrap();
        assert_eq!(report.status, Status::Warn);
        let skipped = value_rows(&report, "skipped");
        assert_eq!(skipped.len(), 4);
        assert!(skipped.iter().all(|row| row == &skipped[0]));
        assert_eq!(skipped[0][1], "E-H1");
        assert_eq!(value_rows(&report, "total")[0][0], "4");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
