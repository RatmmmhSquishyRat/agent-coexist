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
//! * Windows §10.2.5–§10.2.6 — an external no-sharing open yields an
//!   exit-3 failure whose `reason` row is distinct from the one produced
//!   by a real `LockFileEx` range lock (`E-LOCKED`); a genuine
//!   delete-pending file (delete disposition set on a held handle) stays
//!   an ordinary OS failure — exit 1, `reason` carrying the raw
//!   `{os error}` (§8.3.5) — never a retryable lock id (exit 3) and never
//!   a file-missing classification, and the name can be recreated through
//!   the `post` create form once released;
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
use std::time::{Duration, Instant};

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
        .env("HOME", cwd)
        .env("USERPROFILE", cwd)
        .env("XDG_CONFIG_HOME", cwd)
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
        .env("HOME", cwd)
        .env("USERPROFILE", cwd)
        .env("XDG_CONFIG_HOME", cwd)
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
    /// `FILE_INFO_BY_HANDLE_CLASS::FileDispositionInfo` — sets the delete
    /// disposition on an open handle.
    const FILE_DISPOSITION_INFO: i32 = 4;
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
        fn SetFileInformationByHandle(
            file: Handle,
            file_information_class: i32,
            file_information: *const core::ffi::c_void,
            buffer_size: u32,
        ) -> i32;
        fn CloseHandle(object: Handle) -> i32;
    }

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
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
            !handle.is_null() && handle != INVALID_HANDLE_VALUE,
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

    /// An open handle (full sharing) whose delete disposition has been set
    /// (`SetFileInformationByHandle` + `FILE_DISPOSITION_INFO`): the name
    /// stays present, but new opens fail with `ERROR_ACCESS_DENIED` until
    /// this guard is dropped, which completes the deletion (§10.2.6). A
    /// plain `DeleteFileW` is not usable for staging this state on current
    /// Windows builds: when every open handle allows sharing it unlinks
    /// the name immediately instead of leaving it delete-pending.
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
        // Stage genuine pending-delete on the held handle: the name stays
        // visible, but while this handle is open every ordinary open of
        // the file fails with `ERROR_ACCESS_DENIED` (§10.2.6). Dropping
        // the guard closes the handle and completes the deletion.
        let delete_file: u8 = 1; // FILE_DISPOSITION_INFO::DeleteFile (BOOLEAN)
        let ok = unsafe {
            SetFileInformationByHandle(
                handle,
                FILE_DISPOSITION_INFO,
                &delete_file as *const u8 as *const core::ffi::c_void,
                1,
            )
        };
        assert_eq!(
            ok,
            1,
            "SetFileInformationByHandle(FileDispositionInfo) failed on {}",
            path.display()
        );
        // Self-check: a plain read open (what the CLI does) must now fail,
        // otherwise the fixture would silently test a writable file.
        let probe = wide(path);
        let leaked = unsafe {
            CreateFileW(
                probe.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                core::ptr::null_mut(),
            )
        };
        if !leaked.is_null() && leaked != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(leaked) };
            panic!(
                "file is not delete-pending: a read open still succeeds on {}",
                path.display()
            );
        }
        PendingDelete(handle)
    }
}

// ---------------------------------------------------------------------------
// output parsing (§8.1: two-space columns, `<status>  <command>  <object>`)
// ---------------------------------------------------------------------------

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("")
}

/// Split a report row into its columns: runs of two or more spaces
/// separate columns and never occur inside a value (§8.1.4). Splitting on
/// `"  "` leaves one stray space on the next piece whenever a padding run
/// has odd width, so every piece is trimmed and only then are the empty
/// pieces (the other halves of wider runs) dropped.
fn columns(line: &str) -> Vec<&str> {
    line.trim()
        .split("  ")
        .map(str::trim)
        .filter(|col| !col.is_empty())
        .collect()
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

/// The value of a `reason` row in a reason/fix failure report (§8.3).
/// Lets tests assert the *shape* of a failure and *distinctness* between
/// two failure classes without pinning the English wording owned by the
/// diagnostic catalog.
fn reason_row(stdout: &str) -> Option<&str> {
    stdout.lines().find_map(|line| {
        let cols = columns(line);
        if cols.first() == Some(&"reason") {
            Some(cols.get(1).copied().unwrap_or(""))
        } else {
            None
        }
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

/// First-line object column. Safe on folded display values (no two-space
/// runs); never used to split a `fix` payload.
#[cfg(unix)]
fn first_object(stdout: &str) -> &str {
    columns(first_line(stdout)).get(2).copied().unwrap_or("")
}

/// Payload of every `fix` row, diagnostic (`         fix  …`) or reason-form
/// (`  fix` padded to width 12). The value is kept intact — splitting on
/// two spaces would destroy operand whitespace such as `a  b`.
fn fix_payloads(stdout: &str) -> Vec<&str> {
    const DIAG_FIX: &str = "         fix  ";
    stdout
        .lines()
        .filter_map(|line| {
            if let Some(rest) = line.strip_prefix(DIAG_FIX) {
                return Some(rest);
            }
            let rest = line.strip_prefix("  ")?;
            let after_key = rest.strip_prefix("fix")?;
            // KEY_WIDTH 12: 9 pad spaces after `fix`, then the two-space
            // column separator, then the payload.
            after_key.get(11..)
        })
        .collect()
}

/// Extract the displayed heading example without copying surrounding prose.
/// Its bytes are passed straight to the real file parser by the tests below.
fn copyable_heading(fix: &str) -> &str {
    fix.split('"')
        .find(|part| part.starts_with("## "))
        .unwrap_or_else(|| panic!("fix has no copyable heading:\n{fix}"))
}

/// These fixtures emit either a command or `prose: command`. Discard only
/// that prose prefix; leave shell parsing to bash, independent of wording.
#[cfg(unix)]
fn shell_command_from_fix(fix: &str) -> &str {
    fix.split_once(": ").map_or(fix, |(_, command)| command)
}

#[cfg(unix)]
fn heading_section<'a>(content: &'a str, title: &str) -> &'a str {
    let heading = format!("## {title}");
    let start = content
        .find(&heading)
        .unwrap_or_else(|| panic!("missing heading {title:?} in:\n{content}"));
    let after = start + heading.len();
    let end = content[after..]
        .find("\n## ")
        .map(|i| after + i)
        .unwrap_or(content.len());
    &content[start..end]
}

#[cfg(unix)]
fn section_has_sha256(section: &str) -> bool {
    section.lines().any(|line| {
        line.strip_prefix("- hash: ")
            .is_some_and(|value| value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()))
    })
}

#[cfg(unix)]
fn seed_two_space_and_one_space_twins(cwd: &Path) {
    fs::create_dir_all(cwd.join("a  b")).unwrap();
    fs::create_dir_all(cwd.join("a b")).unwrap();
    let out = run(cwd, &["profile", "a  b/alice"]);
    assert!(out.status.success(), "{}", describe(&out));
    let out = run(cwd, &["profile", "a b/alice"]);
    assert!(out.status.success(), "{}", describe(&out));
    let out = run(
        cwd,
        &[
            "post",
            "a  b/onlyhere",
            "--as",
            "alice",
            "-m",
            "two-space only",
        ],
    );
    assert!(out.status.success(), "{}", describe(&out));
    let out = run(
        cwd,
        &["post", "a b/twin", "--as", "bob", "-m", "one-space only"],
    );
    assert!(out.status.success(), "{}", describe(&out));
}

/// Run the emitted fix through real bash in the case temp dir. Synthesized
/// commands call `agent-coexist` via a function that forwards to `BIN`;
/// `{argv}` retries already name `BIN` and run as written.
#[cfg(unix)]
fn consume_emitted(cwd: &Path, fix: &str) -> Output {
    let command = shell_command_from_fix(fix);
    let script =
        format!("agent-coexist() {{\n  command \"$AGENT_COEXIST_BIN\" \"$@\"\n}}\n{command}\n");
    AssertCommand::new("bash")
        .arg("-c")
        .arg(script)
        .env_clear()
        .env("AGENT_COEXIST_BIN", BIN)
        .env("AGENT_COEXIST_NOW", NOW)
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", cwd)
        .env("USERPROFILE", cwd)
        .env("XDG_CONFIG_HOME", cwd)
        .current_dir(cwd)
        .timeout(Duration::from_secs(120))
        .output()
        .expect("bash failed to run emitted fix")
}

#[cfg(unix)]
fn consume_emitted_ok(cwd: &Path, fix: &str) -> String {
    let out = consume_emitted(cwd, fix);
    assert!(
        out.status.success(),
        "emitted command failed\nfix={fix}\n{}",
        describe(&out)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
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
        columns(
            stdout
                .lines()
                .find(|l| columns(l).first() == Some(&"comments"))
                .unwrap_or("")
        )
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
    let out = run(
        cwd,
        &[
            "post",
            "standup",
            "--as",
            "alice",
            "-m",
            "original comment body",
        ],
    );
    assert!(out.status.success(), "{}", describe(&out));
    let out = run(
        cwd,
        &[
            "post",
            "notes",
            "--as",
            "carol",
            "-m",
            "sibling comment body",
        ],
    );
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
        reason_row(&stdout).is_some(),
        "lock failure must report in reason/fix form:\n{stdout}"
    );

    // Single-file read #2: the other single-file read entry point agrees.
    let out = run(
        cwd,
        &["lookfor-comment", "standup", "--keyword", "original"],
    );
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "lookfor-comment");
    assert!(
        reason_row(&stdout).is_some(),
        "lock failure must report in reason/fix form:\n{stdout}"
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
        reason_row(&stdout).is_some(),
        "lock failure must report in reason/fix form:\n{stdout}"
    );

    let bytes_now = snapshot(&mut lock_file);
    assert_eq!(
        bytes_now, baseline,
        "file bytes must be unchanged after failed write"
    );
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
        columns(
            stdout
                .lines()
                .find(|l| columns(l).first() == Some(&"total"))
                .unwrap_or("")
        ),
        vec!["total", "2"],
        "busy leaf and sibling both count:\n{stdout}"
    );

    // RELEASE: closing the handle drops the native lock. This is the sync
    // point — everything after it may see the file free. No sleeps.
    drop(lock_file);

    let out = run(cwd, &["check", "standup"]);
    assert!(out.status.success(), "{}", describe(&out));
    assert_first_line(&out, "ok", "check");

    let out = run(
        cwd,
        &[
            "post",
            "standup",
            "--as",
            "bob",
            "-m",
            "lands after release",
        ],
    );
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

    let out = run(
        cwd,
        &[
            "post",
            "standup",
            "--as",
            "alice",
            "-m",
            "shared baseline body",
        ],
    );
    assert!(out.status.success(), "{}", describe(&out));
    let baseline = fs::read(&file).expect("baseline read");

    // Phase 1 — an editor-like program holds the file with share mode 0.
    let no_share = native::open_without_sharing(&file);

    let out = run(cwd, &["check", "standup"]);
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    // Capture, don't pin: the no-sharing shape and the real range-lock
    // shape later in this test must be observably *distinct* retryable
    // failures (§10.2.5 vs §10.2), asserted by comparing the two reason
    // rows; the English wording itself belongs to the diag catalog.
    let inuse_reason = reason_row(&stdout)
        .unwrap_or_else(|| panic!("no-sharing failure missing a reason row:\n{stdout}"));

    let out = run_with_lock_wait(
        cwd,
        "250",
        &[
            "post",
            "standup",
            "--as",
            "bob",
            "-m",
            "blocked by no sharing",
        ],
    );
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "post");
    assert!(
        reason_row(&stdout).is_some(),
        "no-sharing write failure must report in reason/fix form:\n{stdout}"
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
    let locked_reason = reason_row(&stdout)
        .unwrap_or_else(|| panic!("range-lock failure missing a reason row:\n{stdout}"));
    assert_ne!(
        inuse_reason, locked_reason,
        "no-sharing and range-lock failures must remain distinct retryable IDs:\n{stdout}"
    );

    drop(lock_file);
    let out = run(cwd, &["check", "standup"]);
    assert!(out.status.success(), "{}", describe(&out));
    assert_first_line(&out, "ok", "check");
}

