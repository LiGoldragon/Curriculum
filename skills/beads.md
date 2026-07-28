---
description: Work must be tracked across sessions or between agents.
dependencies: [secrets]
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

## Store

Verify the selected store before using it.
Run `bd init` only when `.beads` is absent.
Run `bd bootstrap` when an existing scaffold has no usable database.
Use embedded Dolt sequentially.
Wait and retry an embedded-Dolt lock instead of concurrent access.

When repository metadata identifies the owner, repository name, and GitHub visibility, create and attach its missing DoltHub database.
Use one database per repository, named from that metadata, with matching visibility and no suffix.
Create hosted state through DoltHub API v2.
Configure the Beads remote with `bd dolt remote add`.
Use `secrets` for PAT and JWK delivery.
When Dolt needs a missing JWK credential, import it through Dolt’s supported stdin contract.
Persistent credential import is allowed when that supported contract requires it and the task authorizes credential setup.
Synchronize without force.
Stop only when owner, name, or visibility cannot be derived, or an unexpected conflict or destructive repair is required.
