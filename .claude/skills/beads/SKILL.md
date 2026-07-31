---
name: beads
description: 'Work must be tracked across sessions or between agents. Requires: secrets.'
---

Verify the consumer’s official stdin or file-descriptor interface first.
Pipe GoPass producer output directly to that consumer.
Use `set -o pipefail`.
Use absolute or verified executables where appropriate.
Suppress secret-bearing output.
Never use command substitution, argv, environment, clipboard, `tee`, filters, process substitution, temporary files, or unsupported prompt automation.
Name the unavoidable crypto-backend, kernel, and consumer boundary without claiming more isolation.
Persistent import is allowed when the consumer’s supported contract requires it and the task authorizes credential setup.

Run `bd` from the repository the work belongs to.

### Fields

`--title` — the outcome, not the activity.
`-d/--description` — what an agent with no context needs: what is true now, what is wrong, where to look.
`--design` — how this work is to be done, and why that way.
`--acceptance` — what makes it done, checkable by someone who did not write it.
`--notes` — current working state. `bd note <id> <text>` appends to it.
`bd comment <id> <text>` — separate append-only stream. Comments are events; notes are state.
`-t/--type` — bug, feature, task, epic, chore, decision, spike, story, milestone.
`-p/--priority` — 0 highest, 2 default.
`--parent`, `--labels`, `--due`, `--estimate`, `--external-ref`.

### Dependencies

`bd dep add <id> <blocker>` — id waits on blocker. Default type `blocks`.
Types: blocks, tracks, related, parent-child, discovered-from, until, caused-by, validates, supersedes.
`bd dep add <new> <origin> -t discovered-from` when work uncovers work.
Beads in another repository are named by id in text. No link crosses databases.

### Lifecycle

`bd ready` — what can be worked now, blockers accounted for. Start here.
`bd update <id> --claim` — assignee and in_progress in one atomic step.
`bd close <id> -r '<proof>'` — the reason carries the evidence; there is no evidence field.
`bd reopen`, `bd defer --until`, `bd blocked`, `bd query`.
`--json` on any read command.

### Store

Verify the selected store before using it.
Do not use Orchestrate claims for Beads database writes.
Run `bd init` only when `.beads` is absent.
Run `bd bootstrap` when an existing scaffold has no usable database.
Use embedded Dolt sequentially.
Wait and retry an embedded-Dolt lock instead of concurrent access.

When repository metadata identifies the owner, repository name, and GitHub visibility, create and attach its missing DoltHub database.
Use one database per repository, named from that metadata, with matching visibility and no suffix.
Use `secrets` to connect `gopass show -o dolthub.com/api-token` directly to curl’s supported secret-input interface; keep the token out of agent output, arguments, and environment.
Authenticate `GET https://www.dolthub.com/api/v1alpha1/user` and `POST https://www.dolthub.com/api/v1alpha1/database` with `Authorization: token ...`; send `ownerName`, `repoName`, and `visibility` to create the matching database.
Treat a missing-database read returning `400 no such repository` as absent and a creation response returning `409 already exists` as success.
Configure the canonical Beads remote with `bd dolt remote add`.
Use `secrets` for PAT and JWK delivery.
When Dolt needs a missing JWK credential, import it through Dolt’s supported stdin contract.
Persistent credential import is allowed when that supported contract requires it and the task authorizes credential setup.
Synchronize without force.
Stop only when owner, name, or visibility cannot be derived, or an unexpected conflict or destructive repair is required.
