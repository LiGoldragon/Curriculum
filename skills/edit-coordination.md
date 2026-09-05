---
description: Another agent may be writing the same paths.
dependencies: [orchestrate]
---

A flow's own directory is never locked: only the flow that has its id ever writes there.

Reserve the complete write set with `Lock` before editing.

Edit only after receiving `Locked`. On `LockRejected` or a client failure, report the failure and do not edit.

Release the returned integer ID with `Release` when editing ends. Read the typed reply.
