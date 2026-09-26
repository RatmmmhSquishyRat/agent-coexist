//! `check-state [DIR] [WALK]` — the directory state overview (spec
//! §7.2, §8.2, §10.5).
//!
//! `shared::collect` builds the §10.5.1 scope (DIR + ancestors +
//! subtree, single implementation for every consumer). A fatal
//! ignore/config source error propagates as [`Fault::Io`] with the
//! original path and cause, matching `list` and `read-notification`.
//! Ordinary unreadable directories stay `skipped` warnings. Output:
//! `inherited` rows for ancestors' valid entries (near → far), DIR's
//! own entries at indent 2, subdirectory groups (canonical order,
//! lexical display) at indent 4, then `skipped` / `partial` rows
//! (§10.5.6) and `canonicalize` warnings before `total`.

use std::collections::HashSet;
use std::path::Path;

use crate::consts;
use crate::error::Fault;
use crate::fsx;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::shared::{self, ScanIssue, StateEntryRec};
use crate::ops::CheckStateParams;
use crate::report::Report;
use crate::time;
use crate::Time;

use super::{dir_error, include_hints};

pub fn check_state(ctx: &mut Ctx, p: &CheckStateParams) -> Result<Report, Fault> {
    fsx::walk::validate(&p.walk)?;
    let object = context::object_of_dir(&p.dir);
    if let Some(rep) = dir_error(ctx, "check-state", &p.dir)? {
        return Ok(rep);
    }
    let scope = shared::collect(ctx, &p.dir, &p.walk)?;
    // Membership uses native identity; the scope traversal below owns order.
    let conflicts: HashSet<_> = shared::conflicts_all(&scope, ctx.now)
        .into_iter()
        .map(|rec| (rec.dir, rec.status, rec.nick, rec.start, rec.until))
        .collect();

    let mut report = Report::new("check-state", object);
    let now = ctx.now;
    let mut count = 0usize;
    let mut warn = false;

    // §8.2: ancestors' valid entries, near → far (scope.ancestors is
    // already in that order).
    for file in &scope.ancestors {
        for entry in file.effective_entries().filter(|entry| entry.valid(now)) {
            let mut values = vec![
                names::display_text(&entry.status),
                entry.nick.clone(),
                span_of(entry),
                file.display_dir.clone(),
            ];
            if conflicts.contains(&conflict_key(entry, &file.dir)) {
                values.push("conflict".to_string());
                warn = true;
            }
            report.record("inherited", values, 2, consts::KEY_WIDTH);
            count += 1;
        }
    }

    if let Some(file) = &scope.self_file {
        for entry in file.effective_entries() {
            let mark = entry_mark(&conflicts, entry, &file.dir, now);
            if mark.is_some() {
                warn = true;
            }
            push_entry(
                &mut report,
                &entry.status,
                &entry.nick,
                entry.start,
                entry.until,
                mark,
                2,
            );
            count += 1;
        }
    }

    // Descendant groups: canonical order, native path relative to DIR
    // then lexical display (§10.5.4 / §10.5.5). Folding happens only at emit.
    for file in &scope.descendants {
        let name = relative_group_name(&file.file, &p.dir);
        report.record(format!("{name}/"), Vec::new(), 2, 0);
        for entry in file.effective_entries() {
            let mark = entry_mark(&conflicts, entry, &file.dir, now);
            if mark.is_some() {
                warn = true;
            }
            push_entry(
                &mut report,
                &entry.status,
                &entry.nick,
                entry.start,
                entry.until,
                mark,
                4,
            );
            count += 1;
        }
    }

    // §10.5.6 / §10.1.5: files and directories that could not
    // contribute, path-ascending (§10.5.4).
    let mut aux: Vec<(&Path, &'static str, Vec<String>)> = Vec::new();
    for (dir, issue) in &scope.issues {
        match issue {
            ScanIssue::Busy => {
                aux.push((dir.as_path(), "skipped", vec!["busy".to_string()]));
            }
            ScanIssue::Blocking(id) => {
                aux.push((dir.as_path(), "skipped", vec![id.to_string()]));
            }
            ScanIssue::Partial { id, line } => {
                aux.push((dir.as_path(), "partial", vec![format!("{id} L{line}")]));
            }
        }
    }
    for dir in &scope.unreadable_dirs {
        aux.push((dir.as_path(), "skipped", vec!["unreadable".to_string()]));
    }
    aux.sort();
    aux.dedup();
    // Deduplication is complete before lossy display formatting.
    let mut aux: Vec<_> = aux
        .into_iter()
        .map(|(dir, key, causes)| (context::object_of_dir(dir), key, causes))
        .collect();
    aux.sort();
    for (dir_display, key, causes) in aux {
        let mut values = vec![dir_display];
        values.extend(causes);
        report.record(key, values, 2, consts::KEY_WIDTH);
        count += 1;
        warn = true;
    }

    // §10.5.2: canonicalize failures fell back to lexical paths — the
    // verdict may be incomplete, so warn (not counted in `total`).
    for dir in &scope.canonicalize_failed {
        report.record(
            "canonicalize",
            vec![context::object_of_dir(dir), "failed".to_string()],
            2,
            consts::KEY_WIDTH,
        );
        warn = true;
    }

    warn |= include_hints(&mut report, &scope.zero_includes);
    report.total(count);
    if warn {
        report.warn();
    }
    Ok(report)
}

fn push_entry(
    report: &mut Report,
    status: &str,
    nick: &str,
    start: Time,
    until: Time,
    mark: Option<&'static str>,
    indent: usize,
) {
    let mut values = vec![nick.to_string(), span_of_times(start, until)];
    if let Some(mark) = mark {
        values.push(mark.to_string());
    }
    report.record(
        names::display_text(status),
        values,
        indent,
        consts::KEY_WIDTH,
    );
}

type ConflictKey<'a> = (&'a Path, &'a str, &'a str, Time, Time);

