---
description: Editing files means committing and pushing them.
dependencies: []
---

Commit and push every change your work produces in every affected repository, including generated output.

Commit existing dirty changes first with an appropriate message
before starting new work.

The sequence for landing work:

    jj commit -m 'short imperative message' path ...
    jj bookmark set main -r @-
    jj git push --bookmark main

`jj commit` snapshots the working copy. After it, `@-` is that
commit. `jj bookmark set main -r @-` advances main to it. Then
push.

A commit names the files it lands: `jj commit -m 'message' path ...`, and only the files this flow edited, usually inside its own flow directory. A commit without paths takes the whole working copy and is made only while the whole repository is locked, when nobody else may be editing.

FLOW_ID=<id> field-clj '#commit ["message" ["path" ...]]' runs this landing under one rule: it commits exactly the named repository-relative paths with a `Flow: <id>` trailer, leaves other dirty paths uncommitted, refuses when `FLOW_ID` is unset, when a named path is clean, or when `jj diff -r @- --name-only` differs from the named set, and prints one positional variant, `#success [flow commit [path ...] :main :pushed :present]` or `#refused …`.

Every `jj` command that takes a description uses `-m`. Never open
an editor. Never use raw `git`.

Clone a working copy from its real remote URL, never from another local checkout (`git clone --shared <local-path>` repoints `origin` at that checkout, and a push there never reaches the real remote). Before reporting a push landed, confirm the pushed revision against the real remote directly — `git ls-remote <real-remote-url>` — not merely against the checkout's configured `origin`, which some checkouts point at a mirror (gitolite, or another local clone) distinct from it.

A source file is written in pieces of a few hundred lines; a module that would exceed that is split.
