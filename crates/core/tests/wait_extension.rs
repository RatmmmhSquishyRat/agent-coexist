//! Extension-only regression tests for the three wait commands.
//!
//! Each test targets a genuinely uncertain mechanism: projection semantics
//! (newly-relevant messages, reply-target resolution), whole-baseline
//! advancement on unqualified changes, lifecycle transitions (creation,
//! deletion, zero-byte, over-limit, duplicate identities), and cancellation. Sync points
//! are the ready callback plus whole polling cycles derived from
//! `WAIT_POLL_MS` — no fixed-sleep readiness guessing. The base eleven
//! commands have their own suites; these tests are never a base-completion
//! gate.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use coexist_core::error::Fault;
use coexist_core::report::{Report, Row, Status};
use coexist_core::wait::{run, Cancellation, Filter, WAIT_POLL_MS};

/// Long enough for at least one full poll cycle to observe (or not observe)
/// a change; derived from the poll interval, not an arbitrary sleep.
const ONE_CYCLE: u64 = 2 * WAIT_POLL_MS + 500;

struct Waiting {
    result: mpsc::Receiver<Result<Report, Fault>>,
    ready: mpsc::Receiver<()>,
    cancel: Arc<Cancellation>,
}

impl Drop for Waiting {
    fn drop(&mut self) {
        // Cancel and then wait for the worker's terminal result (or its
        // channel disconnection) so no wait thread outlives the test.
        self.cancel.cancel();
        let _ = self.result.recv_timeout(Duration::from_secs(10));
    }
}

fn spawn(path: &Path, filter: Filter) -> Waiting {
    let (result_tx, result) = mpsc::channel();
    let (ready_tx, ready) = mpsc::channel();
    let cancel = Arc::new(Cancellation::new());
    let thread_cancel = Arc::clone(&cancel);
    let thread_path: PathBuf = path.to_path_buf();
    std::thread::spawn(move || {
        let _ = result_tx.send(run(&thread_path, filter, &thread_cancel, || {
            let _ = ready_tx.send(());
        }));
    });
    Waiting {
        result,
        ready,
        cancel,
    }
}

impl Waiting {
    fn wait_ready(&self) {
        assert!(
            self.ready.recv_timeout(Duration::from_secs(10)).is_ok(),
            "ready signal never fired"
        );
    }

    fn assert_not_ready(&self) {
        assert!(
            self.ready.recv_timeout(Duration::from_millis(300)).is_err(),
            "ready fired although the baseline was invalid"
        );
    }

    /// `None` when no result arrived within one poll cycle (still waiting).
    /// A disconnected channel means the worker died; that is never
    /// equivalent to "still waiting".
    fn settle(&self) -> Option<Result<Report, Fault>> {
        match self.result.recv_timeout(Duration::from_millis(ONE_CYCLE)) {
            Ok(result) => Some(result),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("wait worker exited without producing a result")
            }
        }
    }

    fn finish(&self) -> Result<Report, Fault> {
        match self.result.recv_timeout(Duration::from_secs(10)) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                panic!("wait did not finish in time")
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("wait worker exited without producing a result")
            }
        }
    }

    fn expect_hit(&self, context: &str) -> Report {
        match self.finish() {
            Ok(report) if report.exit == 0 => report,
            Ok(report) => panic!(
                "expected a hit ({}), got error report exit {}:\n{}",
                context,
                report.exit,
                row_text(&report)
            ),
            Err(fault) => panic!(
                "expected a hit ({}), got: {}",
                context,
                fault_summary(&fault)
            ),
        }
    }
}

