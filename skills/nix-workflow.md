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
Test with binaries and scripts packaged by Nix, built through configured remote builders; do not silently fall back to raw local compilation, Cargo tests, or local package-manager rebuilds. A Nix command alone does not prove offload: retain the remote-builder evidence for a claimed remote build.
Create Rust-only repositories for Rust source and keep mutable data or content in separate data inputs or repositories, so data changes do not invalidate Rust compilation.
Treat managed output as evidence, not a patch target.
Keep evaluation and activation evidence separate.
