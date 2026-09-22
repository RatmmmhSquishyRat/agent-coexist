//! Real-process native-filesystem tests for the `agent-coexist` binary
//! (M4/M6 native acceptance).
//!
//! Covered contracts, against the real binary as real child processes:
//!
//! * §10.2 concurrency — several concurrent `post` writers against one file
//!   (which may start missing) serialize through the native exclusive lock:
//!   every unique body lands exactly once and comment numbering stays
//!   unique and consecutive from 1;
//! * §10.2 / §8.3 / §11 — while the *test process* holds a real native
//!   whole-file exclusive lock (Unix `flock`, Windows `LockFileEx`),
//!   single-file reads fail bounded with exit 3 `E-LOCKED`, writes fail
//!   bounded with exit 3 `E-LOCKED`, the file bytes never change, and a
//!   directory `list` continues past the busy file (busy leaf, sibling
//!   still listed, first line `warn`); after the parent releases the lock,
//!   the same read and write succeed;
//! * Windows §10.2.5–§10.2.6 — an external no-sharing open yields
//!   `E-INUSE` (exit 3), distinct from a real `LockFileEx` range lock
//!   which yields `E-LOCKED`; a genuine delete-pending file stays an OS
//!   failure (exit 1, `cannot ...: os error`), never a lock id, and the
//!   name can be recreated through the `post` create form once released;
//! * §10.1.1 walk skip — `list` never enters a real directory symlink
//!   (Unix) / a real junction created with `mklink /J` (Windows).
//!
//! Conventions:
//!
//! * every case runs in its own `tempfile` directory with a cleared child
//!   environment (`AGENT_COEXIST_NOW` fixed clock, minimal `PATH`);
//! * the parent's native lock is the only synchronization point: children
//!   are spawned after the lock is taken, and again only after it has been
//!   released — no sleep-guessed readiness;
//! * child processes are always reaped before assertions run on their
//!   captured (separate) stdout/stderr.

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process::{Output, Stdio};
use std::time::Duration;

use assert_cmd::Command as AssertCommand;

const BIN: &str = env!("CARGO_BIN_EXE_agent-coexist");
/// Fixed clock so every child sees the same `now` (spec §8.2 preamble).
const NOW: &str = "2026-09-17T10:00:00Z";

// ---------------------------------------------------------------------------
// child-process helpers
// ---------------------------------------------------------------------------

/// A CLI invocation with an isolated environment: cleared env, fixed
/// `AGENT_COEXIST_NOW`, `PATH` (and `SYSTEMROOT` on Windows, where some
/// Win32 calls need it) re-added, cwd set to the case's temp dir.
fn cli(cwd: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::new(BIN);
    cmd.env_clear()
        .env("AGENT_COEXIST_NOW", NOW)
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .current_dir(cwd)
        .timeout(Duration::from_secs(120));
    #[cfg(windows)]
    cmd.env(
        "SYSTEMROOT",
        std::env::var_os("SYSTEMROOT").unwrap_or_default(),
    );
    cmd
}

/// Run the binary once, capturing stdout and stderr separately.
fn run(cwd: &Path, args: &[&str]) -> Output {
    cli(cwd)
        .args(args)
        .output()
        .expect("failed to run agent-coexist")
}

/// Run the binary with an `AGENT_COEXIST_LOCK_WAIT_MS` override, bounding
/// the `post` lock wait (§10.2.9) so lock-busy write failures are fast.
fn run_with_lock_wait(cwd: &Path, lock_wait_ms: &str, args: &[&str]) -> Output {
    let mut cmd = cli(cwd);
    cmd.env("AGENT_COEXIST_LOCK_WAIT_MS", lock_wait_ms);
    cmd.args(args)
        .output()
        .expect("failed to run agent-coexist")
}

/// A raw spawn-ready command with the same isolated environment, for cases
/// that need several real children alive at the same time.
fn detached_cli(cwd: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(BIN);
    cmd.env_clear()
        .env("AGENT_COEXIST_NOW", NOW)
        .env("AGENT_COEXIST_LOCK_WAIT_MS", "30000")
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .current_dir(cwd);
    #[cfg(windows)]
    cmd.env(
        "SYSTEMROOT",
        std::env::var_os("SYSTEMROOT").unwrap_or_default(),
    );
    cmd
}

// ---------------------------------------------------------------------------
// native-lock helpers (the parent side of §10.2)
// ---------------------------------------------------------------------------

