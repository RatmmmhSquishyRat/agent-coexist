//! The `state` command (spec §6.4 declare / remove forms, §8.2 state).
//!
//! Both forms go through [`fsx::mutate`] with `FASTFAIL_GRACE_MS` and
//! `text::apply` splices only — every untouched byte is preserved. The
//! declare form computes conflicts strictly after the write, flush and lock
//! release (§6.4.1); state writes are never functional-lock blocked
//! (§6.4.3). Business failures come back as [`Report`]s; only usage-class
//! faults (from [`time::until`]) and OS-level I/O faults propagate.

use std::path::Path;

use crate::consts;
use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx;
use crate::names;
use crate::report::{Report, Status};
use crate::text::{self, Span, Splice};
use crate::time;
use crate::Time;

use super::context::{self, Ctx};
use super::{reason_report, shared, StateParams};

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
    if let Some(rep) = check_dir(ctx, &p.dir)? {
        return Ok(rep);
    }
    let dctx = context::diag_context_for_dir(ctx, "state", &p.dir);
    let object = context::object_of_dir(&p.dir);

    let for_secs = p.for_secs.unwrap_or(if p.status == "locked" {
        consts::DEFAULT_FOR_LOCKED
    } else {
        consts::DEFAULT_FOR_OTHER
    });
    let now = ctx.now;
    let until = time::until(now, for_secs)?;

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
        |bytes: Vec<u8>, _initial: bool| {
            let plan = plan_declare(&bytes, &file_for_parse, now, &nick, &status, until)?;
            Ok(match plan {
                DeclarePlan::Write {
                    bytes: new_bytes,
                    cleaned,
                } => (
                    fsx::Mutation::Write(new_bytes),
                    DeclareOut::Written { cleaned },
                ),
                DeclarePlan::Refused(r) => (fsx::Mutation::Unchanged, DeclareOut::Refused(r)),
            })
        },
    );

    let result = match outcome {
        Ok(r) => r,
        Err(Fault::Reason {
            id: "E-FILE-MISSING",
            mut params,
        }) => {
            // Should not happen with create: true; render honestly.
            if !params.iter().any(|(k, _)| *k == "dir") {
                params.push(("dir", parent_display(&p.dir)));
            }
            return Ok(reason_report(
                &dctx,
                "state",
                &object,
                "E-FILE-MISSING",
                params,
            ));
        }
        Err(Fault::Reason { id, params }) => {
            return Ok(reason_report(&dctx, "state", &object, id, params));
        }
        Err(Fault::Diagnostics(ds)) => return Ok(refusal_report(&object, &target, None, ds)),
        Err(f) => return Err(f),
    };

    let mut report = Report::new("state", object.clone());
    match result.value {
        DeclareOut::Refused(r) => {
            if let Some(actual) = r.e_type {
                return Ok(e_type_report(&dctx, &object, actual));
            }
            return Ok(refusal_report(&object, &target, r.kind, r.diagnostics));
        }
        DeclareOut::Written { cleaned } => {
            if result.created {
                report.kv("created", vec![context::display_file(&target)]);
            }
            report.kv(
                "set",
                vec![
                    p.status.clone(),
                    p.nick.clone(),
                    format!("{}..{}", time::format(now), time::format(until)),
                ],
            );
            for c in cleaned {
                report.kv(
                    "cleaned",
                    vec![
                        c.status,
                        c.nick,
                        format!("{}..{}", time::format(c.start), time::format(c.until)),
                    ],
                );
            }
        }
    }

    // Conflict scan strictly after write + flush + lock release (§6.4.1);
    // state writes are never functional-lock blocked (§6.4.3).
    let scope = shared::collect(ctx, &p.dir, &default_walk_options());
    for dir in &scope.canonicalize_failed {
        report.kv("canonicalize", vec![dir.clone(), "failed".to_string()]);
        report.warn();
    }
    let conflicts = shared::conflicts_with(&scope, now, &p.nick);
    let mut has_conflict = false;
    for c in &conflicts {
        report.kv(
            "conflict",
            vec![
                c.status.clone(),
                c.nick.clone(),
                format!("{}..{}", time::format(c.start), time::format(c.until)),
                c.dir_display.clone(),
            ],
        );
        has_conflict = true;
    }
    if has_conflict {
        report.warn();
    }
    let mut tail: Vec<(String, &'static str, String)> = Vec::new();
    for (dir, issue) in &scope.issues {
        match issue {
            shared::ScanIssue::Busy => {
                tail.push((dir.clone(), "skipped", "busy".to_string()));
            }
            shared::ScanIssue::Blocking(id) => {
                tail.push((dir.clone(), "skipped", (*id).to_string()));
            }
            shared::ScanIssue::Partial { id, line } => {
                tail.push((dir.clone(), "partial", format!("{id} L{line}")));
            }
        }
    }
    for dir in &scope.unreadable_dirs {
        tail.push((dir.clone(), "skipped", "unreadable".to_string()));
    }
    tail.sort_by(|a, b| a.0.cmp(&b.0));
    tail.dedup_by(|a, b| *a == *b);
    if !tail.is_empty() {
        for (dir, key, detail) in tail {
            report.kv(key, vec![dir, detail]);
        }
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
    Written { cleaned: Vec<CleanedEntry> },
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
            )))
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
                let h2 = &d.sections[sec.section].head;
                let after = past_terminator(source, h2);
                let (at, prefix) = h2_insert_point(source, after);
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
        None => return Ok(state_notfound(&dctx, &object, &p.nick)),
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
        |bytes: Vec<u8>, _initial: bool| {
            let plan = plan_remove(
                &bytes,
                &file_for_parse,
                now,
                &nick,
                status_filter.as_deref(),
            )?;
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
            return Ok(state_notfound(&dctx, &object, &p.nick));
        }
        Err(Fault::Reason { id, params }) => {
            return Ok(reason_report(&dctx, "state", &object, id, params));
        }
        Err(Fault::Diagnostics(ds)) => return Ok(refusal_report(&object, &target, None, ds)),
        Err(f) => return Err(f),
    };

    let mut report = Report::new("state", object.clone());
    match result.value {
        RemoveOut::NotFound => return Ok(state_notfound(&dctx, &object, &p.nick)),
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
                    ("nick", p.nick.clone()),
                    ("dir", object.clone()),
                    ("line", line.to_string()),
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
                        c.status,
                        c.nick,
                        format!("{}..{}", time::format(c.start), time::format(c.until)),
                    ],
                );
            }
            for row in rows {
                report.kv("removed", vec![row.status, p.nick.clone()]);
                if let Some((a, b)) = row.removed_lines {
                    report.kv("removed-lines", vec![format!("L{a}..L{b}")]);
                }
            }
            if result.deleted {
                report.kv("deleted", vec![context::display_file(&target)]);
            }
        }
    }
    if let Some(as_nick) = &p.as_nick {
        if as_nick != &p.nick {
            report.kv("removed-other", vec![p.nick.clone()]);
            report.warn();
        }
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
            )))
        }
    }

    // The nick's own parseable declaration on a dirty line of a selected
    // section: dirty lines never participate in writes (§6.2.6), so
    // splicing the clean-line entries would still leave that declaration
    // in place. Refuse the entire removal before any splice, pointing at
    // the actual line for manual repair.
    for sec in d.state.iter() {
        if let Some(sf) = status_filter {
            if sec.status != sf {
                continue;
            }
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
            if has_target && e.nick == nick {
                continue; // the removal itself
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
        let merged = merge_dupes(survivors);

        let mut line_spans: Vec<Span> = sec
            .clean_lines
            .iter()
            .map(|s| norm_span(source, s))
            .collect();
        line_spans.sort_by_key(|s| s.start);

        // Whole-section deletion (§6.4.2's "变空") only when nothing but
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
    let delete_file = is_state_skeleton(&bytes);
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
    while i < span.end {
        let content_end = line_content_end(source, i);
        if !is_blank_content(&source[i..content_end])
            && !clean.iter().any(|s| s.start <= i && content_end <= s.end)
        {
            return false;
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
    for d in diagnostics {
        report.diagnostic(d);
    }
    report.status = Status::Error;
    report.exit = 1;
    report
}

/// E-TYPE reason report. The §9-supplement-1 state fix variant is chosen
/// by the actual type in the shared renderer (A1 repair), not by a `dir`
/// param's presence; the actual type is never `state` here, so the
/// plain fix applies.
fn e_type_report(dctx: &diag::Context, object: &str, actual: &'static str) -> Report {
    reason_report(
        dctx,
        "state",
        object,
        "E-TYPE",
        vec![
            ("actual", actual.to_string()),
            ("expected", "state".to_string()),
            ("cmd_for_actual", cmd_for_kind(actual).to_string()),
        ],
    )
}

fn cmd_for_kind(kind: &str) -> &'static str {
    match kind {
        "post" => "post",
        "brief" => "brief",
        "agent-profile" => "profile",
        _ => "state",
    }
}

fn state_notfound(dctx: &diag::Context, object: &str, nick: &str) -> Report {
    reason_report(
        dctx,
        "state",
        object,
        "E-STATE-NOTFOUND",
        vec![("nick", nick.to_string()), ("dir", object.to_string())],
    )
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
            vec![("dir", object.clone()), ("parent", parent_display(dir))],
        ))),
        Err(e) => Err(Fault::io("stat", dir.to_path_buf(), e)),
        Ok(md) if !md.is_dir() => Ok(Some(reason_report(
            &dctx,
            "state",
            &object,
            "E-NOT-DIR",
            vec![
                ("path", object.clone()),
                ("cmd", "state".to_string()),
                ("parent", parent_display(dir)),
            ],
        ))),
        Ok(_) => Ok(None),
    }
}

