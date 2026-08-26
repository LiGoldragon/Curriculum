---
description: An ordinary Orchestrate Lock request must be constructed, submitted, or interpreted.
dependencies: []
---

Use `orchestrate` for ordinary requests. The client takes exactly one inline Datom value and no flags. The installed wrapper supplies `ORCHESTRATE_SOCKET`; a direct client binary requires it.

Acquire a Lock:

    orchestrate 'Lock.{<name> <flow> [<absolute-path> ...] <reason>}'

`Locked` returns the complete Lock: its integer ID, name, Flow, paths, and reason. `LockRejected` is a typed refusal. Paths are absolute.

Release by the returned integer ID:

    orchestrate 'Release.{<lock-id>}'

`Released` returns the complete Lock. `ReleaseRejected` is a typed refusal.

Observe current Locks:

    orchestrate 'Observe.Locks'

`Observed` carries one complete point-in-time Lock snapshot. It is not a subscription.

Treat a client failure as a failed operation.
