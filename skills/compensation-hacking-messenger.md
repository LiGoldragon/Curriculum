---
description: A flow runs hm-send, hm-send-abrupt, hm-list, hm-register, hm-rebind, hm-move or hm-retire, reads what one of them printed, or changes Hacking Messenger (HM) itself.
dependencies: [messaging, herdr]
---

HM's source is the HackingMessenger repository under `Repository root`; the `hm-*` commands on `PATH` are links into its `bin/`. Whoever changes HM updates `skills/compensation-hacking-messenger.md` in Curriculum in the same landing.

## Commands

`FLOW_ID=<self> hm-send FLOW 'MESSAGE' [--wait-presented] [--hold-seconds N]` prompts the pane registered to FLOW with the message in a `Machine.Relay` envelope. The message is one shell argument: nonempty, at most 64 KiB, no control characters except newline and tab. A missing or in-transition registration is waited for up to N seconds (0 to 60, default 10).

`hm-send-abrupt` takes the same arguments and interrupts the turn first: one Escape for Codex; two Escapes for Claude, then Enter after the prompt. Text already in the input box is not cleared and joins the message.

`hm-list` prints `FLOW AGENT SESSION STATE` for every live Herdr agent (`-` for no registered Flow), then each registered Flow with no live agent as `STALE`.

`hm-register FLOW AGENT_NAME [--session S] [--native-thread T] [--readiness-probe MARKER --rollout TRANSCRIPT]` binds FLOW to the one live agent of that name. It refuses a Flow bound to another terminal, a retired native thread, and a held route.

`hm-rebind FLOW NEW_NAME --old-name --session --pane-id --terminal-id --agent --native-thread` renames an otherwise unchanged route.

`hm-move FLOW DEST_WORKSPACE --session --pane-id --terminal-id --name --agent --native-thread --process-pid` moves the pane to a new tab under a route hold and follows its new pane ID.

`hm-retire FLOW --session --pane-id --terminal-id --name --agent --native-thread --evidence RECEIPT --evidence-sha256 SHA` blocks every later send to FLOW and registration of that native thread.

`python3 hm.py deregister FLOW --session --pane-id --terminal-id --name` removes a registration as route repair; it does not retire the Flow. `hm.py import-retirement` takes `hm-retire`'s arguments after a witnessed deregistration.

## Receipts

`Transported.{ FLOW STATUS }` on stdout: Herdr accepted the prompt for the exact binding.
`Presented.{ FLOW STATUS }` with `--wait-presented`: Herdr also saw the pane react within five seconds.
Neither is a read receipt.
`Held.{ FLOW REASON attempt-ID }` on stderr, exit 1: nothing was typed. REASON is `NotRegistered`, `InTransition`, `RouteHold`, `PaneMissing`, `IdentityChanged`, `ProcessMismatch`, `NotReady`, `Blocked`, or `Uncertain` (agent status not idle, working or done). `Held` after `NotRegistered` or `InTransition` keeps the text in the registry's `pending/`.
`Held.{ FLOW Stalled ... }` with `--wait-presented`: Herdr reported the prompt stalled; the text may be in the pane.
`hm: Uncertain.{ FLOW attempt-ID } ...`: the prompt failed or the pane changed after it; the text may have arrived. Look at the pane before sending again.
Any other `hm: ...` line is a refusal before anything was typed.

## Registration on this Herdr

When `herdr agent get PANE` shows no `interactive_ready: true`, plain `hm-register` refuses with "Agent is not interactively ready". Register with the probe:

    FLOW_ID=FLOW hm-register FLOW AGENT_NAME --session messaging-build \
      --native-thread EXACT_SESSION_ID \
      --readiness-probe HM_READY_FLOW_YYYYMMDD_HHMM \
      --rollout TRANSCRIPT_JSONL

The probe prompts the pane to reply with the marker and waits about five seconds for that exact reply from the same native thread in TRANSCRIPT_JSONL: the Claude transcript `Claude transcript root/<project>/<session>.jsonl` or the Codex rollout under `Codex transcript root`. The marker matches `HM_READY_[A-Za-z0-9_-]{8,96}` and is new each time. A Herdr `agent_not_ready` refusal while the pane shows `launch_pending` clears by itself; wait and register again.

## When a send fails

A pane may be prompted directly when `hm-send` fails: `herdr --session SESSION agent prompt PANE 'TEXT'`, with SESSION and PANE read from `hm-list` and `herdr agent list`. Report Herdr's reply as the grade, not an HM receipt.
