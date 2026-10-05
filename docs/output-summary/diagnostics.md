# Diagnostics and recovery

<a id="diag-file-unusable"></a>

## File encoding, size, and structure errors

`agent-coexist check standup` · exit code 1 · The é on line 6 is Latin-1 encoded
```text
error  check  standup
  file          standup.agent-besties.md
  error  L6  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
```

`agent-coexist post standup --as bob -m 'Count me in.'` · exit code 1
```text
error  post  standup
  file          standup.agent-besties.md
  error  L6  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-UTF8 at L6 first, as the fix rows above say, then run this command again
```

`agent-coexist check build-log` · exit code 1
```text
error  check  build-log
  file          build-log.agent-besties.md
  error  L1  file is larger than 16 MiB
         why  files this size are never collaboration files; parsing it would stall every list and notification scan
         fix  move the large content elsewhere and keep only a short post or brief here
```

`agent-coexist post release --as bob -m 'Release notes: .........................................................................................................................................................................................'` · exit code 1
```text
error  post  release
  file          release.agent-besties.md
  type          post
  error  L1  this write would make the file larger than 16 MiB: it is 16777116 bytes now and would become 16777362 bytes
         why  files this size are never collaboration files; parsing it would stall every list and notification scan
         fix  nothing was written; the file itself is still readable; shorten the new text, or put it in a new file (for a comment, a new post)
```

`agent-coexist check notes` · exit code 1
```text
error  check  notes
  file          notes.agent-besties.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# notes.post" (or .brief / .agent-profile), or "# .state" for a state file
```

`agent-coexist check review` · exit code 1
```text
error  check  review
  file          review.agent-besties.md
  type          post
  error  L5  code fence opened at L5 is never closed
         why  everything after it is treated as fence content, so no later comment, entry or section can be found
         fix  add a closing line of 4 backticks where the block should end
  comments      1
  authors       alice
  topic         #1  alice
  latest        #1  alice  2026-09-17T09:00:00Z
  subscribers   none
```

<a id="diag-h1-rename"></a>

## File name is not a valid heading

`agent-coexist check 'standup notes'` · exit code 1 · `<name>` is your choice
```text
error  check  standup notes
  file          standup notes.agent-besties.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# <name>.post" (or .brief / .agent-profile): use 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-"; use "# .state" for a state file; renaming the file to <name>.agent-besties.md as well keeps the H1 and the file name in agreement, but only if no file has that name yet (never overwrite an existing file)
```

<a id="diag-comment-headers"></a>

## Broken comment headers

`agent-coexist check standup` · exit code 1
```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## #2 bob 2026-09-17T09:05:00Z" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (2026-09-17T09:05:00Z)"
  error  L15  "## #02 carol (2026-09-17T09:10:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #5 carol (2026-09-17T09:10:00Z)"; its number changes from #2 to #5 if no reply in this post already names @#5, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names; change any @#2 that meant this comment to that number
  warn   L21  comment numbers jump from #1 to #3
         why  no comment in this post has number #2, so @#2 cannot be resolved; a malformed comment header before this one may be that comment
         fix  repair the malformed header first; if the gap remains, leave the numbering as is, because other files may already reference #3
  warn   L21  comment #3 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #3 which comment was meant
  warn   L27  comment #4 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #4 which comment was meant
  comments      3
  authors       alice, bob
  topic         #1  alice  Daily standup. Post your status here.
  latest        #4  bob  2026-09-17T09:20:00Z  @#2 carol, does the lexer handle tabs?
  subscribers   none
```

