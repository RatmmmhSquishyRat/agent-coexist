//! Real output-device errors must not become successful deliveries or panics.

#[cfg(target_os = "linux")]
mod linux {
    use std::fs::OpenOptions;
    use std::path::Path;
    use std::process::{Command, Stdio};

    fn full_device() -> Stdio {
        Stdio::from(OpenOptions::new().write(true).open("/dev/full").unwrap())
    }

    fn cli(cwd: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_agent-coexist"));
        command
            .env_clear()
            .env("AGENT_COEXIST_NOW", "2026-10-07T02:00:00Z")
            .env("HOME", cwd)
            .env("USERPROFILE", cwd)
            .env("XDG_CONFIG_HOME", cwd)
            .current_dir(cwd);
        command
    }

    #[test]
    fn information_commands_report_undelivered_stdout() {
        for args in [
            vec!["--help"],
            vec!["--version"],
            vec!["help"],
            vec!["help", "post"],
            vec!["guide", "--help"],
            vec!["guide"],
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_agent-coexist"))
                .args(&args)
                .stdout(full_device())
                .stderr(Stdio::piped())
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1), "{args:?}");
            assert!(output.stderr.is_empty(), "{args:?}: {:?}", output.stderr);
        }
    }

    #[test]
    fn usage_errors_remain_exit_two_when_stderr_cannot_be_written() {
        let dir = tempfile::tempdir().unwrap();
        for args in [
            vec!["not-a-command"],
            vec!["post", "thread", "--as", "invalid nick", "-m", "body"],
        ] {
            let output = cli(dir.path())
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(full_device())
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2), "{args:?}");
            assert!(output.stdout.is_empty());
        }
    }

    #[test]
    fn an_undelivered_write_report_keeps_the_committed_record() {
        let dir = tempfile::tempdir().unwrap();
        let output = cli(dir.path())
            .args(["post", "thread", "--as", "alice", "-m", "saved"])
            .stdout(full_device())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let record = std::fs::read_to_string(dir.path().join("thread.agent-besties.md")).unwrap();
        assert!(record.contains("## #1 alice"));
        assert!(record.contains("saved"));
    }
}
