---
description: The shape of a `<repo>-test` repository or its Nix code is being discussed with the living psyche and the reasoning behind it is needed.
dependencies: [compensation-nix]
---

Tests change often; the code they test changes less. In the tested repository's flake, every added or edited test changes that flake's source, so Nix rebuilds and re-checks the Rust build on a push that touched no Rust. In a separate `<repo>-test` repository the Rust build is rebuilt only when its own source changes, and the test repository pins the code repository as an input it updates when it chooses. This is nix-workflow's split of mutable content from Rust source, applied to tests. The living asked for it on 2026-09-26 (`flows/e167d8/vision/testRepos.md`), with the name rule: add the test suffix, create a new repository.

Scenarios are named by their components because the living described them that way: a test that includes Message and Flow is a Message and Flow test, and another scenario adds a third component or swaps one. Components in `lib/components/` make that addition or swap a one-line change in the scenario.

Blueprint is the house layout: CriomOS and CriomOS-home use it, and its directory tree is the flake's output tree, so a flow finds an output by its path and adds one by adding a file. Every per-system file receives `pkgs`, `inputs`, `flake`, and `system`, so no file wires arguments by hand. Blueprint turns each package into a `pkgs-<name>` check and has no `apps` folder; `nix run` falls back to packages. flake-parts is the other widely used structure; plain `forAllSystems` flakes grow into one long `flake.nix`, as CriomOS-test-cluster's 586 lines show.

Semi-sandboxes are runners, not checks. A check must be reproducible and runs without network or credentials. Impure derivations are an experimental feature, and a relaxed sandbox with `__noChroot` weakens the builder for every build. Anything a derivation reads lands in the world-readable store. A runner reads the living's logins only at run time, into a root it deletes, and the pure checks stay pure.

nixfmt is the official formatter since RFC 166; in current nixpkgs `pkgs.nixfmt` is the RFC-style implementation and `nixfmt-rfc-style` an alias. deadnix finds unused bindings and arguments; statix finds anti-patterns. The gate was seen failing on unformatted code and on unused lambda arguments, then passing, built on the remote builder.

Evaluating with import-from-derivation disallowed and `--no-build` catches most mistakes in seconds without a builder; a flow then spends builds only on code that evaluates.

Sources: numtide.github.io/blueprint (folder structure); flake.parts; github.com/NixOS/rfcs/pull/166; github.com/NixOS/nixfmt; nixpkgs manual, testers (`runNixOSTest`); nix.dev manual, import from derivation and `nix flake check`; zombiezen.com/blog/2023/11/impure-nix-derivations; github.com/NixOS/nix/issues/8258 (`__noChroot`); discourse.nixos.org/t/help-with-flakes-that-require-authentication-credentials/18183; nixos.asia/en/howto/local-flake-input.
