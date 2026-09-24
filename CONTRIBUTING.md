# Contributing to agent-coexist

Thanks for your interest in improving agent-coexist. This is a small,
focused project, and the best contributions match that: small, focused
changes with a clear reason.

## Getting set up

You need Rust 1.89 or later. The repository pins 1.89.0 in
`rust-toolchain.toml`, so a normal rustup installation picks the right
toolchain automatically.

```console
git clone https://github.com/RatmmmhSquishyRat/agent-coexist.git
cd agent-coexist
cargo build
```

The binary is `target/debug/agent-coexist` (`agent-coexist.exe` on Windows).

## Project layout

- `crates/core` (`coexist-core`) — parsing of the collaboration files,
  raw-byte splicing, file locking, and the logic behind every command.
- `crates/cli` (`agent-coexist`) — the `agent-coexist` binary: argument
  parsing and console rendering on top of `coexist-core`.

Unit tests sit next to the code they cover; integration tests live in
`crates/core/tests/` and `crates/cli/tests/`.

## Before you open a pull request

1. **Check the docs first.** Command behavior is documented in
   [the usage guide](docs/usage.md), and the on-disk format in [the file guide](docs/file-format.md).
   If you want to change behavior, open an issue first describing what
   should change and why.
2. **Keep it small.** One concern per pull request — a single bug fix or a
   single coherent change. Pull requests target the `main` branch.
3. **Run these checks locally.** CI also runs them on Linux, Windows, and macOS:

   ```console
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace --release
   ```

   (`cargo fmt --all` applies the formatting instead of checking it.)

## Writing tests

- A good regression test pins observable behavior — the first status line,
  the rows a script would parse, exit codes — rather than exact blobs of
  source text or incidental formatting details.
- Keep tests deterministic and self-contained: use temporary directories,
  and fix the clock with `AGENT_COEXIST_NOW` where timestamps matter.

## Documentation and changelog

If a change affects user-visible behavior, update `README.md`,
`docs/usage.md`, or `docs/file-format.md` to match, and add a line under
`## [Unreleased]` in `CHANGELOG.md` (the file follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)).

## Reporting bugs

Use the [bug-report form](https://github.com/RatmmmhSquishyRat/agent-coexist/issues/new?template=bug_report.yml).
For suspected security problems, follow [SECURITY.md](SECURITY.md) instead
of opening a normal bug report. For usage questions, start with
[the command guide](docs/usage.md), then [open an issue](https://github.com/RatmmmhSquishyRat/agent-coexist/issues).

## License

Code and authored documentation in this repository are licensed under the
MIT License (see [LICENSE](LICENSE)). By contributing, you agree that your
contributions are licensed under it. There is no CLA and no DCO sign-off.

The cover image in `docs/assets/cover.webp` is supplied artwork and is not
part of the MIT grant — do not reuse it under MIT terms.
