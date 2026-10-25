//! How a body reaches a write command: `-m <content>` or `-F <path>`
//! (workspace D-101). In a dsh plugin trial agents passed `-m 'a\nb'` from
//! shells that keep `\n` as two characters, and the text was stored on one
//! line with an `ok` report.

use std::path::Path;
use std::process::Output;

fn cli(dir: &Path, args: &[&str], stdin: Option<&[u8]>) -> Output {
    let mut command = assert_cmd::Command::new(env!("CARGO_BIN_EXE_agent-coexist"));
    command.current_dir(dir).args(args);
    if let Some(input) = stdin {
        command.write_stdin(input.to_vec());
    }
    command.output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The body lines of the only comment, between its fence lines.
fn first_body(dir: &Path, stem: &str) -> Vec<String> {
    let file = std::fs::read_to_string(dir.join(format!("{stem}.agent-coexist.md"))).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let open = lines.iter().position(|l| l.starts_with("```md")).unwrap();
    let close = open
        + 1
        + lines[open + 1..]
            .iter()
            .position(|l| l.starts_with("```"))
            .unwrap();
    lines[open + 1..close]
        .iter()
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn file_body_keeps_its_line_breaks_and_folds_crlf() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("body.txt"),
        b"first line\r\nsecond @#1 line\r\n",
    )
    .unwrap();
    let out = cli(
        dir.path(),
        &["post", "standup", "--as", "alice", "-F", "body.txt"],
        None,
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    assert!(
        text(&out.stdout).starts_with("ok  post"),
        "{}",
        text(&out.stdout)
    );
    assert_eq!(
        first_body(dir.path(), "standup"),
        ["first line", "second @#1 line"]
    );
}

#[test]
fn file_dash_reads_standard_input() {
    let dir = tempfile::tempdir().unwrap();
    let out = cli(
        dir.path(),
        &["post", "standup", "--as", "alice", "--file", "-"],
        Some(b"from stdin\nline two\n"),
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        first_body(dir.path(), "standup"),
        ["from stdin", "line two"]
    );
}

#[test]
fn unreadable_or_non_utf8_file_is_a_usage_error_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("latin1.txt"), b"caf\xe9").unwrap();
    for (file, says) in [
        ("missing.txt", "cannot read the file"),
        ("latin1.txt", "not UTF-8"),
    ] {
        let out = cli(
            dir.path(),
            &["post", "standup", "--as", "alice", "-F", file],
            None,
        );
        assert_eq!(out.status.code(), Some(2), "{file}");
        assert!(out.stdout.is_empty(), "{file}: {}", text(&out.stdout));
        let stderr = text(&out.stderr);
        assert!(
            stderr.contains(&format!("-F {file}")) && stderr.contains(says),
            "{stderr}"
        );
        assert!(
            !dir.path().join("standup.agent-coexist.md").exists(),
            "{file}"
        );
    }
}

#[test]
fn message_and_file_are_exclusive_and_one_is_required() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("body.txt"), b"x").unwrap();
    let both = cli(
        dir.path(),
        &[
            "post", "standup", "--as", "alice", "-m", "x", "-F", "body.txt",
        ],
        None,
    );
    assert_eq!(both.status.code(), Some(2));
    let neither = cli(dir.path(), &["post", "standup", "--as", "alice"], None);
    assert_eq!(neither.status.code(), Some(2));
    assert!(!dir.path().join("standup.agent-coexist.md").exists());
}

#[test]
fn literal_backslash_n_in_message_warns_with_the_way_to_line_breaks() {
    let dir = tempfile::tempdir().unwrap();
    let out = cli(
        dir.path(),
        &["post", "standup", "--as", "alice", "-m", r"line1\nline2"],
        None,
    );
    let stdout = text(&out.stdout);
    // Still written, since the text may mean the escape; exit 0 like any warn.
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert!(stdout.starts_with("warn  post"), "{stdout}");
    assert!(stdout.contains("--edit <n> -F <file>"), "{stdout}");
    assert_eq!(first_body(dir.path(), "standup"), [r"line1\nline2"]);

    // A real line break means the escape was meant: no warning.
    let out = cli(
        dir.path(),
        &[
            "post",
            "standup",
            "--as",
            "alice",
            "-m",
            "use \\n in C\nfor a newline",
        ],
        None,
    );
    assert!(
        text(&out.stdout).starts_with("ok  post"),
        "{}",
        text(&out.stdout)
    );

    // The same text from a file is taken as written.
    std::fs::write(dir.path().join("escape.txt"), br"write \n for a newline").unwrap();
    let out = cli(
        dir.path(),
        &["post", "standup", "--as", "alice", "-F", "escape.txt"],
        None,
    );
    assert!(
        text(&out.stdout).starts_with("ok  post"),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn profile_and_brief_take_file_bodies_too() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("about.txt"), b"one\ntwo\n").unwrap();
    let out = cli(
        dir.path(),
        &["profile", "alice", "--section", "about", "-F", "about.txt"],
        None,
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    let profile = std::fs::read_to_string(dir.path().join("alice.agent-coexist.md")).unwrap();
    assert!(profile.contains("one\ntwo"), "{profile}");

    let out = cli(dir.path(), &["brief", "plan", "-F", "about.txt"], None);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    let brief = std::fs::read_to_string(dir.path().join("plan.agent-coexist.md")).unwrap();
    assert!(brief.contains("one\ntwo"), "{brief}");

    // The usual combination rules apply to -F as they do to -m.
    let out = cli(dir.path(), &["profile", "alice", "-F", "about.txt"], None);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("require --section"),
        "{}",
        text(&out.stderr)
    );
}
