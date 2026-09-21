---
description: A flow is using an installed typed FlowMessage admission and delivery path.
dependencies: [datom, messaging, refresh]
---

Use this path only after the selected Flow and Message components, generated signal contracts, and the exact target binding have an installed, witnessed API. Resolve one explicit recipient; a vector-shaped transport field does not authorize broadcast.

Acquire is the admission linearization point. An accepted acquire commits one permit against the current exact Flow binding. A permit committed before BeginRefresh may finish its admitted delivery attempt; BeginRefresh closes new admission immediately, including when that earlier permit remains active. There is no second consume or admission boundary at transport.

The sender keeps the permit through delivery and waits for the matching typed CONFIRMED evidence. ReleaseConfirmed must compare the same current Flow identity, binding, nonce, and generation; it clears that permit only, not RefreshHeld. Transport acceptance, terminal presentation, and a sender timeout are not CONFIRMED or Read. An ambiguous transport outcome, deadline, or cancellation retains the durable permit and requires reconciliation; none may call a Flow unlock shortcut.

Ready or reattach while a permit is ambiguous must refuse. After reconciliation, a successor-ready compare-and-swap must witness the exact current binding and generation before queued delivery proceeds. A restart, changed pane, or display name alone does not substitute for this binding proof. Report submitted, transported, presented, read, and completed at their separate observed grades.

Use the component's final typed request and response variants, not an improvised Datom or string wrapper. The exact callable names, release response, reconciliation operation, and installed version must be read from the current generated signal contract and API before making a command. If that contract or deployment receipt is unavailable, hold the operation and report the missing interface rather than guessing.
