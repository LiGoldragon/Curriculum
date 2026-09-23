---
description: A Herdr-hosted Claude or Codex session is registered, exits, moves, or is tested as a message recipient.
dependencies: [testing, herdr]
---

Treat the session registry as a binding from a Flow ID to one native session, Herdr session, pane, terminal, harness, process, and lifecycle state. A SessionStart hook registers that binding atomically in the registry. `flow-id` binds the Flow ID to the native session in that same registry. The hook records only startup; `/clear` and resume are out of scope while seats are launcher-controlled. A changed session ID is refused by the send-time identity check and is never silently followed.

An optional SessionEnd hook may mark a binding as exiting early. The authoritative unregister signal is the Herdr event supervisor: `pane_exited`, `pane_closed`, a null agent, and `pane_moved` mark the binding exited or moved. Keep the Flow record and its history. Never delete it, retire the Flow, or infer a successor from a replacement pane.

The launcher or Field supervisor alone sets `Transition.{ predecessor successor-expected since deadline }` before a controlled replacement and clears it after the successor has a registered, live, identity-checked binding. A sender never creates a transition. During a missing, exited, or transition binding, `hm-send` waits only through its bounded hold window. It either sends once to the declared, verified successor or returns `Held.{ FLOW InTransition|NotRegistered attempt-<id> }`; the sender does not resend. A held attempt carries its predecessor header and is delivered once only to that declared successor.

Test this contract on a disposable seat. Cover registration at startup; normal exit; kill -9; pane close; pane move; terminal or process replacement; an unregistered Flow; and a declared successor appearing during and after the hold window. Observe that each refusal types no text, an exit retains rather than retires the Flow record, and no pending attempt reaches a guessed successor.
