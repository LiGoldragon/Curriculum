---
description: The change lands in Nix.
dependencies: []
---

Keep declarative ownership setup-independent.
Treat `flake.nix` as an index of named implementations.
Test central package-set customization against the exact final package set.
Pin portable inputs in the lock file and keep local overrides transient.
Ask Nix or source, not the store filesystem.
Build only through configured remote builders.
Keep evaluation, build, deployment-admission, activation, current-profile, and runtime witnesses separate.
Keep managed-output provenance only when an observed failure needs it.
