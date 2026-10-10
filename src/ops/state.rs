//! The `state` command (spec §6.4 declare / remove forms, §8.2 state).
//!
//! Both forms go through [`fsx::mutate`] with `FASTFAIL_GRACE_MS` and
//! `text::apply` splices only — every untouched byte is preserved. The
//! declare form computes conflicts strictly after the write, flush and lock
//! release (§6.4.1); state writes are never functional-lock blocked
//! (§6.4.3). A post-write conflict-scan I/O fault stays a warning on the
//! successful write report — the committed mutation is not rolled back or
//! reported unwritten. Business failures come back as [`Report`]s; only
//! usage-class faults (from [`time::until`]) and write-path OS-level I/O
//! faults propagate.

use std::collections::HashMap;
use std::path::Path;

use crate::Time;
use crate::consts;
use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::report::{Report, Status};
use crate::text::{self, Span, Splice};
use crate::time;
use crate::types::is_reader_comment;

use super::context::{self, Ctx};
use super::{StateParams, reason_report, shared};

/// The fixed reader comment written into every new state file (§6.4.1).
const STATE_COMMENT: &str = "locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.";

/// Dispatch the `state` command.
pub fn run(ctx: &mut Ctx, p: &StateParams) -> Result<Report, Fault> {
    match p {
        StateParams::Declare(d) => declare(ctx, d),
        StateParams::Remove(r) => remove(ctx, r),
    }
}

// ---------------------------------------------------------------------------
// Declare form (§6.4.1)
// ---------------------------------------------------------------------------

fn declare(ctx: &mut Ctx, p: &super::StateDeclare) -> Result<Report, Fault> {
    let for_secs = p.for_secs.unwrap_or(if p.status == "locked" {
        consts::DEFAULT_FOR_LOCKED
    } else {
        consts::DEFAULT_FOR_OTHER
    });
    // Reject an unrepresentable duration before accessing any file. The
    // actual start and end are computed together under the write lock.
    time::until(ctx.now, for_secs)?;
    if let Some(rep) = check_dir(ctx, &p.dir)? {
        return Ok(rep);
    }
    let dctx = context::diag_context_for_dir(ctx, "state", &p.dir);
    let object = context::object_of_dir(&p.dir);

    let target = match fsx::walk::state_file(&p.dir)? {
        Some(f) => f,
        None => p.dir.join(format!("state{}", consts::SUFFIX)),
    };
    let file_for_parse = target.clone();
    let nick = p.nick.clone();
    let status = p.status.clone();

    let outcome = fsx::mutate(
        &target,
        fsx::WritePolicy {
            wait_ms: consts::FASTFAIL_GRACE_MS,
            create: true,
            create_parents: false,
        },
        |bytes: &[u8], _initial: bool| {
            // A declaration queued across a whole-second boundary must
            // remain visible after a successful intervening scan. Keep
            // its full duration relative to this same event-time sample.
            let now = ctx.write_now();
            let until = time::until(now, for_secs)?;
            let plan = plan_declare(bytes, &file_for_parse, now, &nick, &status, until)?;
            Ok(match plan {
                DeclarePlan::Write {
                    bytes: new_bytes,
                    cleaned,
                } => (
                    fsx::Mutation::Write(new_bytes),
                    DeclareOut::Written {
                        cleaned,
                        now,
                        until,
                    },
                ),
                DeclarePlan::Refused(r) => (fsx::Mutation::Unchanged, DeclareOut::Refused(r)),
            })
        },
    );

    let result = match outcome {
        Ok(r) => r,
        Err(Fault::Reason { id, params }) => {
            return Ok(reason_report(&dctx, "state", &object, id, params));
        }
        Err(Fault::Diagnostics(ds)) => return Ok(refusal_report(&object, &target, None, ds)),
        Err(f) => return Err(f),
    };

    let mut report = Report::new("state", object.clone());
    let now = match result.value {
        DeclareOut::Refused(r) => {
            if let Some(actual) = r.e_type {
                return Ok(e_type_report(&dctx, &object, actual));
            }
            return Ok(refusal_report(&object, &target, r.kind, r.diagnostics));
        }
        DeclareOut::Written {
            cleaned,
            now,
            until,
        } => {
            if result.created {
                report.kv("created", vec![context::display_file(&target)]);
            }
            report.kv(
                "set",
                vec![
                    names::display_text(&p.status),
                    p.nick.clone(),
                    format!("{}..{}", time::format(now), time::format(until)),
                ],
            );
            for c in cleaned {
                report.kv(
                    "cleaned",
                    vec![
                        names::display_text(&c.status),
                        c.nick,
                        format!("{}..{}", time::format(c.start), time::format(c.until)),
                    ],
                );
            }
            now
        }
    };

    // Conflict scan strictly after write + flush + lock release (§6.4.1);
    // state writes are never functional-lock blocked (§6.4.3).
    let scope = match shared::collect(ctx, &p.dir, &default_walk_options()) {
        Ok(scope) => scope,
        Err(fault) => {
            // The declaration is already committed; only the conflict check
            // is missing. The `scan` row keeps the actual cause, folded so
            // it stays splittable (§8.1.4–5, §10.9.4). The fix resolves that
            // cause and reruns just the read-only conflict check, never the
            // write (§10.5.4).
            let dir_arg = diag::path_operand(p.dir.as_os_str());
            let (scan, fix) = match fault {
                Fault::Source {
                    path,
                    problem,
                    cause,
                } => {
                    let source = context::display_file(&path);
                    let check = dir_arg
                        .as_deref()
                        .map(|dir| format!("agent-coexist check-state {dir}"));
                    // Without a copyable command the sentence still names the
                    // command, as other manual fixes do (`run agent-coexist
                    // <sub> by hand`); its operand is this report's directory.
                    let next = if check.is_some() {
                        "check for conflicts without declaring again"
                    } else {
                        "check for conflicts without declaring again: run agent-coexist check-state on this directory"
                    };
                    let fix = problem.fix(&source, next, check.as_deref());
                    (vec![source, names::display_text(&cause)], fix)
                }
                Fault::Io { path, source, .. } => (
                    vec![
                        context::display_file(&path),
                        names::display_text(&source.to_string()),
                    ],
                    scan_stopped_fix(dir_arg.as_deref()),
                ),
                // `collect` returns no other fault for these walk options
                // (no include/exclude patterns to reject); should one
                // appear, the committed write is still never reported as
                // unwritten.
                _ => (
                    vec![object.clone(), "failed".to_string()],
                    scan_stopped_fix(dir_arg.as_deref()),
                ),
            };
            report.kv("scan", scan);
            report.kv("fix", vec![fix]);
            report.warn();
            return Ok(report);
        }
    };
    for fail in &scope.canonicalize_failed {
        report.kv(
            "canonicalize",
            shared::canonicalize_values(&fail.dir, &fail.reason),
        );
        report.warn();
    }
    let conflicts = shared::conflicts_with(&scope, now, &p.nick);
    let mut has_conflict = false;
    for (c, _) in &conflicts {
        report.kv(
            "conflict",
            vec![
                c.status.to_string(),
                c.nick.to_string(),
                format!("{}..{}", time::format(c.start), time::format(c.until)),
                c.dir_display.to_string(),
            ],
        );
        has_conflict = true;
    }
    if has_conflict {
        report.warn();
    }
    // The declaration is committed; a skipped or partial state file, an
    // unlisted directory, or a directory whose identity fell back to
    // lexical paths (`canonicalize` rows) may hold a conflicting lock
    // this check did not see. Name what to open, never a second
    // declaration (§10.5.6).
    let rows = shared::scan_rows(&scope.issues, &scope.unreadable_dirs);
    // Identity fell back to lexical paths: the whole chain is re-checked
    // from the declared directory itself, once the path exists and every
    // ancestor has read and search permission (the write commands' lock
    // scan names the written file's parent instead).
    let declared =
        (!scope.canonicalize_failed.is_empty()).then(|| shared::Follow::DeclaredDir(p.dir.clone()));
    if !rows.is_empty() || declared.is_some() {
        for row in &rows {
            report.kv(row.key, row.values.clone());
        }
        report.kv(
            "fix",
            vec![shared::scan_fix(
                rows.iter().map(|row| &row.follow).chain(declared.iter()),
                diag::FIX_STATE_SCAN_INCOMPLETE,
                diag::FIX_STATE_SCAN_INCOMPLETE_MANUAL,
            )],
        );
        report.warn();
    }
    Ok(report)
}

