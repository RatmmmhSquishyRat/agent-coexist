//! `check-state [DIR] [WALK]` — the directory state overview (spec
//! §7.2, §8.2, §10.5).
//!
//! `shared::collect` builds the §10.5.1 scope (DIR + ancestors +
//! subtree, single implementation for every consumer). Output:
//! `inherited` rows for ancestors' valid entries (near → far), DIR's
//! own entries at indent 2, subdirectory groups (canonical order,
//! lexical display) at indent 4, then `skipped` / `partial` rows
//! (§10.5.6) and `canonicalize` warnings before `total`.

use crate::consts;
use crate::error::Fault;
use crate::fsx;
use crate::ops::context::{self, Ctx};
use crate::ops::shared::{self, ConflictRec, ScanIssue, StateEntryRec};
use crate::ops::CheckStateParams;
use crate::report::Report;
use crate::time;
use crate::Time;

use super::dir_error;

pub fn check_state(ctx: &mut Ctx, p: &CheckStateParams) -> Result<Report, Fault> {
    fsx::walk::validate(&p.walk)?;
    let object = context::object_of_dir(&p.dir);
    if let Some(rep) = dir_error(ctx, "check-state", &p.dir)? {
        return Ok(rep);
    }
    let scope = shared::collect(ctx, &p.dir, &p.walk);
    let conflicts = shared::conflicts_all(&scope, ctx.now);

    let mut report = Report::new("check-state", object.clone());
    let dir_display = object;
    let now = ctx.now;
    let mut count = 0usize;
    let mut warn = false;

    // §8.2: ancestors' valid entries, near → far (scope.ancestors is
    // already in that order).
    for file in &scope.ancestors {
        for entry in file.entries.iter().filter(|entry| entry.valid(now)) {
            let mut values = vec![
                entry.status.clone(),
                entry.nick.clone(),
                span_of(entry),
                file.display_dir.clone(),
            ];
            if conflicts
                .iter()
                .any(|rec| same_entry(rec, entry, &file.display_dir))
            {
                values.push("conflict".to_string());
                warn = true;
            }
            report.record("inherited", values, 2, consts::KEY_WIDTH);
            count += 1;
        }
    }

    if let Some(file) = &scope.self_file {
        for entry in &file.entries {
            let mark = entry_mark(&conflicts, entry, &file.display_dir, now);
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

    // Descendant groups: canonical order, lexical display names relative
    // to DIR (§10.5.4 / §10.5.5).
    for file in &scope.descendants {
        let name = relative_dir_name(&file.display_dir, &dir_display);
        report.record(format!("{name}/"), Vec::new(), 2, 0);
        for entry in &file.entries {
            let mark = entry_mark(&conflicts, entry, &file.display_dir, now);
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
    let mut aux: Vec<(String, &'static str, Vec<String>)> = Vec::new();
    for (dir_display, issue) in &scope.issues {
        match issue {
            ScanIssue::Busy => {
                aux.push((dir_display.clone(), "skipped", vec!["busy".to_string()]));
            }
            ScanIssue::Blocking(id) => {
                aux.push((dir_display.clone(), "skipped", vec![id.to_string()]));
            }
            ScanIssue::Partial { id, line } => {
                aux.push((
                    dir_display.clone(),
                    "partial",
                    vec![format!("{id} L{line}")],
                ));
            }
        }
    }
    for dir_display in &scope.unreadable_dirs {
        aux.push((
            dir_display.clone(),
            "skipped",
            vec!["unreadable".to_string()],
        ));
    }
    aux.sort_by(|a, b| a.0.cmp(&b.0));
    for (dir_display, key, causes) in aux {
        let mut values = vec![dir_display];
        values.extend(causes);
        report.record(key, values, 2, consts::KEY_WIDTH);
        count += 1;
        warn = true;
    }

    // §10.5.2: canonicalize failures fell back to lexical paths — the
    // verdict may be incomplete, so warn (not counted in `total`).
    for dir_display in &scope.canonicalize_failed {
        report.record(
            "canonicalize",
            vec![dir_display.clone(), "failed".to_string()],
            2,
            consts::KEY_WIDTH,
        );
        warn = true;
    }

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
    report.record(status.to_string(), values, indent, consts::KEY_WIDTH);
}

/// §8.1.11 marks: `conflict` via §10.5 membership, else `expired` when
/// the entry is no longer valid (§1 — `start > until` counts as expired,
/// §6.2.5).
fn entry_mark(
    conflicts: &[ConflictRec],
    entry: &StateEntryRec,
    dir_display: &str,
    now: Time,
) -> Option<&'static str> {
    if conflicts
        .iter()
        .any(|rec| same_entry(rec, entry, dir_display))
    {
        Some("conflict")
    } else if !entry.valid(now) {
        Some("expired")
    } else {
        None
    }
}

fn same_entry(rec: &ConflictRec, entry: &StateEntryRec, dir_display: &str) -> bool {
    rec.nick == entry.nick
        && rec.status == entry.status
        && rec.start == entry.start
        && rec.until == entry.until
        && rec.dir_display == dir_display
}

fn span_of(entry: &StateEntryRec) -> String {
    span_of_times(entry.start, entry.until)
}

fn span_of_times(start: Time, until: Time) -> String {
    format!("{}..{}", time::format(start), time::format(until))
}

/// Group heading for a descendant: its display path relative to DIR.
fn relative_dir_name(display: &str, dir_display: &str) -> String {
    let display = strip_dot_slash(display);
    let base = strip_dot_slash(dir_display);
    if base == "." {
        return display.to_string();
    }
    let prefix = format!("{base}/");
    match display.strip_prefix(&prefix) {
        Some(rest) if !rest.is_empty() => rest.to_string(),
        _ => display.to_string(),
    }
}

/// Lexical `./` prefix strip for display paths (a DIR of `.`-relative form
/// leaves it; comparing both sides on the same stripped basis).
fn strip_dot_slash(s: &str) -> &str {
    s.strip_prefix("./").unwrap_or(s)
}
