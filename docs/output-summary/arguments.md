# Arguments and startup errors

<a id="arg-no-command"></a>

## No subcommand

`agent-coexist` · stderr · exit code 2
```text
error: 'agent-coexist' requires a subcommand but one was not provided
  [subcommands: guide, list, check, profile, post, subscribe, brief, state, read-notification, lookfor-comment, read-brief, check-state, wait, wait-post, wait-state, help]

Usage: agent-coexist <COMMAND>

For more information, try '--help'.
```

<a id="arg-unknown-command"></a>

## Unknown subcommand

`agent-coexist lsit` · stderr · exit code 2
```text
error: unrecognized subcommand 'lsit'

  tip: a similar subcommand exists: 'list'

Usage: agent-coexist <COMMAND>

For more information, try '--help'.
```

`agent-coexist frobnicate` · stderr · exit code 2
```text
error: unrecognized subcommand 'frobnicate'

Usage: agent-coexist <COMMAND>

For more information, try '--help'.
```

<a id="arg-unknown-option"></a>

## Unknown option

`agent-coexist list --bogus` · stderr · exit code 2
```text
error: unexpected argument '--bogus' found

  tip: 'list' has no option --bogus; 'agent-coexist list --help' lists its options; only if '--bogus' is the DIR itself, write it last, after '--'

Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]

For more information, try '--help'.
```

<a id="arg-missing-arguments"></a>

## Missing argument or option value

`agent-coexist check` · stderr · exit code 2
```text
error: the following required arguments were not provided:
  <PATH>

Usage: agent-coexist check <PATH>

For more information, try '--help'.
```

`agent-coexist list --type` · stderr · exit code 2
```text
error: a value is required for '--type <type>' but none was supplied

For more information, try '--help'.
```

<a id="arg-enum-values"></a>

## Invalid value for a fixed choice

`agent-coexist list --type posts` · stderr · exit code 2
```text
error: invalid value 'posts' for '--type <type>': 'posts' is not a valid type; expected post, brief, agent-profile (alias: profile) or state

For more information, try '--help'.
```

<a id="arg-name-values"></a>

## Malformed names

`agent-coexist post standup --as 'alice smith' -m hello` · stderr · exit code 2
```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice smith')
```

`agent-coexist lookfor-comment standup --mention @bob` · stderr · exit code 2
```text
error: --mention must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '@bob'); write 'bob' without the '@'
```

<a id="arg-numbers"></a>

## Non-numeric values

`agent-coexist lookfor-comment standup --reply-to first` · stderr · exit code 2
```text
error: invalid value 'first' for '--reply-to <n>': expected a decimal comment number from 0 to 18446744073709551615

For more information, try '--help'.
```

<a id="arg-time-durations"></a>

## Malformed times and durations

`agent-coexist state --as alice --status locked --for 30m1h` · stderr · exit code 2
```text
error: --for must be a duration such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '30m1h')
```

`agent-coexist state --as alice --status locked --for 3000000d` · stderr · exit code 2
```text
error: the declaration would end after 9999-12-31T23:59:59Z, the latest representable time; use a shorter --for (without --for, 'locked' lasts 2h and other statuses 8h)
```

`agent-coexist read-notification --as alice --since '2026-09-17 09:00:00Z'` · stderr · exit code 2
```text
error: --since must be an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '2026-09-17 09:00:00Z'); separate the date and time with 'T', not a space
```

<a id="arg-glob-regex"></a>

## Malformed filter globs and regexes

`agent-coexist list --include 'docs/[ab'` · stderr · exit code 2
```text
error: --include 'docs/[ab' is not a valid glob: unclosed character class; missing ']'
```

`agent-coexist brief onboarding --target src/main.rs --regex 'fn main('` · stderr · exit code 2
```text
error: --regex 'fn main(' does not compile: unclosed group
```

<a id="arg-target-required"></a>

## Missing companion option

`agent-coexist brief onboarding --entry 'parser entry point' --regex 'fn main'` · stderr · exit code 2
```text
error: --regex requires --target <path>
```

<a id="arg-conflicting-flags"></a>

## Extra and conflicting arguments

`agent-coexist check standup notes` · stderr · exit code 2
```text
error: unexpected argument 'notes' found

Usage: agent-coexist check <PATH>

For more information, try '--help'.
```

`agent-coexist brief onboarding --refresh -m note` · stderr · exit code 2
```text
error: --refresh cannot be combined with --target, --regex, -m or --remove
```

<a id="arg-env-overrides"></a>

## Invalid environment variable values

`AGENT_COEXIST_NOW='2026-09-17 10:00:00Z' agent-coexist check standup` · stderr · exit code 2
```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got '2026-09-17 10:00:00Z'); separate the date and time with 'T', not a space, or unset it to use the system clock
```

<a id="arg-dash-values"></a>

## Values starting with a dash

`agent-coexist post standup --as alice -m '- parser done'` · stderr · exit code 2
```text
error: unexpected argument '- ' found

  tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, attach it with '=': -m='- parser done'

Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]

For more information, try '--help'.
```

<a id="arg-invisible-whitespace"></a>

## Invisible whitespace

`agent-coexist post standup --as $'alice\r' -m hello` · stderr · exit code 2
```text
error: --as must not contain control characters (got 'alice<U+000D>'); remove the whitespace before or after 'alice'
```

<a id="arg-path-lookup"></a>

## PATH is a directory

`agent-coexist lookfor-comment docs` · exit code 1
```text
error  lookfor-comment  docs
  reason        docs.agent-besties.md does not exist; docs is a directory, and this command needs one collaboration file
  fix           rerun with the path of one collaboration file (the .agent-besties.md suffix may be omitted); collaboration files in that directory: agent-coexist list docs
```
