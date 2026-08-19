---
description: A Nix flake input revision must change or be removed.
dependencies: [nix-workflow]
---

Map each dependency to its final provider before updating.
Push a producer before its consumer lock changes.
Verify a required fix by the commit that contains it.
For a named input update, change or remove each naming-contract pin in the same update round.
For every local patch, remove it when upstream already contains it, retain it when it only reconciles a fork, or rebase and verify it when it remains unmerged or repository-specific.
Remove an input only after confirming that it has no consumers and no naming-contract pins. Then prune only unreachable nodes from the lock graph.
Evaluate every consumer.
