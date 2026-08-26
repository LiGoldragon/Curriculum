---
description: A skill is being written or changed.
dependencies: []
---

Write skills with brutal minimalism.
State unusual, impactful instructions once and directly.
Flag anything noisy, unclear, unsafe, or misplaced. Explain what each proposed change preserves, changes, or removes.

Search `Reference skill collections` for prior art on the situation before writing.

A description names the situation that should make an agent load the skill,
in the words of the task at hand. State a trigger, not a topic.
Open with the situation itself. A shared formula carries nothing.
Repeat neither the skill's name nor the word "skill".
No two descriptions may match the same situation.
Length is free. Spend it on what separates this skill from its neighbours.
Nothing in a description appears in the skill. The description is the situation
before loading; the skill is what to do after.

## Cut these

A line whose meaning a competent reader must guess.
A line that restates the skill or role name.
A line true of any competent agent.
A line naming the desired end state without teaching the move, the test, or the case.
A line pairing a goal with a mechanism. The reader cannot tell which one binds.
A line that explains or justifies a rule instead of directing an action.
A line that restates a rule another skill holds.

## Keep these

Minimal is the requirement. Imperative is often the shortest form of it, not the only one.
Agents already behave in the usual way, so an instruction matching default behavior changes nothing.
Removal is better than addition, when the expected behavior is the desired behavior.
State a rule only where it diverges from what an agent would do untold.
Unusual lines carry the behavior change. Remove them only after everything else.
Agents connect surfaces that use the same term. Do not add a line telling them where to look.
Skills name capabilities. Workspaces name the implementations that provide them.
Write each rule as a plain sentence. Do not shape a line for memorability.
A rule names a thing and what must be done with it, with no actor: "A claim must be relayed as a claim."
A line must hold beyond the document, tool, or incident that prompted it.
Write a rule only when it prevents a failure that has happened, or states a choice an agent cannot derive.
Name the incident or the choice. If you can name neither, do not write the rule.
{% raw %}
Target-specific text in a flat source uses `{% if claude %}`, `{% if codex %}`, or `{% if pi %}`, with `{% else %}` and `{% endif %}` alone on their lines; every other character is literal skill content.
{% endraw %}

## Skill types

`user-only: true` — the skill enters only through the user's
typed prompt; the flow cannot load it. It deploys as
`disable-model-invocation: true` in Claude Code and as
$-name-only injection in Codex.

A role skill carries an aspect's identity and names its
dependencies. Mark role skills user-only.

A skill's reasoning and concepts live in a parallel <skill>-rationale skill, loaded by psyche-facing flows only.
