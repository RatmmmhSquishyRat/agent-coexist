# Arguments and startup errors

<a id="arg-unknown-command"></a>

## Unknown subcommand

`agent-coexist lsit` · stderr · exit code 2
```text
error: unrecognized subcommand 'lsit'

  tip: a similar subcommand exists: 'list'

Usage: agent-coexist <COMMAND>

For more information, try '--help'.
```

<a id="arg-missing-arguments"></a>

## Missing required path argument

`agent-coexist check` · stderr · exit code 2
```text
error: the following required arguments were not provided:
  <PATH>

Usage: agent-coexist check <PATH>

For more information, try '--help'.
```

<a id="arg-conflicting-flags"></a>

## Incompatible brief options

`agent-coexist brief onboarding --refresh -m note` · stderr · exit code 2
```text
error: --refresh cannot be combined with --target, --regex, -m or --remove
```

<a id="arg-value-validation"></a>

## Invalid nick

`agent-coexist post standup --as 'alice smith' -m hello` · stderr · exit code 2
```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice smith')
```

<a id="arg-dash-values"></a>

## Values starting with a dash

`agent-coexist post standup --as alice -m '- parser done'` · stderr · exit code 2
```text
error: unexpected argument '- ' found

  tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, attach it with '=': -m='- parser done'

Usage: agent-coexist post <PATH> --as <nick> (-m <content> | -F <path>) [--edit <n>]

For more information, try '--help'.
```

<a id="arg-glob-regex"></a>

## Invalid regex

`agent-coexist brief onboarding --target src/main.rs --regex 'fn main('` · stderr · exit code 2
```text
error: --regex 'fn main(' does not compile: unclosed group
```
