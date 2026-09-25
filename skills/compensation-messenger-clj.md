---
description: A flow runs an hm-* command, reads its output, or changes messenger-clj.
dependencies: [messaging, herdr]
---

The `hm-*` shorthands are provided by the standalone messenger-clj repository under `Repository root`, on its default branch. The nine unprefixed commands on `PATH` target the typed Clojure implementation: `hm-send`, `hm-send-abrupt`, `hm-list`, `hm-register`, `hm-deregister`, `hm-rebind`, `hm-move`, `hm-retire`, and `hm-heartbeat-state`. Whoever changes messenger-clj updates `skills/compensation-messenger-clj.md` in the same landing.

`FLOW_ID=<self> hm-send TARGET BODY [--wait-presented] [--hold-seconds N] [--pane PANE]` sends one complete, unlimited machine body as `#msg [sender machine-prose]`. `FLOW_ID=<self> hm-send TARGET --psyche CONTEXT VERBATIM [--wait-presented] [--hold-seconds N] [--pane PANE]` delivers the living words as sequential, numbered full `#psyche` envelopes of at most 800 characters each. Messenger splits the exact verbatim on word or whitespace boundaries without changing it; concatenating the part fields reproduces the verbatim, including whitespace, newlines, and Unicode. Context goes before the verbatim in the first envelope where it fits; later envelopes have no context. A context or word that cannot fit is held durably before any prompt. Pass a body or the context and verbatim, never a prebuilt envelope. A body that parses as one complete `#msg` or `#psyche` form is rejected as nesting; ordinary prose may mention either tag.

Time, recipient, harness, route, attempts, pending messages, and retirements stay in the typed Datalevin ledger. The state root is `HM_REGISTRY` or the installed default. Each record retains the variant, its fields, and the exact submitted envelope.

`hm-send-abrupt` takes the send arguments and interrupts the active turn first. `hm-list` prints live and stale registrations. `hm-heartbeat-state` takes no body or arguments and prints a read-only view of typed routes and retirements; it sends and mutates nothing. `hm-register`, `hm-deregister`, `hm-rebind`, `hm-move`, and `hm-retire` mutate one exact typed route or retirement; follow their required identity arguments and refusal output. The `hm-clj-` maintenance commands are not ordinary send commands.

Report the printed receipt without upgrading it. `Transported` is Herdr acceptance for the checked binding. `Presented` includes the requested target reaction observation. `Fallback-Presented` is an observed send to a pane resolved by `--pane`, stored name, or flow title when the registered route is not exact. Neither proves a read. `Held` means HM typed nothing, the body is pending, and its printed reason names the refusal. `Uncertain` means the text may have arrived: inspect the target and never retry blindly. For a multi-part psyche delivery, it stops before later parts when any prompt is uncertain.

When HM refuses, a flow may prompt the target pane directly through Herdr, and names the refusal in that message.
