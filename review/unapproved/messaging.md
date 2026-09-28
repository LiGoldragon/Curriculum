# messaging: created after the baseline

Created in 6c161a0, 2026-09-18 10:15 -0600: Define Field and evidence-disciplined messaging operations. Made by: no model trailer (a Codex seat by the audit reading).
Later edits: 451f513 2026-09-24 “Simplify neutral Datom messaging guidance”; b52929d 2026-09-25 “Teach the live Clojure HM message boundary”; 867b060 2026-09-25 “Apply HM skill semantic audit”; 226f2e5 2026-09-25 “Rename authored messenger references”; 5b36d7c 2026-09-25 “Document full Messenger body and psyche variants”.

## View

The living asked Psyche to design a messaging skill ("The living approves before it enters Curriculum"); what landed on 2026-09-18 (6c161a0, a Codex seat) was component naming and receipt grades instead. Later edits: 451f513 with the living ("make sure we are not forcing the agents to put information in there that's not necessary"); the rest by flows around the messenger rewrite.
View: fold the body-only line into compensation-messenger-clj and drop the rest: Flow Nexus 0.3 and Message Nexus 0.12/0.13 are dead references, and "Use a safe isolated test before relying on a route" goes by the living's ruling of 2026-09-28: "Probes wake seats. Don't do it."

## The whole skill as it stands on main (3726da5)

````markdown
---
description: A flow must send, receive, route, or verify an operational message.
dependencies: [behavior, herdr, testing, vocabulary]
---

Name the layer before claiming delivery.

Herdr 0.8.2 is the live terminal-workspace transport. It can inject into a running terminal through its own witnessed APIs; it is not durable message storage or identity resolution.

messenger-clj is the live compatibility bridge behind the `hm-*` command shorthands: it resolves a running target and prompts it through Herdr. A successful submission is not a read receipt.

Flow Nexus 0.3 is the identity and resolution design: it binds the exact logical flow identity to the exact live target. It does not itself prove transport or durable delivery.

Message Nexus 0.12 is installed for durable attempts and receipts. State the observed operation and receipt, not an assumed semantic outcome. The published-not-deployed 0.13 receipt query is not live. The central-messenger design is vision, not a current service.

Receipt grades are distinct. Submitted means the sender accepted the request. Transported means the selected transport accepted the bytes for the exact binding. Presented means the target terminal or harness received the prompt. Read means an observed target-side read acknowledgment. Completed means the requested work returned its stated completion evidence. Never upgrade one grade into another.

Write the recipient-facing body only. Preserve the submitted bytes in the receipt.

Resolve the recipient immediately before submission and bind the attempt to that exact identity and live target. Record the binding with the attempt. A terminal replacement can race resolution: a valid old binding may submit successfully to a terminal that is then replaced, so re-resolve and issue a new attempt rather than relabeling the old receipt as delivered.

Use a safe isolated test before relying on a route: disposable recipient, harmless unique marker, one exact identity binding, bounded wait, target-side observation, then cleanup. Test submission and read separately. Do not test against the psyche, a protected Field seat, or production work.

Use setup variables for local sockets, roots, executable paths, and target names. Do not turn a local version, path, or endpoint into a universal fact.
````
