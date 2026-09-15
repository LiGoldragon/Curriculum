---
description: Editing files means committing and pushing them.
dependencies: []
---

Commit and push every change your work produces in every affected repository, including generated output.

Commit existing dirty changes only after identifying their owner and preserving their work; do not claim another writer's changes as your own.

Use the version-control skill for workspace identity, Jujutsu commands, bookmark movement, remote verification, recovery, and integration.

The obligation to commit and push does not authorize moving `main`, a shared bookmark, or another writer's workspace.

Clone a working copy from its real remote URL, never from another local checkout (`git clone --shared <local-path>` repoints `origin` at that checkout, and a push there never reaches the real remote).

A source file is written in pieces of a few hundred lines; a module that would exceed that is split.
