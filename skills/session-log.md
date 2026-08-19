---
description: An aspect of Athena is starting a new session.
dependencies: []
---

One file per session in `sessions/<aspect>/`, named by the session's short
id — the first 8 hex of the session UUID. Create it at the first prompt.
Never rename it.

No frontmatter. The file opens with a very terse summary section, re-edited
as the session goes, so a short read gives the subjects. Findings and
progress accumulate below it. The file changes only when asked, at final
handoff, or when its handoff state materially changes — when the objective,
governing direction, deliverables, blocker, or open forks change enough that
a continuing agent would otherwise act incorrectly. Refinements on the same
subject are not events.

A new session appends its line — aspect, short id, the summary's first
line — to `sessions/index.md`. The tail of the index gives the latest
sessions.

Amend another session's file only by appending an addendum marked with your
own short id; never alter its existing text.
