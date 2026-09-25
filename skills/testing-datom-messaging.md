---
description: A main flow is preparing a machine-origin message for another flow.
dependencies: [datom, messaging, testing-message-route]
---

Datom is sufficient for a machine-origin message. Write one complete Datom value in the recipient's declared root variant and positional form, and pass that value itself as the HM body. Do not construct a provenance header or a fake variant or field merely for transport.

Validate the value against the recipient's declared parser or type before sending, and report that semantic validation separately from the transport grade. When no recipient type exists, establish the type before claiming semantic acceptance; do not replace the missing type with ordinary prose or invented structure. Test syntax locally; reserve a disposable recipient for a changed transport mechanism.

This applies to machine-origin interflow work by main flows. A direct reply to the living remains ordinary prose under its response contract; do not recast the living's words as machine origin.

Choose the smallest explicit recipient set. Do not broadcast a route probe or status test. Do not send an acknowledgment, echo, or hash-heavy checkpoint solely to prove delivery. Keep full integrity digests, native-thread UUIDs, and full VCS revisions in a receipt or exact-value API field unless the recipient needs the exact value for authorized work.