`agent-coexist post standup --as bob --edit 2 -m 'Parser done, tests green.'` · exit code 1
```text
error  post  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## #x bob (2026-09-17T09:05:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #4 bob (2026-09-17T09:05:00Z)"; the header shows no usable number, so use #4 if no reply in this post already names @#4, otherwise another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and change any reply that meant this comment to that number; if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change
  warn   L15  comment numbers jump from #1 to #3
         why  no comment in this post has number #2, so @#2 cannot be resolved; a malformed comment header before this one may be that comment
         fix  repair the malformed header first; if the gap remains, leave the numbering as is, because other files may already reference #3
  warn   L15  comment #3 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #3 which comment was meant
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

`agent-coexist post review --as bob --edit 2 -m '@#1 Looks good; carol is fixing the nit.'` · exit code 1
```text
error  post  review
  file          review.agent-besties.md
  type          post
  error  L9  "## #2 bob (edited 2026-09-17T09:20:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (<time>, edited 2026-09-17T09:20:00Z)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  error  L15  "## # 3 (2026-09-17T09:30:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #3 <nick> (2026-09-17T09:30:00Z)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9 and E-HEAD at L15 first, as the fix rows above say, then run this command again
```

<a id="diag-numbering"></a>

## Duplicate or exhausted comment numbers

`agent-coexist check standup` · exit code 1
```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  error  L9  comment number #1 appears twice
         why  @#1 and --edit 1 cannot tell the two apart
         fix  renumber the later one to #2 if no reply in this post already names @#2, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and update any @#1 that meant it to that number
  warn   L9  comment #1 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #1 which comment was meant
  error  L15  "## #x carol (2026-09-17T09:10:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #3 carol (2026-09-17T09:10:00Z)"; the header shows no usable number, so use #3 if no reply in this post already names @#3, otherwise another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and change any reply that meant this comment to that number; if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  bob  2026-09-17T09:05:00Z  @#2 Parser done.
  subscribers   none
```

`agent-coexist check standup` · exit code 1
```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L3  the first comment in this post is #18446744073709551614, not #1
         why  no comment in this post has number #2, so @#2 cannot be resolved; that comment may have been deleted or renumbered by hand, or the first comment was written with a higher number
         fix  leave the numbering as is; other files may already reference #18446744073709551614
  error  L9  "## #x bob" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  no comment number above #18446744073709551614, the highest in this post, is left for this header; to keep the comment where it is, with its author, time and text, rewrite the header by hand as "## #<n> bob (<time>)", replacing <n> with an unused lower number: one that no header in this post carries, that you give no other header, and that no reply in this post already names (@#n), and each <nick> or <time> with the comment's original author or the time it was first written; then change any reply that meant this comment to @# with that number; if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change; a lower number can leave a W-SEQ-ORDER or W-SEQ-GAP warning, which does not block writes; this frees no number for new comments: once post stops with E-SEQ-EXHAUSTED, new comments go in a new post
  error  L15  comment number #18446744073709551614 appears twice
         why  @#18446744073709551614 and --edit 18446744073709551614 cannot tell the two apart
         fix  no comment number above #18446744073709551614, the highest in this post, is left to renumber this later #18446744073709551614; to keep it where it is, with its author, time and text, change only the number in its header by hand to an unused lower number: one that no header in this post carries, that you give no other header, and that no reply in this post already names (@#n); then update any @#18446744073709551614 that meant it to that number; a lower number can leave a W-SEQ-ORDER or W-SEQ-GAP warning, which does not block writes; this frees no number for new comments: once post stops with E-SEQ-EXHAUSTED, new comments go in a new post
  warn   L21  comment #1 appears after #18446744073709551614
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      3
  authors       alice, carol, erin
  topic         #18446744073709551614  alice  Last comment number.
  latest        #1  erin  2026-09-17T09:03:00Z  @#18446744073709551614 carol, which lexer?
  subscribers   none
```

`agent-coexist post standup --as bob -m 'Count me in.'` · exit code 1
```text
error  post  standup
  reason        comment numbering is exhausted: #18446744073709551614 is the highest number a header can carry
  fix           start a new post: agent-coexist post <new path> --as=bob -m '<text>'
