# stale-lock: created after the baseline

Created in b95f832, 2026-09-25 09:25 -0600: Add stale-lock skill. Made by: Claude Fable 5.1 <noreply@anthropic.com>.
Later edits: dd14ce7 2026-09-25 “Land stale-lock text as the living approved it”.

## View

Asked for by the living (flows/e51411/vision/locks.md: "We need to develop a skill to allow someone to unlock a stale flow lock"), and the text of dd14ce7 approved: "Yeah the edit is good. Put it in." The quoted "break the locks. Nobody in particular owns Flow Source" is his, 2026-09-24.
View: candidate, whole. The receipt step ("record the lock, its paths and the holder's state in a receipt") is the one part to ask about.

## The whole skill as it stands on main (3726da5)

````markdown
---
description: An Orchestrate lock is held by a flow that no longer answers.
dependencies: [orchestrate, messaging]
---

A lock whose holder is stale may be released by any flow. The holder is stale when hm-list shows its binding STALE or exited, Herdr has no live pane for it, and one hm-send to it comes back Held. Before release: Observe.Locks, and record the lock, its paths and the holder's state in a receipt; commit any uncommitted work under its paths as found, naming the holder; then release by ID and take your own lock. A live holder is messaged, never unlocked. The living: "break the locks. Nobody in particular owns Flow Source."
````