/// A genuine delete-pending file (delete disposition set on a held,
/// fully-shared handle) makes CLI read *and* write fail as ordinary OS
/// failures — exit 1, `reason` carrying the raw `{os error}` (§8.3.5) —
/// never a retryable lock id (exit 3) and never a file-missing
/// classification; once the handle is released the name disappears and
/// the `post` create form recreates it.
#[cfg(windows)]
#[test]
fn delete_pending_file_is_os_failure_and_recreation_works() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("standup.agent-besties.md");

    let out = run(
        cwd,
        &[
            "post",
            "standup",
            "--as",
            "alice",
            "-m",
            "doomed baseline body",
        ],
    );
    assert!(out.status.success(), "{}", describe(&out));

    // Mark the file delete-pending but keep the handle open.
    let pending = native::open_delete_pending(&file);

    let out = run(cwd, &["check", "standup"]);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    // The pending-delete open failure must surface as an ordinary OS
    // failure (§8.3.5: `reason` carries the raw `{os error}`) — the exit
    // code above already rules out the retryable lock class, and a
    // friendly file-missing classification would hide the real cause.
    let reason = reason_row(&stdout)
        .unwrap_or_else(|| panic!("delete-pending read failure missing a reason row:\n{stdout}"));
    assert!(
        reason.contains("os error"),
        "delete-pending must surface the raw OS error, got reason {reason:?}:\n{stdout}"
    );

    let out = run_with_lock_wait(
        cwd,
        "250",
        &[
            "post",
            "standup",
            "--as",
            "bob",
            "-m",
            "write against delete pending",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "post");
    let reason = reason_row(&stdout)
        .unwrap_or_else(|| panic!("delete-pending write failure missing a reason row:\n{stdout}"));
    assert!(
        reason.contains("os error"),
        "delete-pending must surface the raw OS error, got reason {reason:?}:\n{stdout}"
    );

    // Releasing the handle completes the deletion...
    drop(pending);
    assert!(!file.exists(), "file must be gone once the handle closes");

    // ...and the create form can bring the name back.
    let out = run(
        cwd,
        &["post", "standup", "--as", "alice", "-m", "recreated body"],
    );
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
        &[
            "post",
            "real/innerpost",
            "--as",
            "alice",
            "-m",
            "inside the linked directory",
        ],
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
        &[
            "post",
            "real/innerpost",
            "--as",
            "alice",
            "-m",
            "inside the junctioned directory",
        ],
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

// ---------------------------------------------------------------------------
// R7 emitted-fix consumption (native CLI, isolated env / fixed clock)
// ---------------------------------------------------------------------------

/// A lock-busy retry must preserve comment bodies as data after release,
/// not apply bash comments, tilde/brace expansion or command substitution.
#[cfg(unix)]
#[test]
fn emitted_lock_retry_preserves_shell_syntax_bodies() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("standup.agent-besties.md");

    let out = run(
        cwd,
        &[
            "post",
            "standup",
            "--as",
            "alice",
            "-m",
            "original comment body",
        ],
    );
    assert!(out.status.success(), "{}", describe(&out));

    let lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&file)
        .expect("open managed file for native lock");
    lock_exclusive(&lock_file);

    let bodies = [
        "#ready",
        "~",
        "{1..1}",
        r#"O'Brien $(printf expanded) `printf expanded` $HOME \;"#,
    ];
    let mut retries = Vec::new();
    for body in bodies {
        let out = run_with_lock_wait(cwd, "250", &["post", "standup", "--as", "bob", "-m", body]);
        assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
        let stdout = assert_first_line(&out, "error", "post");
        retries.push(
            fix_payloads(&stdout)
                .first()
                .expect("E-LOCKED body retry missing")
                .to_string(),
        );
    }
    drop(lock_file);

    for retry in retries {
        consume_emitted_ok(cwd, &retry);
    }

    let content = fs::read_to_string(&file).expect("read post after retry consume");
    for body in bodies {
        assert_eq!(
            content.lines().filter(|line| *line == body).count(),
            1,
            "retry must preserve the literal body {body:?}:\n{content}"
        );
    }
}

/// Empty `--regex` means deletion; losing that argv word must not turn a
/// lock retry into a missing-value usage error or a keep-regex operation.
#[cfg(unix)]
#[test]
fn emitted_retry_preserves_empty_regex_value() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::write(cwd.join("target.txt"), b"source\n").unwrap();
    let file = cwd.join("notes.agent-besties.md");
    fs::write(
        &file,
        format!(
            "# notes.brief\n\n## parser\n- path: target.txt\n- hash: {}\n- regex: .*\n",
            "0".repeat(64)
        ),
    )
    .unwrap();
    let lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&file)
        .unwrap();
    lock_exclusive(&lock_file);
    let out = run(
        cwd,
        &[
            "brief",
            "notes",
            "--target",
            "target.txt",
            "--entry",
            "parser",
            "--regex",
            "",
        ],
    );
    assert_eq!(out.status.code(), Some(3), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "brief");
    let retry = *fix_payloads(&stdout).first().expect("missing lock retry");
    drop(lock_file);
    consume_emitted_ok(cwd, retry);
    let content = fs::read_to_string(file).unwrap();
    assert!(
        !content.lines().any(|line| line.starts_with("- regex:")),
        "empty argv value must delete the regex:\n{content}"
    );
}

