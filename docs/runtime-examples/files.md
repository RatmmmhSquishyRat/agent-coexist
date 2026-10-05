# 写入文件的说明文字

<a id="brief-new-file"></a>

## brief：新建空导读

目录里还没有导读。

`agent-coexist brief onboarding --as alice`
```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
```

文件 onboarding.agent-besties.md：
```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.
```

<a id="state-new-file"></a>

## state：新建状态文件

目录已存在、还没有状态文件。

`agent-coexist state src/parser --as alice --status locked --for 30m`
```text
ok  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

文件 src/parser/state.agent-besties.md：
```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

alice 2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```
