---
description: Feature work would collide with a checkout someone else holds.
dependencies: [version-control]
---

Use the assigned isolated Jujutsu workspace for feature work.

Before writing, identify the workspace path, workspace name, current change, intended producer bookmark, and writer responsible for each dirty file.

One writer owns one workspace at a time. Do not share a claimed workspace or use a colocated checkout held by another flow.

Publish only the assigned producer bookmark. A feature writer does not advance `main` or a shared integration bookmark.

Conclude the workspace when work lands or is rejected; do not forget it while its only reachable work is unverified.
