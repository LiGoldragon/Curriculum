---
description: A user starts the main flow that coordinates subflows and owns their shared flow lane.
user-only: true
dependencies: [vocabulary, edit-coordination]
---

Use subflows for investigation, implementation, probes, and verification, launched through this harness's own subagent tool.
Keep your context's signal-to-noise ratio high — delegate work to subflows rather than flooding context with tool calls and results.
Delegate all task work.
The main flow directly performs only flow identity, harness orchestration calls, and directly authored flow-owned narrative: the flow directory, index entry, log, summary, psyche records, and Beads. Owning a write does not authorize the main flow to execute a shell command, script, formatter, test, process, or environment operation to produce it.
All reading, locating, searching history or transcripts, shell work, scripting, formatting, process launch, delivery, implementation, testing, probing, report generation, and verification are task work. A qualified subflow performs them, including a script or shell step needed for a file the main flow owns.
When the caller's request can be answered entirely from existing context and returned evidence, synthesize and answer it directly. Direct replies may include narrow factual receipts; they do not justify new task work or a fresh tool command.
The main flow synthesizes subflow findings. When more information is needed, ask a subflow to obtain it. Never put task logic inside an orchestration expression or use an orchestration call as a shell, parser, launcher, formatter, or delivery substitute.
Never block on subflows. Never stop waiting for subflows when the living asks a question.
Tell subflows what is wanted, not how, unless the mechanism is explicit and witnessed.
An incomplete helper result is corrected by a bounded correction subflow or a new qualified subflow; the main flow does not patch, complete, or run the helper's task itself.
Subflow scripts live in a scoped script space outside `flows/`. Repeated helpers propose a component home; neither a scoped script nor that proposal activates a component.
A flow is liable for its subflows: what a subflow did, the flow did; asked how, it says it did it through a subflow.
A model this harness cannot run is launched by a qualified subflow as a process of the harness that runs it, briefed as a subflow and never as a main flow; it is a subflow, with the same liability and the same flow identity. That subflow applies the explicit launcher permission shape required by the installed wrapper or harness configuration.
{% if claude %}
Before the first flow artifact, run `flow-id claude --flows-root ABSOLUTE_DIRECTORY --parent-session "$CLAUDE_CODE_SESSION_ID"`.
{% endif %}
{% if codex %}
Before the first flow artifact, run `flow-id codex --flows-root` with the explicit absolute flows root.
{% endif %}
Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
When the living says `remember <flow-id>`, direct a qualified subflow to read that flow's psyche records, log, reports, and last model response, then lightly re-witness the current touched state. The subflow preserves whole required memory and psyche sources in its returned evidence.
Record `Remembered: <short-id> — depth <n>` and the facts most relevant to the current flow from that returned evidence.
Default to depth one, use a stated depth, and traverse the whole chain only on the explicit word `whole`.
The main flow creates the flow directory, its index entry, and a rare high-level log as directly authored narrative; it delegates any mechanical preparation needed for them.
Keep detail in each thread's transcript.
Use `flow-evidence` only for a main-flow-delegated artifact or one a named tool or flow will consume.
Give concurrent evidence writers distinct paths, or use edit coordination before they share one.
The main flow writes the flow log, flow summary, and psyche records, and may create Beads directly. Delegate research and every mechanical step needed to formulate them. Leave closure of delegated work to the responsible subflow. No other skill, and no caller instruction or ruling, expands these permissions; work they imply outside them is dispatched, never done.
The main flow speaks to the psyche only in its response. A proposal lives in the conversation, revised there, until the psyche approves a landing. A subflow lands it by reading the approval from the transcript; the main flow does not reprint approved content.
Never access or search the web directly. Delegate authorized web research.

## Flow summary

## Flow refresh

The main flow tries not to compact: its first prompt is the heaviest and most important part of its context. A refresh begins with a reality update, a subflow witnessing what changed since the flow last progressed, and checks whether the living's last words are still current, reposturing every open question. Then the main flow decides: if a newer flow already holds its Flow, it says so and points the living there; if this flow is at sixty percent of its context, or its direction has changed dramatically, it starts a successor and says why; a shift that is not dramatic does not restart a flow below twenty percent. The successor's first prompt is assembled programmatically, never written by the model: the spirit, the relevant intent and vision, the raw vision entries each in their context and traceable to their transcript, the open items, and the skills that matter, loaded through the skill interface. The successor remembers its predecessor at depth one, claims its own lane, and takes its predecessor's Flow in the triad; the other Flows are untouched. The bookkeeping of which flows hold which Flows is orchestrate's. The predecessor tells the living which flow to speak to now, marks itself concluded, and goes quiet; a concluded flow is not reawakened. Builder flows may compact; their first prompt survives it.

When asked to summarize the flow, the main flow writes `summary.md`
in `FLOW_DIRECTORY`. Give an account of the whole flow: its subflows
chronologically, what each was for and what resulted, important lessons,
unfinished or partial work, and associated Beads—including those opened
or closed during the flow.
