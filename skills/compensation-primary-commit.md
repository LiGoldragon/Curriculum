---
description: Publish one flow's Primary paths without rewriting shared history.
---

Each flow publishes its own paths only.

Outside the PrimaryPublish lock, every jj command run in Primary passes --ignore-working-copy; a read is not exempt.

Under PrimaryPublish, run `jj git fetch`, then commit only the flow's own paths. Before duplication, identify the ORIGINAL by its unique own description. Run `jj duplicate -r <ORIGINAL> -d main@origin`; capture the DISTINCT IMMUTABLE COPY ID from its result. Duplicate only that original: upstream remains retained and other unpublished work stays excluded.

The original working copy and main intentionally diverge. Do not rebase, restore, abandon, or otherwise change the original; reconciliation is separately reviewed.

Inspect the exact COPY ID for conflicts. If it is conflicted, run `jj abandon <COPY ID>` for that copy only, release PrimaryPublish, report, and publish nothing. Otherwise set main to the exact COPY ID, push, and release.
