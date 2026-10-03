---
description: Editing files means committing and pushing them.
dependencies: []
---

Commit and push every change your work produces in every affected repository, including generated output.

Commit existing dirty changes first with an appropriate message
before starting new work.

Primary is shared on disk, but it is not a commit workspace. Publish its paths only from an independent Git clone with its own colocated jj workspace and bookmark namespace; never commit, abandon, rebase, or restore in the shared working copy. Use `compensation-primary-commit` for its path-limited publication. In another repository or independent jj workspace, a commit names only the paths this flow edited: `jj commit -m 'message' path ...`.

The sequence for landing normal non-Primary jj work:

    jj commit -m 'short imperative message'
    jj bookmark set main -r @-
    jj git push --bookmark main

`jj commit` snapshots the working copy. After it, `@-` is that
commit. `jj bookmark set main -r @-` advances main to it. Then
push.

Every `jj` command that takes a description uses `-m`. Never open
an editor. Never use raw `git`.

A source file is written in pieces of a few hundred lines; a module that would exceed that is split.
