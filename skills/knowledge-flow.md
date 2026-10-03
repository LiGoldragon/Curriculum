---
description: What exists today for launching, naming and reaching flows is read, before Flow the Nexus exists.
dependencies: [vision-flow]
---

There is no Flow Nexus yet. A flow claims its identity itself: `flow-id claude --flows-root <abs> --parent-session "$CLAUDE_CODE_SESSION_ID"` (Codex: `flow-id codex --flows-root <abs>`) prints a six-character hex alias, the `FLOW_ID`, and claims `flows/<FLOW_ID>/` in Primary as the `FLOW_DIRECTORY`; the main flow adds one line to `flows/index.md` as `<kind>, <id>, <title>, <description>.`.

A main flow's pane title today is `<Aspect>.{ <Model> <FLOW_ID> }`, for example `Mind.{ Astra 6f51ad }`; voices are not yet named `Aspect.Rank` anywhere running.

Flows reach each other only through messenger-clj: `FLOW_ID=<self> hm-send <target-flow-id> BODY`, the target always a flow id, never a registered name; a message addressed to a registered name is held with RepairRequired and its route cannot be repaired. Registration validates one live Herdr pane.

Subflows run as harness subagents inside the main flow's session; there is no independent subflow process. No harness hook calls any Flow component; a hook today can only route into a session's first turn (SessionStart) or run a shell command on an event.

The only running Nexus is orchestrate, the lock Nexus. The installed `claude` wrapper prepends `--dangerously-skip-permissions` to every call, so every seat and sandbox flow runs in bypass; only the unwrapped binary honours a requested mode.

The semi-sandbox exists as a script (flows/3ec648/witnesses/semi-sandbox-capsule.sh in Primary): it copies `~/.claude/.credentials.json` alone, starts an orchestrate Nexus as a transient user unit under its own `XDG_RUNTIME_DIR` and `XDG_STATE_HOME`, and runs a haiku `claude -p` inside; socket paths must stay under 108 bytes, so it uses a short `/tmp` symlink; a store records its socket paths, so a sandbox starts from a fresh store.