/// E-ENTRY-HASH refresh for `<file>`, `<nick>`, and `-item` must address
/// those entries (no redirection artifacts; `-item` must parse). An ESC
/// title is distinct from U+FFFD: it must not emit a refresh command, and
/// the U+FFFD sibling's refresh must not touch it.
#[cfg(unix)]
#[test]
fn emitted_brief_refresh_consumes_literal_titles_and_keeps_esc_distinct() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    for (name, bytes) in [
        ("file-target.txt", &b"file-bytes"[..]),
        ("nick-target.txt", &b"nick-bytes"[..]),
        ("dash-target.txt", &b"dash-bytes"[..]),
        ("esc-target.txt", &b"esc-bytes"[..]),
        ("fffd-target.txt", &b"fffd-bytes"[..]),
    ] {
        fs::write(cwd.join(name), bytes).unwrap();
    }
    fs::write(
        cwd.join("notes.agent-besties.md"),
        concat!(
            "# notes.brief\n\n",
            "## <file>\n",
            "- path: file-target.txt\n\n",
            "## <nick>\n",
            "- path: nick-target.txt\n\n",
            "## -item\n",
            "- path: dash-target.txt\n\n",
            "## x\u{1b}y\n",
            "- path: esc-target.txt\n\n",
            "## x\u{fffd}y\n",
            "- path: fffd-target.txt\n",
        ),
    )
    .unwrap();
    fs::write(cwd.join("file"), b"keep file input").unwrap();
    fs::write(cwd.join("nick"), b"keep nick input").unwrap();
    fs::write(cwd.join("--refresh"), b"keep redirect victim").unwrap();

    let out = run(cwd, &["check", "notes"]);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    assert!(
        !stdout.contains('\u{1b}'),
        "ESC title must not leak into emitted text:\n{stdout}"
    );

    let refreshes: Vec<&str> = fix_payloads(&stdout)
        .into_iter()
        .filter(|fix| {
            let cmd = shell_command_from_fix(fix);
            cmd.contains("agent-coexist") && cmd.contains("--refresh")
        })
        .collect();
    assert_eq!(
        refreshes.len(),
        4,
        "ESC title must emit static heading guidance, not a refresh:\n{stdout}"
    );

    for fix in &refreshes {
        consume_emitted_ok(cwd, fix);
    }

    assert_eq!(fs::read(cwd.join("file")).unwrap(), b"keep file input");
    assert_eq!(fs::read(cwd.join("nick")).unwrap(), b"keep nick input");
    assert_eq!(
        fs::read(cwd.join("--refresh")).unwrap(),
        b"keep redirect victim",
        "a refresh suggestion must not truncate an unrelated redirection target"
    );

    let content = fs::read_to_string(cwd.join("notes.agent-besties.md")).unwrap();
    assert!(
        section_has_sha256(heading_section(&content, "<file>")),
        "<file> entry must refresh:\n{content}"
    );
    assert!(
        section_has_sha256(heading_section(&content, "<nick>")),
        "<nick> entry must refresh:\n{content}"
    );
    assert!(
        section_has_sha256(heading_section(&content, "-item")),
        "-item entry must refresh via CLI parsing:\n{content}"
    );
    assert!(
        section_has_sha256(heading_section(&content, "x\u{fffd}y")),
        "U+FFFD title must refresh its own entry:\n{content}"
    );
    assert!(
        !section_has_sha256(heading_section(&content, "x\u{1b}y")),
        "ESC title must stay unrefreshed:\n{content}"
    );
}

/// Post W-BODY composer output must be executable, and a `-alice` empty
/// profile must accept the emitted W-PROFILE-EMPTY repair (leading '-'
/// positional).
#[cfg(unix)]
#[test]
fn emitted_wbody_and_leading_dash_profile_repairs_consume() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();

    fs::write(
        cwd.join("standup.agent-besties.md"),
        format!("# standup.post\n\n## #1 alice ({NOW})\n"),
    )
    .unwrap();
    let out = run(cwd, &["check", "standup"]);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    let wbody = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| {
            let cmd = shell_command_from_fix(fix);
            cmd.contains(" post ") && cmd.contains("-m")
        })
        .unwrap_or_else(|| panic!("missing post W-BODY repair:\n{stdout}"));
    consume_emitted_ok(cwd, wbody);
    let posted = fs::read_to_string(cwd.join("standup.agent-besties.md")).unwrap();
    assert!(
        posted.contains("<text>"),
        "W-BODY consume must write the placeholder body:\n{posted}"
    );

    let out = run(cwd, &["profile", "./-alice"]);
    assert!(out.status.success(), "{}", describe(&out));
    let out = run(cwd, &["check", "--", "-alice"]);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    let repair = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| shell_command_from_fix(fix).contains("profile"))
        .unwrap_or_else(|| panic!("missing W-PROFILE-EMPTY repair:\n{stdout}"));
    consume_emitted_ok(cwd, repair);
    let profile = fs::read_to_string(cwd.join("-alice.agent-besties.md")).unwrap();
    assert!(
        profile.contains("## role"),
        "leading-dash profile must accept the emitted repair:\n{profile}"
    );
}

/// Folded display `a b/alice` must not select the one-space twin. Consumed
/// profile repair mutates only `a  b/alice`; consumed DIR list of the
/// emitted parent lists only the two-space tree.
#[cfg(unix)]
#[test]
fn emitted_repair_mutates_two_space_dir_not_folded_twin() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    seed_two_space_and_one_space_twins(cwd);

    let selected = cwd.join("a  b").join("alice.agent-besties.md");
    let twin = cwd.join("a b").join("alice.agent-besties.md");
    let twin_before = fs::read_to_string(&twin).unwrap();

    let out = run(cwd, &["check", "a  b/alice"]);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    assert_eq!(
        first_object(&stdout),
        "a b/alice",
        "first-line object stays folded:\n{stdout}"
    );
    let repair = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| shell_command_from_fix(fix).contains("profile"))
        .unwrap_or_else(|| panic!("missing two-space profile repair:\n{stdout}"));
    consume_emitted_ok(cwd, repair);

    let after_selected = fs::read_to_string(&selected).unwrap();
    let after_twin = fs::read_to_string(&twin).unwrap();
    assert!(
        after_selected.contains("## role"),
        "two-space target must receive the repair:\n{after_selected}"
    );
    assert_eq!(
        after_twin, twin_before,
        "one-space twin must be untouched:\n{after_twin}"
    );

    let out = run(cwd, &["list", "a  b/missing"]);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "list");
    let list_fix = *fix_payloads(&stdout)
        .first()
        .expect("E-DIR-MISSING list repair missing");
    let listed = consume_emitted_ok(cwd, list_fix);
    assert_eq!(
        first_object(&listed),
        "a b",
        "DIR first-line object stays folded:\n{listed}"
    );
    assert!(
        leaf_row(&listed, "onlyhere", "post").is_some(),
        "DIR repair must list the two-space tree:\n{listed}"
    );
    assert!(
        leaf_row(&listed, "twin", "post").is_none(),
        "DIR repair must not list the one-space twin:\n{listed}"
    );
}

/// Runtime targets retain their own whitespace and option-value boundary.
#[cfg(unix)]
#[test]
fn emitted_regex_repair_preserves_runtime_target() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::write(cwd.join("-target  file"), b"<pattern>\n").unwrap();
    fs::write(cwd.join("-target file"), b"different sibling\n").unwrap();
    fs::write(
        cwd.join("notes.agent-besties.md"),
        format!(
            "# notes.brief\n\n## parser\n- path: -target  file\n- hash: {}\n- regex: (\n",
            "0".repeat(64)
        ),
    )
    .unwrap();

    let out = run(cwd, &["check", "notes"]);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    let repair = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| fix.contains("--regex"))
        .unwrap_or_else(|| panic!("missing regex repair:\n{stdout}"));
    consume_emitted_ok(cwd, repair);

    let content = fs::read_to_string(cwd.join("notes.agent-besties.md")).unwrap();
    assert!(content.lines().any(|line| line == "- path: -target  file"));
    assert_eq!(
        fs::read(cwd.join("-target file")).unwrap(),
        b"different sibling\n"
    );
    let checked = run(cwd, &["check", "notes"]);
    assert!(checked.status.success(), "{}", describe(&checked));
    let stdout = String::from_utf8_lossy(&checked.stdout);
    assert!(
        stdout.lines().any(|line| columns(line).contains(&"fresh")),
        "repaired regex and hash must verify against the original target:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// R8 shell identity, diagnostic recovery and usage emission
// ---------------------------------------------------------------------------

/// Bash brace expansion must not silently select the numeric sibling.
#[cfg(unix)]
#[test]
fn emitted_brace_refresh_addresses_literal_entry_not_expanded_sibling() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::write(cwd.join("wanted.txt"), b"wanted contents\n").unwrap();
    fs::write(cwd.join("other.txt"), b"other contents\n").unwrap();
    let file = cwd.join("plan.agent-besties.md");
    let before = concat!(
        "# plan.brief\n\n",
        "## {1..1}\n- path: wanted.txt\n\n",
        "## 1\n- path: other.txt\n",
    );
    fs::write(&file, before).unwrap();

    let out = run(cwd, &["check", "plan"]);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    let repair = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| fix.contains("--refresh"))
        .expect("missing first entry hash repair");
    consume_emitted_ok(cwd, repair);

    let after = fs::read_to_string(&file).unwrap();
    assert!(
        section_has_sha256(heading_section(&after, "{1..1}")),
        "the literal brace entry must receive its hash:\n{after}"
    );
    assert_eq!(
        heading_section(&after, "1"),
        heading_section(before, "1"),
        "the numeric sibling must remain byte-for-byte unchanged"
    );
}

