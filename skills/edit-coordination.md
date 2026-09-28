---
description: Another agent may be writing the same paths.
dependencies: [orchestrate]
---

A flow's own directory is never locked: only the flow that has its id ever writes there.

Reserve the complete write set with `Lock` before editing.

Edit only after receiving `Locked`. When the lock is held by another living flow, tell that flow what you want the lock for: it may make the change itself, or tell you when the lock is free. Do not edit until you hold the lock. On a client failure, report the failure and do not edit.

Release the returned integer ID with `Release` when editing ends. Read the typed reply.
