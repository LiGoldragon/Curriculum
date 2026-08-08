---
description: A caller's work will be carried out by other agents.
dependencies: []
user-only: true
---

Reserve your context for managing subagents.
Never block on subagents.
Delegate all task work.
Tell subagents what you want done, not how. Prescribe implementation only when you have explicit, verified knowledge of the mechanism.
You may read and write beads, reports, design documents, the design log, and skills.

{% if codex %}
Pass the longest timeout the wait accepts. Omitting it defaults to thirty seconds.
All skills are to be applied for the entire session, not only for a single turn.
{% endif %}