/// Filename-origin prose is sanitized without redirecting native lookup
/// to the display-equivalent sibling. Other path display is not escaped.
#[cfg(unix)]
#[test]
fn filename_diagnostics_fold_controls_without_changing_native_identity() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let sibling = "# untouched.agent-profile\n\n## role\n```md\nkeep sibling\n```\n";
    for (stem, display, body, kind) in [
        ("bad\nname", "bad name", "# .state\n", "state"),
        ("other\nname", "other name", "# legal.post\n", "post"),
        ("bad\u{1b}name", "bad\u{fffd}name", "# .state\n", "state"),
        (
            "other\u{1b}name",
            "other\u{fffd}name",
            "# legal.post\n",
            "post",
        ),
    ] {
        let file = cwd.join(format!("{stem}.agent-besties.md"));
        let twin = cwd.join(format!("{display}.agent-besties.md"));
        fs::write(&file, body).unwrap();
        fs::write(&twin, sibling).unwrap();
        let out = run(cwd, &["check", stem]);
        assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
        let stdout = assert_first_line(&out, "warn", "check");
        assert!(
            stdout.lines().any(|line| columns(line) == ["type", kind]),
            "check must read the native filename rather than its display twin:\n{stdout}"
        );
        let diagnostic = stdout
            .lines()
            .find(|line| {
                let fields = columns(line);
                fields.first() == Some(&"warn") && fields.get(1) == Some(&"L1")
            })
            .expect("missing filename warning");
        assert!(
            diagnostic.contains(display),
            "the complete display stem must stay on one diagnostic line:\n{stdout}"
        );
        assert!(!diagnostic.chars().any(char::is_control));
        assert_eq!(fs::read(&file).unwrap(), body.as_bytes());
        assert_eq!(fs::read(&twin).unwrap(), sibling.as_bytes());
    }
}

/// A body repair must not emit a section command that admission rejects.
#[test]
fn invalid_profile_section_body_repair_is_manual() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("invalid-profile.agent-besties.md");
    let before = "# invalid-profile.agent-profile\n\n## my role\n";
    fs::write(&file, before).unwrap();

    let out = run(cwd, &["check", "invalid-profile"]);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    let repairs = fix_payloads(&stdout);
    let body_repair = repairs.first().expect("missing W-BODY repair");
    assert!(
        body_repair.contains("heading") && body_repair.contains("body"),
        "the unaddressable section needs manual heading and body guidance:\n{stdout}"
    );
    assert!(
        repairs.iter().all(|fix| !fix.contains("--section")),
        "invalid section recovery must not emit a known-failing command:\n{stdout}"
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), before);
}

/// clap's invalid-value path must use the same one-line boundary as
/// custom usage errors, retaining the operand, option and help hint.
#[test]
fn parser_error_with_newline_is_one_stderr_line() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let out = run(
        cwd,
        &[
            "post",
            "standup",
            "--as",
            "alice",
            "-m",
            "text",
            "--edit",
            "bad\nvalue",
        ],
    );
    assert_eq!(out.status.code(), Some(2), "{}", describe(&out));
    assert!(out.stdout.is_empty(), "{}", describe(&out));
    let stderr = std::str::from_utf8(&out.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 1, "{stderr:?}");
    assert!(!stderr.contains('\r'), "{stderr:?}");
    assert!(stderr.starts_with("error: "), "{stderr:?}");
    assert_eq!(stderr.matches("error:").count(), 1, "{stderr:?}");
    assert!(stderr.contains("bad value"), "{stderr:?}");
    assert!(stderr.contains("--edit"), "{stderr:?}");
    assert!(stderr.contains("--help"), "{stderr:?}");
    assert!(!cwd.join("standup.agent-besties.md").exists());
}

#[test]
fn explicit_help_and_version_keep_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let help = run(cwd, &["--help"]);
    assert_eq!(help.status.code(), Some(0), "{}", describe(&help));
    assert!(help.stderr.is_empty(), "{}", describe(&help));
    let stdout = std::str::from_utf8(&help.stdout).unwrap();
    assert!(stdout.contains("Usage:"), "{stdout}");
    assert!(stdout.contains("post"), "{stdout}");
    assert!(stdout.lines().count() > 1, "{stdout}");

    let version = run(cwd, &["--version"]);
    assert_eq!(version.status.code(), Some(0), "{}", describe(&version));
    assert!(version.stderr.is_empty(), "{}", describe(&version));
    let stdout = std::str::from_utf8(&version.stdout).unwrap();
    assert_eq!(
        stdout.trim_end(),
        format!("agent-coexist {}", env!("CARGO_PKG_VERSION"))
    );
}

// ---------------------------------------------------------------------------
// R10 recovery: copyable headings, dirty-state argv retry, invalid status
// ---------------------------------------------------------------------------

/// Copy E-HEAD's file-content example, filling only its reader-owned nick.
/// The real parser must recover the comment and its existing fenced body.
#[test]
fn emitted_ehead_header_is_copyable_file_content() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("broken.agent-besties.md");
    let before = concat!(
        "# broken.post\n\n",
        "## #1 alice (not-a-time)\n\n",
        "```text\nretained comment\n```\n",
    );
    fs::write(&file, before).unwrap();

    let out = run(cwd, &["check", "broken"]);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "check");
    let header_fix = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| fix.contains("## #"))
        .unwrap_or_else(|| panic!("missing E-HEAD heading suggestion:\n{stdout}"));
    let heading = copyable_heading(header_fix).replace("<nick>", "alice");
    fs::write(&file, before.replace("## #1 alice (not-a-time)", &heading)).unwrap();

    let checked = run(cwd, &["check", "broken"]);
    assert!(checked.status.success(), "{}", describe(&checked));
    let stdout = assert_first_line(&checked, "ok", "check");
    assert!(
        stdout
            .lines()
            .any(|line| columns(line) == ["comments", "1"]),
        "copied header must parse as one comment:\n{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| columns(line) == ["topic", "#1", "alice", "retained comment"]),
        "recovered comment must expose its author and preserved body:\n{stdout}"
    );
}

/// Copy the Unicode heading from W-SECTION-NAME as file content rather
/// than interpreting it as shell syntax; its section must become usable.
/// An unrelated section survives the copy and a later profile write.
#[test]
fn emitted_section_heading_repairs_unicode_title() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("role-card.agent-besties.md");
    let before = concat!(
        "# role-card.agent-profile\n\n",
        "## notes\n",
        "```md\nkeep-notes-bytes\n```\n\n",
        "## 角色!\n",
        "```md\nkeep this note\n```\n",
    );
    fs::write(&file, before).unwrap();

    let out = run(cwd, &["check", "role-card"]);
    assert!(out.status.success(), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    let heading_fix = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| fix.contains("## "))
        .unwrap_or_else(|| panic!("missing section heading suggestion:\n{stdout}"));
    fs::write(
        &file,
        before.replace("## 角色!", copyable_heading(heading_fix)),
    )
    .unwrap();

    let checked = run(cwd, &["check", "role-card"]);
    assert!(checked.status.success(), "{}", describe(&checked));
    let stdout = assert_first_line(&checked, "ok", "check");
    assert!(
        stdout
            .lines()
            .any(|line| columns(line) == ["section", "角色", "keep this note"]),
        "reparsed Unicode section must retain its content:\n{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| columns(line) == ["section", "notes", "keep-notes-bytes"]),
        "unrelated section must survive the heading copy:\n{stdout}"
    );

    let written = run(
        cwd,
        &[
            "profile",
            "role-card",
            "--section",
            "角色",
            "-m",
            "written-body",
        ],
    );
    assert!(written.status.success(), "{}", describe(&written));
    let after = fs::read_to_string(&file).unwrap();
    assert!(
        after.contains("## notes\n```md\nkeep-notes-bytes\n```\n"),
        "profile write must preserve unrelated section bytes:\n{after}"
    );
    assert!(
        after.contains("written-body"),
        "profile write must address the repaired Unicode section:\n{after}"
    );
}

/// Execute the emitted retry after repairing only the dirty fragment.
/// A scoped observing removal must retain the unrelated live locked entry.
#[cfg(unix)]
#[test]
fn emitted_dirty_observing_retry_keeps_locked() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::create_dir(cwd.join("tree")).unwrap();
    let file = cwd.join("tree/state.agent-besties.md");
    let before = concat!(
        "# .state\n\n",
        "## observing\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z, leftover-garbage\n",
        "\n",
        "## locked\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    fs::write(&file, before).unwrap();

    let out = run(
        cwd,
        &[
            "state",
            "tree",
            "--remove",
            "alice",
            "--status",
            "observing",
            "--as",
            "alice",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "state");
    assert_eq!(fs::read_to_string(&file).unwrap(), before);
    let retry = *fix_payloads(&stdout)
        .first()
        .expect("missing dirty-line retry");

    fs::write(&file, before.replace(", leftover-garbage", "")).unwrap();
    consume_emitted_ok(cwd, retry);
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        concat!(
            "# .state\n\n",
            "## locked\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
        ),
        "retry must remove observing only, preserving the file and live locked declaration"
    );
}

/// Duplication, inverted-range and expiry diagnostics retain their raw
/// invalid status identity and cannot target the valid normalized sibling.
#[test]
fn invalid_stored_status_repairs_are_manual_not_status_commands() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("state.agent-besties.md");
    let before = concat!(
        "# .state\n\n",
        "## Bad Name\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
        "alice 2026-09-17T10:00:00Z..2026-09-17T14:00:00Z\n",
        "bob 2026-09-17T14:00:00Z..2026-09-17T13:00:00Z\n",
        "carol 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z\n",
        "\n",
        "## bad-name\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
        "bob 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
        "carol 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    fs::write(&file, before).unwrap();

    let out = run(cwd, &["check", "state"]);
    assert!(out.status.success(), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    // Each affected recovery path must actually appear, not just the
    // invalid-heading warning at L3: duplicate L5, range L6, expiry L7.
    for anchor in ["L5", "L6", "L7"] {
        assert!(
            stdout
                .lines()
                .any(|line| columns(line).starts_with(&["warn", anchor])),
            "missing declaration diagnostic at {anchor}:\n{stdout}"
        );
    }
    for status in ["Bad Name", "bad-name"] {
        assert!(
            stdout
                .lines()
                .any(|line| columns(line).starts_with(&[status, "alice"])),
            "raw and normalized status identities must remain distinct:\n{stdout}"
        );
    }
    assert!(
        fix_payloads(&stdout)
            .iter()
            .all(|fix| !fix.contains("--status")),
        "unaddressable statuses require manual heading repair, not a status command:\n{stdout}"
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), before);
}