fn fault_summary(fault: &Fault) -> String {
    match fault {
        Fault::Usage(message) => format!("usage: {}", message),
        Fault::Diagnostics(diags) => format!(
            "diagnostics: {}",
            diags
                .iter()
                .map(|diagnostic| diagnostic.id)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Fault::Reason { id, .. } => format!("reason {}", id),
        Fault::Io { action, path, .. } => format!("io {}: {}", action, path.display()),
        Fault::Cancelled => "cancelled".to_string(),
    }
}

fn doc(lines: &[&str]) -> String {
    let mut text = String::new();
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    text
}

/// Whether an I/O error is real lock contention with the sampling reader
/// (Windows sharing/lock violation; Unix mandatory-lock EACCES/EAGAIN per
/// spec §10.2.5). Everything else is a genuine failure and fails the test.
#[cfg(windows)]
fn is_lock_contention(error: &std::io::Error) -> bool {
    matches!(error.raw_os_error(), Some(32) | Some(33))
}

#[cfg(unix)]
fn is_lock_contention(error: &std::io::Error) -> bool {
    matches!(
        error.raw_os_error(),
        Some(libc::EACCES) | Some(libc::EAGAIN)
    )
}

#[cfg(not(any(windows, unix)))]
fn is_lock_contention(_error: &std::io::Error) -> bool {
    false
}

/// Take the exclusive lock BEFORE truncating, so a concurrent sampling read
/// never observes a torn or zero-length intermediate state (the same
/// lock-before-truncate discipline the product's write path uses).
fn locked_write(path: &Path, content: &str) -> std::io::Result<()> {
    use std::io::{Seek, SeekFrom, Write};
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.try_lock()?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    file.unlock()?;
    Ok(())
}

fn write_file(path: &Path, content: &str) {
    let mut last: Option<std::io::Error> = None;
    for _ in 0..400 {
        match locked_write(path, content) {
            Ok(()) => return,
            Err(error) if is_lock_contention(&error) => {
                last = Some(error);
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("write to {:?} failed: {}", path, error),
        }
    }
    panic!(
        "could not write {:?} within the lock-contention budget: {:?}",
        path, last
    );
}

/// Grow a regular managed file past the storage cap without holding a 16 MiB
/// buffer. The exclusive lock is taken first so a concurrent sample cannot
/// observe a torn length.
fn locked_write_over_limit(path: &Path) -> std::io::Result<()> {
    use std::io::{Seek, SeekFrom, Write};
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.try_lock()?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(b"# huge.post\n")?;
    file.set_len((coexist_core::consts::MAX_MANAGED_FILE_BYTES + 1) as u64)?;
    file.sync_all()?;
    file.unlock()?;
    Ok(())
}

fn write_over_limit(path: &Path) {
    let mut last: Option<std::io::Error> = None;
    for _ in 0..400 {
        match locked_write_over_limit(path) {
            Ok(()) => return,
            Err(error) if is_lock_contention(&error) => {
                last = Some(error);
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("over-limit write to {:?} failed: {}", path, error),
        }
    }
    panic!(
        "could not write over-limit {:?} within the lock-contention budget: {:?}",
        path, last
    );
}

fn remove_file(path: &Path) {
    let mut last: Option<std::io::Error> = None;
    for _ in 0..400 {
        match std::fs::remove_file(path) {
            Ok(()) => return,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) if is_lock_contention(&error) => {
                last = Some(error);
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("could not remove {:?}: {}", path, error),
        }
    }
    panic!(
        "could not remove {:?} within the lock-contention budget: {:?}",
        path, last
    );
}

fn row_text(report: &Report) -> String {
    let mut lines = Vec::new();
    for row in &report.rows {
        match row {
            Row::Fields { key, values, .. } => {
                lines.push(format!("{} {}", key, values.join(" ")));
            }
            Row::Text { key, content } => {
                lines.push(format!("{} {}", key, content));
            }
            Row::Diagnostic(_) => {}
        }
    }
    lines.join("\n")
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

fn file_row_mentions(report: &Report, name: &str) -> bool {
    field_values(report, "file")
        .map(|values| values.iter().any(|value| value.contains(name)))
        .unwrap_or(false)
}

fn expect_blocking_report(result: Result<Report, Fault>, context: &str) -> Report {
    match result {
        Ok(report) => {
            assert_eq!(
                report.exit,
                1,
                "{context}: error report must exit 1, rows:\n{}",
                row_text(&report)
            );
            assert_eq!(
                report.status,
                Status::Error,
                "{context}: first line must be error"
            );
            assert!(
                field_values(&report, "file").is_some(),
                "{context}: missing file row:\n{}",
                row_text(&report)
            );
            assert!(
                field_values(&report, "bytes").is_none(),
                "{context}: blocking report must not be a change hit:\n{}",
                row_text(&report)
            );
            report
        }
        Err(fault) => panic!(
            "{context}: expected check-shaped error report, got {}",
            fault_summary(&fault)
        ),
    }
}

fn post_with_first_author(author: &str) -> String {
    doc(&[
        "# standup.post",
        "",
        "## #1 carol (2026-09-17T09:00:00Z)"
            .replace("carol", author)
            .as_str(),
        "",
        "```md",
        "topic",
        "```",
        "",
        "## #2 bob (2026-09-17T09:05:00Z)",
        "",
        "```md",
        "@#1 done",
        "```",
    ])
}

fn plain_post() -> String {
    doc(&[
        "# standup.post",
        "",
        "## #1 alice (2026-09-17T09:00:00Z)",
        "",
        "```md",
        "hello",
        "```",
    ])
}

#[test]
fn post_newly_relevant_via_reply_target_change() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &post_with_first_author("carol"));
    let waiting = spawn(&path, Filter::Post(Some("alice".to_string())));
    waiting.wait_ready();
    // #1's author becomes alice; #2 (body untouched) now replies to alice.
    write_file(&path, &post_with_first_author("alice"));
    let report = waiting.expect_hit("message became relevant via reply target");
    let text = row_text(&report);
    assert!(
        text.contains("update comment #2 relevance"),
        "expected a relevance update for #2, got:\n{}",
        text
    );
    assert!(
        !text.contains("update comment #1 author"),
        "unqualified change to #1 must not be reported:\n{}",
        text
    );
}

#[test]
fn unqualified_deletion_advances_baseline_and_recreation_is_judged_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &plain_post());
    let waiting = spawn(&path, Filter::Post(Some("alice".to_string())));
    waiting.wait_ready();
    // Deletion is unqualified for a post filter: no hit, and the whole
    // baseline advances to Absent within the cycle.
    remove_file(&path);
    assert!(
        waiting.settle().is_none(),
        "deletion must not hit a post filter"
    );
    // Recreate with byte-identical, unrelated #1 and new relevant #2.
    // Advancing to Absent must produce a file-level creation, not merely
    // #2's created timestamp. The filtered report must still omit #1.
    write_file(
        &path,
        &doc(&[
            "# standup.post",
            "",
            "## #1 alice (2026-09-17T09:00:00Z)",
            "",
            "```md",
            "hello",
            "```",
            "",
            "## #2 bob (2026-09-17T09:05:00Z)",
            "",
            "```md",
            "@alice ping",
            "```",
        ]),
    );
    let report = waiting.expect_hit("recreated post with a relevant message");
    let text = row_text(&report);
    assert!(
        report
            .rows
            .iter()
            .any(|row| { matches!(row, Row::Fields { key, .. } if key == "created") }),
        "only an Absent baseline can report creation:\n{}",
        text
    );
    assert!(
        !text.contains("add comment #1"),
        "unrelated comments stay outside the projection even on recreation:\n{}",
        text
    );
    assert!(
        text.contains("add comment #2"),
        "the new relevant message must hit:\n{}",
        text
    );
    assert!(
        !text.contains("update comment"),
        "comments admitted from Absent must be added, not updated:\n{}",
        text
    );
}

