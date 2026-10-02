//! Managed-file lifecycle: locked reads and locked splice writes.
//!
//! This module is the single place that talks to the filesystem for
//! collaboration files (`*.agent-besties.md`). It implements the lock
//! protocol of spec §10.2:
//!
//! * reads: open a regular file (read-only) → shared lock → read all bytes
//!   (bounded by `MAX_MANAGED_FILE_BYTES`) → release. Never reads without holding the
//!   lock.
//! * writes: open (read+write) or `create_new` → exclusive lock → same-file
//!   identity re-check → read all bytes → run the caller's plan on them →
//!   identity re-check → in-place commit through the locked handle
//!   (`commit`: growth first, then the changed bytes, then shrink, then
//!   `sync_data`; a failure before `sync_data` restores the previous bytes
//!   read under the same lock) → release. Deletion (`Mutation::Delete`)
//!   also rechecks identity and happens on the held lock via `remove_file`,
//!   never via truncation.
//!
//! Error signalling (consumed by `ops`):
//!
//! * lock contention up to the caller-supplied deadline, a Windows lock
//!   violation (33) during lock/read/write, and Unix `EWOULDBLOCK`/`EACCES`
//!   observed *during* a locked read all surface as
//!   `Fault::Reason { id: "E-LOCKED" }`; scan loops translate that into
//!   `busy`/`skipped`, single-file commands into the exit-3 report.
//! * a Windows sharing violation (32) at open surfaces as `Reason { id:
//!   "E-INUSE" }` (that id cannot occur elsewhere).
//! * permission errors at open (a genuinely unreadable path), `NotFound`,
//!   and every other OS error stay `Fault::Io` so §8.3.5 mapping keeps the
//!   real cause; they are never disguised as busy.
//! * content beyond `MAX_MANAGED_FILE_BYTES` (and splice results beyond it)
//!   surface as `Fault::Diagnostics([E-TOO-LARGE])` with `params = {limit}`,
//!   leaving the file untouched.
//! * a failed write whose file shows its previous content again (never
//!   changed, or restored) is `Fault::Io { action: "write" }`, or E-LOCKED
//!   for write-stage lock contention: running the command again is safe.
//!   A write that leaves the file changed or unconfirmed is
//!   `Fault::WriteUncertain`, whatever the cause: running it again could
//!   apply the change twice.
//! * `mutate` on a missing file with `create = false` returns
//!   `Reason { id: "E-FILE-MISSING" }` (params `{file}` = cwd-relative
//!   display); the `state --remove` → `E-STATE-NOTFOUND` translation belongs
//!   to ops, not here.
//! * an empty file at the deadline of a non-creating form returns
//!   `Fault::Diagnostics([E-H1])` (empty file = no H1, spec §10.2.3).
//!
//! `read_stable_optional` is the wait-specific stable sample: only a real
//! `NotFound` with a still-valid parent is `None`; parent invalidation,
//! permissions and instability are errors. The base `read` never depends on
//! it.

mod commit;
pub mod walk;

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions, TryLockError};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use self::commit::CommitError;
use crate::consts;
use crate::error::{Diagnostic, Fault};
use crate::names;

/// Windows `ERROR_SHARING_VIOLATION` (32): another program holds the file
/// without sharing. Only possible on Windows (§10.2.5).
const WIN_ERROR_SHARING_VIOLATION: i32 = 32;
/// Windows `ERROR_LOCK_VIOLATION` (33): a mandatory range lock (ours or
/// another process's) blocks the operation.
const WIN_ERROR_LOCK_VIOLATION: i32 = 33;
/// Windows `ERROR_ACCESS_DENIED` (5): at open/`create_new` stage this may be
/// a pending delete, which is retried a bounded number of times (§10.2.6).
const WIN_ERROR_ACCESS_DENIED: i32 = 5;

/// A wait budget computed purely with elapsed/remaining subtraction.
/// `Instant + Duration` panics on overflow (a large `u64` millisecond budget
/// overflows `Instant`'s representable range on Windows); subtracting
/// elapsed time from the budget keeps every legal `u64` budget usable
/// without inventing a cap.
struct Deadline {
    start: Instant,
    budget: Duration,
}

impl Deadline {
    fn new(wait_ms: u64) -> Deadline {
        Deadline {
            start: Instant::now(),
            budget: Duration::from_millis(wait_ms),
        }
    }

    fn expired(&self) -> bool {
        self.start.elapsed() >= self.budget
    }

    fn remaining(&self) -> Option<Duration> {
        self.budget.checked_sub(self.start.elapsed())
    }
}

// ---------------------------------------------------------------------------
// shared-lock bounded read
// ---------------------------------------------------------------------------

/// Read a managed file under a shared lock, polling for up to `wait_ms`.
///
/// Blocks (polling every `LOCK_POLL_MS`) while another agent-coexist process
/// holds the exclusive lock; on timeout returns `Reason { id: "E-LOCKED" }`.
/// `cancel` is checked between attempts. Content larger than
/// `MAX_MANAGED_FILE_BYTES` returns `Diagnostics([E-TOO-LARGE])`.
pub fn read(path: &Path, wait_ms: u64, cancel: Option<&AtomicBool>) -> Result<Vec<u8>, Fault> {
    let deadline = Deadline::new(wait_ms);
    loop {
        if let Some(flag) = cancel {
            if flag.load(Ordering::Relaxed) {
                return Err(Fault::Cancelled);
            }
        }
        match open_regular(path, OpenOptions::new().read(true)) {
            Ok(file) => match file.try_lock_shared() {
                Ok(()) => match read_all_bounded(&file) {
                    Ok(bytes) => {
                        drop(file);
                        return Ok(bytes);
                    }
                    Err(e) if is_read_stage_busy(&e) => { /* contention: keep polling */ }
                    Err(e) => return Err(read_fault(path, e)),
                },
                Err(TryLockError::WouldBlock) => { /* contention: keep polling */ }
                Err(TryLockError::Error(e)) if is_win_error(&e, WIN_ERROR_LOCK_VIOLATION) => {
                    // Someone else's mandatory range even though we hold this handle.
                }
                Err(TryLockError::Error(e)) => {
                    return Err(Fault::io("lock", path.to_path_buf(), e))
                }
            },
            Err(e) => {
                if is_win_error(&e, WIN_ERROR_SHARING_VIOLATION) {
                    return Err(Fault::reason("E-INUSE"));
                }
                // A path that cannot be opened for reading (permissions, NotFound,
                // ...) is a real I/O failure, never lock-busy.
                return Err(Fault::io("open", path.to_path_buf(), e));
            }
        }
        if deadline.expired() {
            return Err(Fault::reason("E-LOCKED"));
        }
        sleep_one_poll(&deadline);
    }
}