// ---------------------------------------------------------------------------
// R11 empty scalar columns stay present as `/`
// ---------------------------------------------------------------------------

/// Ordinary CLI `--entry=` stores an empty title. Create, wait, and note
/// update must keep that title as a `/` column under `/ {2,}/`; stored
/// heading and note bytes change only by the intended note body.
#[test]
fn empty_brief_title_keeps_slash_columns_through_wait_diff() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::write(cwd.join("target.txt"), b"target\n").unwrap();

    let created = run(
        cwd,
        &[
            "brief",
            "b",
            "--target",
            "target.txt",
            "--entry=",
            "-m",
            "old",
        ],
    );
    assert_eq!(created.status.code(), Some(0), "{}", describe(&created));
    let created_out = assert_first_line(&created, "ok", "brief");
    let entry_row = created_out
        .lines()
        .find(|line| columns(line).first() == Some(&"entry"))
        .unwrap_or("");
    assert_eq!(
        columns(entry_row),
        ["entry", "/", "added", "target.txt"],
        "empty title must stay a slash column on create:\n{created_out}"
    );

    let file = cwd.join("b.agent-besties.md");
    let before = fs::read_to_string(&file).expect("read created brief");
    assert!(
        before.contains("## \n"),
        "stored heading must keep the empty title:\n{before}"
    );
    assert!(
        before.contains("```md\nold\n```"),
        "stored note must keep the created body:\n{before}"
    );

    let stdout_path = cwd.join("wait.out");
    let stderr_path = cwd.join("wait.err");
    let mut child = detached_cli(cwd);
    child
        .args(["wait", "b"])
        .stdin(Stdio::null())
        .stdout(fs::File::create(&stdout_path).expect("wait stdout file"))
        .stderr(fs::File::create(&stderr_path).expect("wait stderr file"));
    let mut child = child.spawn().expect("spawn wait");

    let ready_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        if stderr
            .lines()
            .any(|line| columns(line) == ["waiting", "wait", "b"])
        {
            break;
        }
        if let Some(status) = child.try_wait().expect("poll wait before ready") {
            let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
            let _ = child.kill();
            panic!(
                "wait exited before ready: {status:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
            );
        }
        if Instant::now() > ready_deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "wait never printed ready\n--- stderr ---\n{}",
                fs::read_to_string(&stderr_path).unwrap_or_default()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let updated = run(cwd, &["brief", "b", "--entry=", "-m", "new"]);
    if updated.status.code() != Some(0) {
        let _ = child.kill();
        let _ = child.wait();
    }
    assert_eq!(updated.status.code(), Some(0), "{}", describe(&updated));

    let exit_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if child.try_wait().expect("poll wait after update").is_some() {
            break;
        }
        if Instant::now() > exit_deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "wait did not finish after the note update\n--- stdout ---\n{}\n--- stderr ---\n{}",
                fs::read_to_string(&stdout_path).unwrap_or_default(),
                fs::read_to_string(&stderr_path).unwrap_or_default()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let wait_out = fs::read_to_string(&stdout_path).expect("read wait stdout");
    let wait_first = columns(first_line(&wait_out));
    assert!(
        wait_first.len() >= 3
            && wait_first[0] == "ok"
            && wait_first[1] == "wait"
            && wait_first[2] == "b",
        "expected first line `ok  wait  b`, got {:?}\n{wait_out}",
        first_line(&wait_out)
    );
    let update_row = wait_out
        .lines()
        .find(|line| columns(line).first() == Some(&"update"))
        .unwrap_or("");
    assert_eq!(
        columns(update_row),
        ["update", "entry", "/", "note"],
        "empty title must stay a slash column on wait diff:\n{wait_out}"
    );
    assert!(
        wait_out.lines().any(|line| line == "    | old"),
        "wait before text must keep the previous note:\n{wait_out}"
    );
    assert!(
        wait_out.lines().any(|line| line == "    | new"),
        "wait after text must keep the updated note:\n{wait_out}"
    );

    let after = fs::read_to_string(&file).expect("read updated brief");
    let expected = before.replace("```md\nold\n```", "```md\nnew\n```");
    assert_eq!(
        after, expected,
        "only the note body may change; empty title bytes stay"
    );
}

/// A collaboration file whose stem and suffix-stripped object fold to
/// empty still occupies those columns as `/` on `list` and `check`.
#[test]
fn empty_list_stem_and_check_object_keep_slash_columns() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::write(cwd.join(".agent-besties.md"), b"not a heading\n").unwrap();

    let list = run(cwd, &["list", "."]);
    assert!(list.status.success(), "{}", describe(&list));
    let list_out = String::from_utf8_lossy(&list.stdout);
    assert!(
        list_out.lines().any(|line| columns(line) == ["/", "?"]),
        "empty list stem must stay a slash column:\n{list_out}"
    );

    let check = run(cwd, &["check", ".agent-besties.md"]);
    let check_out = String::from_utf8_lossy(&check.stdout);
    assert_eq!(
        columns(first_line(&check_out)).get(2).copied(),
        Some("/"),
        "empty first-line object must stay a slash column:\n{check_out}"
    );
}

/// A valid suffix-only `.agent-besties.md` waits with empty object: both
/// the ready line and the final report keep `/` as that column. Wait does
/// not rewrite stored bytes.
#[test]
fn suffix_only_wait_ready_and_report_keep_slash_object() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join(".agent-besties.md");
    let baseline = "# .state\n";
    fs::write(&file, baseline).unwrap();

    let stdout_path = cwd.join("wait.out");
    let stderr_path = cwd.join("wait.err");
    let mut child = detached_cli(cwd);
    child
        .args(["wait", ".agent-besties.md"])
        .stdin(Stdio::null())
        .stdout(fs::File::create(&stdout_path).expect("wait stdout file"))
        .stderr(fs::File::create(&stderr_path).expect("wait stderr file"));
    let mut child = child.spawn().expect("spawn wait");

    let ready_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        if stderr
            .lines()
            .any(|line| columns(line) == ["waiting", "wait", "/"])
        {
            break;
        }
        if let Some(status) = child.try_wait().expect("poll wait before ready") {
            let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
            let _ = child.kill();
            panic!(
                "wait exited before ready: {status:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
            );
        }
        if Instant::now() > ready_deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "wait never printed ready `/` object\n--- stderr ---\n{}",
                fs::read_to_string(&stderr_path).unwrap_or_default()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    assert_eq!(
        fs::read_to_string(&file).expect("baseline still stored"),
        baseline,
        "wait must not rewrite the suffix-only file while idle"
    );

    let after = concat!(
        "# .state\n\n",
        "## observing\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
    );
    fs::write(&file, after).unwrap();

    let exit_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if child.try_wait().expect("poll wait after change").is_some() {
            break;
        }
        if Instant::now() > exit_deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "wait did not finish after the suffix-only change\n--- stdout ---\n{}\n--- stderr ---\n{}",
                fs::read_to_string(&stdout_path).unwrap_or_default(),
                fs::read_to_string(&stderr_path).unwrap_or_default()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let wait_out = fs::read_to_string(&stdout_path).expect("read wait stdout");
    let wait_first = columns(first_line(&wait_out));
    assert!(
        wait_first.len() >= 3 && wait_first[1] == "wait" && wait_first[2] == "/",
        "final wait object must stay `/`, got {:?}\n{wait_out}",
        first_line(&wait_out)
    );
    let stderr = fs::read_to_string(&stderr_path).expect("read wait stderr");
    assert!(
        stderr
            .lines()
            .any(|line| columns(line) == ["waiting", "wait", "/"]),
        "ready object must stay `/`:\n{stderr}"
    );
    assert_eq!(
        fs::read_to_string(&file).expect("read suffix-only file after hit"),
        after,
        "wait must not rewrite stored bytes on hit"
    );
}

// ---------------------------------------------------------------------------
// R12 selected removal recovery and include display versus native identity
// ---------------------------------------------------------------------------