#[test]
fn state_renewal_hits_but_stored_expiry_alone_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.agent-besties.md");
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "alice 2026-09-17T08:00:00Z..2026-09-17T10:00:00Z",
        ]),
    );
    let waiting = spawn(&path, Filter::State(Some("locked".to_string())));
    waiting.wait_ready();
    // The stored declaration is long expired; identical bytes must not hit.
    assert!(
        waiting.settle().is_none(),
        "clock expiry alone must never match"
    );
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "alice 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
        ]),
    );
    let report = waiting.expect_hit("renewal of an expired-but-stored declaration");
    let text = row_text(&report);
    assert!(
        text.contains("update locked alice until"),
        "renewal of an expired-but-stored declaration must hit:\n{}",
        text
    );
}

#[test]
fn state_file_deletion_with_selected_declarations_hits() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.agent-besties.md");
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "alice 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
        ]),
    );
    let waiting = spawn(&path, Filter::State(Some("locked".to_string())));
    waiting.wait_ready();
    remove_file(&path);
    let report = waiting.expect_hit("file deletion removing selected declarations");
    let text = row_text(&report);
    assert!(
        text.contains("removed"),
        "file removal must be reported:\n{}",
        text
    );
    assert!(
        text.contains("remove locked alice"),
        "selected-status declaration removal must hit:\n{}",
        text
    );
}

