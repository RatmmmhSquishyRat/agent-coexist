//! Post and state activity times remain usable as notification boundaries.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_agent-coexist");

fn cli(dir: &Path) -> Command {
    let mut cmd = Command::new(BIN);
    cmd.env_clear()
        .env("HOME", dir)
        .env("USERPROFILE", dir)
        .env("XDG_CONFIG_HOME", dir)
        .current_dir(dir);
    #[cfg(windows)]
    cmd.env(
        "SYSTEMROOT",
        std::env::var_os("SYSTEMROOT").unwrap_or_default(),
    );
    cmd
}

fn success(output: &Output) -> &str {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::str::from_utf8(&output.stdout).expect("UTF-8 report")
}

#[test]
fn fixed_clock_is_preserved_for_create_append_and_edit() {
    let dir = tempfile::tempdir().unwrap();
    let times = [
        "2026-09-17T10:00:00Z",
        "2026-09-17T10:01:00Z",
        "2026-09-17T10:02:00Z",
    ];
    for (i, stamp) in times.iter().enumerate() {
        let mut cmd = cli(dir.path());
        cmd.env("AGENT_COEXIST_NOW", stamp).args([
            "post",
            "thread",
            "--as",
            "bob",
            "-m",
            "@alice update",
        ]);
        if i == 2 {
            cmd.args(["--edit", "1"]);
        }
        success(&cmd.output().unwrap());
    }
    let stored = fs::read_to_string(dir.path().join("thread.agent-besties.md")).unwrap();
    assert!(stored.contains(&format!("## #1 bob ({}, edited {})", times[0], times[2])));
    assert!(stored.contains(&format!("## #2 bob ({})", times[1])));

    let invalid = cli(dir.path())
        .env("AGENT_COEXIST_NOW", "not a time")
        .args(["post", "untouched", "--as", "bob", "-m", "hello"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    assert!(!dir.path().join("untouched.agent-besties.md").exists());
}

#[test]
fn fixed_state_clock_preserves_duration_and_rejects_overflow_before_creation() {
    let dir = tempfile::tempdir().unwrap();
    let output = cli(dir.path())
        .env("AGENT_COEXIST_NOW", "2026-09-17T10:00:00Z")
        .args([
            "state",
            "--as",
            "alice",
            "--status",
            "observing",
            "--for",
            "1h",
        ])
        .output()
        .unwrap();
    let range = "2026-09-17T10:00:00Z..2026-09-17T11:00:00Z";
    assert!(success(&output).contains(range));
    let stored = fs::read_to_string(dir.path().join("state.agent-besties.md")).unwrap();
    assert!(stored.contains(&format!("alice {range}")));

    let empty = tempfile::tempdir().unwrap();
    let overflow = cli(empty.path())
        .env("AGENT_COEXIST_NOW", "9999-12-31T23:59:59Z")
        .args([
            "state",
            "--as",
            "alice",
            "--status",
            "observing",
            "--for",
            "1s",
        ])
        .output()
        .unwrap();
    assert_eq!(overflow.status.code(), Some(2));
    assert!(overflow.stdout.is_empty());
    assert!(!empty.path().join("state.agent-besties.md").exists());
}

#[cfg(unix)]
mod queued {
    use super::*;
    use coexist_core::fsx::walk::Options;
    use coexist_core::ops::{
        self, Ctx, PostParams, ReadNotificationParams, StateDeclare, StateParams,
    };
    use coexist_core::report::{Report, Row, Status};
    use std::os::fd::AsRawFd;
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn second() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn next_second(after: u64) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while second() <= after {
            assert!(Instant::now() < deadline, "wall clock did not advance");
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn isolated(test: &str) -> bool {
        // Keep the core context's real clock independent of any override
        // inherited by cargo test, without changing the test runner's env.
        const CHILD: &str = "AGENT_COEXIST_WRITE_CLOCK_TEST_CHILD";
        if std::env::var_os(CHILD).as_deref() != Some(std::ffi::OsStr::new(test)) {
            let output = Command::new(std::env::current_exe().unwrap())
                .env_clear()
                .env(CHILD, test)
                .args(["--exact", test, "--nocapture"])
                .output()
                .unwrap();
            success(&output);
            return true;
        }
        false
    }

    #[test]
    fn queued_post_is_seen_when_resuming_a_successful_scan() {
        if isolated("queued::queued_post_is_seen_when_resuming_a_successful_scan") {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("thread.agent-besties.md");
        fs::write(&target, "# thread.post\n").unwrap();
        let held = fs::File::open(&target).unwrap();
        extern "C" {
            fn flock(fd: i32, operation: i32) -> i32;
        }
        assert_eq!(unsafe { flock(held.as_raw_fd(), 1) }, 0, "shared flock");

        // Construct the same context as the CLI before handing it to the
        // writer thread: startup clock capture has definitely finished.
        // Its separate file handle needs an exclusive native lock and cannot
        // commit while this thread holds the shared lock. The following wait
        // crosses a timestamp boundary, not guessed process readiness.
        let mut ctx = Ctx::new(None, None).unwrap();
        ctx.post_wait_ms = 10000;
        let params = PostParams {
            path: target,
            nick: "bob".to_string(),
            body: "@alice queued update".to_string(),
            edit: None,
        };
        let writer = thread::spawn(move || ops::post::run(&mut ctx, &params));
        next_second(second());
        let cutoff = coexist_core::time::format(coexist_core::Time::from(SystemTime::now()));
        let first = cli(dir.path())
            .args([
                "read-notification",
                ".",
                "--as",
                "alice",
                "--since",
                "0001-01-01T00:00:00Z",
                "--no-ignore",
            ])
            .output()
            .unwrap();
        let first_text = success(&first);
        assert!(first_text.starts_with("ok  "), "{first_text}");
        assert!(first_text.contains("total         0"), "{first_text}");
        assert!(!writer.is_finished(), "writer passed a held shared lock");

        drop(held);
        let written = writer.join().unwrap().unwrap();
        assert_eq!(written.exit, 0);
        next_second(second());
        let resumed = cli(dir.path())
            .args([
                "read-notification",
                ".",
                "--as",
                "alice",
                "--since",
                &cutoff,
                "--no-ignore",
            ])
            .output()
            .unwrap();
        let resumed_text = success(&resumed);
        assert!(resumed_text.starts_with("ok  "), "{resumed_text}");
        assert!(resumed_text.contains("mention"), "{resumed_text}");
        assert!(
            resumed_text.contains("@alice queued update"),
            "{resumed_text}"
        );
        assert!(resumed_text.contains("total         1"), "{resumed_text}");
    }

    fn notifications(dir: &Path, since: coexist_core::Time) -> (coexist_core::Time, Report) {
        let mut ctx = Ctx::new(None, None).unwrap();
        let cutoff = ctx.now;
        let report = ops::query::read_notification(
            &mut ctx,
            &ReadNotificationParams {
                dir: dir.to_path_buf(),
                nick: "reader".into(),
                since,
                walk: Options {
                    no_ignore: true,
                    ..Options::default()
                },
            },
        )
        .unwrap();
        (cutoff, report)
    }

    fn state_count(report: &Report) -> usize {
        report
            .rows
            .iter()
            .filter(|row| matches!(row, Row::Fields { key, .. } if key == "state"))
            .count()
    }

    #[test]
    fn queued_state_is_seen_with_its_full_duration_after_a_successful_scan() {
        if isolated("queued::queued_state_is_seen_with_its_full_duration_after_a_successful_scan") {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("state.agent-besties.md");
        fs::write(&target, "# .state\n\n").unwrap();
        for attempt in 0..3 {
            let held = fs::File::open(&target).unwrap();
            held.try_lock_shared().unwrap();
            // Cross the timestamp boundary within state's real 200 ms
            // budget. A heavily delayed scheduling attempt may legitimately
            // time out; retry only that refusal, never a missing event.
            let deadline = Instant::now() + Duration::from_secs(5);
            while !(900..=925).contains(
                &SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_millis(),
            ) {
                assert!(Instant::now() < deadline, "could not align the real clock");
                thread::sleep(Duration::from_millis(1));
            }
            let mut ctx = Ctx::new(None, None).unwrap();
            let invocation = ctx.now;
            let writer_dir = dir.path().to_path_buf();
            let writer = thread::spawn(move || {
                ops::state::run(
                    &mut ctx,
                    &StateParams::Declare(StateDeclare {
                        dir: writer_dir,
                        nick: "writer".into(),
                        status: "observing".into(),
                        for_secs: Some(3600),
                    }),
                )
                .unwrap()
            });
            next_second(invocation.timestamp() as u64);
            let (cutoff, before) = notifications(dir.path(), invocation);
            assert_eq!(before.status, Status::Ok, "{before:?}");
            assert_eq!(state_count(&before), 0, "{before:?}");
            drop(held);
            let written = writer.join().unwrap();
            if written.exit == 3 && attempt < 2 {
                continue;
            }
            assert_eq!(written.exit, 0, "{written:?}");
            let range = written
                .rows
                .iter()
                .find_map(|row| match row {
                    Row::Fields { key, values, .. } if key == "set" => values.get(2),
                    _ => None,
                })
                .expect("set report contains the declaration window");
            let (start, end) = range.split_once("..").unwrap();
            let start = coexist_core::time::parse(start).unwrap();
            let end = coexist_core::time::parse(end).unwrap();
            assert!(start >= cutoff && start > invocation, "{written:?}");
            assert_eq!(end.signed_duration_since(start).num_seconds(), 3600);
            assert!(fs::read_to_string(&target)
                .unwrap()
                .contains(&format!("writer {range}")));

            next_second(second());
            let (_, after) = notifications(dir.path(), cutoff);
            assert_eq!(after.status, Status::Ok, "{after:?}");
            assert_eq!(state_count(&after), 1, "{after:?}");
            return;
        }
    }
}
