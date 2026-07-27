---
description: Work must be tracked across sessions or between agents.
---

Run `bd` from the repository the work belongs to.

## Fields

`--title` — the outcome, not the activity.
`-d/--description` — what an agent with no context needs: what is true now, what is wrong, where to look.
`--design` — how this work is to be done, and why that way.
`--acceptance` — what makes it done, checkable by someone who did not write it.
`--notes` — current working state. `bd note <id> <text>` appends to it.
`bd comment <id> <text>` — separate append-only stream. Comments are events; notes are state.
`-t/--type` — bug, feature, task, epic, chore, decision, spike, story, milestone.
`-p/--priority` — 0 highest, 2 default.
`--parent`, `--labels`, `--due`, `--estimate`, `--external-ref`.

## Dependencies

`bd dep add <id> <blocker>` — id waits on blocker. Default type `blocks`.
Types: blocks, tracks, related, parent-child, discovered-from, until, caused-by, validates, supersedes.
`bd dep add <new> <origin> -t discovered-from` when work uncovers work.
Beads in another repository are named by id in text. No link crosses databases.

## Lifecycle

`bd ready` — what can be worked now, blockers accounted for. Start here.
`bd update <id> --claim` — assignee and in_progress in one atomic step.
`bd close <id> -r '<proof>'` — the reason carries the evidence; there is no evidence field.
`bd reopen`, `bd defer --until`, `bd blocked`, `bd query`.
`--json` on any read command.