/// One stable sample for the wait commands (spec wait plan §5.3).
///
/// Reopens `path` by its original path, takes a shared lock within
/// `READ_WAIT_MS`, verifies the handle still points at the same file before
/// and after reading, and confirms byte stability by re-reading under the
/// same lock. Identity changes and instability cause a bounded reopen (up to
/// `OPEN_RETRIES`, without resetting the deadline of this sample). Only a
/// real `NotFound` while the parent directory is still valid yields
/// `Ok(None)`; a vanished/invalid parent is an error. A present zero-byte
/// file yields `Ok(Some(vec![]))` — treating it as absent or as a failure is
/// the caller's (wait's) decision, not the read's.
pub fn read_stable_optional(
    path: &Path,
    cancel: Option<&AtomicBool>,
) -> Result<Option<Vec<u8>>, Fault> {
    let deadline = Deadline::new(consts::READ_WAIT_MS);
    let mut reopens = 0usize;
    loop {
        if let Some(flag) = cancel {
            if flag.load(Ordering::Relaxed) {
                return Err(Fault::Cancelled);
            }
        }
        match open_regular(path, OpenOptions::new().read(true)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Following a dangling symlink also returns NotFound. A
                // surviving entry (including a replacement) is not absence.
                if !matches!(
                    std::fs::symlink_metadata(path),
                    Err(error) if error.kind() == io::ErrorKind::NotFound
                ) {
                    return Err(Fault::io("open", path.to_path_buf(), e));
                }
                // A deleted parent is an error, not "absent".
                return match parent_is_valid_dir(path) {
                    Ok(true) => Ok(None),
                    Ok(false) => Err(Fault::io("open", path.to_path_buf(), e)),
                    Err(pe) => Err(pe),
                };
            }
            Err(e) if cfg!(windows) && is_win_error(&e, WIN_ERROR_ACCESS_DENIED) => {
                // Possibly a pending delete: bounded reopen, deadline not reset.
                reopens += 1;
                if reopens > consts::OPEN_RETRIES || deadline.expired() {
                    return Err(Fault::io("open", path.to_path_buf(), e));
                }
                sleep_one_poll(&deadline);
                continue;
            }
            Err(e) => {
                if is_win_error(&e, WIN_ERROR_SHARING_VIOLATION) {
                    return Err(Fault::reason("E-INUSE"));
                }
                return Err(Fault::io("open", path.to_path_buf(), e));
            }
            Ok(file) => {
                lock_shared_until(&file, path, &deadline, cancel)?;
                if !same_file_identity(&file, path)? {
                    reopens += 1;
                    if reopens > consts::OPEN_RETRIES {
                        return Err(Fault::reason("E-LOCKED"));
                    }
                    sleep_one_poll(&deadline);
                    continue;
                }
                let first = read_all_bounded(&file).map_err(|e| read_fault(path, e))?;
                // Post-read identity re-check and byte-stability confirmation,
                // reusing the same locked handle.
                if !same_file_identity(&file, path)? {
                    reopens += 1;
                    if reopens > consts::OPEN_RETRIES {
                        return Err(Fault::reason("E-LOCKED"));
                    }
                    sleep_one_poll(&deadline);
                    continue;
                }
                let second = read_all_bounded(&file).map_err(|e| read_fault(path, e))?;
                if second != first {
                    reopens += 1;
                    if reopens > consts::OPEN_RETRIES {
                        return Err(Fault::reason("E-LOCKED"));
                    }
                    sleep_one_poll(&deadline);
                    continue;
                }
                drop(file);
                return Ok(Some(first));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// locked splice writes
// ---------------------------------------------------------------------------

/// How `mutate` may treat a missing target file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WritePolicy {
    /// Total time budget for lock acquisition and retries (single deadline,
    /// never reset): `FASTFAIL_GRACE_MS` for profile/brief/state forms,
    /// `POST_WAIT_MS` (env-overridable via `time::post_wait_ms`) for
    /// post/subscribe.
    pub wait_ms: u64,
    /// Creating forms (`post`, `subscribe`, `brief` create forms, `profile`,
    /// `state` declare) create the file when missing and take over abandoned
    /// zero-byte files at the deadline. Non-creating forms get
    /// `E-FILE-MISSING`.
    pub create: bool,
    /// Create missing parent directories (collected outer→inner in
    /// `WriteResult::created_dirs`). Only meaningful with `create`.
    pub create_parents: bool,
}

/// What the caller's plan decided to do with the file.
#[derive(Debug)]
pub enum Mutation {
    /// Replace the file's content with these bytes, in place (`commit`).
    Write(Vec<u8>),
    /// Leave the file's bytes exactly as they are.
    Unchanged,
    /// Delete the file while still holding the lock (§6.4.2).
    Delete,
}

/// Outcome of a successful `mutate`.
#[derive(Debug)]
pub struct WriteResult<T> {
    pub value: T,
    /// True when this call wrote the file's initial content (fresh
    /// `create_new`, or takeover of an abandoned zero-byte file at the
    /// deadline). False when splicing existing content, including the case
    /// where another writer had already adopted our `create_new` race.
    pub created: bool,
    /// Directories created by this call, outer→inner (§7.3.2 print order).
    pub created_dirs: Vec<PathBuf>,
    /// True only for `Mutation::Delete`.
    pub deleted: bool,
}

/// Only successful exclusive-lock acquisition constructs this wrapper.
/// Merely opening or creating a file never authorizes rollback deletion.
#[derive(Debug)]
struct LockedFile(File);

/// Retain the native identity handle until cleanup or successful reporting.
/// A pathname alone is not evidence that a directory is still ours.
struct CreatedDir {
    path: PathBuf,
    identity: same_file::Handle,
}

/// Run `plan` under an exclusive lock on the real target file.
///
/// `plan(bytes, initial)` receives the file's current bytes (empty in
/// initial-content mode) and `initial == true` exactly when this call is
/// supposed to produce complete initial content (fresh creation or takeover
/// of an abandoned zero-byte file); it returns the splice result and a value
/// to pass through. `plan` borrows the bytes, which stay the restore source
/// for a failed commit; it must not re-create content or perform I/O on the
/// target itself. All of the following live here and only here:
///
/// * `create_new` followed by a mandatory re-read: non-empty means another
///   writer already took over — the existing bytes are spliced instead and
///   `created` stays false;
/// * zero-byte waiting: an existing empty file is released and retried every
///   `LOCK_POLL_MS` under one non-resetting deadline; still empty at the
///   deadline, creating forms take over (`created = true`), non-creating
///   forms get `Diagnostics([E-H1])`;
/// * same-file identity re-check after locking, with bounded reopen
///   (`OPEN_RETRIES`, shared with create collisions and pending deletes),
///   and again after a successful plan before Write/Delete; a changed path
///   then gets E-LOCKED;
/// * held-lock deletion for `Mutation::Delete`;
/// * creation rollback on pre-write and zero-byte write failures only when
///   this call holds the exclusive lock, the created file still matches
///   `path`, and it is still empty. Nonempty/unknown content and contested
///   creations are preserved, together with their directories;
/// * directory rollback is inner→outer, only for still-empty directories
///   matching retained identity handles. Failed identity capture never
///   falls back to path-only deletion. Portable directory creation and
///   identity capture are separate operations, as are the final identity
///   checks and unlink calls; these short windows against non-cooperating
///   replacements are not claimed as atomic ownership guarantees;
/// * in-place commit through the held handle (`commit`); a failure before
///   `sync_data` restores the previous bytes, and a creation is then empty
///   again and rolled back as above. A failed restore or `sync_data` is
///   [`Fault::WriteUncertain`] and keeps the file and its directories.
pub fn mutate<T>(
    path: &Path,
    policy: WritePolicy,
    mut plan: impl FnMut(&[u8], bool) -> Result<(Mutation, T), Fault>,
) -> Result<WriteResult<T>, Fault> {
    let deadline = Deadline::new(policy.wait_ms);
    let mut reopens = 0usize;
    let mut created_dirs = Vec::new();
    let mut parents_ensured = false;

    loop {
        match open_regular(path, OpenOptions::new().read(true).write(true)) {
            Ok(file) => {
                // Without this lock an adopter may still be preparing its
                // first byte. Preserve the file and all created directories.
                let file = lock_exclusive_until(file, path, &deadline)?;
                match same_file_identity(&file.0, path) {
                    Ok(true) => {}
                    Ok(false) => {
                        drop(file);
                        reopens += 1;
                        if reopens > consts::OPEN_RETRIES {
                            return Err(fail_with_rollback(
                                Fault::reason("E-LOCKED"),
                                None,
                                path,
                                false,
                                created_dirs,
                            ));
                        }
                        continue;
                    }
                    Err(fault) => {
                        return Err(fail_with_rollback(
                            fault,
                            Some(file),
                            path,
                            false,
                            created_dirs,
                        ));
                    }
                }
                let bytes = match read_all_bounded(&file.0) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return Err(fail_with_rollback(
                            read_fault(path, e),
                            Some(file),
                            path,
                            false,
                            created_dirs,
                        ));
                    }
                };
                if bytes.is_empty() {
                    if deadline.expired() {
                        if policy.create {
                            // Adopted zero-byte files are never ours to
                            // unlink, even when we supply initial content.
                            return run_plan(
                                file,
                                path,
                                Vec::new(),
                                true,
                                false,
                                created_dirs,
                                &mut plan,
                            );
                        }
                        return Err(fail_with_rollback(
                            Fault::Diagnostics(vec![Diagnostic::new("E-H1", 1).with("line", "")]),
                            Some(file),
                            path,
                            false,
                            created_dirs,
                        ));
                    }
                    // Zero-byte retries do not consume the reopen budget.
                    drop(file);
                    sleep_one_poll(&deadline);
                    continue;
                }
                return run_plan(file, path, bytes, false, false, created_dirs, &mut plan);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if !policy.create {
                    return Err(fail_with_rollback(
                        Fault::reason("E-FILE-MISSING")
                            .with("file", names::display_text(&walk::display_path(path))),
                        None,
                        path,
                        false,
                        created_dirs,
                    ));
                }
                if policy.create_parents && !parents_ensured {
                    if let Err(fault) = ensure_parents(path, &mut created_dirs) {
                        return Err(fail_with_rollback(fault, None, path, false, created_dirs));
                    }
                    parents_ensured = true;
                }
                match OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(path)
                {
                    Ok(file) => {
                        // Creation proves neither exclusive ownership nor
                        // that another writer has not adopted this file.
                        // On lock failure do not unlink or clean directories.
                        let file = lock_exclusive_until(file, path, &deadline)?;
                        match same_file_identity(&file.0, path) {
                            Ok(true) => {}
                            Ok(false) => {
                                drop(file);
                                reopens += 1;
                                if reopens > consts::OPEN_RETRIES {
                                    return Err(fail_with_rollback(
                                        Fault::reason("E-LOCKED"),
                                        None,
                                        path,
                                        false,
                                        created_dirs,
                                    ));
                                }
                                continue;
                            }
                            Err(fault) => {
                                return Err(fail_with_rollback(
                                    fault,
                                    Some(file),
                                    path,
                                    true,
                                    created_dirs,
                                ));
                            }
                        }
                        // Mandatory re-read: another writer may already
                        // have supplied content through our creation.
                        let bytes = match read_all_bounded(&file.0) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                return Err(fail_with_rollback(
                                    read_fault(path, e),
                                    Some(file),
                                    path,
                                    true,
                                    created_dirs,
                                ));
                            }
                        };
                        let initial = bytes.is_empty();
                        return run_plan(
                            file,
                            path,
                            bytes,
                            initial,
                            initial,
                            created_dirs,
                            &mut plan,
                        );
                    }
                    Err(e2) if e2.kind() == io::ErrorKind::AlreadyExists => {
                        // Reopen for a concurrent creator even with no wait
                        // budget. A dangling symlink can repeat NotFound /
                        // AlreadyExists forever, so this also consumes the
                        // bounded lifecycle reopen budget. Sleeps never reset
                        // or exceed the original deadline.
                        reopens += 1;
                        if reopens > consts::OPEN_RETRIES {
                            return Err(fail_with_rollback(
                                Fault::io("create", path.to_path_buf(), e2),
                                None,
                                path,
                                false,
                                created_dirs,
                            ));
                        }
                        sleep_one_poll(&deadline);
                        continue;
                    }
                    Err(e2) if cfg!(windows) && is_win_error(&e2, WIN_ERROR_ACCESS_DENIED) => {
                        // Possible pending delete: bounded retry (§10.2.6).
                        reopens += 1;
                        if reopens > consts::OPEN_RETRIES {
                            return Err(fail_with_rollback(
                                Fault::io("create", path.to_path_buf(), e2),
                                None,
                                path,
                                false,
                                created_dirs,
                            ));
                        }
                        sleep_one_poll(&deadline);
                        continue;
                    }
                    Err(e2) if is_win_error(&e2, WIN_ERROR_SHARING_VIOLATION) => {
                        // The inaccessible target may have been adopted;
                        // preserve its containing directories as well.
                        return Err(Fault::reason("E-INUSE"));
                    }
                    Err(e2) => {
                        return Err(fail_with_rollback(
                            Fault::io("create", path.to_path_buf(), e2),
                            None,
                            path,
                            false,
                            created_dirs,
                        ));
                    }
                }
            }
            Err(e) if cfg!(windows) && is_win_error(&e, WIN_ERROR_ACCESS_DENIED) => {
                reopens += 1;
                if reopens > consts::OPEN_RETRIES {
                    return Err(fail_with_rollback(
                        Fault::io("open", path.to_path_buf(), e),
                        None,
                        path,
                        false,
                        created_dirs,
                    ));
                }
                sleep_one_poll(&deadline);
                continue;
            }
            Err(e) if is_win_error(&e, WIN_ERROR_SHARING_VIOLATION) => {
                // No ownership proof for this contested target.
                return Err(Fault::reason("E-INUSE"));
            }
            Err(e) => {
                return Err(fail_with_rollback(
                    Fault::io("open", path.to_path_buf(), e),
                    None,
                    path,
                    false,
                    created_dirs,
                ));
            }
        }
    }
}