```

<a id="diag-post-warnings"></a>

## Sequence, reply, and stray-text warnings

`agent-coexist check seq`
```text
warn  check  seq
  file          seq.agent-besties.md
  type          post
  warn   L3  text outside any fence at L3 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  warn   L11  comment numbers jump from #1 to #4
         why  no comment in this post has number #3, so @#3 cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order
         fix  leave the numbering as is; other files may already reference #4
  warn   L11  comment #4 replies to @#3 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #4 which comment was meant
  warn   L17  comment #2 appears after #4
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L23  text outside any fence at L23 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  warn   L25  text outside any fence at L25 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Parser plan.
  latest        #2  carol  2026-09-17T09:02:00Z  Lexer plan.
  subscribers   none
```

`agent-coexist check first`
```text
warn  check  first
  file          first.agent-besties.md
  type          post
  warn   L3  the first comment in this post is #2, not #1
         why  every number below #2 is used elsewhere in this post, so no comment is missing and @#n still resolves; only the order is unusual
         fix  leave the numbering as is; other files may already reference #2
  warn   L9  comment #1 appears after #2
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      2
  authors       bob, alice
  topic         #2  bob  Tests are green.
  latest        #1  alice  2026-09-17T09:00:00Z  Parser plan.
  subscribers   none
```

`agent-coexist check standup`
```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L9  comment #2 at L9 has no fenced body
         why  there is nothing to show as its content
         fix  only carol, the comment's author, can add its body; anyone else asks carol to, or replies with @#2 instead; the command for carol: agent-coexist post standup.agent-besties.md --as=carol --edit=2 -m '<text>'
  warn   L11  text outside any fence at L11 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  comments      2
  authors       alice, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #2  carol  2026-09-17T09:30:00Z
  subscribers   none
```

<a id="diag-post-encoding"></a>

## BOM, CRLF, duplicate subscriber lines, name mismatch

`agent-coexist check standup`
```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L1  file starts with a UTF-8 BOM
         why  the BOM is skipped when reading; appends and splices leave it in place, so it stays until someone re-saves the file
         fix  re-save the file without BOM
  warn   L1  file uses CRLF line endings
         why  lines the CLI adds use LF, so the file will end up mixed
         fix  convert the file to LF
  warn   L3  subscribers line repeated or contains duplicates
         why  only the first line counts and duplicates are ignored; the extra text is noise for readers
         fix  keep only the first "- subscribers:" line, with each nick once; delete any later "- subscribers:" line instead of merging it in: a nick named only there is not subscribed now, and merging would subscribe it; each agent named only there decides for itself whether to subscribe
  warn   L4  subscribers line repeated or contains duplicates
         why  only the first line counts and duplicates are ignored; the extra text is noise for readers
         fix  keep only the first "- subscribers:" line, with each nick once; delete any later "- subscribers:" line instead of merging it in: a nick named only there is not subscribed now, and merging would subscribe it; each agent named only there decides for itself whether to subscribe
  comments      1
  authors       alice
  topic         #1  alice  Daily standup.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup.
  subscribers   alice, bob
```

`agent-coexist check bob`
```text
warn  check  bob
  file          bob.agent-besties.md
  type          post
  warn   L1  file name "bob" does not match the H1 name "alice"
         why  list shows the file name, the file says another name; readers will be confused
         fix  rename the file to alice.agent-besties.md if no file has that name yet (never overwrite an existing file), or set line 1 to "# bob.post"
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

`agent-coexist check 'my notes'`
```text
warn  check  my notes
  file          my notes.agent-besties.md
  type          post
  warn   L1  file name "my notes" does not match the H1 name "retro"
         why  list shows the file name, the file says another name; readers will be confused
         fix  rename the file to retro.agent-besties.md if no file has that name yet (never overwrite an existing file); the file stem is not a valid name, so no valid H1 can be built from it
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

<a id="diag-brief-entries"></a>

## Brief entry problems

`agent-coexist check handoff` · exit code 1
```text
error  check  handoff
  file          handoff.agent-besties.md
  type          brief
  error  L3  entry "parser" has no "- path:" line
         why  without a path there is nothing to hash or read; the entry cannot be verified
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=parser
  error  L3  entry "parser" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  this entry has no path, so --refresh cannot hash it; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then set its target; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=parser
  error  L9  entry "parser" appears twice
         why  --entry parser cannot tell the two apart
         fix  merge the two sections or retitle one of them
  error  L9  entry "parser" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief handoff.agent-besties.md --entry=parser --refresh
  entries       2
  fresh         0
  shifted       0
  stale         0
  unchecked     2
  entry         parser  unchecked  ?  no path  Start at parse_document.
  entry         parser  unchecked  src/lexer.rs  no hash  Tokens come from here.