#[derive(Debug)]
struct CleanedEntry {
    status: String,
    nick: String,
    start: Time,
    until: Time,
}

enum DeclarePlan {
    Write {
        bytes: Vec<u8>,
        cleaned: Vec<CleanedEntry>,
    },
    Refused(Refusal),
}

enum DeclareOut {
    Written {
        cleaned: Vec<CleanedEntry>,
        now: Time,
        until: Time,
    },
    Refused(Refusal),
}

/// Compute the declare-form splice plan against the current bytes.
///
/// Geometry (§6.4.1): the written section has every clean line deleted (with
/// terminators) and the merged serialized line inserted at the original
/// first clean line's start; with no clean line the insertion point is after
/// the H2's following blank line (one blank added right after the H2
/// terminator when absent). Every other section only participates through
/// expiry cleanup, with the same per-section geometry. Dirty lines, blank
/// lines and fences are byte-preserved.
fn plan_declare(
    source: &[u8],
    file: &Path,
    now: Time,
    nick: &str,
    status: &str,
    until: Time,
) -> Result<DeclarePlan, Fault> {
    if source.is_empty() {
        // Initial content (§6.4.1): H1 + blank + comment, then the section.
        let head = format!("# .state\n\n> {STATE_COMMENT}\n");
        let entry = entry_text(nick, now, until);
        let section = format!("## {status}\n\n{entry}\n");
        let full = format!("{head}{}{section}", text::append_gap(&head));
        return Ok(DeclarePlan::Write {
            bytes: full.into_bytes(),
            cleaned: Vec::new(),
        });
    }
    let d = match doc::parse(source.to_vec(), file, now) {
        Ok(d) => d,
        Err(Fault::Diagnostics(ds)) => {
            return Ok(DeclarePlan::Refused(Refusal::blocking(None, ds)));
        }
        Err(f) => return Err(f),
    };
    if let Some(r) = blocking_refusal(&d) {
        return Ok(DeclarePlan::Refused(r));
    }
    match d.kind {
        Some(doc::Kind::State) => {}
        Some(other) => return Ok(DeclarePlan::Refused(Refusal::e_type(other))),
        None => {
            return Ok(DeclarePlan::Refused(Refusal::blocking(
                None,
                d.diagnostics.clone(),
            )));
        }
    }

    let target_sec = d.state.iter().position(|s| s.status == status);
    let mut splices: Vec<Splice> = Vec::new();
    let mut cleaned: Vec<CleanedEntry> = Vec::new();

    for (si, sec) in d.state.iter().enumerate() {
        let is_target = Some(si) == target_sec;
        let has_expired_clean = sec.entries.iter().any(|e| e.clean && !e.valid(now));
        if !is_target && !has_expired_clean {
            continue; // untouched sections keep their bytes (§2.7.2)
        }
        let mut survivors: Vec<(String, Time, Time)> = Vec::new();
        for e in sec.entries.iter().filter(|e| e.clean) {
            if is_target && e.nick == nick {
                continue; // replaced by the upsert (renewal, not cleanup)
            }
            if !e.valid(now) {
                cleaned.push(CleanedEntry {
                    status: sec.status.clone(),
                    nick: e.nick.clone(),
                    start: e.start,
                    until: e.until,
                });
                continue;
            }
            survivors.push((e.nick.clone(), e.start, e.until));
        }
        let mut merged = merge_dupes(survivors);
        if is_target {
            merged.push((nick.to_string(), now, until));
        }

        if sec.clean_lines.is_empty() {
            if is_target {
                let section = &d.sections[sec.section];
                let (at, prefix) = h2_insert_point(source, section);
                let mut bytes = prefix.as_bytes().to_vec();
                bytes.extend_from_slice(&serialize_line(&merged));
                splices.push(Splice {
                    range: at..at,
                    replacement: bytes,
                });
            }
            // A non-target section without clean lines has nothing to clean.
        } else {
            let mut spans: Vec<Span> = sec
                .clean_lines
                .iter()
                .map(|s| norm_span(source, s))
                .collect();
            spans.sort_by_key(|s| s.start);
            let first = spans[0].clone();
            for s in &spans[1..] {
                splices.push(Splice {
                    range: s.clone(),
                    replacement: Vec::new(),
                });
            }
            splices.push(Splice {
                range: first,
                replacement: serialize_line(&merged),
            });
        }
    }

    if target_sec.is_none() {
        // Append the new section at EOF with the §10.3.3 gap.
        let entry = entry_text(nick, now, until);
        let section = format!("## {status}\n\n{entry}\n");
        let gap = text::append_gap(&d.source);
        let at = d.source.len();
        splices.push(Splice {
            range: at..at,
            replacement: format!("{gap}{section}").into_bytes(),
        });
    }

    let bytes = text::apply(source, splices)?;
    Ok(DeclarePlan::Write { bytes, cleaned })
}

// ---------------------------------------------------------------------------
// Remove form (§6.4.2)
// ---------------------------------------------------------------------------