/// Apply the plan's mutation to a locked file handle.
///
/// `initial` is the create/takeover flag passed to `plan`; `created_by_us`
/// drives the creation rollback ([`fail_with_rollback`]). `created_dirs` is
/// reported on success and rolled back on failure.
fn run_plan<T, P>(
    mut file: LockedFile,
    path: &Path,
    bytes: Vec<u8>,
    initial: bool,
    created_by_us: bool,
    created_dirs: Vec<CreatedDir>,
    plan: &mut P,
) -> Result<WriteResult<T>, Fault>
where
    P: FnMut(&[u8], bool) -> Result<(Mutation, T), Fault>,
{
    let outcome = (|| {
        let (mutation, value) = plan(&bytes, initial)?;
        if matches!(&mutation, Mutation::Write(_) | Mutation::Delete)
            && !same_file_identity(&file.0, path)?
        {
            // Planning may have taken arbitrarily long. Never apply its
            // result to a replacement path or report a detached write as
            // success. The final check→mutation window is not atomic.
            return Err(Fault::reason("E-LOCKED"));
        }
        match mutation {
            Mutation::Write(new_bytes) => {
                if new_bytes.len() > consts::MAX_MANAGED_FILE_BYTES {
                    return Err(too_large_fault());
                }
                // Through the held handle, never reopened by path: a file
                // renamed meanwhile still gets its own bytes back.
                commit::commit(&mut file.0, &bytes, &new_bytes).map_err(|e| match e {
                    CommitError::Restored(e) => write_fault(path, e),
                    CommitError::Uncertain { cause, left } => Fault::WriteUncertain {
                        path: path.to_path_buf(),
                        cause,
                        left,
                    },
                })?;
                Ok((value, initial, false))
            }
            Mutation::Unchanged => Ok((value, initial, false)),
            Mutation::Delete => {
                // Held lock and fresh identity check, never truncation.
                // Unlink checks directory permissions: Unix EACCES here is
                // not the mandatory-lock contention recognized during I/O.
                std::fs::remove_file(path).map_err(|e| {
                    if is_win_error(&e, WIN_ERROR_LOCK_VIOLATION) {
                        Fault::reason("E-LOCKED")
                    } else {
                        Fault::io("delete", path.to_path_buf(), e)
                    }
                })?;
                Ok((value, false, true))
            }
        }
    })();
    match outcome {
        Err(fault) => Err(fail_with_rollback(
            fault,
            Some(file),
            path,
            created_by_us,
            created_dirs,
        )),
        Ok((value, created, deleted)) => {
            drop(file);
            Ok(WriteResult {
                value,
                created,
                created_dirs: created_dirs.into_iter().map(|dir| dir.path).collect(),
                deleted,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// creation rollback (§10.8.3)
// ---------------------------------------------------------------------------

/// Fail with `fault`, removing only a locked, verified, empty creation and
/// then verified empty directories. An unlocked handle cannot enter this
/// function. Existing/adopted files, nonempty partial writes and unknown
/// identities retain their directories as well. Real cleanup failures
/// surface as `Fault::Io` instead of hiding behind `fault`. A
/// [`Fault::WriteUncertain`] file is kept as reported, with its directories.
///
/// File handles are closed before directory cleanup; directory identity
/// handles are closed inner→outer so Windows pending deletion completes
/// before the containing directory is considered. Identity checks and
/// path-based removals are not atomic against non-cooperating replacements.
fn fail_with_rollback(
    fault: Fault,
    file: Option<LockedFile>,
    path: &Path,
    created_by_us: bool,
    created_dirs: Vec<CreatedDir>,
) -> Fault {
    if matches!(fault, Fault::WriteUncertain { .. }) {
        return fault;
    }
    if let Some(file) = file {
        match rollback_created_file(file, path, created_by_us) {
            Ok(true) => {}
            Ok(false) => return fault,
            Err(cleanup) => return cleanup,
        }
    }
    match remove_empty_created_dirs(created_dirs) {
        Err(cleanup) => cleanup,
        Ok(()) => fault,
    }
}

/// Return whether directory cleanup is safe after releasing this lock.
/// Only an empty creation belonging to this call may be unlinked.
fn rollback_created_file(
    file: LockedFile,
    path: &Path,
    created_by_us: bool,
) -> Result<bool, Fault> {
    if !created_by_us {
        return Ok(false);
    }
    // Check content first: even a detached partial write must retain all
    // its directories. Failure to inspect the bytes is not proof of empty.
    let metadata = file
        .0
        .metadata()
        .map_err(|e| Fault::io("delete", path.to_path_buf(), e))?;
    if metadata.len() != 0 {
        return Ok(false);
    }
    if !same_file_identity(&file.0, path)? {
        // Do not touch a missing or foreign pathname. Independently verified
        // empty directories may still be cleaned up.
        return Ok(true);
    }
    std::fs::remove_file(path).map_err(|e| Fault::io("delete", path.to_path_buf(), e))?;
    Ok(true)
}

/// Consume retained directory handles inner→outer. Only an ordinary,
/// still-empty directory matching its creation identity may be removed.
fn remove_empty_created_dirs(created_dirs: Vec<CreatedDir>) -> Result<(), Fault> {
    for dir in created_dirs.into_iter().rev() {
        if !is_current_created_dir(&dir)? {
            continue;
        }
        let empty = {
            let mut entries = std::fs::read_dir(&dir.path)
                .map_err(|e| Fault::io("delete", dir.path.clone(), e))?;
            match entries.next() {
                None => true,
                Some(Ok(_)) => false,
                Some(Err(e)) => return Err(Fault::io("delete", dir.path, e)),
            }
        };
        // Drop the read_dir iterator before removal, and recheck both
        // identity and directory kind after enumeration.
        if empty && is_current_created_dir(&dir)? {
            std::fs::remove_dir(&dir.path).map_err(|e| Fault::io("delete", dir.path.clone(), e))?;
        }
        // `dir.identity` closes here, before considering the parent.
    }
    Ok(())
}

fn is_current_created_dir(dir: &CreatedDir) -> Result<bool, Fault> {
    let metadata = match std::fs::symlink_metadata(&dir.path) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(Fault::io("delete", dir.path.clone(), e)),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Ok(false);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // Junctions and other reparse points also name foreign objects.
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Ok(false);
        }
    }
    same_file_identity(dir.identity.as_file(), &dir.path)
}

// ---------------------------------------------------------------------------
// directory-entry spelling
// ---------------------------------------------------------------------------

/// `path` with its final component spelled as the directory lists it.
///
/// On Windows and macOS an operand such as `ROOM/STANDUP` opens
/// `room/standup.agent-besties.md`; the file name that `list` shows and
/// that `W-NAME` compares is the entry's own spelling, not the typed one.
/// One native lookup of the final component (Windows `FindFirstFileW`,
/// macOS `getattrlist(ATTR_CMN_NAME)`), neither of which follows a final
/// symlink: an alias keeps its own name. Only a spelling that differs from
/// the typed name by case is adopted. Everything else — no such lookup on
/// this platform (Linux names are case-sensitive), a failed lookup, or a
/// different name (a Windows 8.3 short-name alias) — keeps `path` as
/// typed, which is never a reason to drop a warning.
pub fn listed_path(path: &Path) -> Cow<'_, Path> {
    let Some(typed) = path.file_name() else {
        return Cow::Borrowed(path);
    };
    match listed_name(path) {
        Some(listed) if listed != typed && same_ignoring_case(&listed, typed) => {
            Cow::Owned(path.with_file_name(listed))
        }
        _ => Cow::Borrowed(path),
    }
}

/// Same name up to letter case. Names that are not Unicode never compare
/// equal here, so a lossy decoding cannot make two names look alike.
fn same_ignoring_case(a: &OsStr, b: &OsStr) -> bool {
    match (a.to_str(), b.to_str()) {
        (Some(a), Some(b)) => a
            .chars()
            .flat_map(char::to_lowercase)
            .eq(b.chars().flat_map(char::to_lowercase)),
        _ => false,
    }
}

#[cfg(windows)]
fn listed_name(path: &Path) -> Option<OsString> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    type Handle = *mut core::ffi::c_void;
    const INVALID_HANDLE_VALUE: Handle = !0usize as Handle;

    /// Win32 `WIN32_FIND_DATAW`; `FILETIME` is two `u32`s. Only the name
    /// is read; the other fields size the struct.
    #[repr(C)]
    struct FindData {
        _attributes: u32,
        _times: [[u32; 2]; 3],
        _size: [u32; 2],
        _reserved: [u32; 2],
        file_name: [u16; 260],
        _alternate_file_name: [u16; 14],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn FindFirstFileW(file_name: *const u16, find_data: *mut FindData) -> Handle;
        fn FindClose(find_file: Handle) -> i32;
    }

    // `* ? < > "` are wildcards to FindFirstFileW and could match another
    // entry; no Windows file name contains them (or a NUL, which would cut
    // the lookup short), so such a final component has no listed spelling.
    // Only the final component is checked: `\\?\` prefixes contain `?`.
    if path
        .file_name()?
        .encode_wide()
        .any(|c| matches!(c, 0 | 0x2A | 0x3F | 0x3C | 0x3E | 0x22))
    {
        return None;
    }
    if path.as_os_str().encode_wide().any(|c| c == 0) {
        return None;
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut data = std::mem::MaybeUninit::<FindData>::uninit();
    // SAFETY: `wide` is NUL-terminated and outlives the call; `data` is
    // writable storage of the exact struct the API fills.
    let handle = unsafe { FindFirstFileW(wide.as_ptr(), data.as_mut_ptr()) };
    if handle == INVALID_HANDLE_VALUE || handle.is_null() {
        return None;
    }
    // SAFETY: a valid search handle returned above, closed exactly once.
    unsafe { FindClose(handle) };
    // SAFETY: FindFirstFileW succeeded, so it initialized `data`.
    let data = unsafe { data.assume_init() };
    let len = data
        .file_name
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(data.file_name.len());
    Some(OsString::from_wide(&data.file_name[..len]))
}

#[cfg(target_os = "macos")]
fn listed_name(path: &Path) -> Option<OsString> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    /// `getattrlist` reply for `ATTR_CMN_NAME`: total length, then the
    /// reference whose offset is relative to the reference itself, then
    /// room for the name (at most `NAME_MAX` UTF-8 bytes plus NUL).
    #[repr(C)]
    struct NameReply {
        _length: u32,
        name: libc::attrreference_t,
        _bytes: [u8; 1024],
    }

    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut request = libc::attrlist {
        bitmapcount: libc::ATTR_BIT_MAP_COUNT,
        reserved: 0,
        commonattr: libc::ATTR_CMN_NAME,
        volattr: 0,
        dirattr: 0,
        fileattr: 0,
        forkattr: 0,
    };
    let mut reply = NameReply {
        _length: 0,
        name: libc::attrreference_t {
            attr_dataoffset: 0,
            attr_length: 0,
        },
        _bytes: [0; 1024],
    };
    // SAFETY: `c_path` is NUL-terminated; `request` and `reply` are live,
    // correctly sized buffers for the duration of the call.
    let rc = unsafe {
        libc::getattrlist(
            c_path.as_ptr(),
            (&mut request as *mut libc::attrlist).cast(),
            (&mut reply as *mut NameReply).cast(),
            std::mem::size_of::<NameReply>(),
            libc::FSOPT_NOFOLLOW,
        )
    };
    if rc != 0 {
        return None;
    }
    // SAFETY: `NameReply` is plain integers without padding; viewing it as
    // bytes for its whole size is sound.
    let raw = unsafe {
        std::slice::from_raw_parts(
            (&reply as *const NameReply).cast::<u8>(),
            std::mem::size_of::<NameReply>(),
        )
    };
    let start = std::mem::offset_of!(NameReply, name)
        .checked_add(usize::try_from(reply.name.attr_dataoffset).ok()?)?;
    let len = usize::try_from(reply.name.attr_length).ok()?;
    let name = raw.get(start..start.checked_add(len)?)?;
    let name = name.strip_suffix(&[0]).unwrap_or(name);
    Some(OsString::from_vec(name.to_vec()))
}

/// Linux and other Unix names are case-sensitive: the typed name is the
/// entry's spelling whenever the path opened.
#[cfg(not(any(windows, target_os = "macos")))]
fn listed_name(_path: &Path) -> Option<OsString> {
    None
}

// ---------------------------------------------------------------------------
// internals
// ---------------------------------------------------------------------------

/// Open an existing regular file, following symlinks to regular files.
/// Unix opens must be nonblocking before descriptor validation: a pathname
/// metadata precheck alone can race with replacement by a FIFO. O_NOCTTY also
/// prevents an unsupported terminal device from becoming our controlling TTY.
pub(crate) fn open_regular(path: &Path, options: &mut OpenOptions) -> io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "path does not name a regular file",
        ));
    }
    Ok(file)
}

