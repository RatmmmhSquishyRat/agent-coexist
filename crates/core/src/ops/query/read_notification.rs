//! `read-notification [DIR] --as <nick> --since <time|duration> [WALK]`
//! (spec §7.2, §8.2, §10.6).
//!
//! One subtree walk reads every managed file once; post files yield
//! mention / reply / subscribed records, state files (plus the ancestor
//! state files, §10.6.2) yield `state` records from all parseable
//! declarations of others. Conflicts, `expiring` and `expired` are
//! computed on the scope assembled from those same state files — the
//! tree is never walked twice. Records sort per §10.6.5; `skipped` /
//! `partial` rows follow them and precede `total`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{dir_error, sort_diagnostics};
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::shared::{self, ScanIssue, StateEntryRec, StateFileInfo};
use crate::ops::ReadNotificationParams;
use crate::report::Report;
use crate::time;
use crate::Time;

/// Diagnostics that keep a file out of the result entirely (§10.6.4).
const SKIP_CLASS: [&str; 4] = ["E-UTF8", "E-TOO-LARGE", "E-H1", "E-FENCE"];
/// Diagnostics whose parseable remainder still participates (§10.6.4).
const PARTIAL_CLASS: [&str; 3] = ["E-HEAD", "E-SEQ-DUP", "E-STATUS-DUP"];

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
    // §8.4.1: count files read, not files found.
    ctx.progress(scanned.dirs, 0);

    let mut report = Report::new("read-notification", object.clone());
    report.kv("as", vec![p.nick.clone()]);
    report.kv("since", vec![time::format(p.since)]);

    let now = ctx.now;
    let since = p.since;
    let mut events: Vec<Event> = Vec::new();
    // (path key, row kind, cause columns) — sorted by path before output.
    let mut aux: Vec<(String, &'static str, Vec<String>)> = Vec::new();
    let mut tie: u64 = 0;
    let mut state_infos: Vec<StateFileInfo> = Vec::new();
    let mut seen_state: Vec<PathBuf> = Vec::new();

    for file in &scanned.files {
        ctx.file_read();
        let path_key = context::object_of_path(file);
        match fsx::read(file, consts::SCAN_READ_WAIT_MS, None) {
            Err(Fault::Reason { .. }) => {
                aux.push((path_key, "skipped", vec!["busy".to_string()]));
            }
            Err(Fault::Diagnostics(diags)) => {
                let id = blocking_id(&diags).unwrap_or("E-TOO-LARGE");
                aux.push((path_key, "skipped", vec![id.to_string()]));
            }
            Err(_) => {
                aux.push((path_key, "skipped", vec!["unreadable".to_string()]));
            }
            Ok(bytes) => match doc::parse(bytes, file, now) {
                Err(Fault::Diagnostics(diags)) => {
                    let id = blocking_id(&diags).unwrap_or("E-UTF8");
                    aux.push((path_key, "skipped", vec![id.to_string()]));
                }
                Err(f) => return Err(f),
                Ok(parsed) => match parsed.kind {
                    None => {
                        let id = blocking_id(&parsed.diagnostics).unwrap_or("E-H1");
                        aux.push((path_key, "skipped", vec![id.to_string()]));
                    }
                    Some(doc::Kind::Post) => {
                        if let Some(d) = first_diag(&parsed.diagnostics, &SKIP_CLASS) {
                            aux.push((path_key, "skipped", vec![d.id.to_string()]));
                        } else {
                            if let Some(d) = first_diag(&parsed.diagnostics, &PARTIAL_CLASS) {
                                aux.push((
                                    path_key.clone(),
                                    "partial",
                                    vec![format!("{} L{}", d.id, d.line)],
                                ));
                            }
                            post_events(
                                &parsed,
                                &p.nick,
                                since,
                                now,
                                &path_key,
                                &mut events,
                            );
                        }
                    }
                    Some(doc::Kind::State) => {
                        if let Some(d) = first_diag(&parsed.diagnostics, &SKIP_CLASS) {
                            aux.push((path_key, "skipped", vec![d.id.to_string()]));
                        } else {
                            if let Some(d) = first_diag(&parsed.diagnostics, &PARTIAL_CLASS) {
                                aux.push((
                                    path_key.clone(),
                                    "partial",
                                    vec![format!("{} L{}", d.id, d.line)],
                                ));
                            }
                            let info = build_state_info(file, &parsed);
                            state_events(&info, &p.nick, since, now, &mut events, &mut tie);
                            seen_state.push(canonical_of(file));
                            state_infos.push(info);
                        }
                    }
                    // brief / agent-profile: not notification candidates.
                    Some(_) => {}
                },
            },
        }
    }

    // §10.6.2: ancestor state files. Ancestor enumeration runs on the
    // canonicalized path (§10.5.2; lexical fallback per §13 Q4).
    let root = std::fs::canonicalize(&p.dir).unwrap_or_else(|_| p.dir.clone());
    for ancestor in fsx::walk::ancestors(&root) {
        let state_file = match fsx::state_file(&ancestor) {
            Ok(Some(file)) => file,
            Ok(None) => continue,
            Err(Fault::Io { .. }) => {
                // §10.1.6: an unreadable ancestor is reported, not fatal.
                let dir_display = context::object_of_dir(&ancestor);
                aux.push((dir_display, "skipped", vec!["unreadable".to_string()]));
                continue;
            }
            Err(f) => return Err(f),
        };
        let canonical = canonical_of(&state_file);
        if seen_state.iter().any(|seen| *seen == canonical) {
            continue;
        }
        seen_state.push(canonical);
        ctx.file_read();
        let info = shared::read_state_file(ctx, &state_file);
        let dir_display = info.display_dir.clone();
        match &info.issue {
            Some(ScanIssue::Busy) => {
                aux.push((dir_display, "skipped", vec!["busy".to_string()]));
            }
            Some(ScanIssue::Blocking(id)) => {
                aux.push((dir_display, "skipped", vec![id.to_string()]));
            }
            Some(ScanIssue::Partial { id, line }) => {
                aux.push((dir_display, "partial", vec![format!("{id} L{line}")]));
            }
            None => {}
        }
        state_events(&info, &p.nick, since, now, &mut events, &mut tie);
        state_infos.push(info);
    }

    // §10.1.5: unreadable directories of the subtree walk.
    let unreadable: Vec<String> = scanned
        .unreadable_dirs
        .iter()
        .map(|dir| context::display_file(dir))
        .collect();
    for dir_display in &unreadable {
        aux.push((dir_display.clone(), "skipped", vec!["unreadable".to_string()]));
    }

    // Conflicts and own-entry records on the assembled scope.
    let scope = shared::assemble(&root, state_infos, unreadable);
    let files: Vec<&StateFileInfo> = scope_files(&scope);

    let own_locks: Vec<(&StateFileInfo, &StateEntryRec)> = files
        .iter()
        .flat_map(|file| {
            file.entries
                .iter()
                .filter(move |entry| {
                    entry.nick == p.nick && entry.status == "locked" && entry.valid(now)
                })
                .map(move |entry| (*file, entry))
        })
        .collect();
    for rec in &shared::conflicts_with(&scope, now, &p.nick) {
        // "overlaps your lock on <dir>": the own valid locked entry this
        // record conflicts with — the deepest overlapping declaration.
        let source_dir = files
            .iter()
            .find(|file| file.display_dir == rec.dir_display)
            .map(|file| file.dir.clone());
        let own_dir = match &source_dir {
            Some(source) => own_locks
                .iter()
                .filter(|(file, _)| overlaps(&file.dir, source))
                .max_by_key(|(file, _)| file.dir.components().count())
                .map(|(file, _)| file.display_dir.clone()),
            None => None,
        }
        .unwrap_or_else(|| rec.dir_display.clone());
        events.push(Event {
            when: rec.start,
            path_key: rec.dir_display.clone(),
            kind_rank: 4,
            tie,
            key: "conflict",
            values: vec![
                time::format(rec.start),
                rec.dir_display.clone(),
                rec.status.clone(),
                rec.nick.clone(),
                format!("overlaps your lock on {own_dir}"),
            ],
        });
        tie += 1;
    }

    for file in &files {
        for entry in &file.entries {
            if entry.nick != p.nick {
                continue;
            }
            // §10.6.3: expiring needs no window (own valid entries within
            // EXPIRING_WINDOW of ending); expired uses until ∈ [since, now).
            if entry.valid(now)
                && (entry.until - now).num_seconds() <= consts::EXPIRING_WINDOW
            {
                let minutes = (entry.until - now).num_seconds().max(0) / 60;
                events.push(Event {
                    when: entry.until,
                    path_key: file.display_dir.clone(),
                    kind_rank: 5,
                    tie,
                    key: "expiring",
                    values: vec![
                        time::format(entry.until),
                        file.display_dir.clone(),
                        entry.status.clone(),
                        entry.nick.clone(),
                        format!("ends in {minutes}m"),
                    ],
                });
                tie += 1;
            }
            if entry.until >= since && entry.until < now {
                let at = time::format(entry.until);
                events.push(Event {
                    when: entry.until,
                    path_key: file.display_dir.clone(),
                    kind_rank: 6,
                    tie,
                    key: "expired",
                    values: vec![
                        at.clone(),
                        file.display_dir.clone(),
                        entry.status.clone(),
                        entry.nick.clone(),
                        format!("expired at {at}"),
                    ],
                });
                tie += 1;
            }
        }
    }

    // §10.6.5 ordering; `skipped` / `partial` after the records.
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

    aux.sort_by(|a, b| a.0.cmp(&b.0));
    aux.dedup_by(|a, b| a.0 == b.0);
    let aux_count = aux.len();
    for (path, key, causes) in aux {
        let mut values = vec![path];
        values.extend(causes);
        report.record(key, values, 2, consts::KIND_WIDTH);
    }

    // §10.1.3 / §8.2: include globs that matched nothing (not counted).
    if !p.walk.no_ignore {
        for glob in &scanned.zero_includes {
            report.record(
                "include",
                vec![
                    glob.clone(),
                    "matched 0 files; ignored directories are not reopened (pass --no-ignore)"
                        .to_string(),
                ],
                2,
                consts::KEY_WIDTH,
            );
        }
    }

    report.total(record_count + aux_count);
    // §8.1.11: conflict rows (row-keyed), skipped / partial rows, and
    // the include hint all warn.
    if events.iter().any(|event| event.key == "conflict")
        || aux_count > 0
        || (!p.walk.no_ignore && !scanned.zero_includes.is_empty())
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
        author_of.entry(comment.seq).or_insert(comment.nick.as_str());
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

/// `state` records of one state file: others' declarations, any status,
/// expired included, `start` in the window (§10.6.3).
fn state_events(
    info: &StateFileInfo,
    nick: &str,
    since: Time,
    now: Time,
    events: &mut Vec<Event>,
    tie: &mut u64,
) {
    for entry in &info.entries {
        if entry.nick == nick || !(entry.start >= since && entry.start < now) {
            continue;
        }
        let mut values = vec![
            time::format(entry.start),
            info.display_dir.clone(),
            entry.status.clone(),
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
}

/// Build the scope member for a state file parsed by this command (the
/// single read of the file; §10.5.6 issue classes applied).
fn build_state_info(file: &Path, parsed: &doc::Document) -> StateFileInfo {
    let parent = file.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
    let dir = std::fs::canonicalize(&parent).unwrap_or(parent.clone());
    let display_dir = context::object_of_dir(&parent);
    let mut entries: Vec<StateEntryRec> = Vec::new();
    for section in &parsed.state {
        for decl in &section.entries {
            entries.push(StateEntryRec {
                status: section.status.clone(),
                nick: decl.nick.clone(),
                start: decl.start,
                until: decl.until,
            });
        }
    }
    let mut issue = None;
    let sorted = sort_diagnostics(parsed.diagnostics.clone());
    if let Some(d) = sorted.iter().find(|d| SKIP_CLASS.contains(&d.id)) {
        issue = Some(ScanIssue::Blocking(d.id));
        entries.clear();
    } else if let Some(d) = sorted.iter().find(|d| PARTIAL_CLASS.contains(&d.id)) {
        issue = Some(ScanIssue::Partial { id: d.id, line: d.line });
    }
    StateFileInfo {
        file: file.to_path_buf(),
        dir,
        display_dir,
        entries,
        issue,
    }
}

fn scope_files(scope: &shared::StateScope) -> Vec<&StateFileInfo> {
    let mut files: Vec<&StateFileInfo> = Vec::new();
    if let Some(file) = &scope.self_file {
        files.push(file);
    }
    files.extend(scope.ancestors.iter());
    files.extend(scope.descendants.iter());
    files
}

/// §10.5.3 directory relation on canonicalized paths.
fn overlaps(a: &Path, b: &Path) -> bool {
    a == b || a.starts_with(b) || b.starts_with(a)
}

fn canonical_of(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
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