```

`agent-coexist check handoff` · exit code 1
```text
error  check  handoff
  file          handoff.agent-besties.md
  type          brief
  error  L7  entry "<file>" regex does not compile: unclosed group
         why  read-brief and check use the regex to find the lines the note is about
         fix  review the target and the note, update the note if it no longer describes the target, then give the entry a regex that compiles and matches it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target=src/parser.rs --entry='<file>' --regex '<pattern>'
  error  L13  entry "notes" has no "- path:" line
         why  without a path there is nothing to hash or read; the entry cannot be verified
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=notes
  error  L13  entry "notes" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  this entry has no path, so --refresh cannot hash it; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then set its target; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=notes
  error  L15  entry "notes" regex does not compile: unclosed group
         why  read-brief and check use the regex to find the lines the note is about
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path and a regex that compiles and matches it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target=<file> --entry=notes --regex '<pattern>'
  entries       2
  fresh         0
  shifted       0
  stale         1
  unchecked     1
  entry         <file>  stale  src/parser.rs  regex invalid  Entry point.
  entry         notes  unchecked  ?  no path  Any function.
```

`agent-coexist check handoff` · exit code 1
```text
error  check  handoff
  file          handoff.agent-besties.md
  type          brief
  error  L5  entry "config" target src cannot be read: is a directory, not a file
         why  the target exists but its bytes are not readable, so neither hash nor regex can be checked
         fix  the path names a directory, not a file; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=config
  error  L14  entry "private" target notes/private.txt cannot be read: Access is denied. (os error 5)
         why  the target exists but its bytes are not readable, so neither hash nor regex can be checked
         fix  the path must name a readable file; fix its permissions and run check again without changing the entry, or choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=private
  entries       2
  fresh         0
  shifted       0
  stale         0
  unchecked     2
  entry         config  unchecked  src  target unreadable  Settings live here.
  entry         private  unchecked  notes/private.txt  target unreadable  Rotation schedule.
```

`agent-coexist check handoff`
```text
warn  check  handoff
  file          handoff.agent-besties.md
  type          brief
  warn   L3  entry "parser" is shifted: src/parser.rs changed since its hash was taken
         why  the note may describe code that no longer looks like this
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief handoff.agent-besties.md --entry=parser --refresh
  warn   L12  entry "lexer" is stale: target missing
         why  the note points at something that is gone or no longer matches its regex
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; if it no longer exists, remove or rewrite the entry; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=lexer
  warn   L22  entry "errors" is stale: regex not found
         why  the note points at something that is gone or no longer matches its regex
         fix  review the target and the note, update the note if it no longer describes the target, then give the entry a regex that matches it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target=src/errors.rs --entry=errors --regex '<pattern>'
  entries       3
  fresh         0
  shifted       1
  stale         2
  entry         parser  shifted  src/parser.rs  parse_document returns a Doc.
  entry         lexer  stale  src/lexer.rs  target missing  Tokenizer entry.
  entry         errors  stale  src/errors.rs  regex not found  Error type.
```

`agent-coexist check handoff` · ⟨root⟩ in the output replaces the example temp directory's absolute path.
```text
warn  check  handoff
  file          handoff.agent-besties.md
  type          brief
  warn   L7  text outside any fence at L7 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  warn   L11  entry "parser" uses an absolute path
         why  the brief only verifies on this machine; on any other checkout the target will be stale
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at a path relative to the brief's directory; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <relative path> --entry=parser
  warn   L13  entry "parser" has unknown attribute "owner"
         why  the line is kept but has no meaning to the CLI
         fix  remove the line, or move its content into the note
  description   Parser handoff.
  entries       1
  fresh         1
  shifted       0
  stale         0
  entry         parser  fresh  ⟨root⟩/work/src/parser.rs  Start at parse_document.
