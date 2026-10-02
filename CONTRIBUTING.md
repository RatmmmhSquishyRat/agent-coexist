# Contributing to agent-coexist

Thanks for your interest in improving agent-coexist. This is a small,
focused project, and the best contributions match that: small, focused
changes with a clear reason.

## Getting set up

You need Rust 1.89 or later. The repository pins 1.89.0 in
`rust-toolchain.toml`, so rustup installs and selects that toolchain the
first time you run `cargo` in the checkout. The pin uses rustup's
minimal profile, which leaves out rustfmt and clippy; the local checks
below need both, so add them once after the first build:

```console
git clone https://github.com/RatmmmhSquishyRat/agent-coexist.git
cd agent-coexist
cargo build
rustup component add rustfmt clippy --toolchain 1.89.0
./target/debug/agent-coexist guide
```

The binary is `target/debug/agent-coexist` (`agent-coexist.exe` on Windows).

## Project layout

- `crates/core` (`coexist-core`): parsing of the collaboration files,
  raw-byte splicing, file locking, the diagnostic catalog, and the logic
  behind the fourteen business commands. It does not own `guide`,
  `--help`, or `--version`.
- `crates/cli` (`agent-coexist`): the binary, with argument parsing and
  console rendering. Business commands call into `coexist-core`. `guide`
  is a CLI-only information branch: it embeds `crates/cli/guide/*.md` and
  writes that Markdown to stdout before any cwd, clock, storage, or lock
  setup. Do not add a core Request or Report for it.
- `.agents/skills/agent-coexist/SKILL.md`: the canonical short discovery
  skill. Consuming projects copy that folder; there is no installer.
