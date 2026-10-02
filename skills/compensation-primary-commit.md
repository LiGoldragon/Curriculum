---
description: Publish one flow's Primary paths without rewriting shared history.
---

Each flow publishes its own paths only.

Outside the PrimaryPublish lock, every jj command run in Primary passes --ignore-working-copy; a read is not exempt.

Under PrimaryPublish: `jj git fetch; jj commit of own paths; find the commit by its own description; jj rebase -s <own commit> -d main@origin, carrying the working-copy child intact; if any conflict, jj op restore to the pre-rebase operation, release, report, publish nothing; else jj bookmark set main, jj git push, release.`
