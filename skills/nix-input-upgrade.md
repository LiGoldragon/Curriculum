---
description: A Nix flake input or lock is being upgraded.
dependencies: [nix-workflow]
---

Map each dependency to its final provider before updating.
Push a producer before its consumer lock changes.
Verify a required fix by the commit that contains it.
Update only the named input and preserve contract pins.
Classify patches as historical, reconciliation, or live, then recheck their interfaces.
Prune retired inputs.
Evaluate every consumer.