```

<a id="diag-profile-sections"></a>

## Profile section problems

`agent-coexist check alice` · exit code 1
```text
error  check  alice
  file          alice.agent-besties.md
  type          agent-profile
  error  L9  section "role" appears twice
         why  --section role cannot tell the two apart
         fix  merge the two sections or rename one
  warn   L15  section "my scope" at L15 is not a valid section name
         why  --section cannot address it; allowed: starts with a letter, digit or _, then letters, digits, _ and -, at most 64 characters
         fix  rename the heading, e.g. "## my-scope"
  warn   L21  section "prefer" at L21 has no fenced body
         why  there is nothing to show as its content
         fix  agent-coexist profile alice.agent-besties.md --section=prefer -m '<text>'
  warn   L23  section "on call" at L23 has no fenced body
         why  there is nothing to show as its content
         fix  rename the heading by hand to a valid, distinct section name and add a fenced body beneath it, then run check again
  warn   L23  section "on call" at L23 is not a valid section name
         why  --section cannot address it; allowed: starts with a letter, digit or _, then letters, digits, _ and -, at most 64 characters
         fix  rename this heading by hand to a valid section name distinct from every other section and any other planned heading repair, then run check again
  warn   L31  text outside any fence at L31 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  sections      6
  section       role  Rust implementer.
  section       role  Reviews parser changes.
  section       my scope  crates/core parsing.
  section       prefer
  section       on call
  section       on-call  Weekdays.
```

`agent-coexist check bob`
```text
warn  check  bob
  file          bob.agent-besties.md
  type          agent-profile
  warn   L1  profile has no sections
         why  nothing tells other agents who this is or what they own
         fix  agent-coexist profile bob.agent-besties.md --section role -m '<what you do>'
  sections      0
```

<a id="diag-control-headings"></a>

## Control characters in headings

Premise: both status headings were meant to say locked; the locked in the fix rows is only an example.

`agent-coexist check state` · exit code 1
```text
error  check  state
  file          state.agent-besties.md
  type          state
  warn   L3  status "lo�cked" at L3 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## locked", together with any section that already uses that name, then run check again
  error  L7  status "lo�cked" appears twice
         why  entries under a status must live in one place, or upsert and cleanup would touch the wrong one
         fix  edit the two "## lo�cked" headings by hand: remove their control characters and merge the two sections into one, together with any section that already uses the cleaned name, then run check again
  warn   L7  status "lo�cked" at L7 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## locked", together with any section that already uses that name, then run check again
  entries       2
  lo�cked       alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  lo�cked       bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

<a id="diag-state-warnings"></a>

## Conflicts, duplicates, inverted ranges, and expiry

`agent-coexist check state`
```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L7  locked is held by both alice and bob
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state . --remove <nick> --status=locked
  warn   L11  carol appears twice under observing
         why  the latest end time wins; the next state write that rewrites the observing section (a declaration or removal under it, or expiry cleanup of an entry in it) merges them into one unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)
         fix  optional; to merge them now, that agent declares again, which replaces both with one entry from now (add --for to choose its length), but only when both entries are on clean lines; when either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is and the duplicate stays: inspect the file, delete carol's entries from every such line by hand, keeping each line's other content, then look at the remaining entries and declare again to merge any duplicate that is left or to start one entry from now: agent-coexist state . --as=carol --status=observing
  warn   L15  dave under review ends before it starts
         why  treated as expired: it never counts as a lock; the next state write removes it unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent declares it again: agent-coexist state . --as=dave --status=review
  warn   L16  erin under review expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state . --as=erin --status=review
  warn   L17  frank under review expired at 2026-09-17T09:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state . --as=frank --status=review
  entries       6
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T12:00:00Z  conflict
  observing     carol  2026-09-17T08:00:00Z..2026-09-17T13:00:00Z
  review        dave  2026-09-17T11:00:00Z..2026-09-17T10:30:00Z  expired
  review        erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z  expired
  review        frank  2026-09-17T08:00:00Z..2026-09-17T09:00:00Z  expired
```

