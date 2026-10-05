# Text written to files

<a id="brief-new-file"></a>

## brief: create a new, empty brief

The directory holds no brief yet.

`agent-coexist brief onboarding --as alice`
```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
```

file onboarding.agent-besties.md:
```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.
```

<a id="state-new-file"></a>

## state: create a new state file

The directory exists and holds no state file yet.

`agent-coexist state src/parser --as alice --status locked --for 30m`
```text
ok  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

file src/parser/state.agent-besties.md:
```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

alice 2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```
