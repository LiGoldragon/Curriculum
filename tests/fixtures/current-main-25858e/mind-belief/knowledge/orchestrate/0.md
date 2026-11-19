---
description: An ordinary Orchestrate Lock request must be constructed, submitted, or interpreted.
dependencies: []
---

Use `orchestrate` for ordinary requests. The installed wrapper supplies `ORCHESTRATE_SOCKET`; a direct client binary requires it.

Acquire a Lock. Its four positional fields are `LockName`, `FlowId`, `LockPaths`, and `LockReason`:

    orchestrate 'Lock.{ <LockName> <FlowId> [ <absolute-path> ... ] <LockReason> }'

`LockName` names the work; `FlowId` is the owning flow's `FLOW_ID`; `LockPaths` lists the reserved absolute paths; `LockReason` states why.

Datom strings are in guillemets, «like this».

A single-word reason is bare.

`Locked` returns the complete Lock: its integer ID, name, Flow, paths, and reason. `LockRejected` is a typed refusal. Paths are absolute.

Release by the returned integer ID:

    orchestrate 'Release.<lock-id>'

`Released` returns the complete Lock. `ReleaseRejected` is a typed refusal.

Observe current Locks:

    orchestrate 'Observe.Locks'

`Observed` carries one complete point-in-time Lock snapshot.

