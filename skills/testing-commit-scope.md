---
description: A commit has been made in a working copy other flows also write, and its file list must be shown to hold only this flow's files.
dependencies: [file-editing, edit-coordination]
---

Prove the commit's scope by listing what it actually landed:

    jj diff -r @- --name-only

Every path returned must be one this flow edited. A path this flow did not touch means another flow's work was swept in: say so, name the path, and do not report the commit as clean.

Do this after every commit in a shared working copy, including one made with explicit paths — a path argument that names a directory takes everything under it.

Run it again before setting the bookmark, so a swept file is found before it is pushed.
