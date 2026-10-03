---
description: Read the version-pinned Codex app-server control surface without turning protocol shapes into runtime claims.
---

Record the installed Codex version and its paired source revision before relying on its protocol.

Initialize the app-server connection before requests.

Use `hooks/list` with optional `cwds` to read its `data` entries for configured hooks. It does not load skills.

The hook events are `preToolUse`, `permissionRequest`, `postToolUse`, `preCompact`, `postCompact`, `sessionStart`, `sessionEnd`, `userPromptSubmit`, `subagentStart`, `subagentStop`, `stop`, and `interrupt`.

`hook/started` and `hook/completed` carry `run`, `threadId`, and nullable `turnId`.

Treat command-hook JSON on stdin only as tagged-source evidence. Do not infer command input, inherited environment, or per-thread hook installation from schema acceptance.

Answer `item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, and `item/permissions/requestApproval` only through their matching typed replies.

Command decisions may accept, accept for a session, apply an offered execpolicy or network amendment, decline, or cancel.

File decisions have no amendment variant.

Permissions replies carry structured `permissions`, `scope` defaulting to `turn`, and nullable `strictAutoReview`.

Approval replies never grant Flow authority or override host policy.

Thread start and resume use a thread sandbox enum. Turn start uses structured `sandboxPolicy`.

Preserve selected policies and require a runtime witness before claiming config application or sandbox enforcement.

Keep account allowance reads and sparse updates distinct from token usage and approvals.

Start an allowance collector from a read snapshot, merge sparse updates, and read a new snapshot after reconnect because this protocol declares no cursor, ordering, or replay guarantee.

Keep Flow identity allocation out of this knowledge. Current allocation choices and UUIDv7 collision behavior remain design questions for the living.
