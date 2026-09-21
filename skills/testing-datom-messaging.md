---
description: A main flow is preparing a machine-origin message for another flow.
dependencies: [datom, messaging, testing-message-route]
---

Write each machine-origin interflow message as one complete Datom value against a declared recipient message type. Give it a variant at the root, in the form that type requires, and every positional value; do not substitute ordinary prose, JSON field labels, or an invented head for a missing type. If the receiver has no declared type for the intended message, establish that contract before treating a value as a typed message.

For the current `tools/msg` bridge, the argument after the recipient is the Datom value for the message body. `tools/msg` wraps it in the existing `Machine.Relay.{ ingress from seat heard mode [recipients] quote context }` transport envelope. The messenger codec checks that the quoted body parses as a Datom variant. That structural check does not prove the recipient's semantic type accepts the variant. Validate against the recipient's parser or declared type when one exists, and report that validation separately from transport and target-side receipt grades.

This rule applies to machine-origin interflow work by main flows. A direct reply to the living remains ordinary prose under its own response contract; do not recast the living's words as a `MACHINE` origin. Before sending over an unproven route, apply `testing-message-route` with an exact live binding. A successful `tools/msg` exit is submission evidence, not a target read.

Keep full integrity digests, native thread UUIDs, and full VCS revisions in machine receipts or exact-value API fields. In conversational prompts, checkpoint updates, and acknowledgments, omit them unless the recipient needs the exact value for the authorized operation. When identity matters but the full value does not, use the shortest unambiguous Flow ID or revision prefix and point to the receipt or report. Preserve raw evidence and the living's original words; this is a sender-output rule, not history rewriting. The present `tools/msg` transport accepts opaque typed payloads and cannot enforce this distinction generically.

Choose the smallest explicit recipient set. Never broadcast a route probe or status test. Test syntax with a local codec and mechanism with a disposable target. For an existing flow, prefer passive route and status observations; contact it for useful authorized work, then use its real work reply as the read witness. Do not send an extra acknowledgment, echo, or hash-heavy checkpoint solely to prove delivery.
