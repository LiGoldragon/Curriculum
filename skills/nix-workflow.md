---
description: Nix package, module, flake-output, evaluation, build, or activation work occurs outside an operating-system change or flake-input revision change.
dependencies: []
---

Model services declaratively with typed options.
Maintain each Nix-owned component through the declarative source that owns it.
Use `flake.nix` as a readable entry point.
Keep substantial check and build implementations and long shell programs out of `flake.nix`.
Pin portable inputs in `flake.lock`.
Keep local input overrides transient.
Ask Nix or source, not the store filesystem.
Customize a package at the package-set boundary and test it through the exact final consumer package set.
Run Nix builds only through configured remote builders; never build locally.
Run Nix evaluations and builds independently.
Keep evaluation and activation evidence separate.
