---
description: A message between flows is designed, sent or judged: its datom form, priority, per-harness delivery, or what counts as its witness.
dependencies: [datom]
---

## A message is a datom, and it arrives as one

The message body is a datom that lands in the recipient's prompt as a
datom-formatted object. There is no envelope around it.

## Priority is a head on the datom

    Priority.[HardAbrupt MiddleAbrupt Soft]

## Delivery is harness-specific, and the mechanism differs per tier

Hard abrupt on Codex is one Escape, and the prompt submits itself. Hard
abrupt on Claude is two Escapes — the first is taken by the input editor in
vim mode and never reaches the harness — then the prompt, then an explicit
Enter, because after an interrupt the prompt is placed in the composer
without being submitted. Middle abrupt is the terminal prompt, which a
Claude recipient receives at its next tool boundary. Soft waits for the
recipient to finish.

## An interrupt witness is not a delivery witness

Inducing an interrupt, placing prompt text, submitting it, and the recipient
consuming it are four separate observations. None of them stands for
another, and a submission is never a read receipt.

## Only messages that act, deliver, or block

Send only messages that require the recipient's action, deliver a result it awaits, or report an error or blocker affecting its work; keep routine receipts in durable records for requested status reports.

## Sources

108ab0 operational-messageAsDatomInPrompt
108ab0 operational-messagePriorityTiers
108ab0 operational-abruptPerHarness
108ab0 operational-herderMuxKeypress
1ac573 operational-modelRoles
d66c26 messaging