/// Take a real OS-level exclusive whole-file lock from the test process:
/// Unix `flock(LOCK_EX)` — the same primitive the CLI's shared/exclusive
/// protocol uses. The lock is held exactly as long as the open `File`; in
/// the tests below, dropping that `File` is the release sync point.
#[cfg(unix)]
fn lock_exclusive(file: &fs::File) {
    use std::os::unix::io::AsRawFd;

    extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
    }
    const LOCK_EX: i32 = 2;

    let rc = unsafe { flock(file.as_raw_fd(), LOCK_EX) };
    assert_eq!(rc, 0, "flock(LOCK_EX) failed");
}

/// Windows side of [`lock_exclusive`]: `LockFileEx` over the whole file on
/// an ordinarily-shared read/write handle, mirroring the CLI's own lock.
#[cfg(windows)]
fn lock_exclusive(file: &fs::File) {
    native::lock_whole_file_exclusive(file);
}

/// Read the whole file through a (possibly locked) handle. Reading through
/// the handle that owns the lock is permitted on both platforms; a second
/// handle would be the thing §10.2 blocks.
fn snapshot(file: &mut fs::File) -> Vec<u8> {
    file.seek(SeekFrom::Start(0)).expect("seek managed file");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("read managed file");
    bytes
}

/// Raw Win32 access needed to stage situations std cannot express
/// (no-sharing opens, delete-pending files, raw `LockFileEx`).
#[cfg(windows)]
mod native {
    use std::fs::File;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;

    type Handle = *mut core::ffi::c_void;

    const GENERIC_READ: u32 = 0x8000_0000;
    const DELETE_ACCESS: u32 = 0x0001_0000;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const OPEN_EXISTING: u32 = 3;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
    const LOCKFILE_EXCLUSIVE_LOCK: u32 = 0x0000_0002;
    const INVALID_HANDLE_VALUE: Handle = !0usize as Handle;

    /// The fields of Win32 `OVERLAPPED` that matter to `LockFileEx`.
    #[repr(C)]
    struct Overlapped {
        internal: usize,
        internal_high: usize,
        offset: u32,
        offset_high: u32,
        event: Handle,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            file_name: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *mut core::ffi::c_void,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: Handle,
        ) -> Handle;
        fn LockFileEx(
            file: Handle,
            flags: u32,
            reserved: u32,
            number_of_bytes_to_lock_low: u32,
            number_of_bytes_to_lock_high: u32,
            overlapped: *mut Overlapped,
        ) -> i32;
        fn DeleteFileW(file_name: *const u16) -> i32;
        fn CloseHandle(object: Handle) -> i32;
    }

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }

    fn create_file(path: &Path, desired_access: u32, share_mode: u32) -> Handle {
        let name = wide(path);
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                desired_access,
                share_mode,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                core::ptr::null_mut(),
            )
        };
        assert!(
            handle != core::ptr::null_mut() && handle != INVALID_HANDLE_VALUE,
            "CreateFileW failed on {}",
            path.display()
        );
        handle
    }

    /// `LockFileEx` exclusive over the whole file on `file`'s handle.
    /// Dropping the `File` (closing the handle) releases the lock.
    pub fn lock_whole_file_exclusive(file: &File) {
        let mut overlapped = Overlapped {
            internal: 0,
            internal_high: 0,
            offset: 0,
            offset_high: 0,
            event: core::ptr::null_mut(),
        };
        let ok = unsafe {
            LockFileEx(
                file.as_raw_handle(),
                LOCKFILE_EXCLUSIVE_LOCK,
                0,
                u32::MAX,
                u32::MAX,
                &mut overlapped,
            )
        };
        assert_eq!(ok, 1, "LockFileEx(LOCKFILE_EXCLUSIVE_LOCK) failed");
    }

    /// An external program holding the file open with *no* sharing — the
    /// situation §10.2.5 maps to `E-INUSE`. Closing (drop) releases it.
    pub struct NoSharing(Handle);

    impl Drop for NoSharing {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    pub fn open_without_sharing(path: &Path) -> NoSharing {
        NoSharing(create_file(path, GENERIC_READ, 0))
    }

    /// An open handle (full sharing) whose file has been marked for
    /// deletion via `DeleteFileW`: the name is delete-pending — new opens
    /// fail with `ERROR_ACCESS_DENIED` — until this guard is dropped,
    /// which completes the deletion (§10.2.6).
    pub struct PendingDelete(Handle);

    impl Drop for PendingDelete {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    pub fn open_delete_pending(path: &Path) -> PendingDelete {
        let handle = create_file(
            path,
            GENERIC_READ | DELETE_ACCESS,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        );
        let name = wide(path);
        let ok = unsafe { DeleteFileW(name.as_ptr()) };
        assert_eq!(ok, 1, "DeleteFileW failed on {}", path.display());
        PendingDelete(handle)
    }
}

// ---------------------------------------------------------------------------
// output parsing (§8.1: two-space columns, `<status>  <command>  <object>`)
// ---------------------------------------------------------------------------

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("")
}

