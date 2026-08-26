---
description: An ordinary Orchestrate PathLock request must be constructed, submitted, or interpreted.
dependencies: []
---

Use `orchestrate` for ordinary requests. The client takes exactly one inline Datom value and no flags. The installed wrapper supplies `ORCHESTRATE_SOCKET`; a direct client binary requires it.

Register a path lock:

    orchestrate 'PathLock.{<name> [<absolute-path> ...] (<description>)}'

`PathLockRegistered` accepts it. `PathLockRegistrationRejected` carries `DuplicateActiveName` or `PathOverlap`. An empty path set, non-absolute path, `..`, or repeated normalized path currently fails without a typed reply.

Release a path lock by name:

    orchestrate 'PathLockRelease.{<name>}'

`PathLockReleased` accepts it. `PathLockReleaseRejected` carries `UnknownActiveName`.

The current contract has no operation to list or observe active PathLocks. Treat a parsing, environment, transport, framing, or missing-reply failure as a failed operation.
