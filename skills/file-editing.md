---
description: Editing files means committing and pushing them.
dependencies: []
---

Always commit and push edited work.

Commit existing dirty changes first with an appropriate message
before starting new work.

The sequence for landing work:

    jj commit -m 'short imperative message'
    jj bookmark set main -r @-
    jj git push --bookmark main

`jj commit` snapshots the working copy. After it, `@-` is that
commit. `jj bookmark set main -r @-` advances main to it. Then
push.

Every `jj` command that takes a description uses `-m`. Never open
an editor. Never use raw `git`.

A source file is written in pieces of a few hundred lines; a module that would exceed that is split.
