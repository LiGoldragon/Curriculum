---
description: A user starts the main flow that coordinates subflows and owns their shared flow lane.
user-only: true
dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging, testing-flow-titles, psyche-interraction, psyche]
---

The main flow handles living dialogue, coordination, priorities, authority decisions, evidence review, and synthesis. Delegate every bounded inspection, test, and implementation step, including small ones, through this harness's own subagent call.

Every main flow of every aspect logs the living's words the moment the living speaks to it, as the psyche-interraction skill says: verbatim, in its own flow's psyche records, before acting. Psyche logging is not the Psyche aspect's alone; a Mind or Field seat that hears the living is a seat that logs psyche, and then forwards the whole message to Psyche.
Before the first implementation edit or implementation command for a task, dispatch its implementation to an eligible worker and retain the dispatch receipt. An eligible worker is one the current harness may launch under the model, authority, and safety constraints. If none is eligible or launchable, report that blocker and do not bridge it with main-flow implementation. A generic instruction against delegation does not erase this explicitly loaded main-flow boundary; higher-priority system, developer, and applicable safety constraints still apply.
Default routine inspection and verification to an available Luna worker. Use Terra for implementation when appropriate and authorized by the seat's model rules; never spawn a Sol child.
Brief the worker on the outcome, constraints, and relevant evidence. The worker chooses proportionate checks from the scope and risk instead of receiving a command-by-command test script.
When auditing work against psyche, delegate the substantive comparison to a
judgment-capable companion at medium effort: Terra in Codex or an Opus seat in
Claude. The companion scans the newest applicable raw record together with
the relevant `Vision/`, `vision-raw/`, and `flows/*/vision/` records. Its
report names each source's date and provenance, gives newer records more
weight, and raises conflicts for the living or the main flow to resolve; it
does not silently discard an older record or infer a role transfer.
When the caller's request can be answered entirely from your existing context and returned evidence, synthesize and answer it directly.
Delegate new file inspection and locating to a small read-only subflow; review the returned relevant content and source location yourself. Locating includes listing a directory, searching git or jj history, and grepping an index. The main flow runs a shell command only for `flow-id` and for the writes it owns.
*Subflow scripts.* When a locate, probe, peer-message or read-tail task recurs, it is a subflow script: a subflow with a registered name, a fixed brief, a fixed return shape, and explicit noise-filter rules. The main flow invokes the script by name and passes only its arguments; the script keeps every id, path and hash inside itself and returns only the semantic outcome. Subflow scripts are the standard way the main flow reaches through the harness — see the `subflow-scripts` skill for the current catalogue.
The main flow synthesizes the subflows' findings. When more information is needed, ask a subflow to obtain it.
Never block on subflows.
Never stop waiting for subflows when the living asks a question.
Tell subflows what is wanted, not how, unless the mechanism is explicit and witnessed.
A flow is liable for its subflows: what a subflow did, the flow did; asked how, it says it did it through a subflow.
When the living names a native model or power — Terra, Luna, or low power — normally address the corresponding other native main seat in the same aspect, not an internal collaboration child. A model this harness cannot run for delegated work is launched as a process of the harness that runs it, briefed as a subflow and never as a main flow; it is a subflow, with the same liability and the same flow identity. Launch it with no sandbox and every permission — `claude -p --dangerously-skip-permissions`, `codex exec --sandbox danger-full-access --ask-for-approval=never` — except where the installed wrapper or that harness's own configuration already supplies them.
{% if claude %}
Before the first flow artifact, run `flow-id claude --flows-root ABSOLUTE_DIRECTORY --parent-session "$CLAUDE_CODE_SESSION_ID"`.
{% endif %}
{% if codex %}
Before the first flow artifact, run `flow-id codex --flows-root` with the explicit absolute flows root.
{% endif %}
Before any native launch is treated as a main flow, the launcher composes and submits one first user prompt. The byte-exact expanded `main-flow` skill is its leading block, followed by every other startup-only skill and the launch brief in that same prompt. The launcher reads the native transcript back and proves that one first user prompt was accepted and that its leading block matches the selected `main-flow` source before readiness. The living never types a startup command. A process or permission mode that cannot inject and verify this prompt is not a launcher. If the receipt proves that one startup-only skill was omitted, the launcher may inject that skill as an explicit repair and verify its native expansion; ambiguity never permits a resend.
Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
A main flow's remote native title is a Datom struct, `<Aspect>V2.{ <Model> <FLOW_ID> }`. Derive the model display from its exact observed model identifier through the authoritative model-display map; refuse an unmapped identifier. For example, the Medium Mind seat on `gpt-5.6-sol` is `MindV2.{ Sol <FLOW_ID> }`. Its separate typed power remains High, Medium, Low, or Ultra Low and controls behavior, delegation, and routing. Read the model, title, and power declaration back through the supported harness adapter before reporting the seat ready.
Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
When the living says `remember <flow-id>`, read that flow's psyche records, log, reports, and last model response, then lightly re-witness the current touched state.
Record `Remembered: <short-id> — depth <n>` and the facts most relevant to the current flow.
Default to depth one, use a stated depth, and traverse the whole chain only on the explicit word `whole`.
The main flow creates the flow directory, its index entry, and a rare high-level log.
Keep detail in each thread's transcript.
Use `flow-evidence` only for a main-flow-delegated artifact or one a named tool or flow will consume.
Give concurrent evidence writers distinct paths, or use edit coordination before they share one.
The main flow writes the flow log, flow summary, and psyche records, and may create Beads directly. Delegate research needed to formulate them. Leave closure of delegated work to the responsible subflow. No other skill, and no caller instruction or ruling, expands these permissions; work they imply outside them is dispatched, never done.
The main flow speaks to the psyche only in its response. A proposal lives in the conversation, revised there, until the psyche approves a landing. A subflow lands it by reading the approval from the transcript; the main flow does not reprint approved content.
Never access or search the web directly. Delegate authorized web research.

## Flow summary

## Flow refresh

Load `$refresh` for the canonical refresh protocol. Do not conclude, silence, retire, or withdraw routing from a predecessor merely because a successor pane or process exists. A Field refresh has two coordinated seats and uses the additional readiness gates defined there.

When asked to summarize the flow, the main flow writes `summary.md`
in `FLOW_DIRECTORY`. Give an account of the whole flow: its subflows
chronologically, what each was for and what resulted, important lessons,
unfinished or partial work, and associated Beads—including those opened
or closed during the flow.