/// Missing observing/alice must not invite removal of locked/alice.
/// Consume the emitted inspection using its native, not display-folded, DIR.
#[cfg(unix)]
#[test]
fn absent_observing_recovery_keeps_locked_and_inspects_native_dir() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::create_dir(cwd.join("tree  raw")).unwrap();
    fs::create_dir(cwd.join("tree raw")).unwrap();
    let file = cwd.join("tree  raw/state.agent-besties.md");
    let twin = cwd.join("tree raw/state.agent-besties.md");
    let before = concat!(
        "# .state\n\n",
        "## locked\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    let twin_before = concat!(
        "# .state\n\n",
        "## observing\n",
        "mallory 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    fs::write(&file, before).unwrap();
    fs::write(&twin, twin_before).unwrap();

    let out = run(
        cwd,
        &[
            "state",
            "tree  raw",
            "--remove",
            "alice",
            "--status",
            "observing",
            "--as",
            "bob",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{}", describe(&out));
    let stdout = assert_first_line(&out, "error", "state");
    let reason = reason_row(&stdout).expect("missing selected-removal reason");
    assert!(
        reason.contains("alice") && reason.contains("observing"),
        "absence must name the requested nick and status, not deny every declaration:\n{stdout}"
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), before);

    let fixes = fix_payloads(&stdout);
    assert_eq!(fixes.len(), 1, "expected one recovery:\n{stdout}");
    let inspected = consume_emitted(cwd, fixes[0]);
    assert!(inspected.status.success(), "{}", describe(&inspected));
    let inspection = assert_first_line(&inspected, "ok", "check-state");
    assert!(
        inspection.lines().any(|line| {
            let row = columns(line);
            row.starts_with(&["locked", "alice"])
        }),
        "inspection must show the original directory's live lock:\n{inspection}"
    );
    assert!(
        !inspection.contains("mallory"),
        "inspection must not follow the display-equivalent directory:\n{inspection}"
    );
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        before,
        "consuming recovery must preserve the live lock and its file"
    );
    assert_eq!(fs::read_to_string(&twin).unwrap(), twin_before);
}

/// Include hints are one-line scalar columns, while matching keeps raw
/// whitespace. Different file types expose a folded-name misselection.
#[test]
fn include_hint_columns_fold_without_changing_raw_glob_matching() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let raw_glob = "a  b.agent-besties.md";
    let out = run(
        cwd,
        &[
            "list",
            ".",
            "--include",
            raw_glob,
            "--include",
            "missing\n\u{1b}end",
        ],
    );
    assert!(out.status.success(), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "list");
    let hints: Vec<_> = stdout
        .lines()
        .map(columns)
        .filter(|row| row.first() == Some(&"include"))
        .collect();
    assert_eq!(
        hints.len(),
        2,
        "each unmatched glob needs one row:\n{stdout}"
    );
    assert!(
        hints.iter().all(|row| row.len() == 3),
        "each include row needs a key, one glob column and one hint column:\n{stdout}"
    );
    assert_eq!(hints[0][1], "a b.agent-besties.md");
    assert_eq!(hints[1][1], "missing \u{fffd}end");
    assert!(
        !stdout.contains('\u{1b}'),
        "raw ESC reached output:\n{stdout}"
    );

    fs::write(cwd.join(raw_glob), "# raw.post\n").unwrap();
    fs::write(cwd.join("a b.agent-besties.md"), "# folded.brief\n").unwrap();
    let selected = run(cwd, &["list", ".", "--include", raw_glob]);
    assert!(selected.status.success(), "{}", describe(&selected));
    let selected_out = assert_first_line(&selected, "ok", "list");
    assert!(
        leaf_row(&selected_out, "a b", "post").is_some(),
        "raw double-space glob must select the post:\n{selected_out}"
    );
    assert!(
        leaf_row(&selected_out, "a b", "brief").is_none(),
        "folded single-space sibling must not be selected:\n{selected_out}"
    );
    assert!(
        selected_out
            .lines()
            .any(|line| columns(line) == ["total", "1"]),
        "exactly the raw native filename must match:\n{selected_out}"
    );
}

fn state_mutation_fixes(stdout: &str) -> Vec<&str> {
    fix_payloads(stdout)
        .into_iter()
        .filter(|fix| {
            let command = fix.split_once(": ").map_or(*fix, |(_, command)| command);
            command.contains("agent-coexist state")
        })
        .collect()
}

fn quoted_heading_examples(stdout: &str) -> Vec<&str> {
    fix_payloads(stdout)
        .into_iter()
        .flat_map(|fix| fix.split('"'))
        .filter(|part| part.starts_with("## "))
        .collect()
}

// ---------------------------------------------------------------------------
// R13 recovery: selected-state mutation recipes vs inspected-file guidance
// ---------------------------------------------------------------------------

/// Noncanonical draft `.state` documents must not emit directory-level state
/// writes that would retarget a separately seeded canonical sibling. Check
/// and E-TYPE failure leave both files untouched; a later hand edit changes
/// only the inspected draft.
#[test]
fn noncanonical_state_recovery_stays_manual_and_local() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::create_dir(cwd.join("tree")).unwrap();
    let draft = cwd.join("tree/draft.agent-besties.md");
    let canonical = cwd.join("tree/state.agent-besties.md");
    let draft_before = concat!(
        "# .state\n\n",
        "## observing\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
        "bob 2026-09-17T14:00:00Z..2026-09-17T13:00:00Z\n",
        "carol 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z\n",
        "\n",
        "## locked\n",
        "dave 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
        "erin 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    let canonical_before = concat!(
        "# .state\n\n",
        "## locked\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    fs::write(&draft, draft_before).unwrap();
    fs::write(&canonical, canonical_before).unwrap();

    let out = run(cwd, &["check", "tree/draft"]);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    for anchor in ["L5", "L6", "L7"] {
        assert!(
            stdout
                .lines()
                .any(|line| columns(line).starts_with(&["warn", anchor])),
            "missing declaration diagnostic at {anchor}:\n{stdout}"
        );
    }
    assert!(
        stdout.contains("dave") && stdout.contains("erin"),
        "conflict pair must stay attached to the inspected draft:\n{stdout}"
    );
    assert!(
        state_mutation_fixes(&stdout).is_empty(),
        "noncanonical check must not emit a directory state write:\n{stdout}"
    );
    assert_eq!(fs::read_to_string(&draft).unwrap(), draft_before);
    assert_eq!(fs::read_to_string(&canonical).unwrap(), canonical_before);

    for (command, args) in [
        (
            "post",
            &["post", "tree/draft", "--as", "alice", "-m", "nope"] as &[&str],
        ),
        ("read-brief", &["read-brief", "tree/draft"] as &[&str]),
        ("wait-post", &["wait-post", "tree/draft"] as &[&str]),
    ] {
        let typed = run(cwd, args);
        assert_eq!(
            typed.status.code(),
            Some(1),
            "{command} {}",
            describe(&typed)
        );
        let typed_out = assert_first_line(&typed, "error", command);
        assert!(
            state_mutation_fixes(&typed_out).is_empty(),
            "{command} noncanonical E-TYPE must not redirect a state write at the sibling:\n{typed_out}"
        );
        assert_eq!(fs::read_to_string(&draft).unwrap(), draft_before);
        assert_eq!(fs::read_to_string(&canonical).unwrap(), canonical_before);
    }

    let repaired = concat!(
        "# .state\n\n",
        "## observing\n",
        "drafty 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    fs::write(&draft, repaired).unwrap();
    let recheck = run(cwd, &["check", "tree/draft"]);
    assert!(recheck.status.success(), "{}", describe(&recheck));
    let recheck_out = String::from_utf8_lossy(&recheck.stdout);
    assert!(
        recheck_out
            .lines()
            .any(|line| columns(line).starts_with(&["observing", "drafty"])),
        "hand repair must make the draft's own nick visible:\n{recheck_out}"
    );
    assert_eq!(fs::read_to_string(&draft).unwrap(), repaired);
    assert_eq!(fs::read_to_string(&canonical).unwrap(), canonical_before);
}

/// A cased reserved name is distinct only when the filesystem stores both.
/// Then only the directory-selected file may emit mutation recipes; aliases
/// must not be treated as two files.
#[test]
fn cased_reserved_state_only_selected_emits_mutation_recipes() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let lower = cwd.join("state.agent-besties.md");
    let upper = cwd.join("STATE.agent-besties.md");
    let lower_body = concat!(
        "# .state\n\n",
        "## observing\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    let upper_body = concat!(
        "# .state\n\n",
        "## observing\n",
        "zoe 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
        "zoe 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
    );
    fs::write(&lower, lower_body).unwrap();
    fs::write(&upper, upper_body).unwrap();
    let distinct = fs::read(&lower).unwrap() != fs::read(&upper).unwrap();

    if !distinct {
        let waiting = run(cwd, &["wait-state", "."]);
        assert_eq!(waiting.status.code(), Some(1), "{}", describe(&waiting));
        let out = run(cwd, &["check", "state"]);
        assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
        let stdout = assert_first_line(&out, "warn", "check");
        assert!(
            !state_mutation_fixes(&stdout).is_empty(),
            "aliased reserved name is the selected state file:\n{stdout}"
        );
        assert_eq!(fs::read_to_string(&lower).unwrap(), upper_body);
        assert_eq!(fs::read_to_string(&upper).unwrap(), upper_body);
        return;
    }

    let waiting = run(cwd, &["wait-state", "."]);
    assert_eq!(waiting.status.code(), Some(1), "{}", describe(&waiting));
    assert_eq!(fs::read(&lower).unwrap(), lower_body.as_bytes());
    assert_eq!(fs::read(&upper).unwrap(), upper_body.as_bytes());

    let lower_check = run(cwd, &["check", "state"]);
    let upper_check = run(cwd, &["check", "STATE"]);
    assert_eq!(
        lower_check.status.code(),
        Some(0),
        "{}",
        describe(&lower_check)
    );
    assert_eq!(
        upper_check.status.code(),
        Some(0),
        "{}",
        describe(&upper_check)
    );
    let lower_out = assert_first_line(&lower_check, "warn", "check");
    let upper_out = assert_first_line(&upper_check, "warn", "check");
    let lower_recipes = state_mutation_fixes(&lower_out);
    let upper_recipes = state_mutation_fixes(&upper_out);
    assert_eq!(
        usize::from(!lower_recipes.is_empty()) + usize::from(!upper_recipes.is_empty()),
        1,
        "exactly one of the coexisting reserved names is selected:\nlower:\n{lower_out}\nupper:\n{upper_out}"
    );
    assert_eq!(fs::read(&lower).unwrap(), lower_body.as_bytes());
    assert_eq!(fs::read(&upper).unwrap(), upper_body.as_bytes());

    let probe = run(
        cwd,
        &["state", ".", "--as", "probe", "--status", "observing"],
    );
    assert!(probe.status.success(), "{}", describe(&probe));
    let lower_after = fs::read_to_string(&lower).unwrap();
    let upper_after = fs::read_to_string(&upper).unwrap();
    let lower_hit = lower_after.contains("probe");
    let upper_hit = upper_after.contains("probe");
    assert_eq!(
        usize::from(lower_hit) + usize::from(upper_hit),
        1,
        "directory state write must hit one selected file\nlower:\n{lower_after}\nupper:\n{upper_after}"
    );
    assert_eq!(
        lower_hit,
        !lower_recipes.is_empty(),
        "mutation recipes must belong to the selected file, not the unresolved sibling"
    );
    assert_eq!(upper_hit, !upper_recipes.is_empty());
    if lower_hit {
        assert_eq!(upper_after, upper_body);
    } else {
        assert_eq!(lower_after, lower_body);
    }
}

