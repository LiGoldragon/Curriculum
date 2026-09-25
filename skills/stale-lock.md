---
description: An Orchestrate lock is held by a flow that no longer answers.
dependencies: [orchestrate, messaging]
---

A lock whose holder is stale may be released by any flow. The holder is stale when hm-list shows its binding STALE or exited, Herdr has no live pane for it, and one hm-send to it comes back Held. Before release: Observe.Locks, and record the lock, its paths and the holder's state in a receipt; commit any uncommitted work under its paths as found, naming the holder; then release by ID and take your own lock. A live holder is messaged, never unlocked. The living: "break the locks. Nobody in particular owns Flow Source."
