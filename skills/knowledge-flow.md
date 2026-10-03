---
description: What exists today for launching, naming and reaching flows is read, before Flow the Nexus exists.
dependencies: [vision-flow]
---

A hand-started seat claims its identity itself: `flow-id claude --flows-root <abs> --parent-session "$CLAUDE_CODE_SESSION_ID"` (Codex: `flow-id codex --flows-root <abs>`) prints a six-character hex alias, the `FLOW_ID`, and claims `flows/<FLOW_ID>/` in Primary as the `FLOW_DIRECTORY`; the main flow adds one line to `flows/index.md` as `<kind>, <id>, <title>, <description>.`.

A main flow's pane title today is `<Aspect>.{ <Model> <FLOW_ID> }`, for example `Mind.{ Astra 6f51ad }`; voices are not yet named `Aspect.Rank` anywhere running.

Flows reach each other only through messenger-clj: `FLOW_ID=<self> hm-send <target-flow-id> BODY`, the target always a flow id, never a registered name; a message addressed to a registered name is held with RepairRequired and its route cannot be repaired. Registration validates one live Herdr pane.

Subflows run as harness subagents inside the main flow's session; there is no independent subflow process. No installed harness hook calls any Flow component; a hook can only route into a session's first turn (SessionStart) or run a shell command on an event. Not deployed: flow main 0.22.0 reserves a Claude launch's FlowId before Spawn, exports `FLOW_ID` to the pane and chooses its `--session-id`, and puts `flow-hook` in the launch's `--settings` on SessionStart, PostToolUse and Stop, each a `Report` appended to the flow's events in Flow's Memory.

The installed `claude` wrapper prepends `--dangerously-skip-permissions` to every call, so every seat and sandbox flow runs in bypass; only the unwrapped binary honours a requested mode.

The semi-sandbox is flow-test's gated runner `flow-claude-hook` (`FLOW_TEST_LIVE=1 nix run .#flow-claude-hook`): a short `mktemp -d /tmp` root (socket paths stay under 108 bytes) holds HOME, every XDG root and TMPDIR, only `~/.claude/.credentials.json` is copied in, its own Flow Nexus starts on a fresh store beside a fixture Herdr, and a cheapest-model `claude -p` runs in a 2G user scope for at most 300 s and 4 turns.