/// Canonical selected state files keep usable recipes. Consuming them mutates
/// only that file; a state-like sibling is left byte-identical. Reader-owned
/// placeholders are filled; DUP/RANGE/EXPIRED recipes are already concrete.
#[cfg(unix)]
#[test]
fn canonical_state_recovery_recipes_mutate_selected_only() {
    for (label, body, args, nick, name, remove) in [
        (
            "dup",
            concat!(
                "# .state\n\n",
                "## observing\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
            ),
            &["check", "tree/state"] as &[&str],
            None,
            None,
            false,
        ),
        (
            "conflict",
            concat!(
                "# .state\n\n",
                "## locked\n",
                "dave 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
                "erin 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
            ),
            &["check", "tree/state"] as &[&str],
            Some("erin"),
            None,
            true,
        ),
        (
            "etype",
            concat!(
                "# .state\n\n",
                "## locked\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
            ),
            &["post", "state", "--as", "alice", "-m", "nope"] as &[&str],
            Some("bob"),
            Some("observing"),
            false,
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let cwd = dir.path();
        let nested = label != "etype";
        if nested {
            fs::create_dir(cwd.join("tree")).unwrap();
        }
        let canonical = cwd.join(if nested {
            "tree/state.agent-besties.md"
        } else {
            "state.agent-besties.md"
        });
        let sibling = cwd.join(if nested {
            "tree/draft.agent-besties.md"
        } else {
            "draft.agent-besties.md"
        });
        let sibling_before = concat!(
            "# .state\n\n",
            "## locked\n",
            "mallory 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
        );
        fs::write(&canonical, body).unwrap();
        fs::write(&sibling, sibling_before).unwrap();

        let out = run(cwd, args);
        if label == "etype" {
            assert_eq!(out.status.code(), Some(1), "{label} {}", describe(&out));
        } else {
            assert_eq!(out.status.code(), Some(0), "{label} {}", describe(&out));
        }
        let stdout = if label == "etype" {
            assert_first_line(&out, "error", "post")
        } else {
            assert_first_line(&out, "warn", "check")
        };
        assert_eq!(fs::read_to_string(&canonical).unwrap(), body);
        assert_eq!(fs::read_to_string(&sibling).unwrap(), sibling_before);

        if label == "dup" {
            let waiting = run(cwd, &["wait-state", "tree"]);
            assert_eq!(
                waiting.status.code(),
                Some(1),
                "{label} {}",
                describe(&waiting)
            );
            assert_eq!(fs::read_to_string(&canonical).unwrap(), body);
            assert_eq!(fs::read_to_string(&sibling).unwrap(), sibling_before);
        }

        let recipes = state_mutation_fixes(&stdout);
        let chosen = if remove {
            recipes
                .iter()
                .copied()
                .find(|fix| shell_command_from_fix(fix).contains("--remove"))
                .unwrap_or_else(|| panic!("{label} missing remove recipe:\n{stdout}"))
        } else {
            recipes
                .first()
                .copied()
                .unwrap_or_else(|| panic!("{label} missing state write recipe:\n{stdout}"))
        };
        let mut filled = chosen.to_string();
        if let Some(nick) = nick {
            filled = filled.replace("<nick>", nick);
        }
        if let Some(name) = name {
            filled = filled.replace("<name>", name);
        }
        assert!(
            !filled.contains('<'),
            "{label} leftover reader placeholder:\n{filled}"
        );
        let written = consume_emitted_ok(cwd, &filled);
        assert_eq!(fs::read_to_string(&sibling).unwrap(), sibling_before);
        let after = fs::read_to_string(&canonical).unwrap();
        assert_ne!(after, body, "{label} canonical file must change");
        assert!(
            !after.contains("mallory"),
            "{label} must not copy the sibling into the selected file:\n{after}"
        );
        match label {
            "dup" => {
                assert!(
                    written.lines().any(|line| columns(line).starts_with(&[
                        "set",
                        "observing",
                        "alice"
                    ])),
                    "{label} consume must set the concrete nick:\n{written}"
                );
            }
            "conflict" => {
                assert!(
                    written
                        .lines()
                        .any(|line| columns(line).starts_with(&["removed", "locked", "erin"])),
                    "{label} consume must remove the filled nick:\n{written}"
                );
                assert!(after.contains("dave"), "{label} keeper missing:\n{after}");
                assert!(
                    !after.contains("erin"),
                    "{label} removed nick lingered:\n{after}"
                );
            }
            "etype" => {
                assert!(
                    written.lines().any(|line| columns(line).starts_with(&[
                        "set",
                        "observing",
                        "bob"
                    ])),
                    "{label} consume must use filled nick and status:\n{written}"
                );
                assert!(
                    after.contains("alice"),
                    "{label} prior lock missing:\n{after}"
                );
            }
            _ => unreachable!(),
        }
    }
}

/// Colliding W-SECTION-NAME candidates must not emit a concrete heading that
/// would duplicate an existing or simultaneously suggested title. Safe unique
/// suggestions stay copyable; colliding titles are repaired by hand.
#[test]
fn emitted_section_heading_collisions_stay_manual() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let file = cwd.join("role-card.agent-besties.md");
    let before = concat!(
        "# role-card.agent-profile\n\n",
        "## notes\n",
        "```md\nkeep-notes-bytes\n```\n\n",
        "## role\t\n",
        "```md\nkeep-role\n```\n\n",
        "## role!\n",
        "```md\nkeep-role-bang\n```\n\n",
        "## Role 2!\n",
        "```md\nkeep-safe\n```\n\n",
        "## later!\n",
        "```md\nkeep-later-bang\n```\n\n",
        "## later\n",
        "```md\nkeep-later\n```\n\n",
        "## my role!\n",
        "```md\nkeep-my-a\n```\n\n",
        "## my role?\n",
        "```md\nkeep-my-b\n```\n",
    );
    fs::write(&file, before).unwrap();

    let out = run(cwd, &["check", "role-card"]);
    assert!(out.status.success(), "{}", describe(&out));
    let stdout = assert_first_line(&out, "warn", "check");
    assert_eq!(fs::read_to_string(&file).unwrap(), before);
    let headings = quoted_heading_examples(&stdout);
    assert_eq!(
        headings.len(),
        1,
        "colliding titles must omit every concrete heading:\n{stdout}"
    );
    for banned in ["## role", "## later", "## my-role"] {
        assert!(
            headings.iter().all(|heading| *heading != banned),
            "colliding candidate {banned} must not be copyable:\n{stdout}"
        );
    }
    let heading_fix = fix_payloads(&stdout)
        .into_iter()
        .find(|fix| {
            fix.split('"').any(|part| {
                part.starts_with("## ")
                    && part != "## role"
                    && part != "## later"
                    && part != "## my-role"
            })
        })
        .unwrap_or_else(|| panic!("missing unique safe heading:\n{stdout}"));
    let heading = copyable_heading(heading_fix);
    let title = heading.strip_prefix("## ").expect("copyable heading");
    let mut repaired = before.replace("## Role 2!", heading);
    repaired = repaired.replace("## role!\n", "## role-fix\n");
    repaired = repaired.replace("## later!\n", "## later-fix\n");
    repaired = repaired.replace("## my role!\n", "## my-role-a\n");
    repaired = repaired.replace("## my role?\n", "## my-role-b\n");
    fs::write(&file, &repaired).unwrap();

    let checked = run(cwd, &["check", "role-card"]);
    assert!(checked.status.success(), "{}", describe(&checked));
    let checked_out = assert_first_line(&checked, "ok", "check");
    for (section, excerpt) in [
        (title, "keep-safe"),
        ("role-fix", "keep-role-bang"),
        ("later-fix", "keep-later-bang"),
        ("my-role-a", "keep-my-a"),
        ("my-role-b", "keep-my-b"),
        ("notes", "keep-notes-bytes"),
        ("role", "keep-role"),
        ("later", "keep-later"),
    ] {
        assert!(
            checked_out
                .lines()
                .any(|line| columns(line) == ["section", section, excerpt]),
            "missing recovered section {section}:\n{checked_out}"
        );
    }

    let written = run(
        cwd,
        &[
            "profile",
            "role-card",
            "--section",
            title,
            "-m",
            "written-safe",
        ],
    );
    assert!(written.status.success(), "{}", describe(&written));
    let extra = run(
        cwd,
        &[
            "profile",
            "role-card",
            "--section",
            "role-fix",
            "-m",
            "written-manual",
        ],
    );
    assert!(extra.status.success(), "{}", describe(&extra));
    let after = fs::read_to_string(&file).unwrap();
    assert!(
        after.contains("## notes\n```md\nkeep-notes-bytes\n```\n"),
        "unrelated notes bytes must stay:\n{after}"
    );
    assert!(
        after.contains("## role\t\n```md\nkeep-role\n```\n"),
        "unrelated existing role bytes must stay:\n{after}"
    );
    assert!(
        after.contains("## later\n```md\nkeep-later\n```\n"),
        "unrelated later bytes must stay:\n{after}"
    );
    assert!(
        after.contains("written-safe") && after.contains("written-manual"),
        "profile writes must address repaired titles:\n{after}"
    );
}

