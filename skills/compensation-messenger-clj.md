---
description: A flow runs an hm-* command, reads its output, or changes messenger-clj.
dependencies: []
---

The `hm-*` shorthands are provided by the standalone messenger-clj repository under `Repository root`, on its default branch.

Name commits, flows and artifacts by at most six characters.

`FLOW_ID=<self> hm-send TARGET BODY` sends one complete machine body as `#msg [sender machine-prose]`. Use `--stdin` in place of `BODY` for a large or multiline body. `FLOW_ID=<self> hm-send TARGET --psyche CONTEXT VERBATIM` sends the living's words, context first, as one `#psyche [sender context whole-verbatim]`; `--psyche CONTEXT --stdin` reads the verbatim from standard input. `FLOW_ID=<self> hm-send TARGET --psyches --stdin` reads one EDN vector of `[context verbatim]` pairs and sends them as one `#psyches` envelope. Pass message fields, never a prebuilt envelope. `hm-send-abrupt` takes the same arguments and interrupts the target's active turn first.

TARGET is the recipient's six-character flow id.

Write the recipient-facing body only.

The typed reply is what hm-send itself prints; a flow never waits on or watches the target for an answer.

Report the printed receipt as it is. `Transported` is Herdr's acceptance for the checked binding. `Presented` includes the observed reaction of the target. Neither proves a read. `Held` means nothing was typed and the whole body is pending; its printed reason names the refusal. `RepairRequired` means the recorded candidates must be judged and the route repaired with `hm-repair`. `Uncertain` means the envelope may have arrived: inspect the target and never retry blindly.

A refused send is reported and its route is mended.

Registration validates one exact live Herdr identity and stores its native binding immediately; send readiness is checked separately. A Held send remains pending until an explicit supported delivery action.