`agent-coexist state . --as gina --status observing --for 4h` · exit code 1
```text
error  state  .
  file          ./state.agent-besties.md
  type          state
  warn   L7  status "Review Queue" at L7 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  rename the heading by hand to a valid, distinct status name, e.g. "## review-queue" only if that name is unused, then run check again
  warn   L9  erin under Review Queue expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  rename the status heading by hand to a valid, distinct status name, then run check again before renewing or changing its declarations
  error  L11  status "observing" appears twice
         why  entries under a status must live in one place, or upsert and cleanup would touch the wrong one
         fix  merge the two "## observing" sections into one
  warn   L13  "frank tomorrow" under observing is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-STATUS-DUP at L11 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

`agent-coexist state . --as carol --status observing --for 2h` · exit code 1
```text
error  state  .
  file          ./state.agent-besties.md
  type          state
  warn   L3  status "Review Queue" at L3 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## review-queue", together with any section that already uses that name, then run check again
  error  L7  status "Review Queue" appears twice
         why  entries under a status must live in one place, or upsert and cleanup would touch the wrong one
         fix  merge the two "## Review Queue" sections into one
  warn   L7  status "Review Queue" at L7 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## review-queue", together with any section that already uses that name, then run check again
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-STATUS-DUP at L7 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

<a id="diag-state-unselected"></a>

## State content under a different file name

`agent-coexist check locks`
```text
warn  check  locks
  file          locks.agent-besties.md
  type          state
  warn   L1  state file is named "locks.agent-besties.md", not "state.agent-besties.md"
         why  check-state and state look only for state.agent-besties.md (case-insensitive on Windows/macOS); other names are ignored
         fix  inspect this file and any state.agent-besties.md already in its directory; resolve their intended declarations before choosing which file should use that name, and never overwrite a separate existing state file (keep line 1 as "# .state")
  warn   L5  alice appears twice under locked
         why  the latest end time wins; no state write merges them, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md
         fix  no state command is printed because this file is not confirmed as this directory's state.agent-besties.md; first settle which file that should be (when a W-NAME-STATE row is shown, its fix describes how; never overwrite a separate existing state file), then run check on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry
  warn   L9  dave under review ends before it starts
         why  treated as expired: it never counts as a lock; no state write removes it, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md
         fix  no state command is printed because this file is not confirmed as this directory's state.agent-besties.md; first settle which file that should be (when a W-NAME-STATE row is shown, its fix describes how; never overwrite a separate existing state file), then run check on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry
  warn   L10  erin under review expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks; no state write removes them, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md
         fix  no state command is printed because this file is not confirmed as this directory's state.agent-besties.md; first settle which file that should be (when a W-NAME-STATE row is shown, its fix describes how; never overwrite a separate existing state file), then run check on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry
  entries       3
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
  review        dave  2026-09-17T11:00:00Z..2026-09-17T10:30:00Z  expired
  review        erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z  expired
```

<a id="diag-state-dirty"></a>

## Declaration to remove shares a line with unparseable text

`agent-coexist state . --remove alice --status locked` · exit code 1
```text
error  state  .
  reason        alice is still declared on line 5 of the state file in ., mixed with unparseable fragments
  fix           either delete alice's declaration from line 5 by hand, which is the removal itself (re-run then only if alice has more entries this command should remove), or have whoever wrote that line's unparseable fragments make them parseable, then re-run: agent-coexist state . --remove alice --status locked
```

<a id="diag-functional-locks"></a>

## `locked` declarations block writes

`agent-coexist profile agents/alice --section role -m 'Rust implementer.'` · exit code 3
```text
error  profile  agents/alice
  reason        agents is locked by alice until 2026-09-17T11:00:00Z
  fix           if the lock is yours, add --as=alice before any -- separator; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist profile agents/alice --section role -m 'Rust implementer.'
```

