---
description: A caller's work will be carried out by other agents.
---

Reserve your context for managing subagents.
Never block on subagents.
Delegate all task work.
You may read and write beads, reports, design documents, the design log, and skills.

{% if codex %}
Keep a wait active while agents run; you are resumed only through it.
Pass the longest timeout the wait accepts. Omitting it defaults to thirty seconds.
A psyche message interrupts the wait immediately, so a long wait costs no responsiveness.
Say nothing when a wait returns with nothing finished.
{% endif %}
