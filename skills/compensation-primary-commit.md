---
description: Publish one flow's Primary paths without rewriting shared history.
---

Each flow publishes its own paths only.

Outside the PrimaryPublish lock, a flow runs only read-only jj commands in Primary, each with --ignore-working-copy. A command that rewrites @ or its ancestors never runs with --ignore-working-copy.

A lock's paths name only the checkouts being edited; a flow's own lane under Primary is never locked.

Under the lock: jj git fetch; jj commit of own paths; find it by its description; jj duplicate <own commit> --destination main@origin; on conflict, abandon the copy, release and report; otherwise set main to the copy, push, release.

The original working copy and main intentionally diverge. Do not rebase, restore, abandon, or otherwise change the original; reconciliation is separately reviewed.

