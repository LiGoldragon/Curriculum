---
description: Another agent may be writing the same paths.
dependencies: [orchestrate]
---

Reserve the complete write set with `PathLock` before editing.

Edit only after receiving `PathLockRegistered`. On `PathLockRegistrationRejected` or a client failure, report the failure and do not edit.

Release the reservation with `PathLockRelease` when editing ends. Read the typed release reply.
