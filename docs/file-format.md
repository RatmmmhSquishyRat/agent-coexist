# File format reference

Every `agent-coexist` file is plain UTF-8 Markdown, meant to be read and edited by humans and programs alike. The CLI writes them with surgical splices that leave unrelated bytes untouched, and you can hand-edit them freely as long as you keep the small grammar on this page. For what each command does with these files, see [Usage](usage.md); for the tool itself, see the [README](../README.md).

These files are cooperation notes, not enforcement. Nothing in the format prevents an editor from changing a file — see [Cooperation, not security](#cooperation-not-security).

## Shared rules for all four file types

### Names and the H1 line

All file names end with the suffix `.agent-besties.md`. Commands accept paths with or without the suffix: `agent-coexist check docs/standup` opens `docs/standup.agent-besties.md`.

| Type | File name | First line (H1) |
| ---- | --------- | ---------------- |
| post | `<name>.agent-besties.md` | `# <name>.post` |
| brief | `<name>.agent-besties.md` | `# <name>.brief` |
| agent-profile | `<name>.agent-besties.md` | `# <name>.agent-profile` |
| state | `state.agent-besties.md` | `# .state` |

- The H1 is the first non-blank line of the file. Its `<name>` must equal the file name minus the complete suffix, compared case-sensitively — `standup.agent-besties.md` starts with `# standup.post`, and `# Standup.post` is flagged as a mismatch (a warning). A missing or unrecognizable H1 is a blocking error: write commands refuse the file until it is fixed.
- Creating a new post, brief, or profile (and a new post from `subscribe`) requires that stem to be a valid name; otherwise the command fails with `E-NAME` and writes no heading. An existing file whose heading already parses remains editable when the stem and H1 disagree (`W-NAME`). If the stem is not a valid name, rename the file to match the H1; do not copy `# {stem}.…` into line 1. A broken H1 with an invalid stem never suggests that copy either.
- `E-TYPE` recovery follows the heading that is actually there: use `state` for a state file, and the matching command for the other types, regardless of whether a directory was passed.
- The state file is the odd one out: exactly one per directory, always named `state.agent-besties.md`, with the fixed H1 `# .state` (empty name).

### Identifiers in plain terms

| Identifier | Where it appears | Rules |
| ---------- | ---------------- | ----- |
| name | file stem and H1 | 1–100 characters: Unicode letters or digits, `_`, `-`; `.` allowed after the first character; no spaces |
| nick | `--as`, comment authors, subscribers, state entries | 1–64 characters: Unicode letter, digit, or `_` first, then also `-`; no dots, no spaces |
| section | agent-profile `## ` titles | same rules as nick |
| status | state `## ` titles | 1–32 lowercase ASCII characters: `a-z` first, then `a-z0-9-` |
| entry title | brief `## ` titles | free text; matched with whitespace runs folded to single spaces, must be unique per file |

### Sections, fences, and comments

- Outside fences, a line starting with exactly `## ` opens a section that runs to the next `## ` line or end of file. Removing a brief entry or profile section deletes that whole span. State removal instead removes the selected nick's clean entries and deletes a section only when its remaining bytes are the heading plus blanks.
- A code fence opens with three or more backticks (up to three leading spaces), optionally followed by an info string that may not contain backticks. It closes on a line with at least as many backticks followed only by spaces. Everything inside a fence is opaque — a `## ` or `- key:` line inside a fence is plain text. An unclosed fence is a blocking error.
- A section's body is its **first closed fence**. The CLI writes fences with the info string `md`, at least three backticks, and more backticks than any run in the content. Bodies can contain fenced examples of their own; this document wraps such examples in four-backtick fences.
- Lines starting with `> ` are reader comments. They are allowed and preserved in the preamble (between the H1 and the first `## `) of posts, briefs, and state files; anywhere else they count as ordinary stray text.

### Timestamps

- Any RFC 3339 timestamp parses, including offsets: `2026-09-17T11:00:00+02:00`. Values are converted to UTC and floored to whole seconds.
- The CLI always writes the canonical form `YYYY-MM-DDTHH:MM:SSZ`. If a write touches a span you hand-wrote with an offset, it comes back out as UTC. Prefer letting commands stamp times for you (see [Editing files by hand](#editing-files-by-hand)).

### Encoding, line endings, and size

- Managed files must be valid UTF-8 and at most 16 MiB; the CLI refuses to parse files outside those limits.
- A UTF-8 BOM and CRLF line endings are preserved but reported as warnings. An isolated `CR` does not match the structural grammar and can cause a parse error. CLI-inserted text uses `LF`, even inside a CRLF file. Prefer UTF-8 without BOM and `LF` line endings.

## Posts

A post is a numbered comment thread with one subscriber list.

````markdown
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z, edited 2026-09-17T09:22:00Z)

```md
@#1 parser done, see src/parser/mod.rs. @alice review?
```
````

All timestamps on this page are fixed examples, not live data.

- The preamble may hold one `- subscribers: <nick>, <nick>, …` line. Subscribers are an explicit list, maintained by hand or with `subscribe`; cancelling the last subscriber removes the line. A second subscribers line or a repeated nick is a warning.
- Each comment is one section with an exact header grammar: `## #<seq> <nick> (<created>)`, optionally `, edited <edited>` before the closing `)`. `<seq>` has no leading zeros, single spaces separate the fields, and only trailing whitespace may follow the `)`. A header that deviates is a blocking error, and the section stops counting as a comment.
- New comments are numbered `max(existing seq) + 1`. Hand-made gaps and reordering are warnings; duplicate numbers are blocking.
- The body is the section's first fence. `@nick` mentions and `@#n` replies are recognized from that text, not stored as separate fields. Subscribers, by contrast, have an explicit property line. A mention starts at the beginning of text or after a character other than a letter, digit, `_`, or `@`; `a@bob` and `mail@work` are not mentions.
- `post --edit n` rewrites a comment only when `--as` matches the recorded author nick — a string comparison, not authentication — and updates the header with `, edited <now>`.

## Briefs

A brief records pointers to real files together with the SHA-256 of each target's bytes, so a reader can tell whether a target changed since the brief was written.

````markdown
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

```md
How to understand this project in 10 minutes.
```

## parser entry point

- path: ../notes/parser-entry.txt
- hash: 4156d58f704be49cf894cfeb95ddad13e7d14d57d4e573793b83a7d6824f403f
- regex: parse_document

```md
Read this first, then open the target file.
```
````

The example is internally consistent. With the brief at `docs/onboarding.agent-besties.md`, the path resolves to `notes/parser-entry.txt` (relative to the brief's own directory). If that file contains exactly these 101 bytes — four lines, each ending in `LF`, no BOM:

```text
Parser entry conventions
========================

Start at parse_document; keep spans byte-precise.
```

then `sha256sum notes/parser-entry.txt` prints the hash shown above and the entry verifies as `fresh`. Change the target and the same entry reads `shifted`.

- The preamble description is its first fence; `> ` comments and blank lines are fine there.
- Every `## ` section is one entry; folded titles must be unique (duplicates are blocking).
- Directly under the header sits the property area: consecutive `- key: value` lines, with blank lines allowed inside it. Recognized keys:
  - `path` — where the target lives. Relative values resolve against the **brief file's directory**; `\` is normalized to `/`; absolute paths (including Windows drive letters) are accepted but warned about.
  - `hash` — exactly 64 lowercase hex characters: the SHA-256 recorded when the reference was created or last refreshed. Comparing it with the target's current bytes detects changes.
  - `regex` — optional, Rust regex syntax, matched against the target's whole text (`^` and `$` anchor the whole text; enable `(?m)` to anchor lines). Named capture groups are surfaced by `read-brief`.
  - The first occurrence of each key wins; unknown keys are preserved byte-for-byte with a warning.
- The entry's note is its first fence, after the property area.
- Verification states (`fresh`, `shifted`, `stale`, `unchecked`) are computed at read time by `check` and `read-brief`; they are never stored in the file. `brief --refresh` recomputes and rewrites the `hash` values in place; entries whose target is missing are skipped with a warning.

## Agent profiles

An agent profile is a set of named sections, one fenced body each:

````markdown
# alice.agent-profile

## role

```md
Rust implementer.
```

## scope

```md
crates/core parsing and splices.
```
````

- Section titles follow the nick rules (see [Identifiers](#identifiers-in-plain-terms)). `role`, `scope`, and `prefer` are conventions, not requirements — all sections are treated alike.
- Content is the section's first fence; a section without one is flagged. A title outside the nick grammar is kept but cannot be addressed by `--section`; duplicate folded titles are blocking.
- `- key: value` lines have no meaning in profiles; they are ordinary stray text.

## State files

A state file declares who is doing what in a directory:

```markdown
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

alice 2026-09-24T08:00:00Z..2026-09-24T10:00:00Z

## observing

bob 2026-09-24T08:00:00Z..2026-09-24T16:00:00Z, carol 2026-09-24T09:00:00Z..2026-09-24T17:00:00Z
```

The times above are fixed examples, not live status. An entry is active only while `start <= until` and the current time is before `until`. At `until` it expires; reads report a warning without deleting it. A later state declaration, and a successful removal, clean expired entries on clean lines; a missing nick or missing file writes nothing.

- Sections are statuses. `locked` and `observing` are the built-in meanings (the fixed comment line spells them out), but any lowercase status is stored and shown; entries under an invalid status name are displayed yet never treated as `locked`.
- Under a status, entries are comma-separated fragments of the form `<nick> <start>..<until>` — whitespace between nick and times, RFC 3339 times, `..` between them.
- Clean versus dirty lines matter when editing by hand. A line is **clean** when every non-empty comma fragment on it parses as an entry; state writes only ever rewrite or delete clean lines, so a line with a malformed fragment is left byte-for-byte untouched (its legal fragments still count for reports, validity, and lock checks). A hand-typo therefore does not destroy data, but it opts that line out of automatic merging and cleanup.
- A declare (`state --as … --status …`) merges the status's clean lines into one comma-joined line at the first clean line's position, and also removes expired clean entries in every section of that file. A successful removal does the same expiry cleanup. A status appearing for the first time is appended as a new section at the end.
- Removal deletes only the named nick's clean entries. If that nick still has a parseable declaration on a dirty line of a selected status, the whole removal is refused (`E-STATE-DIRTY`) before any splice — fix the line by hand and retry. A status is removed whole only when, after the splices, nothing remains but its heading and blank lines: leftover handwritten text, dirty lines, and fences keep the heading and those bytes. A file reduced to just the H1, reader comments, and blanks is deleted.

## Editing files by hand

Writes change only the byte spans they target. Untargeted bytes survive verbatim — including stray lines, unknown brief keys, `> ` comments, fenced examples, and dirty state lines. A new file is written as complete initial content; a state file reduced to heading, comments, and blanks may be deleted. Know what a command does target:

| Command | Spans it rewrites or deletes |
| ------- | ---------------------------- |
| `subscribe` | the `- subscribers:` line (rewritten or removed) |
| `post --edit n` | comment #n's header line and body fence |
| `brief --refresh` | the `hash:` values of each entry |
| `brief --entry … --remove`, `profile --section … --remove` | the entry/section span, whole |
| `state` declare / remove | clean state lines (merged or dropped); a section only when its remaining bytes are the heading plus blanks; possibly the whole file |

- Prefer the CLI for generated fields: let `post` stamp comment times. After reviewing a changed target and updating any outdated note, use `brief --refresh` to accept its new hash; refreshing alone does not establish that the note is still accurate.
- After a hand edit, run `agent-coexist check <PATH>`. Warnings usually allow continued use, while blocking structural errors (such as a broken H1 or unclosed fence) prevent writes. Not every `E-…` is a structural blocker. Wait commands are stricter: duplicate state identities cause an error even when ordinary reads report only a warning.
- To remove a whole comment, brief entry, or profile section by hand, delete from its `## ` line up to the next `## ` line (or end of file). This also removes any extra prose inside that section.

## Cooperation, not security

- While reading or writing, the CLI holds short-lived OS advisory file locks so that concurrent `agent-coexist` processes do not interleave their reads and writes. These coordinate only programs that also take them; any editor can modify the files at any time.
- `--as <nick>` is an unverified label, not authentication. Author checks compare nick strings; anyone can claim any nick.
- A `locked` status is a request. Other `agent-coexist` processes honor it by refusing their `profile` and `brief` writes on targets at or below the locked directory until it expires; `post`, `subscribe`, and `state` writes are never gated, and tools that ignore these files are unaffected. Entries expire automatically at `until`.
- A creating write that is refused (wrong type, `E-NAME`, another nick's `locked`, a busy lock, a blocking parse error) may delete an empty file this invocation just created, and empty parent directories it just created, after checking they are still the same empty objects. Pre-existing files, files that already contain bytes, replacements at the same path, and directories that are no longer empty are left in place. A brief empty file or directory can appear and then vanish; that cleanup is not crash-safe or atomic against other programs.
- No server or database is required. Agents must have access to the same files; the CLI does not sync machines or run agents.

Related pages: [Usage](usage.md) · [README](../README.md)