/// Split a report row into its columns: two or more spaces separate
/// columns and never occur inside a value (§8.1.4).
fn columns(line: &str) -> Vec<&str> {
    line.trim().split("  ").filter(|col| !col.is_empty()).collect()
}

/// Assert the first line is `<status>  <command>  <object>` and return the
/// full stdout for further row assertions.
fn assert_first_line(out: &Output, status: &str, command: &str) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let cols = columns(first_line(&stdout));
    assert!(
        cols.len() >= 2 && cols[0] == status && cols[1] == command,
        "expected first line `{status}  {command} ...`, got {:?}\n{}",
        first_line(&stdout),
        describe(out)
    );
    stdout
}

/// Find a `list` leaf row `<name> ... <kind>` (busy leaf, typed sibling).
fn leaf_row<'a>(stdout: &'a str, name: &str, kind: &str) -> Option<&'a str> {
    stdout.lines().find(|line| {
        let cols = columns(line);
        !cols.is_empty() && cols[0] == name && cols[cols.len() - 1] == kind
    })
}

/// Comment numbers of every `## #<n> <nick> (<time>)` header in the file.
fn comment_seqs(content: &str) -> Vec<u64> {
    content
        .lines()
        .filter_map(|line| line.strip_prefix("## #"))
        .map(|rest| {
            let number = rest.split([' ', '(']).next().unwrap_or_default();
            number
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("malformed comment header: {rest}"))
        })
        .collect()
}

