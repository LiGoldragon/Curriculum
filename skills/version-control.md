---
description: Jujutsu workspaces, changes, bookmarks, remote publication, integration, or recovery are involved.
dependencies: []
---

Use Jujutsu for repository operations.

Before a write, identify the repository, isolated workspace path and name, current change ID, intended bookmark, remote, and every dirty file's writer.

Commit only files your flow owns. Preserve every other dirty file in place and report its writer; do not absorb it into a convenience commit.

One writer owns one workspace. A colocated checkout is shared state; do not run Jujutsu commands there while another flow owns it.

Use an isolated workspace created from the real remote or an assigned clean workspace. Never clone from another local checkout.

Use `-m` or the equivalent headless message on every description-taking Jujutsu command.

Commit the owned change with `jj commit -m 'short imperative message'`.

Publish only the assigned producer bookmark with `jj bookmark set <producer-bookmark> -r @-` and `jj git push --bookmark <producer-bookmark>`.

Before reporting publication, verify the exact pushed revision against the real remote directly. A successful push exit status or a local bookmark alone is insufficient.

Do not move `main` or any shared bookmark unless an explicit authority names that exact target and the named integrator is operating from its clean integration workspace.

Freeze the reviewed producer revisions before integration. Resolve conflicts and test in the integrator's workspace before moving the authorized target bookmark.

## Branch protocol (draft for living approval)

Give each work item one producer bookmark, named `flow/<id>` or `proposal/<flow>-<item>`.

Keep a lane inventory for every repository. For each producer bookmark, record its purpose and one state: `open`, `candidate`, `merged`, or `abandoned`.

Call a revision a candidate only when it has been reviewed, all applicable checks are green, and its deployment shape is stated.

A named integrator may promote an approved candidate only from the clean integration workspace and with authority to advance `main`. Treat the promotion and retirement of the producer bookmark on the remote as one coordinated operation. Git and distributed Jujutsu operations may not be atomic: verify that the exact approved revision is included in `main` on the real remote before deleting the remote producer bookmark, and preserve the review, check, deployment, and inclusion evidence.

When a producer bookmark is abandoned, preserve its evidence, then delete its remote bookmark as a coordinated operation.

Refresh each repository's lane inventory at least weekly. Resolve every remote bookmark with no lane entry as an orphan.

These draft lines do not authorize a remote deletion, a `main` move, integrator nomination, or adoption of this law; the living approves their final wording and any such action separately.

Treat Jujutsu reads in a live colocated repository as potentially stateful because working-copy snapshots and Git-ref imports may occur. Prefer an isolated clone, an operation-specific read, or recorded evidence.

Use the operation log to investigate recovery. Do not prescribe a blanket `jj op undo`, reset, rebase, restore, abandon, workspace forget, or bookmark move in a shared store; first identify concurrent operations, owners, reachable revisions, and the authorized recovery target.

Do not forget or delete a workspace until its intended pushed revision and owner have been independently verified.
