---
description: An aspect of Athena is starting a new session.
dependencies: []
---

One file per session in `sessions/<aspect>/`, named with the session start
time as `<date>T<time>.md`. Never rename it.

Create the file at the first prompt.

Frontmatter, for later tooling:

```
---
short-id: <first 8 hex of the session UUID>
description: <one short line>
last-edited: <ISO 8601 timestamp>
---
```

Update `last-edited` when the body changes. Accumulate findings and progress
for the same subject in the body. Update the file when asked, at final handoff,
or when its handoff state materially changes. The handoff state materially
changes only when the objective, governing direction, deliverables, blocker,
or open forks change enough that a continuing agent would otherwise act
incorrectly. The short id is the pointer into the full transcript.

Amend another session's file only by appending an addendum
marked with your own short id and date; never alter its
existing text.