fn describe(out: &Output) -> String {
    format!(
        "exit={:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

// ---------------------------------------------------------------------------
// (a) §10.2 concurrent real writers
// ---------------------------------------------------------------------------

/// Several real `post` processes race on one file that starts missing.
/// All must exit 0, every unique body must land exactly once, and comment
/// numbering must come out unique and consecutive from 1.
#[test]
fn concurrent_post_writers_land_each_body_once_with_clean_numbering() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();

    const WRITERS: usize = 4;
    let nicks: Vec<String> = (0..WRITERS).map(|i| format!("w{i}")).collect();
    let bodies: Vec<String> = (0..WRITERS)
        .map(|i| format!("concurrent writer body {i}"))
        .collect();

    // Spawn every writer before waiting on any of them, so they genuinely
    // overlap on the create/lock/append sequence.
    let mut children = Vec::new();
    for i in 0..WRITERS {
        let mut cmd = detached_cli(cwd);
        cmd.arg("post")
            .arg("standup")
            .arg("--as")
            .arg(&nicks[i])
            .arg("-m")
            .arg(&bodies[i])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        children.push(cmd.spawn().expect("spawn concurrent writer"));
    }

    // Reap every child before asserting anything, so a failing assertion
    // can never leave a writer process behind.
    let mut waits = Vec::new();
    for child in children.drain(..) {
        waits.push(child.wait_with_output());
    }
    let outputs: Vec<Output> = waits
        .into_iter()
        .map(|wait| wait.expect("wait for concurrent writer"))
        .collect();

    for (i, out) in outputs.iter().enumerate() {
        assert!(
            out.status.success(),
            "concurrent writer {} failed\n{}",
            i,
            describe(out)
        );
        assert_first_line(out, "ok", "post");
    }

    let file = cwd.join("standup.agent-besties.md");
    let content = fs::read_to_string(&file).expect("read merged post");
    assert_eq!(first_line(&content), "# standup.post");

    // The exact expected set of bodies, each exactly once — not a length.
    for body in &bodies {
        assert_eq!(
            content.matches(body.as_str()).count(),
            1,
            "body {body:?} must appear exactly once in:\n{content}"
        );
    }

    // Unique, legal, consecutive-from-1 comment numbering.
    let mut seqs = comment_seqs(&content);
    let count = seqs.len();
    seqs.sort_unstable();
    seqs.dedup();
    assert_eq!(count, WRITERS, "one comment per writer in:\n{content}");
    assert_eq!(seqs, (1..=WRITERS as u64).collect::<Vec<u64>>());

    // A post-lock read agrees: four comments in the merged file.
    let out = run(cwd, &["check", "standup"]);
    assert!(out.status.success(), "{}", describe(&out));
    let stdout = assert_first_line(&out, "ok", "check");
    assert!(
        columns(stdout.lines().find(|l| columns(l).first() == Some(&"comments")).unwrap_or(""))
            .get(1)
            == Some(&"4"),
        "expected `comments  4` row:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// (b) §10.2 native exclusive lock held by the parent process
// ---------------------------------------------------------------------------

/// While the test process holds a real native whole-file exclusive lock:
/// single-file reads and writes fail bounded with exit 3 / `E-LOCKED`
/// without touching the bytes; `list` continues past the busy file and
/// still lists a readable sibling (first line `warn`); after the lock is
/// released the same operations succeed.
#[test]
fn parent_native_exclusive_lock_bounds_access_then_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("standup.agent-besties.md");

    // Seed the busy managed file through the real create path, plus a
    // readable sibling collaboration file in the same directory.
    let out = run(cwd, &["post", "standup", "--as", "alice", "-m", "original comment body"]);
    assert!(out.status.success(), "{}", describe(&out));
    let out = run(cwd, &["post", "notes", "--as", "carol", "-m", "sibling comment body"]);
    assert!(out.status.success(), "{}", describe(&out));

    // The parent takes a real native exclusive lock; children below are
    // only ever spawned while it is held — the lock itself is the barrier.
    let mut lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&file)
        .expect("open managed file for native lock");
    let baseline = snapshot(&mut lock_file);
    assert!(
        String::from_utf8_lossy(&baseline).contains("original comment body"),
        "seeded content missing:\n{}",
        String::from_utf8_lossy(&baseline)
    );
    lock_exclusive(&lock_file);

    // Single-file read #1: `check` fails bounded (READ_WAIT_MS) with
    // exit 3 and the E-LOCKED reason/fix form (§8.3).
    let out = run(cwd, &["check", "standup"]);
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    assert!(
        stdout.contains("file is locked by another process"),
        "E-LOCKED reason missing:\n{stdout}"
    );
    assert!(
        stdout.contains("retry shortly: "),
        "E-LOCKED fix form missing:\n{stdout}"
    );

    // Single-file read #2: the other single-file read entry point agrees.
    let out = run(cwd, &["lookfor-comment", "standup", "--keyword", "original"]);
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "lookfor-comment");
    assert!(
        stdout.contains("file is locked by another process"),
        "E-LOCKED reason missing:\n{stdout}"
    );

    assert_eq!(
        snapshot(&mut lock_file),
        baseline,
        "file bytes must be unchanged after failed reads"
    );

    // Write: `post` (lock wait bounded by the env override) fails with
    // exit 3 / E-LOCKED and its body never lands.
    let out = run_with_lock_wait(
        cwd,
        "250",
        &["post", "standup", "--as", "bob", "-m", "must never land"],
    );
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "post");
    assert!(
        stdout.contains("file is locked by another process"),
        "E-LOCKED reason missing:\n{stdout}"
    );

    let bytes_now = snapshot(&mut lock_file);
    assert_eq!(bytes_now, baseline, "file bytes must be unchanged after failed write");
    assert!(
        !String::from_utf8_lossy(&bytes_now).contains("must never land"),
        "failed write leaked its body"
    );

    // Scan: `list` continues past the busy file (busy leaf, first line
    // warn, warn is still exit 0) and lists the readable sibling.
    let out = run(cwd, &["list"]);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "list");
    assert!(
        leaf_row(&stdout, "standup", "busy").is_some(),
        "busy leaf row for standup missing:\n{stdout}"
    );
    assert!(
        leaf_row(&stdout, "notes", "post").is_some(),
        "readable sibling notes must still be listed:\n{stdout}"
    );
    assert_eq!(
        columns(stdout.lines().find(|l| columns(l).first() == Some(&"total")).unwrap_or("")),
        vec!["total", "2"],
        "busy leaf and sibling both count:\n{stdout}"
    );

    // RELEASE: closing the handle drops the native lock. This is the sync
    // point — everything after it may see the file free. No sleeps.
    drop(lock_file);

    let out = run(cwd, &["check", "standup"]);
    assert!(out.status.success(), "{}", describe(&out));
    assert_first_line(&out, "ok", "check");

    let out = run(cwd, &["post", "standup", "--as", "bob", "-m", "lands after release"]);
    assert!(out.status.success(), "{}", describe(&out));
    assert_first_line(&out, "ok", "post");

    let content = fs::read_to_string(&file).expect("read post after release");
    assert!(content.contains("original comment body"), "{content}");
    assert!(content.contains("lands after release"), "{content}");
}

