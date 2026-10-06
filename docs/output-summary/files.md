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

<a id="post-new-file"></a>

## post: create a post with its first comment

The post does not exist yet; current time: 2026-09-17T10:00:00Z.

`agent-coexist post parser-review --as alice -m $'Decision: keep the parser API unchanged; adjust the error mapping.\nEvidence: src/parser/mod.rs and reports/parser-check.txt.\nRequest: @bob review the mapping before integration; reply here.'`
```text
ok  post  parser-review
  created       parser-review.agent-besties.md
  comment       #1  L3
  mentions      bob
```

file parser-review.agent-besties.md:
````text
# parser-review.post

## #1 alice (2026-09-17T10:00:00Z)

```md
Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Request: @bob review the mapping before integration; reply here.
```
````

<a id="profile-new-file"></a>

## profile: create a profile with a scope section

The `team/` directory does not exist yet.

`agent-coexist profile team/agents/alice --as alice --section scope -m 'Parser error mapping. Send review requests to parser-review.'`
```text
ok  profile  team/agents/alice
  created       team/
  created       team/agents/
  created       team/agents/alice.agent-besties.md
  section       scope  set
```

file team/agents/alice.agent-besties.md:
````text
# alice.agent-profile

## scope

```md
Parser error mapping. Send review requests to parser-review.
```
````

<a id="brief-populated-file"></a>

## brief: create a brief with an entry

`src/parser/mod.rs` is an existing example file; `onboarding` does not exist yet.

`agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document; check its error mapping before changing callers.'`
```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         parser entry point  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  note          set
```

file onboarding.agent-besties.md:
````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/parser/mod.rs
- hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

```md
Start at parse_document; check its error mapping before changing callers.
```
````
