---
description: A flow is about to write a file for its own work.
dependencies: []
---

## Scratch goes to a temporary directory

Build output, a log, a diff, a probe, every file a flow makes for its own work, goes to a temporary directory: the harness scratchpad, or one made with mktemp -d. The primary workspace and the flow directory hold only what is landed or what the living reads, a proposal or a handover.

```sh
# read back by tail or grep
nix build 2>&1 > "$SCRATCH/build.log"
```
