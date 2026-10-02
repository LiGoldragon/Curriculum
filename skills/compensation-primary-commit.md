---
description: Publish one flow's Primary paths without rewriting shared history.
---

Each flow publishes its own paths only.

Outside the PrimaryPublish lock, every jj command run in Primary passes --ignore-working-copy; a read is not exempt.

Under the lock, the sequence is: jj commit of own paths; jj git fetch; jj rebase -r of that one commit onto main@origin; jj bookmark set main; jj git push; release. Nothing else is rebased.

Under PrimaryPublish, a flow finds its commit by its own description, never by assuming @-.

Never rewrite or rebase a mutable revision that is not your own commit. A flow that finds a conflict reports it rather than repairing it.