fn remove(ctx: &mut Ctx, p: &super::StateRemove) -> Result<Report, Fault> {
    if let Some(rep) = check_dir(ctx, &p.dir)? {
        return Ok(rep);
    }
    let dctx = context::diag_context_for_dir(ctx, "state", &p.dir);
    let object = context::object_of_dir(&p.dir);

    // A missing state file (DIR exists) is E-STATE-NOTFOUND; --remove never
    // creates files (§6.4.2, §7.3.2).
    let target = match fsx::walk::state_file(&p.dir)? {
        Some(f) => f,
        None => return Ok(state_notfound(&dctx, &object, &p.nick, p.status.as_deref())),
    };
    let now = ctx.now;
    let file_for_parse = target.clone();
    let nick = p.nick.clone();
    let status_filter = p.status.clone();

    let outcome = fsx::mutate(
        &target,
        fsx::WritePolicy {
            wait_ms: consts::FASTFAIL_GRACE_MS,
            create: false,
            create_parents: false,
        },
        |bytes: &[u8], _initial: bool| {
            let plan = plan_remove(bytes, &file_for_parse, now, &nick, status_filter.as_deref())?;
            Ok(match plan {
                RemovePlan::Write {
                    bytes: new_bytes,
                    rows,
                    cleaned,
                    delete_file,
                } => {
                    let mutation = if delete_file {
                        fsx::Mutation::Delete
                    } else {
                        fsx::Mutation::Write(new_bytes)
                    };
                    (mutation, RemoveOut::Done { rows, cleaned })
                }
                RemovePlan::Dirty { line } => (fsx::Mutation::Unchanged, RemoveOut::Dirty { line }),
                RemovePlan::Refused(r) => (fsx::Mutation::Unchanged, RemoveOut::Refused(r)),
                RemovePlan::NotFound => (fsx::Mutation::Unchanged, RemoveOut::NotFound),
            })
        },
    );

    let result = match outcome {
        Ok(r) => r,
        Err(Fault::Reason {
            id: "E-FILE-MISSING",
            ..
        }) => {
            return Ok(state_notfound(&dctx, &object, &p.nick, p.status.as_deref()));
        }
        Err(Fault::Reason { id, params }) => {
            return Ok(reason_report(&dctx, "state", &object, id, params));
        }
        Err(Fault::Diagnostics(ds)) => return Ok(refusal_report(&object, &target, None, ds)),
        Err(f) => return Err(f),
    };

    let mut report = Report::new("state", object.clone());
    match result.value {
        RemoveOut::NotFound => {
            return Ok(state_notfound(&dctx, &object, &p.nick, p.status.as_deref()));
        }
        RemoveOut::Dirty { line } => {
            // The nick still has a parseable declaration on a dirty line
            // of a selected section: point at the actual line for manual
            // repair (§6.2.6 — writes never touch dirty lines) instead
            // of claiming the nick is absent or partially removing it.
            return Ok(reason_report(
                &dctx,
                "state",
                &object,
                "E-STATE-DIRTY",
                vec![
                    ("nick", p.nick.clone().into()),
                    ("dir", object.clone().into()),
                    ("line", line.to_string().into()),
                ],
            ));
        }
        RemoveOut::Refused(r) => {
            if let Some(actual) = r.e_type {
                return Ok(e_type_report(&dctx, &object, actual));
            }
            return Ok(refusal_report(&object, &target, r.kind, r.diagnostics));
        }
        RemoveOut::Done { rows, cleaned } => {
            // §6.4.4 row order: cleaned before removed.
            for c in cleaned {
                report.kv(
                    "cleaned",
                    vec![
                        names::display_text(&c.status),
                        c.nick,
                        format!("{}..{}", time::format(c.start), time::format(c.until)),
                    ],
                );
            }
            for row in rows {
                report.kv(
                    "removed",
                    vec![names::display_text(&row.status), p.nick.clone()],
                );
                if let Some((a, b)) = row.removed_lines {
                    report.kv("removed-lines", vec![format!("L{a}..L{b}")]);
                }
            }
            if result.deleted {
                report.kv("deleted", vec![context::display_file(&target)]);
            }
        }
    }
    if let Some(as_nick) = &p.as_nick
        && as_nick != &p.nick
    {
        report.kv("removed-other", vec![p.nick.clone()]);
        report.warn();
    }
    Ok(report)
}

#[derive(Debug)]
struct SectionRows {
    status: String,
    removed_lines: Option<(usize, usize)>,
}

#[derive(Debug)]
enum RemovePlan {
    Write {
        bytes: Vec<u8>,
        rows: Vec<SectionRows>,
        cleaned: Vec<CleanedEntry>,
        delete_file: bool,
    },
    Dirty {
        line: usize,
    },
    Refused(Refusal),
    NotFound,
}

#[derive(Debug)]
enum RemoveOut {
    Done {
        rows: Vec<SectionRows>,
        cleaned: Vec<CleanedEntry>,
    },
    Dirty {
        line: usize,
    },
    Refused(Refusal),
    NotFound,
}

