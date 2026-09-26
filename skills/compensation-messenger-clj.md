---
description: A flow runs an hm-* command, reads its output, or changes messenger-clj.
dependencies: [messaging, herdr]
---

The `hm-*` shorthands are provided by the standalone messenger-clj repository under `Repository root`, on its default branch. The nine unprefixed commands on `PATH` target the typed Clojure implementation: `hm-send`, `hm-send-abrupt`, `hm-list`, `hm-register`, `hm-deregister`, `hm-rebind`, `hm-move`, `hm-retire`, and `hm-heartbeat-state`. Whoever changes messenger-clj updates `skills/compensation-messenger-clj.md` in the same landing.

`FLOW_ID=<self> hm-send TARGET BODY [--wait-presented] [--hold-seconds N] [--pane PANE]` sends one complete, unlimited machine body as `#msg [sender machine-prose]`. `FLOW_ID=<self> hm-send TARGET --psyche CONTEXT VERBATIM [--wait-presented] [--hold-seconds N] [--pane PANE]` sends one complete, unlimited `#psyche [sender context whole-verbatim]` envelope. The context and verbatim are separate fields, and the verbatim is preserved exactly, including whitespace, newlines, and Unicode. There is no 800-character cap, part-number field, splitting, overflow file, or pointer. A long or multiline envelope may appear in a Claude pane inside Claude's `pasted_content` wrapper; that accepted presentation does not change the envelope or authorize a retry. Pass a body or the context and verbatim, never a prebuilt envelope. A body that parses as one complete `#msg` or `#psyche` form is rejected as nesting; ordinary prose may mention either tag.

Time, recipient, harness, route, attempts, pending messages, and retirements stay in the typed Datalevin ledger. The state root is `HM_REGISTRY` or the installed default. Each record retains the variant, its fields, and the exact submitted envelope.

`hm-send-abrupt` takes the send arguments and interrupts the active turn first. `hm-list` prints live and stale registrations. `hm-heartbeat-state` takes no body or arguments and prints a read-only view of typed routes and retirements; it sends and mutates nothing. `hm-register`, `hm-deregister`, `hm-rebind`, `hm-move`, and `hm-retire` mutate one exact typed route or retirement; follow their required identity arguments and refusal output. The `hm-clj-` maintenance commands are not ordinary send commands.

Report the printed receipt without upgrading it. `Transported` is Herdr acceptance for the checked binding. `Presented` includes the requested target reaction observation. `Fallback-Presented` is an observed send to a pane resolved by `--pane`, stored name, or flow title when the registered route is not exact. Neither proves a read. `Held` means HM typed nothing, the whole body is pending, and its printed reason names the refusal. `Uncertain` means the one whole envelope may have arrived: inspect the target and never retry blindly.

When HM refuses, a flow may prompt the target pane directly through Herdr, and names the refusal in that message.