// ---------------------------------------------------------------------------
// R14 state recovery: status scope and report-time target selection
// ---------------------------------------------------------------------------

/// Consuming a conflict repair releases only the chosen nick's lock, not its
/// observing or custom declarations. These operands need no shell quoting, so
/// the emitted argv can be passed to the real binary on every native platform.
#[test]
fn emitted_conflict_removal_keeps_other_live_statuses() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    fs::create_dir(cwd.join("tree")).unwrap();
    let file = cwd.join("tree/state.agent-besties.md");
    let before = concat!(
        "# .state\n\n",
        "## locked\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
        "bob 2026-09-17T09:00:00Z..2026-09-17T16:00:00Z\n",
        "\n",
        "## observing\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T14:00:00Z\n",
        "\n",
        "## coding\n",
        "alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z\n",
    );
    fs::write(&file, before).unwrap();

    let checked = run(cwd, &["check", "tree/state"]);
    assert!(checked.status.success(), "{}", describe(&checked));
    let stdout = assert_first_line(&checked, "warn", "check");
    let recipe = state_mutation_fixes(&stdout)
        .into_iter()
        .find(|fix| fix.contains("--remove"))
        .unwrap_or_else(|| panic!("missing conflict release recipe:\n{stdout}"));
    let command = recipe
        .split_once(": ")
        .map_or(recipe, |(_, command)| command)
        .replace("<nick>", "alice");
    let args: Vec<_> = command.split_ascii_whitespace().collect();
    assert_eq!(args.first(), Some(&"agent-coexist"));
    let consumed = run(cwd, &args[1..]);
    assert!(consumed.status.success(), "{}", describe(&consumed));
    let consumed_out = assert_first_line(&consumed, "ok", "state");
    let removed: Vec<_> = consumed_out
        .lines()
        .map(columns)
        .filter(|row| row.first() == Some(&"removed"))
        .collect();
    assert_eq!(removed, [vec!["removed", "locked", "alice"]]);
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        concat!(
            "# .state\n\n",
            "## locked\n",
            "bob 2026-09-17T09:00:00Z..2026-09-17T16:00:00Z\n",
            "\n",
            "## observing\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T14:00:00Z\n",
            "\n",
            "## coding\n",
            "alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z\n",
        ),
        "conflict release must preserve every other live declaration"
    );
}

/// Both wait entry points retain warnings from a removed baseline but must
/// not offer a renewal that recreates it or mutates a newly selected cased
/// sibling. Filesystem changes happen only after the real ready line.
#[test]
fn removed_wait_state_recovery_does_not_recreate_or_retarget() {
    for (command, args, object, removed_path) in [
        (
            "wait",
            &["wait", "state"] as &[&str],
            "state",
            "state.agent-besties.md",
        ),
        (
            "wait-state",
            &["wait-state", "."] as &[&str],
            ".",
            "./state.agent-besties.md",
        ),
    ] {
        for with_sibling in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let cwd = dir.path();
            let file = cwd.join("state.agent-besties.md");
            let sibling = cwd.join("STATE.agent-besties.md");
            let baseline = concat!(
                "# .state\n\n",
                "## observing\n",
                "alice 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z\n",
            );
            let sibling_before = concat!(
                "# .state\n\n",
                "## observing\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
                "\n",
                "## coding\n",
                "mallory 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z\n",
            );
            fs::write(&file, baseline).unwrap();
            if with_sibling {
                // create_new distinguishes native case aliases without ever
                // overwriting the inspected baseline. Only this case needs
                // two distinct names; plain removal runs on every surface.
                match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&sibling)
                {
                    Ok(probe) => {
                        drop(probe);
                        fs::remove_file(&sibling).unwrap();
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("cannot probe native case distinction: {error}"),
                }
            }

            let stdout_path = cwd.join("wait.out");
            let stderr_path = cwd.join("wait.err");
            let mut child = detached_cli(cwd);
            child
                .args(args)
                .stdin(Stdio::null())
                .stdout(fs::File::create(&stdout_path).expect("wait stdout file"))
                .stderr(fs::File::create(&stderr_path).expect("wait stderr file"));
            let mut child = child.spawn().expect("spawn wait");

            let ready_deadline = Instant::now() + Duration::from_secs(15);
            loop {
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                if stderr
                    .lines()
                    .any(|line| columns(line) == ["waiting", command, object])
                {
                    break;
                }
                if let Some(status) = child.try_wait().expect("poll wait before ready") {
                    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                    panic!(
                        "{command} exited before ready: {status:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
                    );
                }
                if Instant::now() > ready_deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("{command} never printed ready\n--- stderr ---\n{stderr}");
                }
                std::thread::sleep(Duration::from_millis(20));
            }

            let changed = if with_sibling {
                fs::write(&sibling, sibling_before).and_then(|()| fs::remove_file(&file))
            } else {
                fs::remove_file(&file)
            };
            if let Err(error) = changed {
                let _ = child.kill();
                let _ = child.wait();
                panic!("cannot remove the admitted wait baseline: {error}");
            }

            let exit_deadline = Instant::now() + Duration::from_secs(15);
            let status = loop {
                if let Some(status) = child.try_wait().expect("poll wait after removal") {
                    break status;
                }
                if Instant::now() > exit_deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "{command} did not finish after removal\n--- stdout ---\n{}\n--- stderr ---\n{}",
                        fs::read_to_string(&stdout_path).unwrap_or_default(),
                        fs::read_to_string(&stderr_path).unwrap_or_default()
                    );
                }
                std::thread::sleep(Duration::from_millis(20));
            };
            let out = Output {
                status,
                stdout: fs::read(&stdout_path).expect("read wait stdout"),
                stderr: fs::read(&stderr_path).expect("read wait stderr"),
            };
            assert_eq!(out.status.code(), Some(0), "{}", describe(&out));
            let stdout = assert_first_line(&out, "warn", command);
            assert_eq!(columns(first_line(&stdout)).get(2), Some(&object));
            for expected in [
                &["bytes", "changed"] as &[&str],
                &["semantic", "changed"],
                &["removed", removed_path],
                &["remove", "observing", "alice"],
            ] {
                assert!(
                    stdout
                        .lines()
                        .any(|line| columns(line).as_slice() == expected),
                    "missing removal report row {expected:?}:\n{stdout}"
                );
            }
            assert!(
                stdout
                    .lines()
                    .any(|line| columns(line).starts_with(&["warn", "L4"])),
                "removed baseline's declaration warning must remain:\n{stdout}"
            );

            let recipes = state_mutation_fixes(&stdout);
            // Consume any leaked renewal as an actual client would. With the
            // stale startup binding it would recreate the missing source or
            // write to the selected sibling; neither is a repair of this file.
            for recipe in &recipes {
                let recipe = *recipe;
                let command = recipe
                    .split_once(": ")
                    .map_or(recipe, |(_, command)| command);
                let args: Vec<_> = command.split_ascii_whitespace().collect();
                assert_eq!(args.first(), Some(&"agent-coexist"));
                let consumed = run(cwd, &args[1..]);
                assert!(consumed.status.success(), "{}", describe(&consumed));
            }
            assert!(
                !file.exists(),
                "wait recovery must not recreate its removed source:\n{stdout}"
            );
            if with_sibling {
                assert_eq!(
                    fs::read_to_string(&sibling).unwrap(),
                    sibling_before,
                    "wait recovery must not mutate the distinct-case sibling"
                );
            }
            assert!(
                recipes.is_empty(),
                "removed wait target must not emit a directory-level mutation:\n{stdout}"
            );
        }
    }
}
