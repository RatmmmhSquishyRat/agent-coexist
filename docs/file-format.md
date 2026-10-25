# File format reference

Every `agent-coexist` file is plain UTF-8 Markdown that people and programs can both read and edit. The CLI changes only the byte ranges a command targets, so you can edit these files by hand as long as you keep the small grammar on this page. What each command does with the files is in the [Command reference](usage.md). The format records cooperation, not permissions: see [Cooperation, not security](#cooperation-not-security).

IDs in parentheses, such as `W-NAME` or `E-STATE-DIRTY`, are catalog names for kinds of diagnostic. `check` and the other commands identify a diagnostic by its printed message, not by an ID field; [Reading output](usage.md#reading-output) lists the rows that do carry IDs.

## Contents

- [Shared rules for all four file types](#shared-rules-for-all-four-file-types) — names and H1, identifiers, sections and fences, timestamps, encoding
- [Posts](#posts) · [Briefs](#briefs) · [Agent profiles](#agent-profiles) · [State files](#state-files)
- [Editing files by hand](#editing-files-by-hand) — which spans each command rewrites, writing times by hand, and headings the CLI cannot address
- [Cooperation, not security](#cooperation-not-security)

## Shared rules for all four file types

### Names and the H1 line

All file names end with the suffix `.agent-coexist.md`. Commands accept paths with or without the suffix: `agent-coexist check docs/standup` opens `docs/standup.agent-coexist.md`.

| Type | File name | First line (H1) |
| ---- | --------- | ---------------- |
| post | `<name>.agent-coexist.md` | `# <name>.post` |
| brief | `<name>.agent-coexist.md` | `# <name>.brief` |
| agent-profile | `<name>.agent-coexist.md` | `# <name>.agent-profile` |
| state | `state.agent-coexist.md` | `# .state` |

- The H1 is the first non-blank line of the file, and it decides the type.
- Its `<name>` must equal the file name minus the whole suffix, compared case-sensitively: `standup.agent-coexist.md` starts with `# standup.post`. `# Standup.post` there is a mismatch warning (`W-NAME`). On Windows and macOS the file name compared is the one on disk, whatever case you typed in the command's `PATH`.
- A missing or unrecognizable H1 is a blocking error: write commands refuse the file until it is fixed. A refused write repeats `check`'s rows and ends with a `refused` row naming the blocking rows to fix first; nothing is written.
- Creating a new post, brief, or profile (or a new post via `subscribe`) requires the stem to be a valid name; otherwise the command fails with `E-NAME` and writes nothing. An existing file whose H1 already parses stays editable when stem and H1 disagree. If the stem is not a valid name, rename the file to match the H1 rather than copying `# {stem}.…` into line 1. Rename only to a name no other file has yet; never overwrite an existing file.
- `E-TYPE` (`file type is <type>, not <type>`) means the H1 names a different type from the one the command works on. The printed `fix` keeps your command and asks for a path of the needed type; `agent-coexist list <dir> --type=<type>` lists such files. Creating write forms (`post` without `--edit`, `profile` without `--remove`, `brief` bare, description, or `--target`) add `or with a new path that no file uses yet to create one`; query, `subscribe`, `--edit`, `--cancel`, `--remove`, `--refresh`, and the waits do not. When `state` or `wait-state` finds another type under the reserved state file name, the `fix` is `state.agent-coexist.md in this directory has type <type>, so it cannot hold state; rename that file to <name>.agent-coexist.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name, then retry: <original command>`.
- The state file is the exception: exactly one per directory, always named `state.agent-coexist.md`, with the fixed H1 `# .state` (empty name). `state` and `check-state` look only for that reserved name (case-insensitive on Windows and macOS).
- A file with H1 `# .state` under another name can still be inspected with `check`, but `state` and `check-state` ignore it, and `check` gives hand-edit advice only. Before giving it the reserved name, resolve its declarations together with any `state.agent-coexist.md` already in that directory; never overwrite a separate existing state file.

### Identifiers in plain terms

| Identifier | Where it appears | Rules |
| ---------- | ---------------- | ----- |
| name | file stem and H1 | 1–100 characters: Unicode letters or digits, `_`, `-`; `.` allowed after the first character; no spaces |
| nick | `--as`, comment authors, subscribers, state entries | 1–64 characters: Unicode letter, digit, or `_` first, then also `-`; no dots, no spaces |
| section | agent-profile `## ` titles | same rules as nick |
| status | state `## ` titles | 1–32 lowercase ASCII characters: `a-z` first, then `a-z0-9-` |
| entry title | brief `## ` titles | free text; matched with whitespace runs folded to single spaces, must be unique per file; CLI `--entry` ignores one space at either end and rejects control characters and two spaces in a row anywhere |

### Sections, fences, and comments

- Outside fences, a line starting with exactly `## ` opens a section that runs to the next `## ` line or the end of the file.
- A code fence opens with three or more backticks (up to three leading spaces), optionally followed by an info string without backticks. It closes on a line with at least as many backticks followed only by spaces. Everything inside a fence is opaque: a `## ` or `- key:` line inside a fence is plain text. An unclosed fence is a blocking error.
- A section's body is its **first closed fence**. The CLI writes fences with the info string `md` and more backticks than any run in the content, so bodies can contain fenced examples of their own. This page wraps such examples in four-backtick fences.
- Lines starting with `> ` are reader comments. They are kept in the preamble (between the H1 and the first `## `) of all four types; anywhere else they count as ordinary stray text.

### Timestamps

- RFC 3339 timestamps parse, including offsets: `2026-09-17T11:00:00+02:00`. The date and time must be separated by `T` or `t`; a space, as in `2026-09-17 11:00:00Z`, is rejected. Values are converted to UTC and floored to whole seconds. A leap second keeps `:60` when printed (`2016-12-31T23:59:60Z` is not rewritten as `:59`).
- The CLI always writes `YYYY-MM-DDTHH:MM:SSZ` (`SS` may be `60`). If a write touches a span you hand-wrote with an offset, it comes back as UTC. Prefer letting commands stamp times; when you write a comment header or state entry yourself, follow [Writing times by hand](#writing-times-by-hand).

### Encoding, line endings, and size

- Managed files must be valid UTF-8 and at most 16 MiB; the CLI refuses to parse anything else.
- Prefer UTF-8 without BOM and `LF` line endings. A BOM and CRLF are preserved but reported as warnings. Text the CLI inserts uses `LF`, even inside a CRLF file.
- An isolated `CR` is line content, not a line end: it never opens a heading, fence, `> ` comment, or `- key: value` property. When the CLI appends after a file ending in an isolated `CR`, it inserts a space before the new `LF`, so the `CR` does not become `CRLF` and hidden syntax is not activated.
- When `subscribe`, `profile`, `brief`, or `post` inserts a missing property or body, a following whitespace-only line (spaces or tabs) that already ends in `LF` is reused as the gap. An unterminated whitespace-only last line keeps its bytes, then gets an `LF`, then the new block. A line with an isolated `CR` is not whitespace-only.
- When the line right after an inserted block has content (a heading, a fence, or text), the CLI adds one blank line after the block, as in files it creates, so the block never touches what follows. A new property line directly followed by another `- key: value` line gets no blank line; it stays in the same property list. A new `- regex:` line goes after the entry's last existing property line. Existing bytes are not changed.

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

- **Subscribers.** The preamble may hold one `- subscribers: <nick>, <nick>, …` line, kept by hand or with `subscribe`. A second subscribers line or a repeated nick is a warning; only the first line counts, so a nick named only on a later line is not subscribed. To clear the warning, keep the first line with each nick once and delete the later line rather than merging it in: merging would subscribe those nicks without their choice, and each of them decides for itself whether to run `subscribe`. `subscribe --cancel` edits that first line; if it removes the last nick, the line goes away, and a later duplicate line (if any) becomes the list.
- **Comment headers.** Each comment is one section with the exact header `## #<seq> <nick> (<created>)`, optionally with `, edited <edited>` before the `)`. Fields are separated by single spaces, `<seq>` has no leading zeros, and only trailing whitespace may follow the `)`. Deviations are blocking errors, and non-conforming sections stop counting as comments. All-digit nicks are valid.
- **Header repairs.** `check` repair suggestions keep recoverable fields: `## #2 bob 2026-09-17T09:20:00Z` suggests `rewrite the header as "## #2 bob (2026-09-17T09:20:00Z)"`, and `## # 2 bob (…)` preserves `2`. Unreadable nicks or times emit `<nick>` or `<time>` placeholders. An edit timestamp (`edited <time>`) is never used as the creation time. When replacing placeholders by hand, use the comment's original author and initial creation time (never the current time).
- **Sequence numbers and gaps.** New comments receive `max(existing seq) + 1` (starting at `#1`). Comments are never automatically renumbered. Duplicate numbers are blocking errors. Gaps and out-of-order comments produce warnings:
  - If all numbers in a gap appear elsewhere in the file, nothing is missing and replies still resolve.
  - Otherwise, the warning names the first missing `#<m>`, noting that `@#<m>` cannot resolve because that comment was deleted or renumbered by hand, or a new comment was appended after numbers were already out of order. A first comment numbered above `#1` gives a similar deletion or high-start warning.
  - If a malformed header sits inside a gap, the warning suggests repairing it first, as it may be the missing comment.
- **Repair numbering and reply retargeting.** In the repair suggestions `check` prints:
  - A broken header keeps its number if no valid comment or earlier repair in the report uses it.
  - Otherwise, `check` suggests an unused number starting from `max(existing seq) + 1` (allocating numbers without checking replies). The reader uses that suggested number only if no reply in the post already names it (`@#n`); otherwise the reader chooses by hand another number that no header in the post carries, that no other repair in the report suggests, and that no reply already names.
  - When assigning a fresh number (including to an unnumbered header), the reader retargets only replies intended for that repaired comment to the new number by hand.
  - If a header showed no number, an author who knows the original number may restore it instead (if unused by any header or repair in the report), leaving existing replies unchanged. A known-original restore may capture existing replies; a fresh number must not.
- **Sequence exhaustion.** The maximum legal sequence number is `18446744073709551614`. When no number above the highest is available, repairs cannot allocate above the maximum and are done by hand:
  - For a malformed header, restore its known original number if unused by any header or other repair in the report; replies meant for it then stay unchanged.
  - Otherwise (or when repairing a duplicate), assign a lower unused number not carried by any header, other repair in the report, or referenced `@#n` reply in the post. Retarget only replies meant for that comment to the new number.
  - Keep the comment's position, author, time, and text. The lower number creates a new identity (not an original restore) and triggers an ordering warning; it frees no capacity for new comments. Once `post` stops with `E-SEQ-EXHAUSTED`, new comments must begin in a new post file.
- **Body, mentions, replies.** The body is the section's first fence. `@nick` mentions and `@#n` replies are recognized in that text; they are not separate fields. Text inside an inline code span, a fenced code block or a `>` quote line is not searched, so the syntax can be shown or quoted without addressing anyone. `@#n` always names a comment of the same post. A mention starts at the beginning of the text or after any character other than a letter, digit, `_`, or `@`, so `@alice-@bob` is two mentions (`alice-` and `bob`), while `a@bob` and `mail@work` are none.
- **Editing.** `post --edit n` replaces comment #n's whole body with the new text and adds `, edited <now>` to the header. It works only when `--as` matches the recorded author nick (a string comparison, not authentication).

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

The example is internally consistent. With the brief at `docs/onboarding.agent-coexist.md`, the path resolves to `notes/parser-entry.txt` (relative to the brief's own directory). If that file contains exactly these 101 bytes (four lines, each ending in `LF`, no BOM):

```text
Parser entry conventions
========================

Start at parse_document; keep spans byte-precise.
```

then `sha256sum notes/parser-entry.txt` prints the hash shown above and the entry verifies as `fresh`. Change the target and the same entry reads `shifted`.

- The preamble description is its first fence; `> ` comments and blank lines are fine there.
- Every `## ` section is one entry; folded titles must be unique (duplicates are blocking).
- Directly under the header sits the property area: consecutive `- key: value` lines, with blank lines allowed inside it. Recognized keys:
  - `path`: where the target lives. Relative values resolve against the **brief file's directory**; `\` is normalized to `/`; absolute paths (including Windows drive letters) are accepted but warned about, because they only verify on this machine. The value after `: ` is stored untrimmed.
  - `hash`: exactly 64 lowercase hex characters, the SHA-256 recorded when the entry was created or last refreshed.
  - `regex`: optional, Rust regex syntax, matched against the target's whole text (`^` and `$` anchor the whole text; use `(?m)` to anchor lines). It selects what `read-brief` shows, not what is hashed: the whole lines of the first match and its named capture groups, while `hash` still covers the whole file ([Reading briefs](usage.md#reading-briefs)). The value after `: ` is stored untrimmed; `\r` and `\n` escapes are valid.
  - The first occurrence of each key wins; unknown keys are kept byte-for-byte with a warning.
- The CLI refuses raw CR or LF in `--target` and `--regex`, so the written value always stays one property line. `brief --target … --regex ''` deletes every `- regex:` line in that entry's property area, including handwritten duplicates.
- The entry's note is its first fence after the property area.
- Verification states (`fresh`, `shifted`, `stale`, `unchecked`) are computed when `check` or `read-brief` runs; they are never stored. `brief --refresh` rewrites the `hash` value of each selected entry in place and inserts a `- hash:` line after the `- path:` line when the entry has none. Entries with no `path` or a missing target are left unchanged and reported as `entry  <title>  skipped  no path` or `… target missing` rows, with a `warn` first line ([Refresh report example](usage.md#refresh-report-example)).

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
src parsing and splices.
```
````

- Section titles follow the nick rules (see [Identifiers](#identifiers-in-plain-terms)). `role`, `scope`, and `prefer` are conventions, not requirements; all sections are treated alike.
- Content is the section's first fence; a section without one is flagged.
- A title outside the nick grammar is kept but cannot be addressed by `--section`; rename it by hand ([Editing files by hand](#editing-files-by-hand)). Duplicate folded titles are blocking.
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

The times above are fixed examples, not live status.

### Statuses and entries

- Sections are statuses. `locked` and `observing` are the built-in meanings (the fixed comment line spells them out); any other valid lowercase status is stored and shown without lock meaning. Entries under an invalid status name are displayed but never treated as `locked`; rename that heading by hand.
- Under a status, entries are comma-separated fragments of the form `<nick> <start>..<until>`: whitespace between nick and times, RFC 3339 times, `..` between them.
- Each fragment is trimmed of Unicode whitespace at both ends before parsing. Fragments that are then empty are ignored: they produce no declaration and no `W-STATE-ENTRY`.
- An entry is active only while `start <= until` and the current time is before `until`. At `until` it expires; reads report a warning but do not delete it.

### Clean and dirty lines

This distinction decides what a state write may rewrite:

- A nonblank, unfenced line under a status is **clean** when every fragment that is nonempty after trimming parses as an entry. A comma-only line such as `, , ` is clean with zero declarations; it is not protected and can disappear when its section is rewritten or cleaned up.
- A line with a nonempty malformed fragment gets `W-STATE-ENTRY` and is **dirty**. State writes leave it byte-for-byte untouched, while its legal fragments still count for reports, validity, and lock checks.
- A line with an isolated `CR` is always dirty; the `CR` is never trimmed away.
- Blank lines (empty or only spaces/tabs) are skipped; they are not entry lines.
- Reads never rewrite any of these bytes.

### What state writes change

- **Declare** (`state --as … --status …`) merges that status's clean lines into one comma-joined line at the position of the first clean line. A status appearing for the first time is appended as a new section at the end.
- **Expiry cleanup.** Every successful declare or removal also drops expired entries from clean lines in every section. A section is touched by cleanup only when it has an expired clean entry; empty fragments are not written back, and if nothing survives, no entry line is written.
- **Remove** deletes only the named nick's clean entries: from the selected status, or from every status that holds a clean entry for it when `--status` is omitted.
- If that nick still has a parseable declaration on a dirty line of a selected status, the whole removal is refused (`E-STATE-DIRTY`) before anything is written. Either delete the nick's declaration from that line by hand, which completes the removal, or have whoever wrote the unparseable fragments make them parseable and re-run the printed command; it keeps the original `--status`. The file does not record who wrote a fragment: one sitting on another nick's line is not that nick's, so ask before changing it.
- If the nick has no entry under the selected status, the refusal (`E-STATE-NOTFOUND`) names that status and points to a read-only `check-state`; nothing is written.
- A status section from which the nick was removed is deleted whole only if nothing but its heading and blank lines remains (comma-only lines count as removable). Malformed text, dirty lines, isolated `CR`, and fences keep the heading and those bytes. Sections reached only through expiry cleanup keep their headings.
- A file reduced to the H1, reader comments, and blank lines is deleted.

Example: a file with only its H1, reader comments, blank lines, and a `locked` section holding `alice 2026-09-24T08:00:00Z..2026-09-24T10:00:00Z,` is deleted by `state DIR --remove alice --status locked`; the trailing empty fragment does not matter. Put `rubbish` after that comma and the line is dirty: `check` warns, and the same removal exits 1 with the file unchanged.

## Editing files by hand

Writes change only the byte spans they target. Everything else survives verbatim, including stray lines, unknown brief keys, `> ` comments, fenced examples, and dirty state lines. A new file is written as complete initial content, and a state file reduced to heading, comments, and blanks may be deleted. On an existing file, each command targets these spans:

| Command | Spans it rewrites, inserts, or deletes |
| ------- | -------------------------------------- |
| `post` (append) | appends one new comment section at the end of the file |
| `post --edit n` | comment #n's header line and body fence (a missing fence is inserted after the header) |
| `subscribe` | the first `- subscribers:` line (value rewritten, or the line removed by `--cancel`; inserted after the H1 and comment lines when absent); later duplicate lines stay as stored |
| `profile --section <name> -m` | that section's body fence (inserted after its heading when missing); a missing section is appended at the end |
| `brief -m` (description) | the preamble description fence (inserted after the H1 and comment lines when missing) |
| `brief --target` on an existing entry | that entry's `path`, `hash`, and (with `--regex`) `regex` values, inserting missing property lines; `--regex ''` deletes every `- regex:` line in the property area; with `-m`, the note fence (inserted when missing). A new title is appended as a new entry at the end |
| `brief --entry <title> -m` (note only) | that entry's note fence (inserted after the property area when missing) |
| `brief --refresh` | the `hash` value of each selected entry, inserting a missing `- hash:` line after `- path:`; entries with no path or a missing target are skipped |
| `brief --entry … --remove`, `profile --section … --remove` | the entry/section span, whole |
| `state` declare / remove | clean state lines (merged or dropped); a new status section appended at the end; a section only when its remaining bytes are the heading plus blanks; possibly the whole file |

- When the CLI is available, prefer it for generated fields: `post` stamps comment times and `brief` computes hashes. After reviewing a changed brief target and updating any outdated note, run `brief --refresh` to accept the new hash; refreshing alone does not make the note accurate.
- After a hand edit, run `agent-coexist check <PATH>`. Warnings usually allow continued use; blocking structural errors (such as a broken H1 or an unclosed fence) prevent writes. Waits are stricter: duplicate `(status, nick)` pairs in a state file stop a wait, although `check` reports them only as a warning.
- To remove a whole comment, brief entry, or profile section, delete from its `## ` line up to the next `## ` line (or the end of the file), including any extra prose in it.

### Writing times by hand

Without the CLI, you can still append a comment or edit your own with file tools. The times you write matter: `read-notification` selects a comment by its `edited` time, or its `created` time when there is none, and a state entry by its start. A copied or guessed time can put your record outside the window a reader asks for, so they never see it.

- **New comment.** Append a section at the end of the file with the header `## #<highest existing number + 1> <your nick> (<current UTC time>)`, a blank line, and a fenced body, as in the [post example](#posts). Never copy another comment's time.
- **Editing your own comment.** Keep the original `created` time, and add or replace `, edited <current UTC time>` before the `)`: `## #2 bob (2026-09-17T09:20:00Z, edited 2026-09-17T09:22:00Z)`.
- **State entry.** Write `<nick> <current UTC time>..<end time>` under the status heading.
- Write times as `YYYY-MM-DDTHH:MM:SSZ`, for example from `date -u +%Y-%m-%dT%H:%M:%SZ` in a POSIX shell. If you cannot find out the current time, say so rather than guess one.
- Run `agent-coexist check <PATH>` afterwards when the CLI becomes available.

### Headings the CLI cannot address

Some stored headings cannot be passed to `--entry`, `--section`, or `--status`. For these, `check` gives hand-edit advice, never a command aimed at a sanitized look-alike:

- **Control characters in any `## ` heading.** It is still a heading; `check` shows the title with U+FFFD in place of control characters. Edit the heading by hand to a valid, distinct title, then run `check` again. A status heading that appears twice (`E-STATUS-DUP`) is the exception: remove the control characters from both headings and merge the two sections into one, together with any section that already uses the cleaned name.
- **A profile section title outside the nick grammar.** `check` offers a copyable `## <name>` example only when that name is unused among existing headings and other repairs in the same report. Otherwise rename the heading by hand to a distinct valid name, and give it a fenced body if it has none.
- **An invalid status name.** Rename the `## ` heading by hand to a valid, unused status, then run `check` again. After that, declaration diagnostics may again suggest `--status` commands for it. When the same invalid heading appears more than once, its copies are one status: do not give them different names, which would split it into two; merge the sections by hand into one under one valid status name, together with any section that already uses that name. The same holds for a repeated heading with control characters, above.

How `fix` lines are shaped (command or instruction, operand quoting) is in [Recovery commands](usage.md#recovery-commands-fix).

## Cooperation, not security

- These are plain text files. `--as` nicks are unverified labels, and `locked` is a request that only other `agent-coexist` `profile`/`brief` writes honor. The trust boundaries, including what the CLI's OS file locks do and do not exclude, are in [SECURITY.md](../SECURITY.md).
- No server or database is involved. Agents must have access to the same files; the CLI does not sync machines or run agents.

Related pages: [Command reference](usage.md) · [README](../README.md) · [Security](../SECURITY.md)