/// Compute the remove-form splice plan. Only clean-line entries are
/// removable (§6.2.6): when the nick still has a parseable declaration
/// on a dirty line of a selected section, the whole removal is refused
/// before any splice (`E-STATE-DIRTY`) — a write must not claim
/// success while a declaration it cannot touch remains, and must not
/// report the nick as absent. A successful removal also runs the
/// §6.3.3 expiry cleanup over every section's clean lines, exactly as
/// the declare form does. A section whose content after the removal
/// and cleanup reduces to its H2 head plus blank lines is deleted
/// whole (§6.4.2, §10.3.5) and reported via `removed-lines` — decided
/// on the residual line classes, never on parsed-entry flags, so
/// pure-garbage lines, dirty entry lines and fences (which create no
/// `Declaration`) always keep the H2 and their bytes. A file reduced
/// to H1 + comment lines + blanks is deleted on the held lock. A
/// missing target changes nothing at all (§6.4.2).
fn plan_remove(
    source: &[u8],
    file: &Path,
    now: Time,
    nick: &str,
    status_filter: Option<&str>,
) -> Result<RemovePlan, Fault> {
    let d = match doc::parse(source.to_vec(), file, now) {
        Ok(d) => d,
        Err(Fault::Diagnostics(ds)) => {
            return Ok(RemovePlan::Refused(Refusal::blocking(None, ds)));
        }
        Err(f) => return Err(f),
    };
    if let Some(r) = blocking_refusal(&d) {
        return Ok(RemovePlan::Refused(r));
    }
    match d.kind {
        Some(doc::Kind::State) => {}
        Some(other) => return Ok(RemovePlan::Refused(Refusal::e_type(other))),
        None => {
            return Ok(RemovePlan::Refused(Refusal::blocking(
                None,
                d.diagnostics.clone(),
            )));
        }
    }

    // The nick's own parseable declaration on a dirty line of a selected
    // section: dirty lines never participate in writes (§6.2.6), so
    // splicing the clean-line entries would still leave that declaration
    // in place. Refuse the entire removal before any splice, pointing at
    // the actual line for manual repair.
    for sec in d.state.iter() {
        if let Some(sf) = status_filter
            && sec.status != sf
        {
            continue;
        }
        if let Some(hit) = sec.entries.iter().find(|e| !e.clean && e.nick == nick) {
            return Ok(RemovePlan::Dirty { line: hit.line });
        }
    }

    let mut splices: Vec<Splice> = Vec::new();
    let mut rows: Vec<SectionRows> = Vec::new();
    let mut cleaned: Vec<CleanedEntry> = Vec::new();

    for sec in d.state.iter() {
        let in_selection = status_filter.is_none_or(|sf| sec.status == sf);
        let has_target = in_selection && sec.entries.iter().any(|e| e.clean && e.nick == nick);
        // §6.3.3 cleanup reaches a section only when some clean entry is
        // expired; otherwise untouched sections keep their bytes (§2.7.2).
        let has_expired_clean = sec.entries.iter().any(|e| e.clean && !e.valid(now));
        if (!in_selection || !has_target) && !has_expired_clean {
            continue;
        }
        let mut survivors: Vec<(String, Time, Time)> = Vec::new();
        for e in sec.entries.iter().filter(|e| e.clean) {
            if !e.valid(now) {
                // §6.4.2: expired selected-target declarations are
                // recorded as cleaned before the removal itself.
                cleaned.push(CleanedEntry {
                    status: sec.status.clone(),
                    nick: e.nick.clone(),
                    start: e.start,
                    until: e.until,
                });
                continue;
            }
            if has_target && e.nick == nick {
                continue; // the removal itself
            }
            survivors.push((e.nick.clone(), e.start, e.until));
        }
        let merged = merge_dupes(survivors);

        let mut line_spans: Vec<Span> = sec
            .clean_lines
            .iter()
            .map(|s| norm_span(source, s))
            .collect();
        line_spans.sort_by_key(|s| s.start);

        // Whole-section deletion (§6.4.2's "becomes empty") only when nothing but
        // the H2 head and blank lines survives the entry splices: lines
        // that create no Declaration — pure-garbage fragments, dirty
        // entry lines, fenced lines — carry no clean-line span, so their
        // presence keeps the section and every unknown byte.
        let section = &d.sections[sec.section];
        let head_end = past_terminator(source, &section.head);
        if has_target
            && merged.is_empty()
            && section_reduces_to_head(source, &line_spans, &section.span, head_end)
        {
            let span = section.span.clone();
            let a = d.line_at(span.start);
            let b = d.line_at(span.end.saturating_sub(1));
            let mut replacement = Vec::new();
            if span.end < source.len() && !preceded_by_blank(source, span.start) {
                replacement.push(b'\n');
            }
            splices.push(Splice {
                range: span,
                replacement,
            });
            rows.push(SectionRows {
                status: sec.status.clone(),
                removed_lines: Some((a, b)),
            });
        } else if !line_spans.is_empty() {
            let first = line_spans[0].clone();
            for s in &line_spans[1..] {
                splices.push(Splice {
                    range: s.clone(),
                    replacement: Vec::new(),
                });
            }
            splices.push(Splice {
                range: first,
                replacement: serialize_line(&merged),
            });
            if has_target {
                rows.push(SectionRows {
                    status: sec.status.clone(),
                    removed_lines: None,
                });
            }
            // A degenerate parse (clean entries without clean-line
            // spans) claims nothing: the bytes stay, no `removed` row.
        }
    }

    if rows.is_empty() {
        // No selected section held a removable clean entry: nothing is
        // written, and §6.3.3 cleanup never runs without a removal.
        return Ok(RemovePlan::NotFound);
    }
    let bytes = text::apply(source, splices)?;
    // Parsing already validated the H1. All splices start after it, so
    // its following offset is unchanged and its grammar need not be repeated.
    let delete_file = is_state_skeleton(&bytes[d.preamble.start..]);
    Ok(RemovePlan::Write {
        bytes,
        rows,
        cleaned,
        delete_file,
    })
}

/// Whether the lines of a section span past its H2 head reduce, once the
/// `clean` full-line spans are spliced away, to blank lines only: every
/// non-blank line must be one of the clean lines. Pure-garbage lines,
/// dirty entry lines and fenced lines produce no clean-line span, so
/// they answer `false` and an "emptied" section keeps its H2 and bytes
/// (§2.7.2 — a removal never sweeps unknown hand-written content).
fn section_reduces_to_head(source: &[u8], clean: &[Span], span: &Span, head_end: usize) -> bool {
    let mut i = head_end.min(span.end);
    let mut c = 0;
    while i < span.end {
        let content_end = line_content_end(source, i);
        if !is_blank_content(line_content(source, i, content_end)) {
            while c < clean.len() && clean[c].start <= i && content_end > clean[c].end {
                c += 1;
            }
            if !(c < clean.len() && clean[c].start <= i && content_end <= clean[c].end) {
                return false;
            }
        }
        i = if content_end < source.len() {
            content_end + 1
        } else {
            content_end
        };
    }
    true
}

// ---------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------

/// A write refusal computed under the lock: rendered by the caller as
/// either an E-TYPE reason report or a check-shaped diagnostic report.
#[derive(Debug)]
struct Refusal {
    kind: Option<&'static str>,
    diagnostics: Vec<Diagnostic>,
    e_type: Option<&'static str>,
}

impl Refusal {
    fn blocking(kind: Option<&'static str>, diagnostics: Vec<Diagnostic>) -> Self {
        Refusal {
            kind,
            diagnostics,
            e_type: None,
        }
    }

    fn e_type(actual: doc::Kind) -> Self {
        let s = actual.as_str();
        Refusal {
            kind: Some(s),
            diagnostics: Vec::new(),
            e_type: Some(s),
        }
    }
}

fn blocking_refusal(d: &doc::Document) -> Option<Refusal> {
    if d.blocking().is_empty() {
        return None;
    }
    Some(Refusal::blocking(kind_name(d.kind), d.diagnostics.clone()))
}

fn kind_name(k: Option<doc::Kind>) -> Option<&'static str> {
    k.map(|x| x.as_str())
}

