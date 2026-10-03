---
description: Publish one flow's Primary paths without rewriting shared history.
---

Each flow publishes only its own paths, from an independent Git clone with its own colocated jj workspace and bookmark namespace, and never commits, abandons, rebases or restores in the shared working copy.

Outside the PrimaryPublish lock, a flow runs only read-only jj commands in Primary, each with --ignore-working-copy. A command that rewrites @ or its ancestors never runs with --ignore-working-copy.

A lock's paths name only the checkouts being edited; a flow's own lane under Primary is never locked. The PrimaryPublish lock's only path is the sentinel `<Primary>/.PrimaryPublish.lock`, a name with no file ever created there, so publishers serialize on one name and lock no checkout.

For publication, create an independent Git clone of Primary and initialize its colocated jj workspace with `jj git init --colocate`. Its `.git` directory and jj bookmark namespace must be independent of the shared working copy. Copy only the flow's verified paths into that clone.

Under the lock, work only in that independent clone: fetch, commit only the flow's paths, find that commit by its description, and duplicate it to `main@origin`. Capture the COPY result and run `jj resolve --list -r <COPY>`: listed paths are a conflict, and `No conflicts found at this revision` is clean. On conflict, abandon only the COPY in that independent clone, release and report. Otherwise set that clone's `main` bookmark to the exact COPY, run `jj git push --bookmark main`, and release. The shared working copy's bookmark is never changed.

The shared working copy and `main` may intentionally diverge. Reconciliation is separately reviewed and never changes the shared working copy.

A reconciliation that lands a flow's path on main is followed, under the same lock, by an independent-clone commit of that path holding exactly the landed content before any newer edit of it, so the flow's next duplicate applies cleanly.
