---
description: An ordinary Orchestrate Lock request must be constructed, submitted, or interpreted.
dependencies: []
---

Use `orchestrate` for ordinary requests. The client takes exactly one inline Datom value and no flags. The installed wrapper supplies `ORCHESTRATE_SOCKET`; a direct client binary requires it.

Acquire a Lock. Its four positional fields are `LockName`, `FlowId`, `LockPaths`, and `LockReason`:

    orchestrate 'Lock.{ <LockName> <FlowId> [ <absolute-path> ... ] <LockReason> }'

`LockName` names the work; `FlowId` is the owning flow's `FLOW_ID`; `LockPaths` lists the reserved absolute paths; `LockReason` states why. Put the actual owning `FLOW_ID` only in `FlowId`, never concatenated into `LockName` merely for uniqueness or title display. A name such as `FlowIdDocumentation` remains valid when flow ID is the subject of the work.

A reason containing a space or a delimiter is written in Datom curly quotes, “like this”; ASCII double quotes are not Datom string delimiters. A copyable multi-word reason example is:

    orchestrate 'Lock.{ OrchestrateDocs 444e5e [ /absolute/path/to/file ] “Clarify Lock fields” }'

A single-word reason is bare.

`Locked` returns the complete Lock: its integer ID, name, Flow, paths, and reason. `LockRejected` is a typed refusal. Paths are absolute.

Release by the returned integer ID using the Orchestrate release request skill variable, substituting the integer for <lock-id>. Read the typed reply.

`Released` returns the complete Lock. `ReleaseRejected` is a typed refusal.

Observe current Locks:

    orchestrate 'Observe.Locks'

`Observed` carries one complete point-in-time Lock snapshot. It is not a subscription.

Treat a client failure as a failed operation.
