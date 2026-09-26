//! CLI-only operational guide: compile-embedded Markdown, no workspace access.
//!
//! Pages are borrowed `&'static str` from `crates/cli/guide/*.md`. Success
//! writes those bytes to stdout and checks flush. Write or flush failure is
//! exit 1 without a status-line report, panic, or file recovery.

use std::io::{self, Write};
use std::process::ExitCode;

use crate::args::GuideTopic;

/// Emit the selected page and map IO failure to exit 1.
pub fn emit(topic: Option<GuideTopic>) -> ExitCode {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match write_page(&mut out, page(topic)) {
        Ok(()) => ExitCode::from(0),
        Err(_) => ExitCode::from(1),
    }
}

fn page(topic: Option<GuideTopic>) -> &'static str {
    match topic {
        None => include_str!("../guide/index.md"),
        Some(GuideTopic::Start) => include_str!("../guide/start.md"),
        Some(GuideTopic::Read) => include_str!("../guide/read.md"),
        Some(GuideTopic::Write) => include_str!("../guide/write.md"),
        Some(GuideTopic::Wait) => include_str!("../guide/wait.md"),
    }
}

fn write_page(out: &mut impl Write, body: &str) -> io::Result<()> {
    out.write_all(body.as_bytes())?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailWrite;
    impl Write for FailWrite {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FailFlush;
    impl Write for FailFlush {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
    }

    #[test]
    fn write_page_fails_on_write_error() {
        assert!(write_page(&mut FailWrite, "page\n").is_err());
    }

    #[test]
    fn write_page_fails_on_flush_error() {
        assert!(write_page(&mut FailFlush, "page\n").is_err());
    }
}
