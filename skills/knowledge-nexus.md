---
description: What the nexuses are today — which run, on which sockets and versions, from which branches — is read, before judging or changing one.
dependencies: [vision-nexus]
---

The retained Home 83 runtime packet records Home Manager generation 1045 as active. Generation 1039 remains an executable rollback.

The running server engines are Orchestrate Nexus 0.37, Flow Nexus 0.23, Message Nexus 0.19, and Lojix Nexus 8.1. Their stable socket pairs are:

- Orchestrate: `/run/user/1001/orchestrate-nexus/orchestrate.sock` and `/run/user/1001/orchestrate-nexus/orchestrate-meta.sock`.
- Flow: `/run/user/1001/flow/flow.sock` and `/run/user/1001/flow/flow-meta.sock`.
- Message: `/run/user/1001/message/message.sock` and `/run/user/1001/message/message-owner.sock`.
- Lojix: `/run/lojix/ordinary.sock` and `/run/lojix/meta.sock`.

The regular `PATH` resolves Flow 0.23 and Message 0.19 clients from the same package closures as their running servers. This packet does not establish store-engine versions, a wire contract, active seats, or a successful launch. Read the current source and the live target's explicit response before making any of those claims or changing a Nexus.