// ---------------------------------------------------------------------------
// (c) Windows-only §10.2.5–§10.2.6: E-INUSE, delete-pending
// ---------------------------------------------------------------------------

/// An external handle with no sharing yields `E-INUSE` (exit 3) for both
/// read and write, with a reason text *distinct* from the `E-LOCKED`
/// produced by a real `LockFileEx` exclusive range lock on the same file.
#[cfg(windows)]
#[test]
fn no_sharing_external_handle_is_e_inuse_distinct_from_e_locked() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("standup.agent-besties.md");

    let out = run(cwd, &["post", "standup", "--as", "alice", "-m", "shared baseline body"]);
    assert!(out.status.success(), "{}", describe(&out));
    let baseline = fs::read(&file).expect("baseline read");

    // Phase 1 — an editor-like program holds the file with share mode 0.
    let no_share = native::open_without_sharing(&file);

    let out = run(cwd, &["check", "standup"]);
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    assert!(
        stdout.contains("file is in use by another program"),
        "E-INUSE reason missing:\n{stdout}"
    );
    assert!(
        stdout.contains("close it, then retry: "),
        "E-INUSE fix form missing:\n{stdout}"
    );
    assert!(
        !stdout.contains("file is locked by another process"),
        "share-mode-0 must not be reported as E-LOCKED:\n{stdout}"
    );

    let out = run_with_lock_wait(
        cwd,
        "250",
        &["post", "standup", "--as", "bob", "-m", "blocked by no sharing"],
    );
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "post");
    assert!(
        stdout.contains("file is in use by another program"),
        "E-INUSE reason missing:\n{stdout}"
    );

    // The external program closes: bytes untouched, access restored.
    drop(no_share);
    assert_eq!(fs::read(&file).expect("reread"), baseline);
    let out = run(cwd, &["check", "standup"]);
    assert!(out.status.success(), "{}", describe(&out));
    assert_first_line(&out, "ok", "check");

    // Phase 2 — a *real* LockFileEx exclusive lock (the CLI's own lock
    // primitive) on a normally-shared handle yields E-LOCKED, not E-INUSE.
    let lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&file)
        .expect("open for LockFileEx");
    lock_exclusive(&lock_file);

    let out = run(cwd, &["check", "standup"]);
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    assert!(
        stdout.contains("file is locked by another process"),
        "E-LOCKED reason missing:\n{stdout}"
    );
    assert!(
        !stdout.contains("file is in use by another program"),
        "a range lock must not be reported as E-INUSE:\n{stdout}"
    );

    drop(lock_file);
    let out = run(cwd, &["check", "standup"]);
    assert!(out.status.success(), "{}", describe(&out));
    assert_first_line(&out, "ok", "check");
}

/// A genuine delete-pending file (FILE_SHARE_DELETE + DeleteFileW with the
/// handle held) makes CLI read *and* write fail as ordinary OS failures —
/// exit 1, `cannot ...: os error` — never `E-LOCKED`; once the handle is
/// released the name disappears and the `post` create form recreates it.
#[cfg(windows)]
#[test]
fn delete_pending_file_is_os_failure_and_recreation_works() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("standup.agent-besties.md");

    let out = run(cwd, &["post", "standup", "--as", "alice", "-m", "doomed baseline body"]);
    assert!(out.status.success(), "{}", describe(&out));

    // Mark the file delete-pending but keep the handle open.
    let pending = native::open_delete_pending(&file);

    let out = run(cwd, &["check", "standup"]);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    assert!(
        stdout.contains("cannot ") && stdout.contains("os error"),
        "expected a `cannot ...: os error` reason:\n{stdout}"
    );
    assert!(
        !stdout.contains("file is locked by another process")
            && !stdout.contains("file is in use by another program"),
        "delete-pending must never be reported as a lock id:\n{stdout}"
    );

    let out = run_with_lock_wait(
        cwd,
        "250",
        &["post", "standup", "--as", "bob", "-m", "write against delete pending"],
    );
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "post");
    assert!(
        stdout.contains("cannot ") && stdout.contains("os error"),
        "expected a `cannot ...: os error` reason:\n{stdout}"
    );
    assert!(
        !stdout.contains("file is locked by another process")
            && !stdout.contains("file is in use by another program"),
        "delete-pending must never be reported as a lock id:\n{stdout}"
    );

    // Releasing the handle completes the deletion...
    drop(pending);
    assert!(!file.exists(), "file must be gone once the handle closes");

    // ...and the create form can bring the name back.
    let out = run(cwd, &["post", "standup", "--as", "alice", "-m", "recreated body"]);
    assert!(out.status.success(), "{}", describe(&out));
    assert_first_line(&out, "ok", "post");
    let content = fs::read_to_string(&file).expect("read recreated file");
    assert_eq!(first_line(&content), "# standup.post");
    assert!(content.contains("recreated body"), "{content}");
}