fn parent_display(dir: &Path) -> String {
    match dir.parent() {
        Some(p) if !p.as_os_str().is_empty() => names::fold(&fsx::walk::display_path(p)),
        _ => ".".to_string(),
    }
}

fn default_walk_options() -> fsx::walk::Options {
    fsx::walk::Options {
        include: Vec::new(),
        exclude: Vec::new(),
        no_ignore: false,
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
    for (nick, start, until) in entries {
        if let Some(slot) = out.iter_mut().find(|m| m.0 == nick) {
            if until > slot.2 {
                slot.1 = start;
                slot.2 = until;
            }
        } else {
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

/// Blank line content (§1): empty or only U+0020 and tabs; a trailing `\r`
/// of a CRLF terminator does not count.
fn is_blank_content(mut c: &[u8]) -> bool {
    if c.ends_with(b"\r") {
        c = &c[..c.len() - 1];
    }
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
fn h2_insert_point(source: &[u8], after: usize) -> (usize, &'static str) {
    if after >= source.len() {
        return (after, "\n");
    }
    let content_end = line_content_end(source, after);
    if is_blank_content(&source[after..content_end]) {
        let past = if content_end < source.len() {
            content_end + 1
        } else {
            content_end
        };
        (past, "")
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

/// Whether only H1, `> ` comment lines and blanks remain (§6.4.2): the
/// file is deleted on the held lock.
fn is_state_skeleton(bytes: &[u8]) -> bool {
    let mut rest = bytes;
    if rest.starts_with(b"\xEF\xBB\xBF") {
        rest = &rest[3..];
    }
    let mut seen_h1 = false;
    let mut i = 0usize;
    while i < rest.len() {
        let content_end = line_content_end(rest, i);
        let mut line = &rest[i..content_end];
        if line.ends_with(b"\r") {
            line = &line[..line.len() - 1];
        }
        if !is_blank_content(line) {
            if !seen_h1 {
                if trim_blank(line) == b"# .state" {
                    seen_h1 = true;
                } else {
                    return false;
                }
            } else if !line.starts_with(b"> ") {
                return false;
            }
        }
        i = if content_end < rest.len() {
            content_end + 1
        } else {
            content_end
        };
    }
    seen_h1
}

fn trim_blank(line: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = line.len();
    while start < end && (line[start] == b' ' || line[start] == b'\t') {
        start += 1;
    }
    while end > start && (line[end - 1] == b' ' || line[end - 1] == b'\t') {
        end -= 1;
    }
    &line[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2026-09-17T10:00:00Z";

    fn now() -> Time {
        crate::time::parse(NOW).unwrap()
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
}
