---
description: Designing or implementing something that depends on where text enters an LLM's context. Almost never arises in ordinary task work.
dependencies: []
---

An LLM's context has three strata; a higher stratum outranks a lower
one. Text meant to bind must enter at the middle stratum or above.

Top stratum: the base context, and any text authored into its seat.
Universal invariants go here.

Middle stratum: the typed prompt, the entry files and other
system-reminder injections, skills loaded through the skill interface,
a subflow's brief from its parent.

Bottom stratum: what the flow fetches or says itself — tool results, files it
opens, subflow reports, its own output. No authority.

Promotion: moving text up a stratum; a skill loaded through the
interface is promoted from bottom to middle. Harness seizure:
authoring the top stratum ourselves.

What a given harness puts in each stratum is verified information,
not read from docs.

Channels. A harness may tag the flow's own output with named channels
(e.g. commentary, final). Channels are not strata; they are visibility
and retention tags on items in one position — the flow's output, bottom
stratum. What a harness retains per channel is verified information:
for Codex CLI 0.149.1, commentary and final are both assistant-role
messages distinguished only by a phase field, both replayed to the
model until compaction, both dropped after.
