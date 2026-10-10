//! Brief targets use the native lock protocol, including aliases to
//! managed files and references to the brief being written.

use std::fs;
use std::path::Path;
use std::process::Output;
use std::time::Duration;

fn run(cwd: &Path, args: &[&str]) -> Output {
    let mut command = assert_cmd::Command::new(env!("CARGO_BIN_EXE_agent-coexist"));
    command
        .env_clear()
        .env("AGENT_COEXIST_NOW", "2026-10-07T02:00:00Z")
        .env("HOME", cwd)
        .env("USERPROFILE", cwd)
        .env("XDG_CONFIG_HOME", cwd)
        .current_dir(cwd)
        .args(args)
        .timeout(Duration::from_secs(8));
    #[cfg(windows)]
    command.env(
        "SYSTEMROOT",
        std::env::var_os("SYSTEMROOT").unwrap_or_default(),
    );
    command.output().expect("run agent-coexist")
}

fn stdout(output: &Output) -> String {
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn busy_alias_targets_are_unchecked_and_cannot_update_briefs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let target = root.join("thread.agent-besties.md");
    // Unique body sentinel: any output showing it leaked target bytes
    // the lock kept unreadable.
    let sentinel = "busy-target-body-6d61c7";
    fs::write(&target, format!("# thread.post\n{sentinel}\n")).unwrap();
    let alias = root.join("alias.txt");
    fs::hard_link(&target, &alias).unwrap();
    assert!(run(root, &["brief", "plan", "--target", "alias.txt"])
        .status
        .success());
    let brief = root.join("plan.agent-besties.md");
    let before = fs::read(&brief).unwrap();
    let writer = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&target)
        .unwrap();
    writer.try_lock().unwrap();

    let check = run(root, &["check", "plan"]);
    assert_eq!(check.status.code(), Some(1));
    let checked = stdout(&check);
    assert!(checked.contains("unchecked"), "{checked}");
    // Recovery retries the read that failed: the fix row's command
    // keeps the original "check plan" operands (the program token is
    // spelled as the runner's binary path) and never routes to a
    // brief --target repoint.
    let fix = checked
        .lines()
        .find(|line| line.starts_with("         fix  "))
        .expect("E-ENTRY-TARGET fix row");
    assert!(fix.ends_with(" check plan"), "{checked}");
    assert!(!fix.contains("--target"), "{checked}");
    let read = run(root, &["read-brief", "plan"]);
    let read = stdout(&read);
    assert!(read.starts_with("warn  read-brief"), "{read}");
    assert!(read.contains("unchecked"), "{read}");
    assert!(!read.contains(sentinel), "{read}");

    let refresh = run(root, &["brief", "plan", "--refresh"]);
    assert_eq!(refresh.status.code(), Some(1));
    assert!(stdout(&refresh).contains("locked by another process"));
    assert_eq!(fs::read(&brief).unwrap(), before);
    let create = run(
        root,
        &["brief", "new/plan", "--target", alias.to_str().unwrap()],
    );
    assert_eq!(create.status.code(), Some(1));
    assert!(!root.join("new").exists());

    drop(writer);
    let check = run(root, &["check", "plan"]);
    assert!(check.status.success(), "{}", stdout(&check));
    assert!(stdout(&check).contains("fresh"));
}

#[test]
fn self_target_and_refresh_hash_the_snapshot_before_the_write() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let brief = root.join("plan.agent-besties.md");
    assert!(run(root, &["brief", "plan"]).status.success());
    let before = coexist_core::hash::read_target(&brief).unwrap().sha256;
    let create = run(
        root,
        &[
            "brief",
            "plan",
            "--target",
            "plan.agent-besties.md",
            "--entry",
            "self",
        ],
    );
    assert!(create.status.success(), "{}", stdout(&create));
    assert!(fs::read_to_string(&brief)
        .unwrap()
        .contains(&format!("- hash: {before}")));

    fs::hard_link(&brief, root.join("alias.txt")).unwrap();
    let before = coexist_core::hash::read_target(&brief).unwrap().sha256;
    let update = run(
        root,
        &["brief", "plan", "--target", "alias.txt", "--entry", "self"],
    );
    assert!(update.status.success(), "{}", stdout(&update));
    assert!(fs::read_to_string(&brief)
        .unwrap()
        .contains(&format!("- hash: {before}")));

    let before = coexist_core::hash::read_target(&brief).unwrap().sha256;
    let refresh = run(root, &["brief", "plan", "--refresh"]);
    assert!(refresh.status.success(), "{}", stdout(&refresh));
    assert!(fs::read_to_string(&brief)
        .unwrap()
        .contains(&format!("- hash: {before}")));
    let check = run(root, &["check", "plan"]);
    assert!(stdout(&check).contains("shifted"));
}