`agent-coexist profile agents/alice --as alcie --section role -m 'Rust implementer.'` · exit code 3
```text
error  profile  agents/alice
  reason        agents is locked by alice until 2026-09-17T11:00:00Z
  fix           if the lock is yours and the --as in this command names someone else, rerun it with --as=alice in place of that --as; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist profile agents/alice --as alcie --section role -m 'Rust implementer.'
```

`agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'` · exit code 3
```text
error  brief  docs/agents/handoff
  reason        . is locked by carol until 2026-09-17T12:00:00Z
  fix           if the lock is yours and the --as in this command names someone else, rerun it with --as=carol in place of that --as; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'
  also-locked   alice  until 2026-09-17T11:00:00Z
  partial       .  W-STATE-ENTRY  L7
```

<a id="diag-ignore-syntax"></a>

## Malformed ignore rules

`agent-coexist list .` · exit code 1
```text
error  list  .
  reason        cannot scan ./.ignore: line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of ./.ignore, then retry: agent-coexist list .
```

`agent-coexist list work` · exit code 1
```text
error  list  work
  reason        cannot scan work/.ignore: line 2: stream did not contain valid UTF-8
  fix           correct line 2 of work/.ignore, then retry: agent-coexist list work
```

`agent-coexist list . --no-ignore` · .ignore has a syntax error
```text
ok  list  .
  ignore        --no-ignore
  build/
    hidden                    post
  docs/
    standup                   post
  total         2
```

<a id="diag-ignore-unreadable"></a>

## Unreadable ignore sources: not a syntax error

The OS message and code in the reason line vary by system; on Windows both cases surface as access denied (os error 5).

`agent-coexist list .` · exit code 1 · .ignore has no read permission
```text
error  list  .
  reason        cannot scan ./.ignore: Access is denied. (os error 5)
  fix           check that ./.ignore can be read, then retry: agent-coexist list .
```

`agent-coexist list .` · exit code 1 · .ignore is a directory
```text
error  list  .
  reason        cannot scan ./.ignore: Access is denied. (os error 5)
  fix           replace ./.ignore with a regular file or remove it, then retry: agent-coexist list .
```

<a id="diag-git-file"></a>

## .git file problems

In a worktree or submodule, .git is a small file pointing at the real git directory.

`agent-coexist list work` · exit code 1
```text
error  list  work
  reason        cannot scan work/.git: invalid gitdir reference
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

`agent-coexist list work` · exit code 1
```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/nowhere: The system cannot find the file specified. (os error 2)
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

`agent-coexist list work` · exit code 1
```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/../gitdata: gitdir reference is not a directory
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

<a id="diag-state-scan-after-write"></a>

## Write succeeds, post-write scan fails

`agent-coexist state src --as alice --status locked` · .ignore has a syntax error
```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of .ignore, then check for conflicts without declaring again: agent-coexist check-state src
```

`agent-coexist state src --as alice --status locked` · .ignore has no read permission
```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  Access is denied. (os error 5)
  fix           check that .ignore can be read, then check for conflicts without declaring again: agent-coexist check-state src
```

`agent-coexist state src --as alice --status locked` · .ignore is a directory
```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  Access is denied. (os error 5)
  fix           replace .ignore with a regular file or remove it, then check for conflicts without declaring again: agent-coexist check-state src
```

<a id="diag-ignore-too-large"></a>

## Combined ignore rules exceed the limit

`agent-coexist list room` · exit code 1
```text
error  list  room
  reason        cannot scan room/.ignore: error building NFA
  fix           shorten, simplify or remove rules in room/.ignore until they fit the pattern matcher's size and nesting limits, then retry: agent-coexist list room
```

`agent-coexist state room --as bob --status observing`
```text
warn  state  room
  created       room/state.agent-besties.md
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  scan          room/.ignore  error building NFA
  fix           shorten, simplify or remove rules in room/.ignore until they fit the pattern matcher's size and nesting limits, then check for conflicts without declaring again: agent-coexist check-state room
```