/// Identity checks also open replaceable paths, including directories. They
/// must not wait for a FIFO writer if a non-cooperating actor swaps the path.
/// Keep same-file's metadata-only, directory-capable open on Windows.
fn open_identity(path: &Path) -> io::Result<same_file::Handle> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY)
            .open(path)?;
        same_file::Handle::from_file(file)
    }
    #[cfg(not(unix))]
    {
        same_file::Handle::from_path(path)
    }
}

/// Map a failed write, after which the file shows its previous content, to
/// `E-LOCKED` for write-stage lock contention and `Fault::Io` otherwise.
fn write_fault(path: &Path, e: io::Error) -> Fault {
    if is_write_stage_busy(&e) {
        Fault::reason("E-LOCKED")
    } else {
        Fault::io("write", path.to_path_buf(), e)
    }
}

/// Consume a handle, returning proof of exclusive locking only on success.
fn lock_exclusive_until(file: File, path: &Path, deadline: &Deadline) -> Result<LockedFile, Fault> {
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(LockedFile(file)),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(e)) if is_win_error(&e, WIN_ERROR_LOCK_VIOLATION) => {}
            Err(TryLockError::Error(e)) => return Err(Fault::io("lock", path.to_path_buf(), e)),
        }
        if deadline.expired() {
            return Err(Fault::reason("E-LOCKED"));
        }
        sleep_one_poll(deadline);
    }
}

