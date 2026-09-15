---
description: Jujutsu workspaces, changes, bookmarks, remote publication, integration, or recovery are involved.
dependencies: []
---

Use Jujutsu for repository operations.

Before a write, identify the repository, isolated workspace path and name, current change ID, intended bookmark, remote, and every dirty file's writer.

One writer owns one workspace. A colocated checkout is shared state; do not run Jujutsu commands there while another flow owns it.

Use an isolated workspace created from the real remote or an assigned clean workspace. Never clone from another local checkout.

Use `-m` or the equivalent headless message on every description-taking Jujutsu command.

Commit the owned change with `jj commit -m 'short imperative message'`.

Publish only the assigned producer bookmark with `jj bookmark set <producer-bookmark> -r @-` and `jj git push --bookmark <producer-bookmark>`.

Before reporting publication, verify the exact pushed revision against the real remote directly. A successful push exit status or a local bookmark alone is insufficient.

Do not move `main` or any shared bookmark unless an explicit authority names that exact target and the named integrator is operating from its clean integration workspace.

Freeze the reviewed producer revisions before integration. Resolve conflicts and test in the integrator's workspace before moving the authorized target bookmark.

Treat Jujutsu reads in a live colocated repository as potentially stateful because working-copy snapshots and Git-ref imports may occur. Prefer an isolated clone, an operation-specific read, or recorded evidence.

Use the operation log to investigate recovery. Do not prescribe a blanket `jj op undo`, reset, rebase, restore, abandon, workspace forget, or bookmark move in a shared store; first identify concurrent operations, owners, reachable revisions, and the authorized recovery target.

Do not forget or delete a workspace until its intended pushed revision and owner have been independently verified.