#[test]
fn wait_hit_and_deletion_report_state_conflict_warnings() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.agent-besties.md");
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "alice 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
        ]),
    );
    let waiting = spawn(&path, Filter::Any);
    waiting.wait_ready();
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "alice 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
            "bob 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
        ]),
    );
    let hit = waiting.expect_hit("locked pair becomes a conflict");
    assert_eq!(hit.status, Status::Warn);
    assert_eq!(hit.exit, 0);
    let hit_lines: Vec<usize> = hit
        .rows
        .iter()
        .filter_map(|row| match row {
            Row::Diagnostic(diagnostic) if diagnostic.id == "W-STATE-CONFLICT" => {
                Some(diagnostic.line)
            }
            _ => None,
        })
        .collect();
    assert!(
        !hit_lines.is_empty() && hit_lines.iter().all(|line| *line > 0),
        "ordinary hit must emit conflict warnings with real anchors, ids={:?} lines={:?}",
        diagnostic_ids(&hit),
        hit_lines
    );

    let waiting = spawn(&path, Filter::Any);
    waiting.wait_ready();
    remove_file(&path);
    let removed = waiting.expect_hit("conflicting state file deleted");
    assert_eq!(removed.status, Status::Warn);
    assert_eq!(removed.exit, 0);
    let removed_lines: Vec<usize> = removed
        .rows
        .iter()
        .filter_map(|row| match row {
            Row::Diagnostic(diagnostic) if diagnostic.id == "W-STATE-CONFLICT" => {
                Some(diagnostic.line)
            }
            _ => None,
        })
        .collect();
    assert!(
        !removed_lines.is_empty() && removed_lines.iter().all(|line| *line > 0),
        "deletion must emit the before snapshot's conflict warnings, ids={:?} lines={:?}",
        diagnostic_ids(&removed),
        removed_lines
    );
}

#[test]
fn state_deletion_without_selected_status_only_advances_to_absent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.agent-besties.md");
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## observing",
            "",
            "eve 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
        ]),
    );
    let waiting = spawn(&path, Filter::State(Some("locked".to_string())));
    waiting.wait_ready();
    remove_file(&path);
    assert!(
        waiting.settle().is_none(),
        "deletion without selected-status declarations must only advance the baseline"
    );
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "bob 2026-09-17T09:00:00Z..2030-01-01T00:00:00Z",
        ]),
    );
    let report = waiting.expect_hit("later selected-status declaration");
    let text = row_text(&report);
    // `created` proves the baseline really reached Absent; a stale Present
    // baseline would report an update-free hit without this row.
    assert!(
        text.contains("created"),
        "only an Absent baseline can report creation:\n{}",
        text
    );
    assert!(
        text.contains("add locked bob"),
        "later selected-status declaration must hit:\n{}",
        text
    );
}