/// Poll `try_lock_shared` until `deadline`.
fn lock_shared_until(
    file: &File,
    path: &Path,
    deadline: &Deadline,
    cancel: Option<&AtomicBool>,
) -> Result<(), Fault> {
    loop {
        if let Some(flag) = cancel {
            if flag.load(Ordering::Relaxed) {
                return Err(Fault::Cancelled);
            }
        }
        match file.try_lock_shared() {
            Ok(()) => return Ok(()),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(e)) if is_win_error(&e, WIN_ERROR_LOCK_VIOLATION) => {}
            Err(TryLockError::Error(e)) => return Err(Fault::io("lock", path.to_path_buf(), e)),
        }
        if deadline.expired() {
            return Err(Fault::reason("E-LOCKED"));
        }
        sleep_one_poll(deadline);
    }
}

/// Read the whole file from the start, at most `MAX_MANAGED_FILE_BYTES`
/// bytes; exceeding the limit returns an `ErrorKind::FileTooLarge` marker
/// that the callers translate via [`read_fault`] into the E-TOO-LARGE
/// diagnostic.
fn read_all_bounded(file: &File) -> io::Result<Vec<u8>> {
    let mut reader = file;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 64 * 1024];
    reader.seek(SeekFrom::Start(0))?;
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        if buf.len() + n > consts::MAX_MANAGED_FILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                "managed file exceeds MAX_MANAGED_FILE_BYTES",
            ));
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    Ok(buf)
}

/// Map a bounded-read failure: over-limit becomes the E-TOO-LARGE
/// diagnostic, lock-style contention becomes E-LOCKED, the rest stays I/O.
fn read_fault(path: &Path, e: io::Error) -> Fault {
    if e.kind() == io::ErrorKind::FileTooLarge {
        too_large_fault()
    } else if is_read_stage_busy(&e) {
        Fault::reason("E-LOCKED")
    } else {
        Fault::io("read", path.to_path_buf(), e)
    }
}

fn too_large_fault() -> Fault {
    let mib = consts::MAX_MANAGED_FILE_BYTES / (1024 * 1024);
    Fault::Diagnostics(vec![
        Diagnostic::new("E-TOO-LARGE", 1).with("limit", mib.to_string())
    ])
}

/// Contention observed *while reading* under a lock: Windows lock violation
/// (33), or Unix `EWOULDBLOCK`/`EACCES` from filesystems that make flock
/// mandatory (SMB) (§10.2.5).
fn is_read_stage_busy(e: &io::Error) -> bool {
    if cfg!(windows) {
        is_win_error(e, WIN_ERROR_LOCK_VIOLATION)
    } else {
        matches!(
            e.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::PermissionDenied
        )
    }
}

/// Contention observed while writing/truncating/syncing under our exclusive
/// lock: another process's mandatory range (§10.2.5).
fn is_write_stage_busy(e: &io::Error) -> bool {
    if cfg!(windows) {
        is_win_error(e, WIN_ERROR_LOCK_VIOLATION)
    } else {
        matches!(
            e.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::PermissionDenied
        )
    }
}

fn is_win_error(e: &io::Error, code: i32) -> bool {
    cfg!(windows) && e.raw_os_error() == Some(code)
}

/// Does the handle still point at the file currently reachable via `path`?
/// `Ok(false)` means deleted/replaced (including Windows pending-delete at
/// open); other errors propagate as real I/O failures.
fn same_file_identity(file: &File, path: &Path) -> Result<bool, Fault> {
    match open_identity(path) {
        Ok(current) => {
            let mine = file
                .try_clone()
                .map_err(|e| Fault::io("open", path.to_path_buf(), e))
                .and_then(|f| {
                    same_file::Handle::from_file(f)
                        .map_err(|e| Fault::io("open", path.to_path_buf(), e))
                })?;
            Ok(mine == current)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) if cfg!(windows) && is_win_error(&e, WIN_ERROR_ACCESS_DENIED) => Ok(false),
        Err(e) => Err(Fault::io("open", path.to_path_buf(), e)),
    }
}

/// Sleep one `LOCK_POLL_MS` interval, never past `deadline`.
fn sleep_one_poll(deadline: &Deadline) {
    if let Some(remaining) = deadline.remaining() {
        let poll = Duration::from_millis(consts::LOCK_POLL_MS);
        std::thread::sleep(if remaining < poll { remaining } else { poll });
    }
}

/// Is the parent of `path` an existing directory? `Ok(false)` for a missing
/// or non-directory parent; non-`NotFound` errors propagate.
fn parent_is_valid_dir(path: &Path) -> Result<bool, Fault> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    match std::fs::metadata(&parent) {
        Ok(md) => Ok(md.is_dir()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(Fault::io("open", parent, e)),
    }
}

