---
description: An aspect of Athena is starting a new session.
dependencies: []
---

One file per session in `sessions/<aspect>/`, named
`<date>T<time>.md`. The time is the last edit's: rename the
file when you update it.

Create the file at the first prompt. Update it at every major
event or turning point in the session, and when asked.

Frontmatter, for later tooling:

```
---
short-id: <first 8 hex of the session UUID>
description: <one short line>
---
```

The body carries a very good summary of everything touched in
the session: subjects, rulings landed, artifacts produced,
dispatches sent, forks left open. The short id is the pointer
into the full transcript.

Amend another session's file only by appending an addendum
marked with your own short id and date; never alter its
existing text.