#[test]
fn duplicate_state_nick_fails_instead_of_waiting() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.agent-besties.md");
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "alice 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
        ]),
    );
    let waiting = spawn(&path, Filter::State(Some("locked".to_string())));
    waiting.wait_ready();
    write_file(
        &path,
        &doc(&[
            "# .state",
            "",
            "## locked",
            "",
            "alice 2026-09-17T08:00:00Z..2030-01-01T00:00:00Z",
            "alice 2026-09-17T09:00:00Z..2030-01-02T00:00:00Z",
        ]),
    );
    let report = expect_blocking_report(waiting.finish(), "duplicate identities");
    assert!(
        file_row_mentions(&report, "state.agent-besties.md"),
        "duplicate failure must name the file:\n{}",
        row_text(&report)
    );
    assert_eq!(field_values(&report, "type").expect("type row"), &["state"]);
    assert!(
        diagnostic_ids(&report).contains(&"W-STATE-DUP"),
        "duplicate nick is W-STATE-DUP, got {:?}",
        diagnostic_ids(&report)
    );
}

#[test]
fn zero_byte_present_file_fails_before_ready() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, "");
    let waiting = spawn(&path, Filter::Post(None));
    let report = expect_blocking_report(waiting.finish(), "zero-byte present file");
    assert!(
        file_row_mentions(&report, "standup.agent-besties.md"),
        "empty present file must name the file:\n{}",
        row_text(&report)
    );
    assert!(
        field_values(&report, "type").is_none(),
        "empty file has no kind: {:?}",
        field_values(&report, "type")
    );
    assert!(
        diagnostic_ids(&report).contains(&"E-H1"),
        "empty file is E-H1, got {:?}",
        diagnostic_ids(&report)
    );
    waiting.assert_not_ready();
}

#[test]
fn malformed_update_after_ready_reports_file_type_and_fence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &plain_post());
    let waiting = spawn(&path, Filter::Post(None));
    waiting.wait_ready();
    write_file(
        &path,
        &doc(&[
            "# standup.post",
            "",
            "## #1 alice (2026-09-17T09:00:00Z)",
            "",
            "```md",
            "never closed",
        ]),
    );
    let report = expect_blocking_report(waiting.finish(), "unclosed fence after ready");
    assert!(
        file_row_mentions(&report, "standup.agent-besties.md"),
        "malformed update must name the file:\n{}",
        row_text(&report)
    );
    assert_eq!(field_values(&report, "type").expect("type row"), &["post"]);
    assert!(
        diagnostic_ids(&report).contains(&"E-FENCE"),
        "unclosed fence is E-FENCE, got {:?}",
        diagnostic_ids(&report)
    );
}

#[test]
fn oversize_present_file_fails_before_ready() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("huge.agent-besties.md");
    write_over_limit(&path);
    let waiting = spawn(&path, Filter::Any);
    let report = expect_blocking_report(waiting.finish(), "over-limit present file");
    assert!(
        file_row_mentions(&report, "huge.agent-besties.md"),
        "over-limit present file must name the file:\n{}",
        row_text(&report)
    );
    assert!(
        field_values(&report, "type").is_none(),
        "unparsed over-limit sample must not invent type: {:?}",
        field_values(&report, "type")
    );
    assert!(
        diagnostic_ids(&report).contains(&"E-TOO-LARGE"),
        "over-limit file is E-TOO-LARGE, got {:?}",
        diagnostic_ids(&report)
    );
    waiting.assert_not_ready();
}

#[test]
fn oversize_update_after_ready_reports_file_without_type() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &plain_post());
    let waiting = spawn(&path, Filter::Post(None));
    waiting.wait_ready();
    write_over_limit(&path);
    let report = expect_blocking_report(waiting.finish(), "over-limit after ready");
    assert!(
        file_row_mentions(&report, "standup.agent-besties.md"),
        "over-limit update must name the file:\n{}",
        row_text(&report)
    );
    assert!(
        field_values(&report, "type").is_none(),
        "unparsed over-limit sample must not reuse the admitted type: {:?}",
        field_values(&report, "type")
    );
    assert!(
        diagnostic_ids(&report).contains(&"E-TOO-LARGE"),
        "over-limit update is E-TOO-LARGE, got {:?}",
        diagnostic_ids(&report)
    );
}