/// §8.1.11 marks: `conflict` via §10.5 membership, else `expired` when
/// the entry is no longer valid (§1 — `start > until` counts as expired,
/// §6.2.5).
fn entry_mark(
    conflicts: &HashSet<ConflictKey<'_>>,
    entry: &StateEntryRec,
    dir: &Path,
    now: Time,
) -> Option<&'static str> {
    if conflicts.contains(&conflict_key(entry, dir)) {
        Some("conflict")
    } else if !entry.valid(now) {
        Some("expired")
    } else {
        None
    }
}

fn conflict_key<'a>(entry: &'a StateEntryRec, dir: &'a Path) -> ConflictKey<'a> {
    (dir, &entry.status, &entry.nick, entry.start, entry.until)
}

fn span_of(entry: &StateEntryRec) -> String {
    span_of_times(entry.start, entry.until)
}

fn span_of_times(start: Time, until: Time) -> String {
    format!("{}..{}", time::format(start), time::format(until))
}

/// Group heading for a descendant: native components of the state file's
/// parent relative to the requested DIR. Folding happens only at emit.
fn relative_group_name(file: &Path, dir: &Path) -> String {
    match fsx::walk::relative_native_parts(file, dir) {
        Some((dirs, _)) => dirs
            .iter()
            .map(|component| names::fold(&component.to_string_lossy()))
            .collect::<Vec<_>>()
            .join("/"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::context::test_ctx;
    use crate::report::{Row, Status};
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-check-state-tests")
            .join(format!(
                "{}-{}-{}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn values<'a>(report: &'a Report, wanted: &str) -> Vec<&'a [String]> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key, values, .. } if key == wanted => Some(values.as_slice()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn overview_uses_latest_declaration_even_when_its_range_is_inverted() {
        let dir = scratch("latest-until");
        std::fs::write(
            dir.join("state.agent-besties.md"),
            concat!(
                "# .state\n\n## locked\n",
                "alice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n",
                "alice 2026-09-25T15:00:00Z..2026-09-25T14:00:00Z\n",
                "bob 2026-09-25T11:00:00Z..2026-09-25T16:00:00Z\n",
            ),
        )
        .unwrap();
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let p = CheckStateParams {
            dir: dir.clone(),
            walk: Default::default(),
        };
        let report = check_state(&mut ctx, &p).unwrap();
        let entries = values(&report, "locked");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0][0], "alice");
        assert_eq!(entries[0].last().unwrap(), "expired");
        assert_eq!(entries[1][0], "bob");
        assert!(!entries
            .iter()
            .any(|row| row.iter().any(|v| v == "conflict")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn folded_sibling_paths_do_not_share_conflict_markers() {
        let dir = scratch("folded-sibling-paths");
        let conflicting = dir.join("a  b");
        let independent = dir.join("a b");
        for (path, nicks) in [
            (&conflicting, &["alice", "bob"][..]),
            (&independent, &["alice"][..]),
        ] {
            std::fs::create_dir(path).unwrap();
            let mut text = "# .state\n\n## locked\n".to_string();
            for nick in nicks {
                text.push_str(&format!(
                    "{nick} 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"
                ));
            }
            std::fs::write(path.join("state.agent-besties.md"), text).unwrap();
        }
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let p = CheckStateParams {
            dir: dir.clone(),
            walk: fsx::walk::Options {
                no_ignore: true,
                ..Default::default()
            },
        };
        let report = check_state(&mut ctx, &p).unwrap();
        let entries = values(&report, "locked");
        assert_eq!(
            entries
                .iter()
                .map(|row| (row[0].as_str(), row.get(2).map(String::as_str)))
                .collect::<Vec<_>>(),
            vec![
                ("alice", Some("conflict")),
                ("bob", Some("conflict")),
                ("alice", None),
            ]
        );
        assert_eq!(values(&report, "total")[0][0], "3");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn colliding_incomplete_sources_each_contribute_a_record() {
        let dir = scratch("colliding-incomplete-sources");
        for (name, contents) in [
            ("bad  source", "# invalid\n"),
            ("bad source", "# invalid\n"),
            (
                "partial  source",
                "# .state\n\n## locked\ninvalid declaration\n",
            ),
            (
                "partial source",
                "# .state\n\n## locked\ninvalid declaration\n",
            ),
        ] {
            let child = dir.join(name);
            std::fs::create_dir(&child).unwrap();
            std::fs::write(child.join("state.agent-besties.md"), contents).unwrap();
        }
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let report = check_state(
            &mut ctx,
            &CheckStateParams {
                dir: dir.clone(),
                walk: fsx::walk::Options {
                    no_ignore: true,
                    ..Default::default()
                },
            },
        )
        .unwrap();
        assert_eq!(report.status, Status::Warn);
        let skipped = values(&report, "skipped");
        assert_eq!(skipped.len(), 2);
        assert_eq!(skipped[0], skipped[1]);
        let partial = values(&report, "partial");
        assert_eq!(partial.len(), 2);
        assert_eq!(partial[0], partial[1]);
        assert_eq!(values(&report, "total")[0][0], "4");

        let declared = crate::ops::state::run(
            &mut ctx,
            &crate::ops::StateParams::Declare(crate::ops::StateDeclare {
                dir: dir.clone(),
                nick: "alice".into(),
                status: "observing".into(),
                for_secs: Some(7200),
            }),
        )
        .unwrap();
        assert_eq!(declared.exit, 0, "{declared:?}");
        assert_eq!(values(&declared, "skipped").len(), 2);
        assert_eq!(values(&declared, "partial").len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn status_titles_are_safe_in_inherited_and_local_rows() {
        let dir = scratch("status-title-controls");
        let child = dir.join("child");
        let deep = child.join("deep");
        for (path, nick) in [(&dir, "carol"), (&child, "alice"), (&deep, "bob")] {
            std::fs::create_dir_all(path).unwrap();
            std::fs::write(
                path.join("state.agent-besties.md"),
                format!(
                    "# .state\n\n## custom\u{1b}[2J\n{nick} 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n"
                ),
            )
            .unwrap();
        }
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let p = CheckStateParams {
            dir: child,
            walk: fsx::walk::Options {
                no_ignore: true,
                ..Default::default()
            },
        };
        let report = check_state(&mut ctx, &p).unwrap();
        let inherited = values(&report, "inherited");
        assert_eq!(inherited.len(), 1);
        assert_eq!(&inherited[0][..2], &["custom\u{FFFD}[2J", "carol"]);
        assert_eq!(
            values(&report, "custom\u{FFFD}[2J")
                .iter()
                .map(|row| row[0].as_str())
                .collect::<Vec<_>>(),
            vec!["alice", "bob"]
        );
        assert_eq!(values(&report, "total")[0][0], "3");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn zero_include_hint_warns_without_counting_as_a_record() {
        let dir = scratch("zero-includes");
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let mut p = CheckStateParams {
            dir: dir.clone(),
            walk: fsx::walk::Options {
                include: vec!["*.agent-besties.md".to_string()],
                ..Default::default()
            },
        };
        let report = check_state(&mut ctx, &p).unwrap();
        assert_eq!(report.status, Status::Warn);
        assert_eq!(values(&report, "include")[0][0], "*.agent-besties.md");
        assert_eq!(values(&report, "total")[0][0], "0");
        p.walk.no_ignore = true;
        let without_ignore = check_state(&mut ctx, &p).unwrap();
        assert!(values(&without_ignore, "include").is_empty());
        assert_eq!(values(&without_ignore, "total")[0][0], "0");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn descendant_groups_are_relative_to_the_literal_requested_dir() {
        let parent = scratch("literal-root");
        let dir = parent.join("room ");
        let child = dir.join("child");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(
            child.join("state.agent-besties.md"),
            concat!(
                "# .state\n\n## observing\n",
                "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
        )
        .unwrap();
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let report = check_state(
            &mut ctx,
            &CheckStateParams {
                dir: dir.clone(),
                walk: fsx::walk::Options {
                    no_ignore: true,
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let groups: Vec<&str> = report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key, values, .. } if values.is_empty() && key.ends_with('/') => {
                    Some(key.as_str())
                }
                _ => None,
            })
            .collect();
        assert_eq!(groups, vec!["child/"]);
        let observing = values(&report, "observing");
        assert_eq!(observing.len(), 1);
        assert_eq!(observing[0][0], "alice");
        assert_eq!(values(&report, "total")[0][0], "1");
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn malformed_ignore_fails_the_overview_like_other_scans() {
        let dir = scratch("malformed-ignore");
        std::fs::write(dir.join(".ignore"), "{broken\n").unwrap();
        let child = dir.join("child");
        std::fs::create_dir(&child).unwrap();
        std::fs::write(
            child.join("state.agent-besties.md"),
            concat!(
                "# .state\n\n## locked\n",
                "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
            ),
        )
        .unwrap();
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let err = check_state(
            &mut ctx,
            &CheckStateParams {
                dir: dir.clone(),
                walk: Default::default(),
            },
        )
        .expect_err("fatal ignore source must fail the overview");
        match err {
            Fault::Io {
                action,
                path,
                source,
            } => {
                assert_eq!(action, "scan");
                assert_eq!(path, dir.join(".ignore"));
                assert_eq!(source.kind(), std::io::ErrorKind::InvalidData);
                assert!(!source.to_string().is_empty());
            }
            other => panic!("expected scan Io, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
