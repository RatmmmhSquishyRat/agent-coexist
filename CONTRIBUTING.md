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
- `vendor/ignore` — pinned `ignore` 0.4.33 with a bounded local patch;
  not a workspace member. See [Vendored `ignore` crate](#vendored-ignore-crate).

Unit tests sit next to the code they cover; integration tests live in
`crates/core/tests/` and `crates/cli/tests/`.

## Vendored `ignore` crate

Directory walking uses one pinned dependency: `ignore` 0.4.33 from
crates.io, patched in-tree.

- The workspace `Cargo.toml` pins `ignore = "=0.4.33"` and redirects it
  with `[patch.crates-io] ignore = { path = "vendor/ignore" }`.
  `vendor/ignore` is listed in workspace `exclude`, so it is not a
  workspace crate.
- Upstream package: [ignore 0.4.33](https://crates.io/crates/ignore/0.4.33).
  The SHA-256 of the published `ignore-0.4.33.crate` archive is
  `00b69833ed729dc5aa7d19541d96d6cf8e9137194207a04916d658e43168402f`.
- License: Unlicense OR MIT. The license texts live at
  `vendor/ignore/COPYING`, `vendor/ignore/LICENSE-MIT`, and
  `vendor/ignore/UNLICENSE`. Keep them byte-for-byte when refreshing
  the pin.
- The local delta is bounded: ignore and git-config sources are opened
  as regular files; glob matching uses native path bytes; load errors
  are delivered to the scanner; Git metadata and source selection follow
  worktree, separate-gitdir, gitfile, and commondir layout, and an
  explicit global config replaces the default global files. Relative
  gitfile paths resolve from the gitfile's directory; relative commondir
  paths resolve from the gitdir; a missing optional commondir keeps that
  gitdir's exclude rules. Ordinary `.git` directories consult `commondir`
  the same way gitfiles do. Gitfile and commondir descriptors are read
  in full; only trailing CR/LF bytes are trimmed, so an embedded LF
  stays in the native path. In a `core.excludesFile` path read from
  git config, only a recognized leading `~` home component expands, at
  a component boundary and using the home directory's native bytes;
  interior or later tildes — including Windows short native names such
  as `RUNNER~1` — and `~user` forms stay literal. Descriptor-safe
  reads, relative bases, optional missing sources, load errors, and
  ignore grammar are unchanged. Unix matcher prefix stripping requires
  a component boundary; upstream raw `./` remainder spelling is kept.
  On Windows, ordinary and verbatim volume prefixes compare as native
  Rust `Prefix` values (`c:` and `C:` are already the same disk; `C:\`
  and `\\?\C:\` match as one volume). Prefix stripping consumes every
  remaining root component, so a shorter candidate cannot be treated as
  under that root; ordinary and verbatim spellings of the same volume
  agree on global directory excludes. Directory components are not given
  extra case-folding; DeviceNS (`\\.\`) is not treated as a disk; user
  verbatim spelling is kept for I/O — paths are not canonicalized or
  stripped of `\\?\`. Upstream glob grammar, type/directory precedence,
  and license texts stay as published. `globset` remains the unmodified
  crates.io crate. The patch does not attempt to maintain every upstream
  API. The old whole-upstream `min_depth` limit remains outside product
  scope.
- Refreshing the pin means re-applying that delta on the new upstream
  tree and re-running the vendor library tests below. Do not drop the
  pin, and do not run `cargo fmt --all` against this tree:
  workspace-wide formatting can follow the path patch and rewrite
  upstream style.

## Before you open a pull request

1. **Check the docs first.** Command behavior is documented in
   [the usage guide](docs/usage.md), and the on-disk format in [the file guide](docs/file-format.md).
   If you want to change behavior, open an issue first describing what
   should change and why.
2. **Keep it small.** One concern per pull request — a single bug fix or a
   single coherent change. Pull requests target the `main` branch.
3. **Run these checks locally.** CI also runs them on Linux, Windows, and macOS:

   ```console
   cargo fmt -p agent-coexist -p coexist-core -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --release --locked
   ```

   (`cargo fmt -p agent-coexist -p coexist-core` applies the formatting
   instead of checking it.) Package-filtered `fmt` is intentional: it
   formats the two product crates and leaves the pinned `ignore` tree
   in upstream style. CI also runs the vendor library tests with
   `cargo test --manifest-path vendor/ignore/Cargo.toml --lib --release --locked --target-dir target/vendor-check`.

## Writing tests

- A good regression test pins observable behavior — the first status line,
  the rows a script would parse, exit codes — rather than exact blobs of
  source text or incidental formatting details.
- Recovery `fix` lines are either literal file content / prose or POSIX
  words over native path operands, not the folded display column. Pin
  that command-form operands survive a POSIX consumer; do not treat
  exact quote spelling as the user API, and do not treat a literal
  header or heading instruction as a quoted command. Pin that a scoped
  dirty retry keeps the original `--status` selection; an invalid
  stored status is repaired by a distinct valid heading, not a
  `--status` command the CLI would reject or normalize. Pin that a
  scoped not-found removal names the supplied status and points at
  read-only `check-state`, not a write that drops `--status`.
- Keep tests deterministic and self-contained: use temporary directories,
  and fix the clock with `AGENT_COEXIST_NOW` where timestamps matter.
  That override is the strict 20-character `YYYY-MM-DDTHH:MM:SSZ` form
  (`:60` is rejected). Same-file lock-conflict warnings (`W-STATE-CONFLICT`)
  are expanded on explicit `check` and qualifying wait-hit reports;
  ordinary parse and scan tests should not expect that pair list. A
  complete check still lists every pair. Wait still refuses a snapshot
  at the first duplicate `(status, nick)`; pairing uses first-wins
  raw-identity indexes while ordered vectors remain the report. Pin
  that unmatched `--include` hint display is folded and
  control-sanitized while matching uses the raw glob. Pin that a
  misnamed or unselected state-content file receives hand-edit
  advice for the inspected file and never a directory-level `state`
  recipe against a sibling reserved file; rename advice must not
  overwrite a separate existing state file. Pin that a profile
  heading example is copyable only when that name is unused among
  existing headings and other repairs in the same report; colliding
  folded, later, or simultaneous names stay manual. Pin that post
  `authors` keep first-occurrence order and write `mentions` /
  `reply-to` rows keep body order, including `(not found)`. Pin
  that `--keyword` matches as typed without whitespace folding.
  Pin that a generated same-file lock-conflict removal keeps
  `--status=locked` and leaves other live statuses; an explicit
  `--remove` without `--status` still spans. Pin that a wait
  `removed` report can keep baseline warnings while refusing a
  mutating recipe against a later selected file. Pin that a
  file-symlink write honors both the supplied-parent and
  canonical-referent-parent lock chains, and that matching `--as`
  still writes. Pin that several header repairs in one post
  report reserve distinct unused sequence numbers, and that
  later exhaustion stays manual. Do not assert universal
  admission or sample latency, do not pin internal parse search,
  membership indexes, or merge/span lookup, and do not drop
  required conflict-pair output.

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