// ---------------------------------------------------------------------------
// §10.1.1 walk skip: directory symlink (Unix) / junction (Windows)
// ---------------------------------------------------------------------------

/// A real directory symlink is never entered by `list`: the collaboration
/// file inside the linked directory appears exactly once (via the real
/// path) and the symlink itself never shows up.
#[cfg(unix)]
#[test]
fn list_does_not_enter_directory_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();

    fs::create_dir_all(cwd.join("real")).unwrap();
    let out = run(
        cwd,
        &["post", "real/innerpost", "--as", "alice", "-m", "inside the linked directory"],
    );
    assert!(out.status.success(), "{}", describe(&out));

    std::os::unix::fs::symlink("real", cwd.join("link")).expect("create directory symlink");
    // The symlink is a genuinely traversable directory: reading through it
    // works, so any second `innerpost` hit below would prove the walk had
    // entered it.
    assert!(fs::read(cwd.join("link").join("innerpost.agent-besties.md")).is_ok());

    let out = run(cwd, &["list"]);
    assert!(out.status.success(), "{}", describe(&out));
    let stdout = assert_first_line(&out, "ok", "list");
    assert!(
        leaf_row(&stdout, "innerpost", "post").is_some(),
        "file via its real path must be listed:\n{stdout}"
    );
    assert_eq!(
        stdout.matches("innerpost").count(),
        1,
        "file must appear exactly once (never via link/):\n{stdout}"
    );
    assert!(
        !stdout.contains("link"),
        "directory symlink must be skipped entirely:\n{stdout}"
    );
}

/// A real junction (`mklink /J`, not a symlink) is never entered by
/// `list`: same expectations as the Unix directory-symlink case.
#[cfg(windows)]
#[test]
fn list_does_not_enter_junction() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();

    fs::create_dir_all(cwd.join("real")).unwrap();
    let out = run(
        cwd,
        &["post", "real/innerpost", "--as", "alice", "-m", "inside the junctioned directory"],
    );
    assert!(out.status.success(), "{}", describe(&out));

    // An actual junction — a reparse point that is not `is_symlink()`,
    // which is exactly why §10.1.1 requires the attribute-level check.
    let target = cwd
        .join("real")
        .to_str()
        .expect("utf-8 temp path")
        .replace('/', "\\");
    let mklink = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J", "link", target.as_str()])
        .current_dir(cwd)
        .output()
        .expect("spawn cmd /C mklink");
    assert!(
        mklink.status.success(),
        "mklink /J failed\n{}",
        describe(&mklink)
    );
    assert!(
        fs::symlink_metadata(cwd.join("link")).is_ok(),
        "junction was not created"
    );
    // The junction is genuinely traversable at the OS level.
    assert!(fs::read(cwd.join("link").join("innerpost.agent-besties.md")).is_ok());

    let out = run(cwd, &["list"]);
    assert!(out.status.success(), "{}", describe(&out));
    let stdout = assert_first_line(&out, "ok", "list");
    assert!(
        leaf_row(&stdout, "innerpost", "post").is_some(),
        "file via its real path must be listed:\n{stdout}"
    );
    assert_eq!(
        stdout.matches("innerpost").count(),
        1,
        "file must appear exactly once (never via link/):\n{stdout}"
    );
    assert!(
        !stdout.contains("link"),
        "junction must be skipped entirely:\n{stdout}"
    );
}
