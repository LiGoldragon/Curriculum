# voice-psyche: created after the baseline

Created in b42c519, 2026-09-18 11:07 -0600: Add Voice Psyche operational skill. Made by: no model trailer (a Codex seat by the audit reading).
Later edits: d8769f0 2026-09-18 “Escalate Voice Psyche investigations from Luna to Terra”.

## View

The living asked for it and said "Sure. For now" to the drafted wording (flows/8393ca/vision/operational-herdrVoiceAccess.md); he asked for the Luna-then-Terra line himself ("Ask Luna. If Luna can't find it, ask Terra").
View: candidate, less the Terra line (Terra is withdrawn). Whether the voice seat still runs is the living's to say.

## The whole skill as it stands on main (3726da5)

````markdown
---
description: The living psyche is speaking through interactive voice while work proceeds through subflows.
user-only: true
dependencies: [main-flow, psyche-interraction]
---

The voice conversation remains available while subflows work.

All investigation, queries, and execution go to subflows. The main flow
holds the conversational context, briefs subflows, and answers directly
from what it already knows.

Send an investigation to Luna first. If Luna cannot resolve it, send Terra
the unresolved question together with Luna's findings and context.

Replies address the living's current words. Subflow activity does not
delay the conversation.

Updates convey useful findings, uncertainty, completion, or a decision
needed from the living. Waiting produces no filler acknowledgements.
````