#[test]
fn unfiltered_format_only_change_hits_semantic_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &plain_post());
    let waiting = spawn(&path, Filter::Any);
    waiting.wait_ready();
    // Stray text outside any fence: no entity or field changes (a retained
    // W-STRAY warning may accompany the report; it is not an entity row).
    write_file(
        &path,
        &doc(&[
            "# standup.post",
            "",
            "## #1 alice (2026-09-17T09:00:00Z)",
            "",
            "```md",
            "hello",
            "```",
            "",
            "loose text",
        ]),
    );
    let report = waiting.expect_hit("format-only change");
    let text = row_text(&report);
    assert!(
        text.contains("bytes changed"),
        "format-only change must report bytes changed:\n{}",
        text
    );
    assert!(
        text.contains("semantic unchanged"),
        "format-only change has no semantic delta:\n{}",
        text
    );
    assert!(
        !text.contains("add comment"),
        "no entity changes may be fabricated:\n{}",
        text
    );
}

#[test]
fn cancellation_returns_cancelled_fault() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &plain_post());
    let waiting = spawn(&path, Filter::Any);
    waiting.wait_ready();
    waiting.cancel.cancel();
    let started = std::time::Instant::now();
    let result = waiting.finish();
    assert!(
        matches!(result, Err(Fault::Cancelled)),
        "expected Fault::Cancelled, got: {}",
        result
            .err()
            .as_ref()
            .map(fault_summary)
            .unwrap_or_else(|| "hit report".to_string())
    );
    assert!(
        started.elapsed() < Duration::from_millis(1500),
        "cancellation must wake the timed wait promptly"
    );
}

#[test]
fn absent_baseline_never_creates_the_file_and_later_creation_hits() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    let waiting = spawn(&path, Filter::Post(Some("alice".to_string())));
    waiting.wait_ready();
    assert!(
        !path.exists(),
        "an absent baseline must not create the file"
    );
    write_file(
        &path,
        &doc(&[
            "# standup.post",
            "",
            "## #1 bob (2026-09-17T09:00:00Z)",
            "",
            "```md",
            "@alice hi",
            "```",
        ]),
    );
    let report = waiting.expect_hit("creation with a relevant message");
    let text = row_text(&report);
    assert!(
        text.contains("created"),
        "creation must be reported:\n{}",
        text
    );
    assert!(
        text.contains("add comment #1"),
        "created relevant message must hit:\n{}",
        text
    );
}

#[test]
fn specialized_command_with_wrong_type_fails_with_e_type() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.agent-besties.md");
    write_file(
        &path,
        &doc(&["# notes.brief", "", "```md", "description", "```"]),
    );
    let waiting = spawn(&path, Filter::Post(None));
    match waiting.finish() {
        Err(Fault::Reason { id, .. }) => assert_eq!(id, "E-TYPE"),
        Err(other) => panic!("expected E-TYPE failure, got: {}", fault_summary(&other)),
        Ok(report) => panic!(
            "wrong type must fail, not hit; rows:\n{}",
            row_text(&report)
        ),
    }
    waiting.assert_not_ready();
}

#[test]
fn generic_wait_reports_legal_type_switch_without_pairing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mixed.agent-besties.md");
    write_file(&path, &plain_post_with_stem("mixed"));
    let waiting = spawn(&path, Filter::Any);
    waiting.wait_ready();
    let hash: String = "ab".repeat(32);
    write_file(
        &path,
        &doc(&[
            "# mixed.brief",
            "",
            "```md",
            "description",
            "```",
            "",
            "## parser entry point",
            "",
            "- path: src/parser/mod.rs",
            &format!("- hash: {}", hash),
        ]),
    );
    let report = waiting.expect_hit("legal type switch");
    let text = row_text(&report);
    assert!(
        text.contains("update document type"),
        "type switch must be reported as a document field update:\n{}",
        text
    );
    assert!(
        text.contains("remove comment #1"),
        "old-type entities must be reported as removals:\n{}",
        text
    );
    assert!(
        text.contains("add entry"),
        "new-type entities must be reported as additions:\n{}",
        text
    );
    assert!(
        text.contains("update description"),
        "type switch must report the entering brief description:\n{}",
        text
    );
    assert!(
        text.contains("after description"),
        "type switch must report the description body in full:\n{}",
        text
    );
    assert_eq!(
        text.matches("update description").count(),
        1,
        "brief description must stay a single complete field update:\n{}",
        text
    );
}

