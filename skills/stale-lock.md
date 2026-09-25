---
description: An Orchestrate lock is held by a flow that no longer answers.
dependencies: [orchestrate, messaging]
---

A lock whose holder is stale may be released by any flow. The holder is stale when `hm-list` gives its binding `STALE` or exited, Herdr shows no live pane for it, and one `hm-send` to it returns `Held` or fails. A live holder is messaged, never unlocked.

Before release: `Observe.Locks` and record the lock, its paths, and the holder's three witnessed states in a receipt; commit uncommitted work under its paths as found, on its own branch, naming the holder in the commit message; then release the lock by its integer ID.

A releaser that will edit the paths takes its own `Lock` at once; a releaser acting for another flow tells that flow the lock is free and takes none.

The by-hand registry of flows is HM: `hm-register` and `hm-list`.
