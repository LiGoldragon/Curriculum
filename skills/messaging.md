---
description: A flow must send, receive, route, or verify an operational message.
dependencies: [behavior, herdr, testing, vocabulary]
---

Name the layer before claiming delivery.

Herdr 0.8.2 is the live terminal-workspace transport. It can inject into a running terminal through its own witnessed APIs; it is not durable message storage or identity resolution.

Hacky Messenger is the live compatibility bridge: it resolves a running target and directly prompts it through Herdr. A successful submission is not a read receipt. It is a bridge, not Flow Nexus or Message Nexus.

Flow Nexus 0.3 is the identity and resolution design: it binds the exact logical flow identity to the exact live target. It does not itself prove transport or durable delivery.

Message Nexus 0.12 is installed for durable attempts and receipts. State the observed operation and receipt, not an assumed semantic outcome. The published-not-deployed 0.13 receipt query is not live. The central-messenger design is vision, not a current service.

Receipt grades are distinct. Submitted means the sender accepted the request. Transported means the selected transport accepted the bytes for the exact binding. Presented means the target terminal or harness received the prompt. Read means an observed target-side read acknowledgment. Completed means the requested work returned its stated completion evidence. Never upgrade one grade into another.

For an authorized native-flow message on an already-known route, `hm-send` is the direct primitive: its normal call performs the current binding, identity, pane, terminal, harness, and readiness checks and returns its receipt. Harness messaging is for an already-spawned internal collaboration worker; it does not establish or substitute for a native HM route. A submitted native message remains only submitted until its printed receipt proves a higher grade. `Held` means no text was typed and owns its declared successor rule; do not retry or reroute it.

The message content is sufficient on its own. Transport envelopes, terminal paste delimiters, attempt ledgers, and renderer wrappers are transport behavior, not message semantics. Do not add provenance XML, a duplicate recipient list, or boilerplate to make a message routable. Preserve the submitted bytes in the receipt. Do not strip, unwrap, split, or resend arbitrary user XML; a rendered wrapper changes only after its actual emitter and supported setting are established.

Resolve the recipient immediately before submission and bind the attempt to that exact identity and live target. Record the binding with the attempt. A terminal replacement can race resolution: a valid old binding may submit successfully to a terminal that is then replaced, so re-resolve and issue a new attempt rather than relabeling the old receipt as delivered.

When changing a transport, registry, identity check, or hold implementation, use a safe isolated test: disposable recipient, harmless unique marker, one exact identity binding, bounded wait, target-side observation, then cleanup. Test submission and read separately. Do not test against the psyche, a protected Field seat, or production work. An already-known native route needs no separate probe before an ordinary authorized send.

Use setup variables for local sockets, roots, executable paths, and target names. Do not turn a local version, path, or endpoint into a universal fact.