/// Create missing parents outer→inner and retain native identity handles.
/// If capture fails, the uncaptured directory is preserved and the error
/// surfaces; it is never entered into a path-only cleanup list. Portable
/// mkdir and identity capture are not atomic against outside replacement.
fn ensure_parents(path: &Path, created: &mut Vec<CreatedDir>) -> Result<(), Fault> {
    let mut missing = Vec::new();
    let mut cur = path.parent();
    while let Some(dir) = cur {
        if !dir.as_os_str().is_empty() && !dir.exists() {
            missing.push(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    // `missing` is inner→outer; create outer first.
    for dir in missing.into_iter().rev() {
        match std::fs::create_dir(&dir) {
            Ok(()) => {
                let identity =
                    open_identity(&dir).map_err(|e| Fault::io("open", dir.clone(), e))?;
                created.push(CreatedDir {
                    path: dir,
                    identity,
                });
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(Fault::io("create", dir, e)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("coexist-fsx-tests").join(format!(
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

    fn policy(wait_ms: u64, create: bool, create_parents: bool) -> WritePolicy {
        WritePolicy {
            wait_ms,
            create,
            create_parents,
        }
    }

    #[test]
    fn mutate_creates_file_and_parents() {
        let dir = scratch("create");
        let target = dir.join("a/b/notes.agent-besties.md");
        let res = mutate(&target, policy(200, true, true), |bytes, initial| {
            assert!(bytes.is_empty());
            assert!(initial);
            Ok((Mutation::Write(b"# notes.post\n".to_vec()), ()))
        })
        .unwrap();
        assert!(res.created);
        assert!(!res.deleted);
        assert_eq!(res.created_dirs.len(), 2);
        assert_eq!(res.created_dirs[0], dir.join("a"));
        assert_eq!(res.created_dirs[1], dir.join("a/b"));
        assert_eq!(std::fs::read(&target).unwrap(), b"# notes.post\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mutate_missing_without_create_reports_file_missing() {
        let dir = scratch("missing");
        let target = dir.join("none.agent-besties.md");
        let err = mutate(
            &target,
            policy(0, false, false),
            |_, _| -> Result<(Mutation, ()), Fault> { unreachable!() },
        )
        .unwrap_err();
        match err {
            Fault::Reason { id, params } => {
                assert_eq!(id, "E-FILE-MISSING");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].0, "file");
            }
            other => panic!("unexpected fault: {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn mutate_dangling_symlink_retries_are_bounded() {
        const CHILD_DIR: &str = "COEXIST_FSX_DANGLING_SYMLINK_CHILD_DIR";
        if let Some(dir) = std::env::var_os(CHILD_DIR) {
            let dir = PathBuf::from(dir);
            let target = dir.join("x.agent-besties.md");
            // Zero budget exercises fastfail; a practically unlimited budget
            // must still stop at the lifecycle retry cap, not spin forever.
            for wait_ms in [0, u64::MAX] {
                let err = mutate::<()>(&target, policy(wait_ms, true, true), |_, _| {
                    panic!("a surviving dangling symlink must never reach the plan")
                })
                .unwrap_err();
                match err {
                    Fault::Io { action, source, .. } => {
                        assert_eq!(action, "create");
                        assert_eq!(source.kind(), io::ErrorKind::AlreadyExists);
                    }
                    other => panic!("unexpected fault: {other:?}"),
                }
            }
            std::fs::write(dir.join("checked"), b"").unwrap();
            return;
        }

        let dir = scratch("dangling-symlink");
        let target = dir.join("x.agent-besties.md");
        let referent = Path::new("missing.agent-besties.md");
        #[cfg(unix)]
        std::os::unix::fs::symlink(referent, &target).unwrap();
        #[cfg(windows)]
        if let Err(error) = std::os::windows::fs::symlink_file(referent, &target) {
            if error.raw_os_error() == Some(1314) {
                // Windows runners without symlink privilege cannot exercise
                // this filesystem input; do not replace it with a mock.
                std::fs::remove_dir_all(&dir).unwrap();
                return;
            }
            panic!("could not create dangling symlink: {error}");
        }
        // Isolate the historical infinite loop in a killable process. A
        // regression must fail this test rather than strand the test runner.
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "fsx::tests::mutate_dangling_symlink_retries_are_bounded",
                "--nocapture",
            ])
            .env(CHILD_DIR, &dir)
            .spawn()
            .unwrap();
        let start = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if start.elapsed() >= Duration::from_secs(10) {
                child.kill().unwrap();
                child.wait().unwrap();
                break None;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let checked = dir.join("checked").is_file();
        let link = std::fs::read_link(&target);
        let referent_state = std::fs::symlink_metadata(dir.join(referent));
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(status.is_some(), "dangling-symlink mutation did not finish");
        assert!(status.unwrap().success(), "child mutation checks failed");
        assert!(checked, "child did not exercise the mutation");
        assert_eq!(link.unwrap(), referent);
        assert_eq!(referent_state.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn mutate_splices_existing_content() {
        let dir = scratch("splice");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"old-bytes").unwrap();
        let res = mutate(&target, policy(200, true, true), |bytes, initial| {
            assert_eq!(bytes, b"old-bytes");
            assert!(!initial);
            Ok((Mutation::Write(b"new-bytes".to_vec()), bytes.len()))
        })
        .unwrap();
        assert!(!res.created);
        assert_eq!(res.value, 9usize);
        assert_eq!(std::fs::read(&target).unwrap(), b"new-bytes");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mutate_shorter_write_removes_old_tail() {
        let dir = scratch("shorter");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"0123456789").unwrap();
        mutate(&target, policy(200, false, false), |_, _| {
            Ok((Mutation::Write(b"abc".to_vec()), ()))
        })
        .unwrap();
        // Final content contains only the replacement, with no old tail.
        assert_eq!(std::fs::read(&target).unwrap(), b"abc");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mutate_delete_removes_file_under_lock() {
        let dir = scratch("delete");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"# .state\n").unwrap();
        let res = mutate(&target, policy(200, false, false), |_, _| {
            Ok((Mutation::Delete, ()))
        })
        .unwrap();
        assert!(res.deleted);
        assert!(!res.created);
        assert!(!target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn mutate_delete_preserves_unlink_permission_error_and_bytes() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("delete-permission");
        let target = dir.join("x.agent-besties.md");
        let original =
            b"# .state\n\n## locked\n\nalice 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n";
        std::fs::write(&target, original).unwrap();
        let original_perms = std::fs::metadata(&dir).unwrap().permissions();
        let mut readonly_perms = original_perms.clone();
        readonly_perms.set_mode(0o500);
        let probe = dir.join("unlink-probe");
        std::fs::write(&probe, b"probe").unwrap();
        std::fs::set_permissions(&dir, readonly_perms.clone()).unwrap();
        let precondition = std::fs::remove_file(&probe);
        std::fs::set_permissions(&dir, original_perms.clone()).unwrap();
        let denied = match precondition {
            Ok(()) => {
                // Root/capabilities can bypass parent directory permissions.
                std::fs::remove_dir_all(&dir).unwrap();
                return;
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => error,
            Err(error) => panic!("unexpected unlink precondition failure: {error}"),
        };
        std::fs::remove_file(&probe).unwrap();
        std::fs::set_permissions(&dir, readonly_perms).unwrap();
        let outcome = mutate(&target, policy(0, false, false), |_, _| {
            Ok((Mutation::Delete, ()))
        });
        // Restore access before inspecting failures or cleaning the fixture.
        std::fs::set_permissions(&dir, original_perms).unwrap();
        match outcome.unwrap_err() {
            Fault::Io { action, source, .. } => {
                assert_eq!(action, "delete");
                assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
                assert_eq!(source.raw_os_error(), denied.raw_os_error());
            }
            other => panic!("unexpected fault: {other:?}"),
        }
        assert_eq!(std::fs::read(&target).unwrap(), original);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mutate_plan_failure_cleans_up_own_creation() {
        let dir = scratch("cleanup");
        let target = dir.join("a/b/x.agent-besties.md");
        let err = mutate(
            &target,
            policy(200, true, true),
            |_, _| -> Result<(Mutation, ()), Fault> { Err(Fault::reason("E-FLOCKED")) },
        )
        .unwrap_err();
        match err {
            Fault::Reason { id, .. } => assert_eq!(id, "E-FLOCKED"),
            other => panic!("unexpected fault: {other:?}"),
        }
        assert!(!target.exists());
        assert!(!dir.join("a/b").exists());
        assert!(!dir.join("a").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mutate_empty_existing_file_without_create_reports_eh1() {
        let dir = scratch("empty-no-create");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"").unwrap();
        let err = mutate(
            &target,
            policy(0, false, false),
            |_, _| -> Result<(Mutation, ()), Fault> { unreachable!() },
        )
        .unwrap_err();
        match err {
            Fault::Diagnostics(diags) => {
                assert_eq!(diags.len(), 1);
                assert_eq!(diags[0].id, "E-H1");
                assert_eq!(diags[0].line, 1);
            }
            other => panic!("unexpected fault: {other:?}"),
        }
        // The pre-existing empty file must be left in place.
        assert!(target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mutate_takes_over_abandoned_empty_file_at_deadline() {
        let dir = scratch("takeover");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"").unwrap();
        let res = mutate(&target, policy(0, true, true), |bytes, initial| {
            assert!(bytes.is_empty());
            assert!(initial);
            Ok((Mutation::Write(b"# x.post\n".to_vec()), ()))
        })
        .unwrap();
        assert!(res.created);
        assert_eq!(std::fs::read(&target).unwrap(), b"# x.post\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_returns_bytes_and_reports_busy_as_elocked() {
        let dir = scratch("read");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"payload").unwrap();
        assert_eq!(read(&target, 100, None).unwrap(), b"payload");

        // An exclusive lock held on a second handle blocks the shared read
        // (separate handles conflict on both Unix flock and LockFileEx).
        let holder = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&target)
            .unwrap();
        holder.try_lock().unwrap();
        let err = read(&target, 0, None).unwrap_err();
        match err {
            Fault::Reason { id, .. } => assert_eq!(id, "E-LOCKED"),
            other => panic!("unexpected fault: {other:?}"),
        }
        holder.unlock().unwrap();
        drop(holder);
        assert_eq!(read(&target, 100, None).unwrap(), b"payload");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    fn make_fifo(path: &Path) {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        // SAFETY: path is a live, NUL-terminated pathname for this call.
        let result = unsafe { libc::mkfifo(path.as_ptr(), 0o600) };
        assert_eq!(result, 0, "mkfifo failed: {}", io::Error::last_os_error());
    }

    #[cfg(unix)]
    #[test]
    fn fifo_reads_and_replacement_checks_are_bounded() {
        use std::os::unix::fs::FileTypeExt;
        const CHILD_DIR: &str = "COEXIST_FSX_FIFO_CHILD_DIR";
        if let Some(dir) = std::env::var_os(CHILD_DIR) {
            let dir = PathBuf::from(dir);
            let target = dir.join("pipe.agent-besties.md");
            // No writer is ever opened. Each public read boundary must reject
            // the object, not block or return a successful empty snapshot.
            for fault in [
                read(&target, 0, None).unwrap_err(),
                read_stable_optional(&target, None).unwrap_err(),
                mutate::<()>(&target, policy(0, false, false), |_, _| {
                    panic!("a FIFO must not reach a mutation plan")
                })
                .unwrap_err(),
            ] {
                assert!(matches!(
                    fault,
                    Fault::Io { source, .. } if source.kind() == io::ErrorKind::Unsupported
                ));
            }
            assert_eq!(
                crate::hash::read_target(&target).unwrap_err().kind(),
                io::ErrorKind::Unsupported
            );

            // A successful initial open is not enough: rollback reopens the
            // path for identity after the plan can replace it with a FIFO.
            let replaced = dir.join("created/notes.agent-besties.md");
            let fault = mutate::<()>(&replaced, policy(0, true, true), |_, _| {
                std::fs::remove_file(&replaced).unwrap();
                make_fifo(&replaced);
                Err(Fault::reason("E-FLOCKED"))
            })
            .unwrap_err();
            assert!(matches!(
                fault,
                Fault::Reason {
                    id: "E-FLOCKED",
                    ..
                }
            ));
            assert!(std::fs::symlink_metadata(&replaced)
                .unwrap()
                .file_type()
                .is_fifo());
            assert!(dir.join("created").is_dir());
            std::fs::write(dir.join("checked"), b"").unwrap();
            return;
        }

        let dir = scratch("fifo-reads");
        let target = dir.join("pipe.agent-besties.md");
        make_fifo(&target);
        // Keep a historical blocking open from stranding the test runner.
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "fsx::tests::fifo_reads_and_replacement_checks_are_bounded",
                "--nocapture",
            ])
            .env(CHILD_DIR, &dir)
            .spawn()
            .unwrap();
        let start = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if start.elapsed() >= Duration::from_secs(10) {
                child.kill().unwrap();
                child.wait().unwrap();
                break None;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let checked = dir.join("checked").is_file();
        let still_fifo = std::fs::symlink_metadata(&target)
            .unwrap()
            .file_type()
            .is_fifo();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(
            status.is_some(),
            "FIFO read or identity check did not finish"
        );
        assert!(status.unwrap().success(), "child FIFO checks failed");
        assert!(checked, "child did not exercise the FIFO boundaries");
        assert!(still_fifo, "unsupported input was changed");
    }

    #[test]
    fn read_missing_file_is_real_io_not_busy() {
        let dir = scratch("read-missing");
        let target = dir.join("none.agent-besties.md");
        let err = read(&target, 0, None).unwrap_err();
        match err {
            Fault::Io { action, source, .. } => {
                assert_eq!(action, "open");
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("unexpected fault: {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mutate_plan_failure_never_deletes_replacement_file() {
        let dir = scratch("rollback-replaced");
        let target = dir.join("x.agent-besties.md");
        let err = mutate::<()>(&target, policy(200, true, false), |_, _| {
            // A non-cooperating writer replaces the path while we hold
            // the lock on the original: rename foreign content over the
            // target, so the path no longer names our handle's file.
            let staging = dir.join("staging.tmp");
            std::fs::write(&staging, b"# foreign.post\n").unwrap();
            std::fs::rename(&staging, &target).unwrap();
            Err(Fault::reason("E-FLOCKED"))
        })
        .unwrap_err();
        match &err {
            Fault::Reason { id, .. } => assert_eq!(*id, "E-FLOCKED"),
            other => panic!("unexpected fault: {other:?}"),
        }
        // The rollback re-verifies identity on the held handle: a path
        // that names someone else's file is never unlinked.
        assert_eq!(std::fs::read(&target).unwrap(), b"# foreign.post\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn successful_plan_refuses_replacement_before_write_or_delete() {
        for delete in [false, true] {
            let dir = scratch("successful-plan-replaced");
            let target = dir.join("x.agent-besties.md");
            let detached = dir.join("detached.tmp");
            std::fs::write(&target, b"original").unwrap();
            let err = mutate(&target, policy(0, false, false), |_, _| {
                std::fs::rename(&target, &detached).unwrap();
                std::fs::write(&target, b"foreign").unwrap();
                let mutation = if delete {
                    Mutation::Delete
                } else {
                    Mutation::Write(b"planned replacement".to_vec())
                };
                Ok((mutation, ()))
            })
            .unwrap_err();
            assert!(matches!(err, Fault::Reason { id: "E-LOCKED", .. }));
            assert_eq!(std::fs::read(&target).unwrap(), b"foreign");
            assert_eq!(std::fs::read(&detached).unwrap(), b"original");
            std::fs::remove_dir_all(&dir).unwrap();
        }
    }

    #[test]
    fn successful_plan_refuses_missing_path_before_write() {
        let dir = scratch("successful-plan-missing");
        let target = dir.join("x.agent-besties.md");
        let detached = dir.join("detached.tmp");
        std::fs::write(&target, b"original").unwrap();
        let err = mutate(&target, policy(0, false, false), |_, _| {
            std::fs::rename(&target, &detached).unwrap();
            Ok((Mutation::Write(b"planned replacement".to_vec()), ()))
        })
        .unwrap_err();
        assert!(matches!(err, Fault::Reason { id: "E-LOCKED", .. }));
        assert!(!target.exists());
        assert_eq!(std::fs::read(&detached).unwrap(), b"original");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mutate_plan_failure_keeps_occupied_created_dir() {
        let dir = scratch("rollback-occupied");
        let target = dir.join("a/b/x.agent-besties.md");
        let err = mutate::<()>(&target, policy(200, true, true), |_, _| {
            // Foreign content lands in a directory this call created:
            // cleanup must keep the directory — only still-empty
            // creations are removed.
            std::fs::create_dir_all(dir.join("a/b")).unwrap();
            std::fs::write(dir.join("a/b/keep.txt"), b"kept").unwrap();
            Err(Fault::reason("E-FLOCKED"))
        })
        .unwrap_err();
        match &err {
            Fault::Reason { id, .. } => assert_eq!(*id, "E-FLOCKED"),
            other => panic!("unexpected fault: {other:?}"),
        }
        assert!(!target.exists());
        assert_eq!(std::fs::read(dir.join("a/b/keep.txt")).unwrap(), b"kept");
        assert!(dir.join("a/b").exists());
        assert!(dir.join("a").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn directory_cleanup_preserves_foreign_empty_replacement() {
        let dir = scratch("rollback-replaced-dir");
        let target = dir.join("a/x.agent-besties.md");
        let mut created_dirs = Vec::new();
        ensure_parents(&target, &mut created_dirs).unwrap();
        std::fs::rename(dir.join("a"), dir.join("moved")).unwrap();
        std::fs::create_dir(dir.join("a")).unwrap();
        remove_empty_created_dirs(created_dirs).unwrap();
        assert!(dir.join("a").is_dir());
        assert!(dir.join("moved").is_dir());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn first_write_failure_cleans_verified_empty_creation() {
        let dir = scratch("rollback-first-write");
        let target = dir.join("a/b/x.agent-besties.md");
        let mut created_dirs = Vec::new();
        ensure_parents(&target, &mut created_dirs).unwrap();
        let creator = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .unwrap();
        // A read-only handle gives a real, deterministic first-write error
        // without process-global quotas, permission assumptions or hooks.
        let readonly = File::open(&target).unwrap();
        drop(creator);
        let file = lock_exclusive_until(readonly, &target, &Deadline::new(0)).unwrap();
        let err = run_plan(
            file,
            &target,
            Vec::new(),
            true,
            true,
            created_dirs,
            &mut |_, _| Ok((Mutation::Write(b"content".to_vec()), ())),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            Fault::Io {
                action: "write",
                ..
            }
        ));
        assert!(!target.exists());
        assert!(!dir.join("a").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn failed_write_cleanup_preserves_partial_bytes_and_directories() {
        let dir = scratch("rollback-partial-write");
        let target = dir.join("a/b/x.agent-besties.md");
        let mut created_dirs = Vec::new();
        ensure_parents(&target, &mut created_dirs).unwrap();
        let creator = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&target)
            .unwrap();
        let mut file = lock_exclusive_until(creator, &target, &Deadline::new(0)).unwrap();
        file.0.write_all(b"partial").unwrap();
        // Exercise cleanup with genuine partial bytes and a real failing
        // write. A mandatory lock may reject the second handle before its
        // read-only access does; neither failure may discard the prefix.
        let mut readonly = File::open(&target).unwrap();
        let fault = write_fault(&target, readonly.write_all(b"unwritten").unwrap_err());
        drop(readonly);
        let err = fail_with_rollback(fault, Some(file), &target, true, created_dirs);
        assert!(matches!(
            err,
            Fault::Io {
                action: "write",
                ..
            } | Fault::Reason { id: "E-LOCKED", .. }
        ));
        assert_eq!(std::fs::read(&target).unwrap(), b"partial");
        assert!(dir.join("a/b").is_dir());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mutate_oversize_result_rolls_back_creation() {
        let dir = scratch("rollback-toolarge");
        let target = dir.join("a/b/x.agent-besties.md");
        let err = mutate(
            &target,
            policy(200, true, true),
            |_, _| -> Result<(Mutation, ()), Fault> {
                Ok((
                    Mutation::Write(vec![0u8; consts::MAX_MANAGED_FILE_BYTES + 1]),
                    (),
                ))
            },
        )
        .unwrap_err();
        match &err {
            Fault::Diagnostics(ds) => {
                assert_eq!(ds.len(), 1);
                assert_eq!(ds[0].id, "E-TOO-LARGE");
            }
            other => panic!("unexpected fault: {other:?}"),
        }
        // Refused before any byte: the creation leaves no trace.
        assert!(!target.exists());
        assert!(!dir.join("a/b").exists());
        assert!(!dir.join("a").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A write that runs into the process file-size limit (a real `EFBIG`
    /// from the kernel, not a mock) must leave an existing file with its
    /// previous bytes, including when the plan also changes a byte inside
    /// them, and must leave no trace of a creation. The historical order
    /// (rewrite from offset 0) left the new prefix and a cut-off tail.
    #[cfg(unix)]
    #[test]
    fn file_size_limit_failure_keeps_previous_bytes_and_removes_creation() {
        const CHILD_DIR: &str = "COEXIST_FSX_FSIZE_CHILD_DIR";
        const LIMIT: u64 = 4096;
        if let Some(dir) = std::env::var_os(CHILD_DIR) {
            let dir = PathBuf::from(dir);
            // SAFETY: plain libc calls on this single-test child process;
            // ignoring SIGXFSZ turns the limit into an `EFBIG` result.
            unsafe {
                libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
                let mut limit: libc::rlimit = std::mem::zeroed();
                assert_eq!(libc::getrlimit(libc::RLIMIT_FSIZE, &mut limit), 0);
                limit.rlim_cur = LIMIT as libc::rlim_t;
                assert_eq!(libc::setrlimit(libc::RLIMIT_FSIZE, &limit), 0);
            }
            let is_limit = |fault: &Fault| {
                matches!(fault, Fault::Io { action: "write", source, .. }
                    if source.kind() == io::ErrorKind::FileTooLarge)
            };
            let target = dir.join("x.agent-besties.md");
            let original = std::fs::read(&target).unwrap();
            for change_inside in [false, true] {
                let fault = mutate::<()>(&target, policy(0, false, false), |bytes, _| {
                    let mut new = bytes.to_vec();
                    if change_inside {
                        new[12] = b'!';
                    }
                    new.resize(bytes.len() + 5000, b'b');
                    Ok((Mutation::Write(new), ()))
                })
                .unwrap_err();
                assert!(is_limit(&fault), "{fault:?}");
                assert_eq!(std::fs::read(&target).unwrap(), original);
            }
            let created = dir.join("a/b/new.agent-besties.md");
            let fault = mutate::<()>(&created, policy(0, true, true), |_, _| {
                Ok((Mutation::Write(vec![b'c'; 5000]), ()))
            })
            .unwrap_err();
            assert!(is_limit(&fault), "{fault:?}");
            assert!(!dir.join("a").exists());
            std::fs::write(dir.join("checked"), b"").unwrap();
            return;
        }

        let dir = scratch("fsize-limit");
        let target = dir.join("x.agent-besties.md");
        let original = b"# x.post\n\n## #1 alice (2026-09-17T09:12:00Z)\n\n```md\nhi\n```\n";
        std::fs::write(&target, original).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "fsx::tests::file_size_limit_failure_keeps_previous_bytes_and_removes_creation",
                "--nocapture",
            ])
            .env(CHILD_DIR, &dir)
            .status()
            .unwrap();
        let checked = dir.join("checked").is_file();
        let content = std::fs::read(&target).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(status.success(), "child write-failure checks failed");
        assert!(checked, "child did not exercise the writes");
        assert_eq!(content, original);
    }

    #[test]
    fn read_stable_optional_distinguishes_absent_zero_and_present() {
        let dir = scratch("stable");
        let target = dir.join("x.agent-besties.md");
        // Absent with a valid parent -> None, file not created.
        assert!(read_stable_optional(&target, None).unwrap().is_none());
        assert!(!target.exists());
        // Present zero-byte -> Some(empty): absence is never faked.
        std::fs::write(&target, b"").unwrap();
        assert_eq!(
            read_stable_optional(&target, None).unwrap(),
            Some(Vec::new())
        );
        // Present content -> Some(bytes).
        std::fs::write(&target, b"body").unwrap();
        assert_eq!(
            read_stable_optional(&target, None).unwrap(),
            Some(b"body".to_vec())
        );
        // Parent gone -> error, not None.
        let nested = dir.join("gone/x.agent-besties.md");
        std::fs::create_dir(dir.join("gone")).unwrap();
        std::fs::write(&nested, b"body").unwrap();
        std::fs::remove_file(&nested).unwrap();
        std::fs::remove_dir(dir.join("gone")).unwrap();
        let err = read_stable_optional(&nested, None).unwrap_err();
        match err {
            Fault::Io { action, .. } => assert_eq!(action, "open"),
            other => panic!("unexpected fault: {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn stable_read_rejects_dangling_replacement_without_faking_removal() {
        #[cfg(unix)]
        use std::os::unix::fs::symlink as symlink_file;
        #[cfg(windows)]
        use std::os::windows::fs::symlink_file;

        let dir = scratch("stable-dangling");
        let target = dir.join("notes.agent-besties.md");
        let referent = dir.join("ordinary.agent-besties.md");
        let bytes = b"# notes.post\n";
        std::fs::write(&target, bytes).unwrap();
        assert_eq!(
            read_stable_optional(&target, None).unwrap(),
            Some(bytes.to_vec())
        );

        // Following a symlink to an ordinary file remains supported by all
        // three read entry points; rejecting every symlink is not the fix.
        std::fs::rename(&target, &referent).unwrap();
        if let Err(error) = symlink_file("ordinary.agent-besties.md", &target) {
            #[cfg(windows)]
            if error.raw_os_error() == Some(1314) {
                std::fs::remove_dir_all(&dir).unwrap();
                return;
            }
            panic!("could not create file symlink: {error}");
        }
        assert_eq!(read(&target, 0, None).unwrap(), bytes);
        assert_eq!(
            read_stable_optional(&target, None).unwrap(),
            Some(bytes.to_vec())
        );
        assert_eq!(
            crate::hash::read_target(&target)
                .unwrap()
                .content
                .as_deref(),
            Some("# notes.post\n")
        );

        std::fs::remove_file(&target).unwrap();
        symlink_file("missing.agent-besties.md", &target).unwrap();
        assert!(matches!(
            read_stable_optional(&target, None),
            Err(Fault::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound
        ));
        assert_eq!(
            std::fs::read_link(&target).unwrap(),
            Path::new("missing.agent-besties.md")
        );
        assert_eq!(std::fs::read(&referent).unwrap(), bytes);

        // Only removing the actual directory entry restores genuine absence.
        std::fs::remove_file(&target).unwrap();
        assert!(read_stable_optional(&target, None).unwrap().is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn huge_wait_budgets_do_not_panic() {
        // u64::MAX milliseconds must be a legal budget: `Instant +
        // Duration` would overflow-panic (notably on Windows), the
        // elapsed/remaining accounting must not.
        let dir = scratch("huge-budget");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"payload").unwrap();
        assert_eq!(read(&target, u64::MAX, None).unwrap(), b"payload");
        let res = mutate(&target, policy(u64::MAX, false, false), |bytes, initial| {
            assert_eq!(bytes, b"payload");
            assert!(!initial);
            Ok((Mutation::Write(b"replaced".to_vec()), ()))
        })
        .unwrap();
        assert!(!res.created);
        assert_eq!(std::fs::read(&target).unwrap(), b"replaced");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_failure_surfaces_instead_of_the_plan_fault() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("cleanup-visible");
        let target = dir.join("x.agent-besties.md");
        let original_perms = std::fs::metadata(&dir).unwrap().permissions();
        let mut readonly_perms = original_perms.clone();
        readonly_perms.set_mode(0o500);
        let probe = dir.join("unlink-probe");
        std::fs::write(&probe, b"probe").unwrap();
        std::fs::set_permissions(&dir, readonly_perms.clone()).unwrap();
        let precondition = std::fs::remove_file(&probe);
        std::fs::set_permissions(&dir, original_perms.clone()).unwrap();
        match precondition {
            Ok(()) => {
                // Root/capabilities may bypass mode bits. This fixture can
                // prove cleanup errors only when the OS actually denies unlink.
                std::fs::remove_dir_all(&dir).unwrap();
                return;
            }
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                std::fs::remove_file(&probe).unwrap();
            }
            Err(e) => panic!("unexpected unlink precondition failure: {e}"),
        }
        let outcome = mutate(
            &target,
            policy(200, true, false),
            |_, _| -> Result<(Mutation, ()), Fault> {
                std::fs::set_permissions(&dir, readonly_perms.clone()).unwrap();
                Err(Fault::reason("E-FLOCKED"))
            },
        );
        // Restore access before asserting, including on an unexpected outcome.
        std::fs::set_permissions(&dir, original_perms).unwrap();
        match outcome.unwrap_err() {
            // The real cleanup failure must win over the caller's E-FLOCKED.
            Fault::Io { action, source, .. } => {
                assert_eq!(action, "delete");
                assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
            }
            other => panic!("unexpected fault: {other:?}"),
        }
        assert!(target.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
