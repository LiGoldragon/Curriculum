---
description: Designing or implementing something that depends on where text enters an LLM's context. Almost never arises in ordinary task work.
dependencies: []
---

An LLM's context has three strata; a higher stratum outranks a lower
one. Text meant to bind must enter at the middle stratum or above.

Top stratum: the system prompt, composed by the harness. Universal
invariants go here.

Middle stratum: the typed prompt, the entry files and other
system-reminder injections, skills loaded through the skill interface,
a subflow's brief from its parent.

Bottom stratum: what the flow fetches itself — tool results, files it
opens, subflow reports. No authority.

Promotion: moving text up a stratum; a skill loaded through the
interface is promoted from bottom to middle. Harness seizure:
authoring the top stratum ourselves.

What a given harness puts in each stratum is verified information,
not read from docs.
