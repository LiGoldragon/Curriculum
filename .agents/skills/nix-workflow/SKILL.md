---
name: nix-workflow
description: 'The change lands in Nix.'
---

Model services declaratively with typed options.
Pin portable inputs in the lock file.
Build and deploy reproducible source.
Keep `flake.nix` readable as an index.
Keep substantial check and build implementations and long shell programs out of `flake.nix`.
Ask Nix or source, not the store filesystem.
Any part of an environment already owned by Nix, CriomOS, or CriomOS-home is fixed, updated, and maintained through that owning declarative source.
Keep local overrides transient.
Run Nix builds only through configured remote builders; never build locally.
Run Nix evaluations and builds independently.
Treat managed output as evidence, not a patch target.
Keep evaluation and activation evidence separate.
