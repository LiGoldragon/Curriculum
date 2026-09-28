# testing-push-landed: created after the baseline

Created in dad80ad, 2026-09-19 20:34 -0600: Add a round of agent-written testing skills. Made by: Claude Fable 5.1 <noreply@anthropic.com>.

## View

Flow f38926's round (dad80ad).
View: drop. file-editing already says to confirm against the real remote with `git ls-remote`.

Under the kinds the living settled on 2026-09-28 this skill would be named `test-push-landed`.

## The whole skill as it stands on main (3726da5)

````markdown
---
description: A push has been made and the revision must be shown present on the real remote before the work is reported as landed.
dependencies: [behavior, file-editing]
---

Prove a push by the hash on the real remote: record the local revision, push, then read the remote ref and compare.

    jj log -r @- --no-graph -T 'commit_id'
    git ls-remote <real-remote-url> refs/heads/main

Equal hashes are the proof. A push command that exited zero is not, and neither is the checkout's own view of its bookmark.

Resolve the real remote URL before reading it. When `git remote get-url origin` yields a filesystem path or a mirror host, that is not the forge; use the forge URL the repository publishes to, and say which URL you queried.

When the hashes differ, report the push as not landed and give both hashes.
````
