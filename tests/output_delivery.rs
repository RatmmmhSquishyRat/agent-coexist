//! Real output-device errors must not become successful deliveries or panics.

use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
fn failing_device() -> Stdio {
    use std::fs::OpenOptions;
    Stdio::from(OpenOptions::new().write(true).open("/dev/full").unwrap())
}

#[cfg(windows)]
fn failing_device() -> Stdio {
    use std::os::windows::io::FromRawHandle;
    unsafe extern "system" {
        fn CreatePipe(
            hReadPipe: *mut *mut std::ffi::c_void,
            hWritePipe: *mut *mut std::ffi::c_void,
            lpPipeAttributes: *const std::ffi::c_void,
            nSize: u32,
        ) -> i32;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    }
    #[repr(C)]
    #[allow(non_snake_case)]
    struct SecurityAttributes {
        nLength: u32,
        lpSecurityDescriptor: *mut std::ffi::c_void,
        bInheritHandle: i32,
    }
    let sa = SecurityAttributes {
        nLength: std::mem::size_of::<SecurityAttributes>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: 1,
    };
    let mut read_handle = std::ptr::null_mut();
    let mut write_handle = std::ptr::null_mut();
    unsafe {
        let ret = CreatePipe(
            &mut read_handle,
            &mut write_handle,
            &sa as *const _ as *const std::ffi::c_void,
            0,
        );
        assert_ne!(ret, 0, "CreatePipe failed");
        CloseHandle(read_handle);
        Stdio::from_raw_handle(write_handle)
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
fn failing_device() -> Stdio {
    use std::os::fd::FromRawFd;
    let mut fds = [0i32; 2];
    unsafe {
        assert_eq!(libc::pipe(fds.as_mut_ptr()), 0);
        libc::close(fds[0]);
        Stdio::from_raw_fd(fds[1])
    }
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
    #[cfg(windows)]
    for var in ["SystemRoot", "SYSTEMROOT", "windir", "TMP", "TEMP"] {
        if let Some(val) = std::env::var_os(var) {
            command.env(var, val);
        }
    }
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
            .stdout(failing_device())
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
            .stderr(failing_device())
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
        .stdout(failing_device())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let record = std::fs::read_to_string(dir.path().join("thread.agent-besties.md")).unwrap();
    assert!(record.contains("## #1 alice"));
    assert!(record.contains("saved"));
}
