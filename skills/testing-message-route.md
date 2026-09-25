---
description: A flow sends an authorized message over an existing Herdr route, or changes that route.
dependencies: [messaging, behavior]
---

For one authorized message, make one `FLOW_ID=<self> hm-send <FLOW> "<text>"` call with the body only. Its in-call registry resolution and checks of the live pane, terminal, harness process, identity, and readiness are the route proof for that exact send. Report the receipt grade it prints, never a higher grade.

`Transported` means Herdr accepted the prompt for the exact checked binding. `Presented` is available only when `--wait-presented` reports a lifecycle change; include the pre-send status because an already-working target can make that observation ambiguous. A useful work reply is the read or completion witness. A zero exit alone is not a read witness.

`Held.{ FLOW <reason> }` means no text was typed to a target. Do not retry an `Uncertain` result, reroute, or fall back to another channel. The held attempt and its declared successor rule own later delivery.

Use a disposable recipient only while changing the transport, registry, identity check, or hold implementation. Test the changed mechanism with a unique marker and target-side observation, then remove the recipient. Do not probe a psyche, Field seat, or production flow. A routine send needs no preflight probe or second resolution.