#[test]
fn generic_wait_reports_brief_description_on_create_and_delete() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.agent-besties.md");
    let body = doc(&["# notes.brief", "", "```md", "only description", "```"]);

    let created_wait = spawn(&path, Filter::Any);
    created_wait.wait_ready();
    assert!(
        !path.exists(),
        "an absent baseline must not create the file"
    );
    write_file(&path, &body);
    let created = created_wait.expect_hit("brief creation with a description");
    let created_text = row_text(&created);
    assert!(
        created_text.contains("created"),
        "creation must be reported:\n{}",
        created_text
    );
    assert!(
        created_text.contains("update description"),
        "created brief must report the description:\n{}",
        created_text
    );
    assert!(
        created_text.contains("after only description"),
        "created description must appear in full:\n{}",
        created_text
    );
    assert_eq!(
        created_text.matches("only description").count(),
        1,
        "creation must not duplicate the description:\n{}",
        created_text
    );

    let deleted_wait = spawn(&path, Filter::Any);
    deleted_wait.wait_ready();
    remove_file(&path);
    let removed = deleted_wait.expect_hit("brief deletion with a description");
    let removed_text = row_text(&removed);
    assert!(
        removed_text.contains("removed"),
        "deletion must be reported:\n{}",
        removed_text
    );
    assert!(
        removed_text.contains("update description"),
        "deleted brief must report the description:\n{}",
        removed_text
    );
    assert!(
        removed_text.contains("before only description"),
        "deleted description must appear in full:\n{}",
        removed_text
    );
    assert_eq!(
        removed_text.matches("only description").count(),
        1,
        "deletion must not duplicate the description:\n{}",
        removed_text
    );
}

#[test]
fn self_mention_qualifies_without_author_exclusion() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &plain_post());
    let waiting = spawn(&path, Filter::Post(Some("alice".to_string())));
    waiting.wait_ready();
    // alice mentions herself: the wait projection has no self-exclusion.
    write_file(
        &path,
        &doc(&[
            "# standup.post",
            "",
            "## #1 alice (2026-09-17T09:00:00Z)",
            "",
            "```md",
            "hello",
            "```",
            "",
            "## #2 alice (2026-09-17T09:05:00Z)",
            "",
            "```md",
            "note to @alice self",
            "```",
        ]),
    );
    let report = waiting.expect_hit("self-mention");
    let text = row_text(&report);
    assert!(
        text.contains("add comment #2"),
        "self-mention must qualify (no notification-style self-exclusion):\n{}",
        text
    );
}

#[test]
fn same_length_rewrite_is_discovered_by_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("standup.agent-besties.md");
    write_file(&path, &plain_post());
    let waiting = spawn(&path, Filter::Any);
    waiting.wait_ready();
    // Same total length, different content: discovery must not depend on
    // size or timestamps.
    write_file(
        &path,
        &doc(&[
            "# standup.post",
            "",
            "## #1 alice (2026-09-17T09:00:00Z)",
            "",
            "```md",
            "HHell",
            "```",
        ]),
    );
    let report = waiting.expect_hit("same-length rewrite");
    let text = row_text(&report);
    assert!(
        text.contains("update comment #1 body"),
        "same-length rewrite must be discovered by content:\n{}",
        text
    );
}

fn plain_post_with_stem(stem: &str) -> String {
    doc(&[
        &format!("# {}.post", stem),
        "",
        "## #1 alice (2026-09-17T09:00:00Z)",
        "",
        "```md",
        "hello",
        "```",
    ])
}
