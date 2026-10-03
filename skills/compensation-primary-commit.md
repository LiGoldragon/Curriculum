---
description: Publish one flow's Primary paths without rewriting shared history.
---

Each flow publishes its own paths only.

Outside the PrimaryPublish lock, a flow runs only read-only jj commands in Primary, each with --ignore-working-copy. A command that rewrites @ or its ancestors never runs with --ignore-working-copy.

A lock's paths name only the checkouts being edited; a flow's own lane under Primary is never locked.
The PrimaryPublish lock's only path is the sentinel `<Primary>/.PrimaryPublish.lock`, a name with no file ever created there, so publishers serialize on one name and lock no checkout.

Under the lock: jj git fetch; jj commit of own paths; find it by its description; jj duplicate <own commit> --destination main@origin. Capture the COPY commit id from the duplicate's result and run `jj resolve --list -r <COPY>`: listed paths are a conflict, and `No conflicts found at this revision` is clean. On conflict, abandon the copy, release and report; otherwise set main to the copy, push, release.

The original working copy and main intentionally diverge. Do not rebase, restore, abandon, or otherwise change the original; reconciliation is separately reviewed.

A reconciliation that lands a flow's path on main is followed, under the same lock, by a working-copy commit of that path holding exactly the landed content, before any newer edit of it, so the flow's next duplicate applies cleanly.
