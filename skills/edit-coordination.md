---
description: Another agent may be writing the same paths.
dependencies: []
---

Reserve the complete write set before editing.

    orchestrate 'PathLock.{<name> [<absolute-path> ...] (<description>)}'

Only edit after receiving `PathLockRegistered`. On
`PathLockRegistrationRejected` or a transport failure, report the
failure and do not edit.

Release a registered PathLock after editing.

    orchestrate 'PathLockRelease.{<name>}'

Read the typed `PathLockReleased` or `PathLockReleaseRejected` reply.