/// Check-shaped refusal (§8.3.2): `file` row, `type` row when the type is
/// determinable, then the diagnostic rows in anchor order, exit 1.
fn refusal_report(
    object: &str,
    file: &Path,
    kind: Option<&'static str>,
    diagnostics: Vec<Diagnostic>,
) -> Report {
    let mut report = Report::new("state", object);
    report.kv("file", vec![context::display_file(file)]);
    if let Some(k) = kind {
        report.kv("type", vec![k.to_string()]);
    }
    let refused = diag::write_refusal(&diagnostics);
    for d in diagnostics {
        report.diagnostic(d);
    }
    if let Some(text) = refused {
        report.kv("refused", vec![text]);
    }
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// E-TYPE reason report: `state.agent-besties.md` holds another type. The
/// renderer's expected-state branch asks to rename it and retry `{argv}`.
fn e_type_report(dctx: &diag::Context, object: &str, actual: &'static str) -> Report {
    reason_report(
        dctx,
        "state",
        object,
        "E-TYPE",
        vec![("actual", actual.into()), ("expected", "state".into())],
    )
}

fn state_notfound(dctx: &diag::Context, object: &str, nick: &str, status: Option<&str>) -> Report {
    let mut params = Vec::with_capacity(2 + usize::from(status.is_some()));
    params.push(("nick", nick.into()));
    params.push(("dir", object.into()));
    if let Some(status) = status {
        params.push(("status", status.into()));
    }
    reason_report(dctx, "state", object, "E-STATE-NOTFOUND", params)
}

/// DIR validation for both forms (§6.4.1): must exist and be a directory;
/// the CLI never creates directories for state.
fn check_dir(ctx: &Ctx, dir: &Path) -> Result<Option<Report>, Fault> {
    let dctx = context::diag_context_for_dir(ctx, "state", dir);
    let object = context::object_of_dir(dir);
    match std::fs::metadata(dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Some(reason_report(
            &dctx,
            "state",
            &object,
            "E-DIR-MISSING",
            vec![
                ("dir", object.clone().into()),
                (
                    "parent_arg",
                    context::parent_dir(dir).as_os_str().to_os_string(),
                ),
            ],
        ))),
        Err(e) => Err(Fault::io("stat", dir.to_path_buf(), e)),
        Ok(md) if !md.is_dir() => Ok(Some(reason_report(
            &dctx,
            "state",
            &object,
            "E-NOT-DIR",
            vec![
                ("path", object.clone().into()),
                ("cmd", "state".into()),
                (
                    "parent_arg",
                    context::parent_dir(dir).as_os_str().to_os_string(),
                ),
            ],
        ))),
        Ok(_) => Ok(None),
    }
}

fn default_walk_options() -> fsx::walk::Options {
    fsx::walk::Options {
        include: Vec::new(),
        exclude: Vec::new(),
        no_ignore: false,
    }
}

