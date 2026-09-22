//! Managed-file lifecycle: locked reads and locked splice writes.
//!
//! This module is the single place that talks to the filesystem for
//! collaboration files (`*.agent-besties.md`). It implements the lock
//! protocol of spec §10.2:
//!
//! * reads: open (read-only) → shared lock → read all bytes (bounded by
//!   `MAX_MANAGED_FILE_BYTES`) → release. Never reads without holding the
//!   lock.
//! * writes: open (read+write) or `create_new` → exclusive lock → same-file
//!   identity re-check → read all bytes → run the caller's plan → `seek(0)` →
//!   `write_all` → `set_len` (write before truncate) → `flush` → `sync_data` →
//!   release. Deletion (`Mutation::Delete`) happens on the held lock via
//!   `remove_file`, never via truncation.
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

pub mod walk;

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

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
        match File::open(path) {
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
        match File::open(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Real absence requires the parent directory to still be valid;
                // a deleted parent is an error, not "absent".
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
    /// Replace the file's content with these bytes (write before truncate).
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

/// Run `plan` under an exclusive lock on the real target file.
///
/// `plan(bytes, initial)` receives the file's current bytes (empty in
/// initial-content mode) and `initial == true` exactly when this call is
/// supposed to produce complete initial content (fresh creation or takeover
/// of an abandoned zero-byte file); it returns the splice result and a value
/// to pass through. `plan` must not re-create content or perform I/O on the
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
///   (`OPEN_RETRIES`, only identity changes count);
/// * held-lock deletion for `Mutation::Delete`;
/// * creation-failure cleanup: if `plan` fails on a file this call just
///   `create_new`-created and that is still empty, the file is deleted on
///   the held lock and the directories created by this call (still empty)
///   are removed inner→outer, so a rejected `profile`/`brief` leaves no
///   trace (§10.8.3);
/// * write before `set_len`, then `flush` and `sync_data`.
pub fn mutate<T>(
    path: &Path,
    policy: WritePolicy,
    mut plan: impl FnMut(Vec<u8>, bool) -> Result<(Mutation, T), Fault>,
) -> Result<WriteResult<T>, Fault> {
    let deadline = Deadline::new(policy.wait_ms);
    let mut identity_reopens = 0usize;
    let mut created_dirs: Vec<PathBuf> = Vec::new();
    let mut parents_ensured = false;
    // True only while the file on disk is one this call created via
    // `create_new` and nobody has written to it yet.
    let mut created_by_us = false;

    loop {
        match OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => {
                lock_exclusive_until(&file, path, &deadline)?;
                if !same_file_identity(&file, path)? {
                    // Deleted or replaced under us: back to the create dispatch.
                    drop(file);
                    created_by_us = false;
                    identity_reopens += 1;
                    if identity_reopens > consts::OPEN_RETRIES {
                        return Err(Fault::reason("E-LOCKED"));
                    }
                    continue;
                }
                let bytes = read_all_bounded(&file).map_err(|e| read_fault(path, e))?;
                if bytes.is_empty() {
                    if deadline.expired() {
                        if policy.create {
                            // Abandoned creation: we hold the lock, write the
                            // complete initial content now (§10.2.3). A file not
                            // created by us is never deleted on later failure.
                            return run_plan(
                                file,
                                path,
                                Vec::new(),
                                true,
                                created_by_us,
                                created_dirs,
                                &mut plan,
                            );
                        }
                        return Err(Fault::Diagnostics(vec![
                            Diagnostic::new("E-H1", 1).with("line", "")
                        ]));
                    }
                    // Release the lock, wait one poll, reopen; zero-byte retries
                    // do not consume the identity-reopen budget.
                    drop(file);
                    created_by_us = false;
                    sleep_one_poll(&deadline);
                    continue;
                }
                return run_plan(file, path, bytes, false, false, created_dirs, &mut plan);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if !policy.create {
                    // §10.1.9/§10.9: displayed paths fold whitespace runs;
                    // display_text also keeps the value single-line safe.
                    return Err(Fault::reason("E-FILE-MISSING")
                        .with("file", names::display_text(&walk::display_path(path))));
                }
                if policy.create_parents && !parents_ensured {
                    ensure_parents(path, &mut created_dirs)?;
                    parents_ensured = true;
                }
                match OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(path)
                {
                    Ok(file) => {
                        lock_exclusive_until(&file, path, &deadline)?;
                        // Our freshly created file can already have been
                        // deleted and recreated by someone else before we
                        // locked it; mutating our handle would then write an
                        // unlinked file nobody ever sees. Same identity rule
                        // as the existing path (§10.2.6).
                        if !same_file_identity(&file, path)? {
                            drop(file);
                            created_by_us = false;
                            identity_reopens += 1;
                            if identity_reopens > consts::OPEN_RETRIES {
                                return Err(Fault::reason("E-LOCKED"));
                            }
                            continue;
                        }
                        // Mandatory re-read: a non-empty file means another
                        // writer took over our creation.
                        let bytes = read_all_bounded(&file).map_err(|e| read_fault(path, e))?;
                        if bytes.is_empty() {
                            return run_plan(
                                file,
                                path,
                                Vec::new(),
                                true,
                                true,
                                created_dirs,
                                &mut plan,
                            );
                        }
                        return run_plan(file, path, bytes, false, false, Vec::new(), &mut plan);
                    }
                    Err(e2) if e2.kind() == io::ErrorKind::AlreadyExists => {
                        // Concurrent creation: fall through to the existing path.
                        continue;
                    }
                    Err(e2) if cfg!(windows) && is_win_error(&e2, WIN_ERROR_ACCESS_DENIED) => {
                        // Possible pending delete: bounded retry (§10.2.6).
                        identity_reopens += 1;
                        if identity_reopens > consts::OPEN_RETRIES {
                            return Err(Fault::io("create", path.to_path_buf(), e2));
                        }
                        sleep_one_poll(&deadline);
                        continue;
                    }
                    Err(e2) if is_win_error(&e2, WIN_ERROR_SHARING_VIOLATION) => {
                        return Err(Fault::reason("E-INUSE"));
                    }
                    Err(e2) => return Err(Fault::io("create", path.to_path_buf(), e2)),
                }
            }
            Err(e) if cfg!(windows) && is_win_error(&e, WIN_ERROR_ACCESS_DENIED) => {
                identity_reopens += 1;
                if identity_reopens > consts::OPEN_RETRIES {
                    return Err(Fault::io("open", path.to_path_buf(), e));
                }
                sleep_one_poll(&deadline);
                continue;
            }
            Err(e) if is_win_error(&e, WIN_ERROR_SHARING_VIOLATION) => {
                return Err(Fault::reason("E-INUSE"));
            }
            Err(e) => return Err(Fault::io("open", path.to_path_buf(), e)),
        }
    }
}

/// Apply the plan's mutation to a locked file handle.
///
/// `initial` is the create/takeover flag passed to `plan`; `created_by_us`
/// drives the creation-failure cleanup. `created_dirs` is reported on
/// success and used for cleanup on failure.
fn run_plan<T>(
    mut file: File,
    path: &Path,
    bytes: Vec<u8>,
    initial: bool,
    created_by_us: bool,
    created_dirs: Vec<PathBuf>,
    plan: &mut dyn FnMut(Vec<u8>, bool) -> Result<(Mutation, T), Fault>,
) -> Result<WriteResult<T>, Fault> {
    let outcome = plan(bytes, initial);
    let (mutation, value) = match outcome {
        Ok(ok) => ok,
        Err(fault) => {
            // Creation-failure cleanup (§10.8.3): remove only what this
            // call created — a still-empty file we just `create_new`d
            // (deleted on the held lock), then the directories we created,
            // but only once the handle is released: a Windows pending-delete
            // handle keeps the directory entry alive and would make
            // `remove_dir` fail. Real cleanup failures surface as
            // `Fault::Io` instead of hiding behind the caller's fault; an
            // adopted pre-existing zero-byte file is never deleted.
            if created_by_us {
                let file_was_empty = match file.metadata() {
                    Ok(md) => md.len() == 0,
                    Err(e) => return Err(Fault::io("delete", path.to_path_buf(), e)),
                };
                if file_was_empty {
                    if let Err(e) = std::fs::remove_file(path) {
                        return Err(Fault::io("delete", path.to_path_buf(), e));
                    }
                }
                drop(file);
                for dir in created_dirs.iter().rev() {
                    match std::fs::read_dir(dir) {
                        Ok(mut entries) => match entries.next() {
                            // Owned-directory cleanup: only a directory this
                            // call created, and only while it is still empty.
                            None => {
                                if let Err(e) = std::fs::remove_dir(dir) {
                                    return Err(Fault::io("delete", dir.clone(), e));
                                }
                            }
                            // A real directory-entry error is a cleanup
                            // failure, not an occupied directory.
                            Some(Err(e)) => return Err(Fault::io("delete", dir.clone(), e)),
                            Some(Ok(_)) => {}
                        },
                        Err(e) => return Err(Fault::io("delete", dir.clone(), e)),
                    }
                }
            }
            return Err(fault);
        }
    };
    match mutation {
        Mutation::Write(new_bytes) => {
            if new_bytes.len() > consts::MAX_MANAGED_FILE_BYTES {
                // Rejected before any byte is touched (§10.2.2).
                return Err(too_large_fault());
            }
            let new_len = new_bytes.len() as u64;
            write_step(path, || file.seek(SeekFrom::Start(0)).map(|_| ()))?;
            // Write before truncate: a crash leaves "new content + old tail"
            // instead of wiping old comments (§10.2.2, §13 Q12).
            write_step(path, || file.write_all(&new_bytes))?;
            write_step(path, || file.set_len(new_len))?;
            write_step(path, || file.flush())?;
            write_step(path, || file.sync_data())?;
            Ok(WriteResult {
                value,
                created: initial,
                created_dirs,
                deleted: false,
            })
        }
        Mutation::Unchanged => Ok(WriteResult {
            value,
            created: initial,
            created_dirs,
            deleted: false,
        }),
        Mutation::Delete => {
            // Delete on the held lock; never truncate first (§10.2.7). Our
            // handle was opened with FILE_SHARE_DELETE on Windows.
            std::fs::remove_file(path).map_err(|e| {
                if is_write_stage_busy(&e) {
                    Fault::reason("E-LOCKED")
                } else {
                    Fault::io("delete", path.to_path_buf(), e)
                }
            })?;
            drop(file);
            Ok(WriteResult {
                value,
                created: false,
                created_dirs,
                deleted: true,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// internals
// ---------------------------------------------------------------------------

/// Execute one step of the write sequence, mapping lock violations to
/// `E-LOCKED` and everything else to `Fault::Io`.
fn write_step(path: &Path, step: impl FnOnce() -> io::Result<()>) -> Result<(), Fault> {
    step().map_err(|e| {
        if is_write_stage_busy(&e) {
            Fault::reason("E-LOCKED")
        } else {
            Fault::io("write", path.to_path_buf(), e)
        }
    })
}

/// Poll `try_lock` until `deadline` (exclusive).
fn lock_exclusive_until(file: &File, path: &Path, deadline: &Deadline) -> Result<(), Fault> {
    loop {
        match file.try_lock() {
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
    match same_file::Handle::from_path(path) {
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

/// Create the missing parent chain of `path`, outer→inner, recording what
/// this call created. A directory that appears concurrently is fine (not
/// recorded); real failures surface as `Fault::Io`.
fn ensure_parents(path: &Path, created: &mut Vec<PathBuf>) -> Result<(), Fault> {
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
            Ok(()) => created.push(dir),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(Fault::io("create", dir, e)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn mutate_shorter_write_truncates_after_writing() {
        let dir = scratch("shorter");
        let target = dir.join("x.agent-besties.md");
        std::fs::write(&target, b"0123456789").unwrap();
        mutate(&target, policy(200, false, false), |_, _| {
            Ok((Mutation::Write(b"abc".to_vec()), ()))
        })
        .unwrap();
        // The write-then-set_len order must leave exactly the new bytes.
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
        let err = mutate(
            &target,
            policy(200, true, false),
            |_, _| -> Result<(Mutation, ()), Fault> {
                // Flip the parent to read-only inside the plan (after
                // create_new, under the held lock) so the mandated cleanup
                // cannot unlink the freshly created empty file.
                let mut perms = std::fs::metadata(&dir).unwrap().permissions();
                perms.set_mode(0o500);
                std::fs::set_permissions(&dir, perms).unwrap();
                Err(Fault::reason("E-FLOCKED"))
            },
        )
        .unwrap_err();
        match err {
            // The plan fault is E-FLOCKED, but the mandated cleanup could
            // not complete: the real I/O failure wins instead of hiding
            // behind the caller's fault.
            Fault::Io { action, source, .. } => {
                assert_eq!(action, "delete");
                assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
            }
            other => panic!("unexpected fault: {other:?}"),
        }
        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(&dir, perms).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
