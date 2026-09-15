---
description: A breaking change must be deployed.
dependencies: [documentation-placement]
---

Document how to deploy each breaking change in the repository's `UPGRADES.md`.
Land the documentation with the breaking change.
If deployment fails or partially fails, correct the documentation before continuing.
Before deploying a breaking change to production, arm an automatic countdown rollback to the last known working stack.
Cancel the countdown only after a witness confirms that both network connectivity and remote access work on the new stack, from the host itself or a watch flow on another host.
Do not deploy a change that can lose remote access without that timeout.
Record the rollback target, timeout, cancellation authority, and witness in `UPGRADES.md`.