/// Fix row after a committed declaration whose conflict scan stopped on an
/// error that is not an ignore/config source (the `scan` row shows it):
/// resolve it, then run only the read-only conflict check on `dir_arg`
/// (§8.3.4-quoted DIR), or the by-hand form when DIR cannot be quoted.
fn scan_stopped_fix(dir_arg: Option<&str>) -> String {
    match dir_arg {
        Some(dir) => diag::FIX_STATE_SCAN_STOPPED.replace("{dir_arg}", dir),
        None => diag::FIX_STATE_SCAN_STOPPED_MANUAL.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Serialization (§6.2.6)
// ---------------------------------------------------------------------------

fn entry_text(nick: &str, start: Time, until: Time) -> String {
    format!("{} {}..{}", nick, time::format(start), time::format(until))
}

/// One merged clean line for a section: entries joined with `, `, file
/// order, LF-terminated. Empty input serializes to no line at all.
fn serialize_line(entries: &[(String, Time, Time)]) -> Vec<u8> {
    if entries.is_empty() {
        return Vec::new();
    }
    let joined = entries
        .iter()
        .map(|(n, s, u)| entry_text(n, *s, *u))
        .collect::<Vec<_>>()
        .join(", ");
    let mut out = joined.into_bytes();
    out.push(b'\n');
    out
}

/// Merge duplicate nicks keeping the latest `until`, first-occurrence
/// order (§6.2.4, §6.4.1).
fn merge_dupes(entries: Vec<(String, Time, Time)>) -> Vec<(String, Time, Time)> {
    let mut out: Vec<(String, Time, Time)> = Vec::new();
    let mut at: HashMap<String, usize> = HashMap::new();
    for (nick, start, until) in entries {
        if let Some(&idx) = at.get(&nick) {
            let slot = &mut out[idx];
            if until > slot.2 {
                slot.1 = start;
                slot.2 = until;
            }
        } else {
            at.insert(nick.clone(), out.len());
            out.push((nick, start, until));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Byte geometry helpers (terminator-safe, CRLF-aware)
// ---------------------------------------------------------------------------

/// End of the line starting at `from`: index of its `\n` or EOF.
fn line_content_end(source: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < source.len() && source[i] != b'\n' {
        i += 1;
    }
    i
}

/// Bytes of the line starting at `start` up to `content_end` (`\n` or EOF),
/// with a CRLF terminator's CR removed. A standalone CR is left in place
/// as unknown content (§2.1.2).
fn line_content(source: &[u8], start: usize, content_end: usize) -> &[u8] {
    let c = &source[start..content_end];
    if c.ends_with(b"\r") && content_end < source.len() && source[content_end] == b'\n' {
        &c[..c.len() - 1]
    } else {
        c
    }
}

/// Blank line content (§1): empty or only U+0020 and tabs. A standalone
/// CR is not a terminator and is not blank.
fn is_blank_content(c: &[u8]) -> bool {
    c.iter().all(|&b| b == b' ' || b == b'\t')
}

/// Extend a line span to include its terminator when the parser's span
/// excluded it (H2 heads are terminator-excluded; this accepts both
/// conventions for clean-line spans too).
fn norm_span(source: &[u8], span: &Span) -> Span {
    let end = span.end.min(source.len());
    if end > span.start && source[end - 1] == b'\n' {
        return span.start..end;
    }
    if end < source.len() {
        if source[end] == b'\n' {
            return span.start..end + 1;
        }
        if source[end] == b'\r' && end + 1 < source.len() && source[end + 1] == b'\n' {
            return span.start..end + 2;
        }
    }
    span.start..end
}

/// Byte offset just past a line span's terminator.
fn past_terminator(source: &[u8], span: &Span) -> usize {
    norm_span(source, span).end
}

/// Insertion point after an H2 with no clean lines (§6.4.1): after the
/// blank line following the H2, or right after the H2 terminator with a
/// blank line added when there is none (or nothing follows at all).
/// Only the chosen section is inspected; the next H2 is outside its bounds.
/// Unterminated opaque body text stays after the insertion boundary. In
/// particular, inserting before a lone CR never turns it into a CRLF or
/// activates a previously opaque heading.
fn h2_insert_point(source: &[u8], section: &doc::Section) -> (usize, &'static str) {
    let source = &source[..section.span.end];
    let after = past_terminator(source, &section.head);
    if after >= source.len() {
        let gap = if source.ends_with(b"\n") {
            "\n"
        } else {
            "\n\n"
        };
        return (after, gap);
    }
    let content_end = line_content_end(source, after);
    if is_blank_content(line_content(source, after, content_end)) {
        if content_end < source.len() {
            (content_end + 1, "")
        } else {
            (content_end, "\n")
        }
    } else {
        (after, "\n")
    }
}

/// Whether the complete line immediately before `start` is blank (§10.3.5's
/// separation guarantee for whole-section deletion).
fn preceded_by_blank(source: &[u8], start: usize) -> bool {
    let mut i = start;
    if i == 0 {
        return true;
    }
    if source[i - 1] == b'\n' {
        i -= 1;
        if i > 0 && source[i - 1] == b'\r' {
            i -= 1;
        }
    }
    let mut ls = i;
    while ls > 0 && source[ls - 1] != b'\n' {
        ls -= 1;
    }
    is_blank_content(&source[ls..i])
}

/// Whether only reader comments and blanks remain after the already
/// validated H1 (§6.4.2): the file is deleted on the held lock.
fn is_state_skeleton(bytes: &[u8]) -> bool {
    let mut i = 0usize;
    while i < bytes.len() {
        let content_end = line_content_end(bytes, i);
        let line = line_content(bytes, i, content_end);
        if !is_blank_content(line) && !is_reader_comment(line) {
            return false;
        }
        i = if content_end < bytes.len() {
            content_end + 1
        } else {
            content_end
        };
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{ProfileAction, ProfileParams, StateDeclare, StateRemove};
    use crate::report::Row;
    use std::path::PathBuf;

    const NOW: &str = "2026-09-17T10:00:00Z";

    fn now() -> Time {
        crate::time::parse(NOW).unwrap()
    }

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-state-tests")
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

    fn row_values(report: &Report, key: &str) -> Vec<Vec<String>> {
        report
            .rows
            .iter()
            .filter_map(|r| match r {
                Row::Fields { key: k, values, .. } if k == key => Some(values.clone()),
                _ => None,
            })
            .collect()
    }

    fn plan(src: &str, nick: &str, filter: Option<&str>) -> Result<RemovePlan, Fault> {
        plan_remove(
            src.as_bytes(),
            Path::new("state.agent-besties.md"),
            now(),
            nick,
            filter,
        )
    }

    fn written(plan: RemovePlan) -> (Vec<u8>, Vec<SectionRows>, Vec<CleanedEntry>) {
        match plan {
            RemovePlan::Write {
                bytes,
                rows,
                cleaned,
                delete_file,
            } => {
                assert!(!delete_file);
                (bytes, rows, cleaned)
            }
            other => panic!("expected a write plan, got {other:?}"),
        }
    }

    fn cleaned_pairs(cleaned: &[CleanedEntry]) -> Vec<(&str, &str)> {
        cleaned
            .iter()
            .map(|c| (c.status.as_str(), c.nick.as_str()))
            .collect()
    }

    #[test]
    fn declaration_before_unterminated_next_heading_locks_the_requested_status() {
        let dir = scratch("section-boundary");
        let file = dir.join("state.agent-besties.md");
        std::fs::write(&file, b"# .state\n\n## locked\n## observing").unwrap();
        let mut ctx = context::test_ctx(now());
        let report = run(
            &mut ctx,
            &StateParams::Declare(StateDeclare {
                dir: dir.clone(),
                nick: "alice".into(),
                status: "locked".into(),
                for_secs: Some(7200),
            }),
        )
        .unwrap();
        assert_eq!(report.exit, 0, "{report:?}");
        assert_eq!(
            std::fs::read(&file).unwrap(),
            b"# .state\n\n## locked\n\nalice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z\n## observing"
        );

        let target = dir.join("probe.agent-besties.md");
        let blocked = crate::ops::profile::run(
            &mut ctx,
            &ProfileParams {
                path: target.clone(),
                as_nick: Some("bob".into()),
                section: Some("role".into()),
                action: ProfileAction::SetBody("dev".into()),
            },
        )
        .unwrap();
        assert_eq!(blocked.status, Status::Error, "{blocked:?}");
        assert_eq!(blocked.exit, 3, "{blocked:?}");
        assert!(!target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn removing_last_entry_preserves_cr_tainted_reader_comment() {
        let dir = scratch("cr-comment");
        let file = dir.join("state.agent-besties.md");
        std::fs::write(
            &file,
            concat!(
                "# .state\n\n> preserve\rthis\n\n## locked\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            ),
        )
        .unwrap();
        let report = run(
            &mut context::test_ctx(now()),
            &StateParams::Remove(StateRemove {
                dir: dir.clone(),
                nick: "alice".into(),
                as_nick: None,
                status: None,
            }),
        )
        .unwrap();
        assert_eq!(report.exit, 0, "{report:?}");
        assert_eq!(
            std::fs::read(&file).unwrap(),
            b"# .state\n\n> preserve\rthis\n\n"
        );
        assert!(row_values(&report, "deleted").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn removing_last_entry_deletes_skeleton_with_unicode_h1_padding() {
        let dir = scratch("h1-padding");
        let file = dir.join("state.agent-besties.md");
        std::fs::write(
            &file,
            concat!(
                "# .state\u{a0}\n\n## observing\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            ),
        )
        .unwrap();
        let report = run(
            &mut context::test_ctx(now()),
            &StateParams::Remove(StateRemove {
                dir: dir.clone(),
                nick: "alice".into(),
                as_nick: None,
                status: None,
            }),
        )
        .unwrap();
        assert_eq!(report.exit, 0, "{report:?}");
        assert!(!file.exists());
        assert_eq!(row_values(&report, "deleted").len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_write_reports_sanitize_retained_status_titles_only_for_display() {
        let dir = scratch("status-display");
        let file = dir.join("state.agent-besties.md");
        std::fs::write(
            &file,
            concat!(
                "# .state\n\n## custom\x1b[2J\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, ",
                "bob 2026-09-17T08:00:00Z..2026-09-17T09:30:00Z\n",
                "keep this note\n",
            ),
        )
        .unwrap();
        let declared = run(
            &mut context::test_ctx(now()),
            &StateParams::Declare(StateDeclare {
                dir: dir.clone(),
                nick: "carol".into(),
                status: "observing".into(),
                for_secs: Some(7200),
            }),
        )
        .unwrap();
        assert_eq!(declared.exit, 0, "{declared:?}");
        assert_eq!(
            row_values(&declared, "cleaned"),
            vec![vec![
                "custom\u{fffd}[2J".to_string(),
                "bob".to_string(),
                "2026-09-17T08:00:00Z..2026-09-17T09:30:00Z".to_string(),
            ]]
        );
        assert!(
            std::fs::read(&file).unwrap().starts_with(
                concat!(
                    "# .state\n\n## custom\x1b[2J\n",
                    "alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z\n",
                    "keep this note\n",
                )
                .as_bytes()
            )
        );

        let removed = run(
            &mut context::test_ctx(time::parse("2026-09-17T11:00:00Z").unwrap()),
            &StateParams::Remove(StateRemove {
                dir: dir.clone(),
                nick: "alice".into(),
                as_nick: Some("carol".into()),
                status: None,
            }),
        )
        .unwrap();
        assert_eq!(removed.exit, 0, "{removed:?}");
        assert_eq!(
            row_values(&removed, "cleaned"),
            vec![vec![
                "custom\u{fffd}[2J".to_string(),
                "alice".to_string(),
                "2026-09-17T09:00:00Z..2026-09-17T11:00:00Z".to_string(),
            ]]
        );
        assert_eq!(
            row_values(&removed, "removed"),
            vec![vec!["custom\u{fffd}[2J".to_string(), "alice".to_string()]]
        );
        assert_eq!(
            row_values(&removed, "removed-other"),
            vec![vec!["alice".to_string()]]
        );
        assert!(
            std::fs::read(&file)
                .unwrap()
                .starts_with(b"# .state\n\n## custom\x1b[2J\nkeep this note\n")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn declare_keeps_the_write_when_post_write_conflict_scan_hits_a_fatal_ignore() {
        let dir = scratch("declare-scan-fatal");
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
        let mut ctx = context::test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let report = run(
            &mut ctx,
            &StateParams::Declare(StateDeclare {
                dir: dir.clone(),
                nick: "bob".into(),
                status: "observing".into(),
                for_secs: None,
            }),
        )
        .expect("committed write must not fail as if unwritten");
        assert_eq!(report.exit, 0, "{report:?}");
        assert_eq!(report.status, Status::Warn);
        let set = row_values(&report, "set");
        assert_eq!(set.len(), 1);
        assert_eq!(set[0][0], "observing");
        assert_eq!(set[0][1], "bob");
        assert!(
            row_values(&report, "skipped")
                .iter()
                .all(|row| { row.get(1).map(String::as_str) != Some("unreadable") }),
            "must not label the readable root unreadable: {report:?}"
        );
        let scans = row_values(&report, "scan");
        assert_eq!(scans.len(), 1, "{report:?}");
        assert_eq!(scans[0][0], context::display_file(&dir.join(".ignore")));
        assert_ne!(scans[0][1], "unreadable");
        assert!(!scans[0][1].is_empty());
        // The fix repairs the named line and reruns only the conflict
        // check: the declaration must not be written a second time.
        let fixes = row_values(&report, "fix");
        assert_eq!(fixes.len(), 1, "{report:?}");
        let (advice, command) = fixes[0][0]
            .rsplit_once(": ")
            .expect("fix ends in a command");
        assert!(advice.contains("line 1 of"), "{advice}");
        assert!(
            command.starts_with("agent-coexist check-state "),
            "{command}"
        );
        let written = std::fs::read_to_string(dir.join("state.agent-besties.md")).unwrap();
        assert!(
            written.contains("bob 2026-09-25T12:00:00Z..2026-09-25T20:00:00Z"),
            "committed observing line missing: {written:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// With the current directory deleted, the post-write conflict scan
    /// stops on an OS error (global ignore rules need the cwd). The
    /// declaration stays written and the fix reruns only the read-only
    /// conflict check on DIR, never the declaration.
    #[cfg(unix)]
    #[test]
    fn declare_keeps_the_write_when_post_write_conflict_scan_hits_an_io_error() {
        const CHILD: &str = "COEXIST_STATE_SCAN_CWD_GONE_CHILD";
        let Some(dir) = std::env::var_os(CHILD).map(PathBuf::from) else {
            let dir = scratch("declare-scan-io");
            let result = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "ops::state::tests::declare_keeps_the_write_when_post_write_conflict_scan_hits_an_io_error",
                ])
                .env(CHILD, &dir)
                .status()
                .expect("spawn isolated cwd child");
            let _ = std::fs::remove_dir_all(&dir);
            assert!(result.success(), "post-write scan I/O child failed");
            return;
        };
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();
        let alice = concat!(
            "# .state\n\n## locked\n",
            "alice 2026-09-25T11:00:00Z..2026-09-25T14:00:00Z\n",
        );
        std::fs::write(sub.join("state.agent-besties.md"), alice).unwrap();
        let gone = dir.join("gone");
        std::fs::create_dir(&gone).unwrap();
        std::env::set_current_dir(&gone).unwrap();
        std::fs::remove_dir(&gone).unwrap();

        let mut ctx = context::test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let report = run(
            &mut ctx,
            &StateParams::Declare(StateDeclare {
                dir: dir.clone(),
                nick: "bob".into(),
                status: "locked".into(),
                for_secs: None,
            }),
        )
        .expect("committed write must not fail as if unwritten");
        assert_eq!(report.exit, 0, "{report:?}");
        assert_eq!(report.status, Status::Warn);
        assert_eq!(row_values(&report, "set").len(), 1, "{report:?}");
        // The scan row keeps the walker's actual cause against DIR.
        let scans = row_values(&report, "scan");
        assert_eq!(scans.len(), 1, "{report:?}");
        assert_eq!(scans[0][0], context::display_file(&dir));
        assert_ne!(scans[0][1], "failed");
        // The fix ends in the read-only conflict check on DIR.
        let fixes = row_values(&report, "fix");
        assert_eq!(fixes.len(), 1, "{report:?}");
        let (_, command) = fixes[0][0]
            .rsplit_once(": ")
            .expect("fix ends in a command");
        assert_eq!(
            command,
            format!(
                "agent-coexist check-state {}",
                diag::path_operand(dir.as_os_str()).unwrap()
            )
        );
        let written = std::fs::read_to_string(dir.join("state.agent-besties.md")).unwrap();
        assert!(
            written.contains("bob 2026-09-25T12:00:00Z..2026-09-25T14:00:00Z"),
            "committed locked line missing: {written:?}"
        );
        assert_eq!(
            std::fs::read_to_string(sub.join("state.agent-besties.md")).unwrap(),
            alice
        );
    }

    // -- E-STATE-DIRTY: honest refusal before any splice --

    #[test]
    fn remove_refuses_when_the_targets_only_declaration_is_dirty() {
        // alice shares her line with an unparseable fragment: the line is
        // dirty (§6.2.6) and no clean entry exists. The refusal points
        // at the actual line instead of claiming the nick is absent.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, broken fragment\n",
        );
        match plan(src, "alice", None).unwrap() {
            RemovePlan::Dirty { line } => assert_eq!(line, 5),
            other => panic!("expected a dirty refusal, got {other:?}"),
        }
    }

    #[test]
    fn remove_refuses_whole_command_when_clean_and_dirty_declarations_mix() {
        // alice is on a clean line AND on a dirty line: splicing the
        // clean one would report `removed` while her dirty declaration
        // stays. The entire removal refuses before any splice.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            "alice 2026-09-17T08:00:00Z..2026-09-17T13:00:00Z, junk\n",
        );
        match plan(src, "alice", None).unwrap() {
            RemovePlan::Dirty { line } => assert_eq!(line, 6),
            other => panic!("expected a dirty refusal, got {other:?}"),
        }
    }

    #[test]
    fn remove_dirty_declaration_outside_the_status_selection_does_not_refuse() {
        // carol's dirty declaration lives under `observing`; the removal
        // targets `locked` only. It proceeds, and the untouched observing
        // bytes — dirty line included — survive verbatim.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            "\n",
            "## observing\n",
            "\n",
            "carol 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z, junk\n",
        );
        let (bytes, rows, cleaned) = written(plan(src, "alice", Some("locked")).unwrap());
        assert!(cleaned.is_empty());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "locked");
        // locked reduces to its head: whole-section delete with
        // removed-lines (H2 at L3 through the blank before the next H2).
        assert_eq!(rows[0].removed_lines, Some((3, 6)));
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            concat!(
                "# .state\n",
                "\n",
                "## observing\n",
                "\n",
                "carol 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z, junk\n",
            )
        );
    }

    // -- expiry cleanup on success, declare-consistent (§6.3.3) --

    #[test]
    fn remove_cleans_expired_clean_entries_across_sections() {
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, bob 2026-09-17T08:00:00Z..2026-09-17T09:30:00Z\n",
            "\n",
            "## observing\n",
            "\n",
            "carol 2026-09-17T07:00:00Z..2026-09-17T08:00:00Z\n",
        );
        let (bytes, rows, cleaned) = written(plan(src, "alice", None).unwrap());
        // alice removed; bob and carol's expired clean entries cleaned;
        // locked (emptied by removal + cleanup) goes whole, observing
        // keeps its H2 (cleanup geometry, not a removal).
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "locked");
        assert!(rows[0].removed_lines.is_some());
        assert_eq!(
            cleaned_pairs(&cleaned),
            vec![("locked", "bob"), ("observing", "carol")]
        );
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            concat!("# .state\n", "\n", "## observing\n", "\n")
        );
    }

    #[test]
    fn remove_missing_target_writes_nothing_despite_expired_entries() {
        // bob's expired entry must not be swept by a removal that has no
        // target: a missing nick refuses without any cleanup write.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "bob 2026-09-17T08:00:00Z..2026-09-17T09:30:00Z\n",
        );
        match plan(src, "alice", None).unwrap() {
            RemovePlan::NotFound => {}
            other => panic!("expected not-found, got {other:?}"),
        }
    }

    #[test]
    fn remove_deletes_section_and_skeleton_file_when_only_entries_remain() {
        let src = concat!(
            "# .state\n",
            "\n",
            "> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, bob 2026-09-17T08:00:00Z..2026-09-17T09:30:00Z\n",
        );
        match plan(src, "alice", None).unwrap() {
            RemovePlan::Write {
                bytes,
                rows,
                cleaned,
                delete_file,
            } => {
                assert!(delete_file); // H1 + comment + blanks only
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].removed_lines, Some((5, 7)));
                assert_eq!(cleaned_pairs(&cleaned), vec![("locked", "bob")]);
                assert_eq!(
                    bytes,
                    b"# .state\n\n> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.\n\n"
                );
            }
            other => panic!("expected a write plan, got {other:?}"),
        }
    }

    // -- unknown hand-written content survives a removal --

    #[test]
    fn remove_keeps_unknown_text_and_fences_when_last_clean_nick_goes() {
        // Pure-garbage lines and fences create no Declaration, so an
        // entries-based emptiness test never saw them; removing alice
        // must keep the H2 and every unknown byte, not delete the
        // section span under them.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            "hand-written note: do not remove\n",
            "```\n",
            "fenced preserved text\n",
            "```\n",
        );
        let (bytes, rows, cleaned) = written(plan(src, "alice", None).unwrap());
        assert!(cleaned.is_empty());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "locked");
        assert_eq!(rows[0].removed_lines, None); // section kept — not emptied
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            concat!(
                "# .state\n",
                "\n",
                "## locked\n",
                "\n",
                "hand-written note: do not remove\n",
                "```\n",
                "fenced preserved text\n",
                "```\n",
            )
        );
    }

    #[test]
    fn remove_keeps_dirty_entry_line_when_other_nicks_stay() {
        // A dirty line with parseable declarations of OTHER nicks keeps
        // the section alive too: only the clean lines are rewritten.
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            "bob 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z, junk\n",
        );
        let (bytes, rows, _cleaned) = written(plan(src, "alice", None).unwrap());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].removed_lines, None);
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            concat!(
                "# .state\n",
                "\n",
                "## locked\n",
                "\n",
                "bob 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z, junk\n",
            )
        );
    }

    #[test]
    fn opaque_cr_heading_cannot_capture_a_declared_lock() {
        let dir = scratch("opaque-cr-heading");
        let file = dir.join("state.agent-besties.md");
        std::fs::write(&file, b"# .state\n\n## locked\n## observing\r").unwrap();
        let mut ctx = context::test_ctx(now());
        let report = run(
            &mut ctx,
            &StateParams::Declare(StateDeclare {
                dir: dir.clone(),
                nick: "alice".into(),
                status: "locked".into(),
                for_secs: Some(7200),
            }),
        )
        .unwrap();
        assert_eq!(report.exit, 0, "{report:?}");
        let stored = std::fs::read(&file).unwrap();
        assert!(stored.ends_with(b"## observing\r"));
        let parsed = doc::parse(stored, &file, now()).unwrap();
        assert_eq!(
            parsed
                .state
                .iter()
                .flat_map(|section| section
                    .entries
                    .iter()
                    .map(move |entry| { (section.status.as_str(), entry.nick.as_str()) }))
                .collect::<Vec<_>>(),
            vec![("locked", "alice")]
        );

        let target = dir.join("probe.agent-besties.md");
        let blocked = crate::ops::profile::run(
            &mut ctx,
            &ProfileParams {
                path: target.clone(),
                as_nick: Some("bob".into()),
                section: Some("role".into()),
                action: ProfileAction::SetBody("dev".into()),
            },
        )
        .unwrap();
        assert_eq!(blocked.status, Status::Error, "{blocked:?}");
        assert_eq!(blocked.exit, 3, "{blocked:?}");
        assert!(!target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_preserves_lone_cr_and_keeps_section() {
        let mut src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
        )
        .as_bytes()
        .to_vec();
        src.push(b'\r');
        match plan_remove(
            &src,
            Path::new("state.agent-besties.md"),
            now(),
            "alice",
            None,
        )
        .unwrap()
        {
            RemovePlan::Write {
                bytes,
                rows,
                cleaned,
                delete_file,
            } => {
                assert!(!delete_file);
                assert!(cleaned.is_empty());
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].status, "locked");
                assert_eq!(rows[0].removed_lines, None);
                assert!(bytes.contains(&b'\r'));
                assert!(!bytes.windows(5).any(|w| w == b"alice"));
                assert!(bytes.windows(10).any(|w| w == b"## locked\n"));
            }
            other => panic!("expected a write plan, got {other:?}"),
        }
    }

    #[test]
    fn remove_reports_cleaned_for_expired_selected_target() {
        let src = concat!(
            "# .state\n",
            "\n",
            "## locked\n",
            "\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T09:30:00Z\n",
        );
        match plan(src, "alice", None).unwrap() {
            RemovePlan::Write {
                rows,
                cleaned,
                delete_file,
                ..
            } => {
                assert!(delete_file);
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].status, "locked");
                assert_eq!(cleaned.len(), 1);
                assert_eq!(cleaned[0].status, "locked");
                assert_eq!(cleaned[0].nick, "alice");
                assert_eq!(time::format(cleaned[0].start), "2026-09-17T09:00:00Z");
                assert_eq!(time::format(cleaned[0].until), "2026-09-17T09:30:00Z");
            }
            other => panic!("expected a write plan, got {other:?}"),
        }
    }
}
