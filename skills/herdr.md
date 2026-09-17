---
description: A flow references Herder / Herdr, invokes `herdr`, or reasons about terminal workspace management for AI agents on this cluster.
dependencies: []
glance-approved-by: 108ab0
---

Herdr is a terminal workspace manager for AI coding agents,
installed as `herdr` (five letters — H-E-R-D-R, not "herder") at
~/.nix-profile/bin/herdr. Version at the time of this skill:
0.8.2. It runs as a persistent server; a client attaches to it
by session name. When the psyche says "Herder," they mean this
tool.

Herdr's own top-level verbs, by shape:
- session / workspace / worktree / tab / pane — pane and workspace
  management primitives.
- agent — dedicated subcommands for AI agents running inside a
  pane.
- api — programmatic API, probably reached over a Unix socket.
- notification — inbound message routing surface.
- integration — harness-specific hooks (Claude Code, Codex, others).
- server / channel / update / --handoff — running-server model
  with a stable/preview channel and self-update.
- --remote <ssh-target> — remote sessions supported, not local only.

To learn what a subcommand does before using it, always run
`herdr <sub> --help` — never guess by name.

Herdr is the substrate the message CLI and the flow CLI will ride
on. Do not reinvent multiplexer plumbing. The tier-priority
messaging system (hard abrupt / middle / soft) delivers its bytes
through herdr's existing pane + notification + agent APIs.

Common failure mode: searching for the spelling "herder" and
concluding the tool does not exist. It does. The tool is `herdr`.
