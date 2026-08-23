---
description: An external manager for coding harnesses must be selected, packaged, installed, configured, or integrated.
dependencies: [nix-workflow]
---

Treat an external harness manager as distinct from the Claude or Codex harnesses it coordinates.

Obtain current release, packaging, installation, and integration facts from authoritative upstream sources before choosing or changing an integration.

Put durable packages and configuration in the declarative source that owns that environment.

Put a distinct reusable package in its own public package repository; a home-environment source consumes a pinned package output.

Give an agent manager a package and executable name that cannot collide with an unrelated existing package; StablyAI Orca is `orca-ide`, not GNOME `orca`.

Do not run an upstream integration installer that mutates a configuration Nix owns; express the intended configuration in its declarative owner.

Evaluation is not package proof: build the artifact and behavior-smoke every claimed CLI, GUI, and headless surface.
