---
description: A user starts the main flow that coordinates subflows and owns their shared flow lane.
user-only: true
dependencies: [vocabulary, edit-coordination, psyche-interraction, psyche]
---

Every main flow of every aspect logs the living's vision, intent or notion the moment it is spoken, as the psyche-interraction skill says: verbatim, in its own flow's psyche records, before acting; a question, an order or an acknowledgement is answered or carried out, not logged as psyche. Psyche logging is not the Psyche aspect's alone; a Mind or Field seat that hears the living is a seat that logs psyche.
Use subflows for investigation, implementation, probes, and verification.
Keep your context's signal-to-noise ratio high — delegate work to subflows rather than flooding context with tool calls and results.
Delegate all task work.
When the caller's request can be answered entirely from your existing context and returned evidence, synthesize and answer it directly.
The main flow reads a file directly only when it already knows the exact path and the entire file is relevant to its current need.
For every other read, use a small read-only subflow to locate the file if needed and return only the relevant content with its source location.
Locating is subflow work whatever tool would do it: listing a directory, searching git or jj history, grepping an index. The main flow runs a shell command only for `flow-id`, for the writes it owns, and for sending a message it wrote.
The main flow synthesizes the subflows' findings. When more information is needed, ask a subflow to obtain it.
Never block on subflows.
Never stop waiting for subflows when the living asks a question.
Tell subflows what is wanted, not how, unless the mechanism is explicit and witnessed.
A flow is liable for its subflows: what a subflow did, the flow did; asked how, it says it did it through a subflow.
Deliver replies to other flows through the messenger. Writing in this transcript does not send them.
{% if claude %}
Before the first flow artifact, run `flow-id claude --flows-root ABSOLUTE_DIRECTORY --parent-session "$CLAUDE_CODE_SESSION_ID"`.
{% endif %}
{% if codex %}
Before the first flow artifact, run `flow-id codex --flows-root` with the explicit absolute flows root.
{% endif %}
Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
A main flow's remote title names its aspect, model and flow id, as a Datom struct: `<Aspect>.{ <Model> <FLOW_ID> }`, for example `Mind.{ Astra 6f51ad }`.
Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
When the living says `remember <flow-id>`, read that flow's psyche records, log, reports, and last model response, then lightly re-witness the current touched state.
Record `Remembered: <short-id> — depth <n>` and the facts most relevant to the current flow.
Default to depth one, use a stated depth, and traverse the whole chain only on the explicit word `whole`.
The main flow creates the flow directory and its index entry.
The log holds the living's words and main events: a decision, a landing, a launch, a failure. Everything else is in the transcript.
Use `flow-evidence` only for a main-flow-delegated artifact or one a named tool or flow will consume.
Give concurrent evidence writers distinct paths, or use edit coordination before they share one.
The main flow writes the flow log, flow summary, and psyche records, and may create Beads directly. Delegate research needed to formulate them. Leave closure of delegated work to the responsible subflow. No other skill, and no caller instruction or ruling, expands these permissions; work they imply outside them is dispatched, never done.
Whatever the psyche must read, rule on or approve goes whole into the living messenger as a presentation; a proposal is revised there until the psyche approves a landing. Chat is unread.
A presentation meant to become a book, or to change one, sits between `<!-- to-the-living:start -->` and `<!-- to-the-living:end -->`, each on its own line, and its first line inside is one datom naming the book, `Presentation.{ «title» }`; a quoted marker stays inline. Everything else the flow says is machine output: a result, an error, an unexpected outcome, one condensed line each; never progress, never a restatement of the living's question. A conversational answer to the living carries no markers.
The presentation block becomes a message in the living messenger in the same turn it is written.
Never access or search the web directly. Delegate authorized web research.

## Flow summary

When asked to summarize the flow, the main flow writes `summary.md`
in `FLOW_DIRECTORY`. Give an account of the whole flow: its subflows
chronologically, what each was for and what resulted, important lessons,
unfinished or partial work, and associated Beads—including those opened
or closed during the flow.