- `vendor/ignore`: pinned `ignore` 0.4.33 with a bounded local patch; not
  a workspace member. See [Vendored `ignore` crate](#vendored-ignore-crate).

Unit tests sit next to the code they cover; integration tests live in
`crates/core/tests/` and `crates/cli/tests/`.

## Before you open a pull request

1. **Behavior changes start with an issue.** Describe what should change
   and why. Current behavior is documented in
   [the built-in guide](crates/cli/guide/index.md) (`agent-coexist guide`)
   for operational methods, [the command reference](docs/usage.md) for
   commands, flags, and the line protocol, and
   [the file format reference](docs/file-format.md) for the on-disk format.
2. **Keep it small.** One concern per pull request: a single bug fix or a
   single coherent change. Pull requests target the `main` branch.
3. **Update docs and the changelog** in the same change when a user or
   script will notice a difference; see
   [Documentation and changelog](#documentation-and-changelog).
4. **Run these checks locally.** CI runs the same on Linux, Windows, and
   macOS:

   ```console
   cargo fmt -p agent-coexist -p coexist-core -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --release --locked
   cargo test --manifest-path vendor/ignore/Cargo.toml --lib --release --locked --target-dir target/vendor-check
   ```

   `cargo fmt -p agent-coexist -p coexist-core` (without `--check`)
   applies the formatting. The package filter is intentional: it formats
   the two product crates and leaves the pinned `ignore` tree in upstream
   style. The last command runs the vendor library tests.

## Writing tests

Test what a user or script can observe: the first status line of a
business report, the rows a script would parse, exit codes, and the bytes
left on disk. Keep tests deterministic and self-contained: use temporary
directories, and fix the clock with `AGENT_COEXIST_NOW` (strict
20-character `YYYY-MM-DDTHH:MM:SSZ`; `:60` is rejected) where timestamps
matter.

Do not pin:

- the full wording of `crates/cli/guide/*.md`, `SKILL.md`, or diagnostic
  prose;
- exact quote spelling of a `fix` command (test that its operands survive
  a real POSIX shell instead);
- internal search, index, or merge structures, or timing and latency.

If your change touches one of the areas below, keep tests covering the
behavior listed for it; a regression there would mislead or harm a user.
Each item links to the reference section that defines its terms:

- **Information commands** ([Built-in guide](docs/usage.md#built-in-guide)).
  `guide`, `help [COMMAND]`, `--help`, and `--version` ignore business
  environment variables and storage; an unknown guide topic is a usage
  error. The guide and business grammars stay separate: `guide` rejects
  business options such as `guide --as alice`; bare `read` and `start`
  are usage errors, not commands; `wait PATH` and `list` still parse as
  business commands, not guide pages.
- **Recovery `fix` lines** ([Recovery commands](docs/usage.md#recovery-commands-fix)).
  Command-form fixes address the
  [native operand](docs/usage.md#how-operands-are-written) (not the folded
  display), and a POSIX shell round-trip reaches the same file. Literal
  headers and hand-edit instructions are never quoted as commands.
  Operands that cannot be represented produce whole-recipe
  [manual guidance](docs/usage.md#when-there-is-no-command).
  Placeholder-looking runtime values stay literal values.
- **Never widen a destructive recipe** ([Removing](docs/usage.md#removing)).
  A dirty-line or not-found state removal keeps the original `--status`;
  a same-file lock-conflict fix keeps `--status=locked`; an explicit
  `--remove` without `--status` still spans statuses. An unselected or
  misnamed state-content file (see the
  [reserved file](docs/usage.md#declaring)) gets hand-edit advice and
  never a directory-level `state` command against a sibling reserved
  file; rename advice never overwrites an existing state file. A wait
  `removed` report prints no diagnostics from the deleted file
  ([The wait report](docs/usage.md#the-wait-report)).
- **Heading repairs** ([Headings the CLI cannot address](docs/file-format.md#headings-the-cli-cannot-address)).
  Suggested profile section names and post header sequence numbers are
  unique across existing headings and every other repair in the same
  report; collisions and exhaustion fall back to manual advice. Invalid
  stored statuses and control-bearing headings get manual advice, not a
  command the CLI would reject or normalize.
- **Locks** ([Functional lock](docs/usage.md#functional-lock)). A
  file-symlink write honors both the supplied-parent and the
  canonical-target-parent lock chains; a matching `--as` still writes;
  incomplete scans warn rather than invent a lock
  ([Incomplete lock and conflict scans](docs/usage.md#incomplete-lock-and-conflict-scans)).
- **Order and matching** ([Comments, mentions, and replies](docs/usage.md#comments-mentions-and-replies),
  [Scans](docs/usage.md#scans-what-is-visited-and-filter-precedence)).
  Post `authors` keep first-occurrence order; write-report mentions and
  `reply-to` rows keep body order, including `(not found)`. `--keyword`
  and `--include` match the raw input; only hint display is folded and
  sanitized.
- **Conflict output** ([Scope, inheritance, and conflicts](docs/usage.md#scope-inheritance-and-conflicts)).
  `W-STATE-CONFLICT` pair diagnostics appear only on explicit `check` and
  on qualifying wait hits other than a deletion hit, and a complete check
  lists every pair. Wait
  refuses a baseline with a duplicate `(status, nick)` pair
  ([Lifecycle](docs/usage.md#lifecycle)).

## Documentation and changelog

If a change affects user-visible behavior, update the matching surface in
the same change: `crates/cli/guide/` for operational methods, `--help`
text, `README.md`, `docs/usage.md`, `docs/file-format.md`, or
`.agents/skills/agent-coexist/SKILL.md`. Add a line under
`## [Unreleased]` in `CHANGELOG.md` (the file follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)) describing what a
user or script will notice.

Keep one tutorial source: the guide pages own the operational walkthrough,
and `docs/usage.md` stays a lookup reference rather than a second copy.

## Vendored `ignore` crate

You need this section only when changing scan behavior or refreshing the
pin. Directory walking uses one pinned dependency: `ignore` 0.4.33 from
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
  `vendor/ignore/UNLICENSE`. Keep them byte-for-byte when refreshing the
  pin.
- The local delta is limited to what scans need:
  - Ignore and git-config sources are opened as regular files, and load
    errors reach the scanner. When a file's rules parse but cannot be
    compiled together (a size or nesting limit), the error names that file.
  - Glob matching uses native path bytes. Unix prefix stripping requires
    a component boundary; upstream raw `./` remainder spelling is kept.
  - Git layouts: worktree, separate-gitdir, gitfile, and `commondir`
    layouts are followed. Relative gitfile paths resolve from the
    gitfile's directory and relative `commondir` paths from the gitdir;
    ordinary `.git` directories honor `commondir` the same way; a missing
    optional `commondir` keeps that gitdir's exclude rules; a gitdir that
    is missing or not a directory is reported against that gitdir on every
    platform (Unix `ENOTDIR` included). Gitfile and
    `commondir` descriptors are read in full, with only trailing CR/LF
    trimmed. An explicit global config replaces the default global files.
  - `core.excludesFile`: only a recognized leading `~` home component
    expands, at a component boundary and using the home directory's
    native bytes; `~user` and later tildes (such as Windows short names
    like `RUNNER~1`) stay literal.
  - Windows volumes: ordinary and verbatim prefixes compare as native
    Rust `Prefix` values (`C:\` and `\\?\C:\` are one volume); prefix
    stripping consumes every remaining root component; directory
    components get no extra case-folding; DeviceNS (`\\.\`) is not a
    disk; I/O keeps the user's spelling, without canonicalizing or
    stripping `\\?\`.

  Upstream glob grammar, type/directory precedence, and license texts
  stay as published; `globset` is the unmodified crates.io crate. The
  patch does not maintain every upstream API, and upstream's whole-tree
  `min_depth` limit is outside product scope.
- To refresh the pin, re-apply that delta on the new upstream tree and
  re-run the vendor library tests. Do not drop the pin, and do not run
  `cargo fmt --all`: workspace-wide formatting can follow the path patch
  and rewrite upstream style.

## Reporting bugs

Use the [bug-report form](https://github.com/RatmmmhSquishyRat/agent-coexist/issues/new?template=bug_report.yml).
For suspected security problems, follow [SECURITY.md](SECURITY.md) instead
of opening a normal bug report. For usage questions, start with
`agent-coexist guide`, then [the command reference](docs/usage.md), then
[open an issue](https://github.com/RatmmmhSquishyRat/agent-coexist/issues).

## License

The project's own code and authored documentation are licensed under the
MIT License (see [LICENSE](LICENSE)). By contributing, you agree that your
contributions are licensed under it. There is no CLA and no DCO sign-off.

The vendored `ignore` crate in `vendor/ignore/` keeps its upstream
copyright notices and license (Unlicense OR MIT; see
[Vendored `ignore` crate](#vendored-ignore-crate)). The cover image in
`docs/assets/cover.webp` is supplied artwork and is not part of the MIT
grant; do not reuse it under MIT terms.
