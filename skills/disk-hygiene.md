---
description: Space must be reclaimed, or data removed to reclaim it.
dependencies: []
---

Measure before and after cleanup.
A maintenance chore is a bounded, repeatable inventory and reclamation operation with an explicit scope, owner, authorization, and recorded result.
Inventory is read-only and reports per-user retention categories for store, build, cache, repository, dirty state, and oversized data.
Shared paths are attributed once to their physical owner and separately marked when retained by another user; never double-count them.
Reclamation requires separate per-host and per-user authorization naming the understood targets and retention reason.
Delete only authorized, understood data.
Never delete boot state, rollback state, or unknown or unreadable data.
End every chore with a per-user report of measured before and after state, retained references, skipped unknowns, and suggested follow-up actions.
Preserve boot and rollback state when reclaiming generations.
