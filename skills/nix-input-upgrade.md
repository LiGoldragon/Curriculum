---
description: Nix flake inputs are being upgraded across a layered repo structure.
dependencies: [nix-workflow]
---

Map the dependency chain before touching anything. Update bottom-up: eval-cache boundary repos (like CriomOS-pkgs) must be committed and pushed before their consumers update their locks.

Before advancing an external package input, verify its final provider, immutable upstream payload and hash, embedded runtimes, contract-pinned consumers, built launchers, and resident versions.

A version number does not prove a specific fix is included. Verify the fix against upstream commit history, not the release tag. GTK 4.22.4 (nixos-unstable, Aug 2026) did not contain MR !10130 despite being the latest stable; the fix appeared only in 4.23.3. Use fetchpatch when the fix is merged upstream but not yet released.

A nixpkgs fork with no local commits can be fast-forwarded via the GitHub API without cloning. Check local commits before pulling the full repo.

The same package may be sourced from multiple inputs at different versions. Identify which input actually provides each package in the final closure before deciding which input to update. niri-flake pinned v25.08 while nixpkgs already carried v26.04.

Some repos enforce specific input revisions in CI (main-contract-pins checks). Identify which inputs are pinned-by-contract before updating them.

When auditing local patches across a version jump, classify each patch:
- Historical: already merged upstream before the old pin — no action.
- Reconciliation: applied once to align a fork — no action.
- Live: not yet upstream, or CriomOS-specific — must rebase and verify applicability.

Patch interface changes silently across minor versions. A patch that applied cleanly at v0.80 may require rebase at v0.84 because the surrounding function signature changed (Pi v0.83 grew a new argument).

npm-fetching packages require a failed build attempt to obtain the correct hash. Supply a placeholder, let the build fail, copy the hash from the error.

Packages are removed from nixpkgs without notice. Each missing package needs an individual replacement decision; there is no bulk fallback.

Breaking API changes in transitive dependencies (e.g. home-manager renaming `programs.vscode` to `programs.vscodium`) surface only at eval time in consumer repos. Eval each consumer after updating its inputs.
