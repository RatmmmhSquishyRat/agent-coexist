# Diagnostics and recovery

<a id="diag-file-unusable"></a>

## File UTF-8 and missing heading errors

`agent-coexist check standup` · exit code 1 · The é on line 6 is Latin-1 encoded
```text
error  check  standup
  file          standup.agent-coexist.md
  error  L6  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
```

`agent-coexist check notes` · exit code 1
```text
error  check  notes
  file          notes.agent-coexist.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# notes.post" (or .brief / .agent-profile), or "# .state" for a state file
```

<a id="diag-comment-headers"></a>

## Broken comment headers

`agent-coexist check standup` · exit code 1
```text
error  check  standup
  file          standup.agent-coexist.md
  type          post
  error  L9  "## #2 bob 2026-09-17T09:05:00Z" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (2026-09-17T09:05:00Z)"
  error  L15  "## #02 carol (2026-09-17T09:10:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #5 carol (2026-09-17T09:10:00Z)"; its number changes from #2 to #5 if no reply in this post already names @#5, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply in this post already names. Change any @#2 that meant this comment to that number
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

<a id="diag-post-warnings"></a>

## Comment sequence warning

`agent-coexist check seq`
```text
warn  check  seq
  file          seq.agent-coexist.md
  type          post
  warn   L3  text outside any fence at L3 is not part of the record
         why  this text stays in the file but is not included in CLI reports
         fix  the writer may move this text into a fenced body or delete it. Others must ask the writer, not infer who wrote it from a nearby heading; only a comment's author edits that comment
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
         why  this text stays in the file but is not included in CLI reports
         fix  the writer may move this text into a fenced body or delete it. Others must ask the writer, not infer who wrote it from a nearby heading; only a comment's author edits that comment
  warn   L25  text outside any fence at L25 is not part of the record
         why  this text stays in the file but is not included in CLI reports
         fix  the writer may move this text into a fenced body or delete it. Others must ask the writer, not infer who wrote it from a nearby heading; only a comment's author edits that comment
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Parser plan.
  latest        #2  carol  2026-09-17T09:02:00Z  Lexer plan.
  subscribers   none
```

<a id="diag-brief-entries"></a>

## Brief entry drift warning

`agent-coexist check handoff`
```text
warn  check  handoff
  file          handoff.agent-coexist.md
  type          brief
  warn   L3  entry "parser" is shifted: src/parser.rs changed since its hash was taken
         why  the note may describe code that no longer looks like this
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief handoff.agent-coexist.md --entry=parser --refresh
  warn   L12  entry "lexer" is stale: target missing
         why  the note points at something that is gone or no longer matches its regex
         fix  choose a file you know is the right target, review that file and the note, update the note if needed, then point the entry at it; if it no longer exists, remove or rewrite the entry. This command also records the target's current hash: agent-coexist brief handoff.agent-coexist.md --target <file> --entry=lexer
  warn   L22  entry "errors" is stale: regex not found
         why  the note points at something that is gone or no longer matches its regex
         fix  review the target and the note, update the note if it no longer describes the target, then give the entry a regex that matches it; this command also records the target's current hash: agent-coexist brief handoff.agent-coexist.md --target=src/errors.rs --entry=errors --regex '<pattern>'
  entries       3
  fresh         0
  shifted       1
  stale         2
  entry         parser  shifted  src/parser.rs  parse_document returns a Doc.
  entry         lexer  stale  src/lexer.rs  target missing  Tokenizer entry.
  entry         errors  stale  src/errors.rs  regex not found  Error type.
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

## `locked` declarations block brief write

`agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'` · exit code 3
```text
error  brief  docs/agents/handoff
  reason        . is locked by carol until 2026-09-17T12:00:00Z
  fix           if the lock is yours and the --as in this command names someone else, rerun it with --as=carol in place of that --as. Otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'
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

<a id="diag-state-scan-after-write"></a>

## Write succeeds, post-write scan fails

`agent-coexist state src --as alice --status locked` · .ignore has a syntax error
```text
warn  state  src
  created       src/state.agent-coexist.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of .ignore, then check for conflicts without declaring again: agent-coexist check-state src
```
