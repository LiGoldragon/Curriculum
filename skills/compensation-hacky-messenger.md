---
description: A flow runs an hm-* command, reads its output, or changes Hacky Messenger.
dependencies: [messaging, herdr]
---

HM's live source is the `clojure` branch of the HackyMessenger repository under `Repository root`. The nine unprefixed commands on `PATH` target the typed Clojure implementation: `hm-send`, `hm-send-abrupt`, `hm-list`, `hm-register`, `hm-deregister`, `hm-rebind`, `hm-move`, `hm-retire`, and `hm-heartbeat-state`. Whoever changes HM updates this authored source in the same landing.

`FLOW_ID=<self> hm-send FLOW 'MESSAGE' [--wait-presented] [--hold-seconds N] [--pane PANE]` sends one body. HM alone constructs the one-line pane envelope `#msg ["FLOW_ID" "text"]`; its only fields are sender and text. Pass the body, never a prebuilt envelope. A body that parses as one complete `#msg` form is rejected; prose may mention `#msg`.

Time, recipient, harness, route, attempts, pending messages, and retirements stay in the typed Datalevin ledger. The state root is `HM_REGISTRY` or the installed default. Long bodies are stored under the sender's flow directory and the pane receives a short file pointer inside the same envelope.

`hm-send-abrupt` takes the send arguments and interrupts the active turn first. `hm-list` prints live and stale registrations. `hm-heartbeat-state` takes no body or arguments and prints a read-only view of typed routes and retirements; it sends and mutates nothing. `hm-register`, `hm-deregister`, `hm-rebind`, `hm-move`, and `hm-retire` mutate one exact typed route or retirement; follow their required identity arguments and refusal output. The `hm-clj-` maintenance commands are not ordinary send commands.

Report the printed receipt without upgrading it. `Transported` is Herdr acceptance for the checked binding. `Presented` includes the requested target reaction observation. `Fallback-Presented` is an observed send to a pane resolved by `--pane`, stored name, or flow title when the registered route is not exact. Neither proves a read. `Held` means HM typed nothing, the body is pending, and its printed reason names the refusal. `Uncertain` means the text may have arrived, so inspect the target before any retry.

Do not bypass an HM refusal with a direct Herdr prompt. Repair the route or return the refusal to the owning flow.
