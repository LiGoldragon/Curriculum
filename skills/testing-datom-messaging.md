---
description: A main flow is preparing a machine-origin message for another flow.
dependencies: [datom, messaging, testing-message-route]
---

Send a machine-origin message with `FLOW_ID=<self> hm-send <FLOW> "<body>"`. The relay header is the typed `Machine.Relay` machine envelope. Until the recipient declares a message type for the intended body, the body may be ordinary prose. Do not invent a variant or delay useful authorized work to discover a missing type.

When the recipient declares a type, write one complete Datom value in that type's required root variant and positional form. Validate it against the declared parser or type before sending, and report that semantic validation separately from the `hm-send` transport grade. Test syntax locally; reserve a disposable recipient for a changed transport mechanism.

This applies to machine-origin interflow work by main flows. A direct reply to the living remains ordinary prose under its response contract; do not recast the living's words as machine origin.

Choose the smallest explicit recipient set. Do not broadcast a route probe or status test. Do not send an acknowledgment, echo, or hash-heavy checkpoint solely to prove delivery. Keep full integrity digests, native-thread UUIDs, and full VCS revisions in a receipt or exact-value API field unless the recipient needs the exact value for authorized work.
