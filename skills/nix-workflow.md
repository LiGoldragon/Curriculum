---
description: The change lands in Nix.
dependencies: []
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
Test with binaries and scripts packaged by Nix.
A Rust runtime has its own Rust-only repository, holding only what compiling its executable or library needs; mutable data and content live in separate data inputs or repositories, so data changes do not invalidate Rust compilation. Do not modify what is not Rust in a Rust executable repository.
Treat managed output as evidence, not a patch target.
Keep evaluation and activation evidence separate.
