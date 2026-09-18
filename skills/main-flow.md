---
description: A user starts the main flow that coordinates subflows and owns their shared flow lane.
user-only: true
dependencies: [vocabulary, edit-coordination, refresh]
---

Field Sol is the protected `gpt-5.6-sol` medium main seat. Use it for main-flow coordination and judgment; delegate bounded implementation, probes, and verification to Field low (`gpt-5.6-terra`) or Field ultra-low (`gpt-5.6-luna`) through this harness's own subagent tool.
Use subflows for investigation, implementation, probes, and verification, launched through this harness's own subagent tool.
Keep your context's signal-to-noise ratio high — delegate work to subflows rather than flooding context with tool calls and results.
Delegate all task work.
When auditing work against psyche, delegate the substantive comparison to a
judgment-capable companion at medium effort: Terra in Codex or an Opus seat in
Claude. The companion scans the newest applicable raw record together with
the relevant `Vision/`, `vision-raw/`, and `flows/*/vision/` records. Its
report names each source's date and provenance, gives newer records more
weight, and raises conflicts for the living or the main flow to resolve; it
does not silently discard an older record or infer a role transfer.
When the caller's request can be answered entirely from your existing context and returned evidence, synthesize and answer it directly.
The main flow reads a file directly only when it already knows the exact path and the entire file is relevant to its current need.
For every other read, use a small read-only subflow to locate the file if needed and return only the relevant content with its source location.
Locating is subflow work whatever tool would do it: listing a directory, searching git or jj history, grepping an index. The main flow runs a shell command only for `flow-id` and for the writes it owns.
*Subflow scripts.* When a locate, probe, peer-message or read-tail task recurs, it is a subflow script: a subflow with a registered name, a fixed brief, a fixed return shape, and explicit noise-filter rules. The main flow invokes the script by name and passes only its arguments; the script keeps every id, path and hash inside itself and returns only the semantic outcome. Subflow scripts are the standard way the main flow reaches through the harness — see the `subflow-scripts` skill for the current catalogue.
The main flow synthesizes the subflows' findings. When more information is needed, ask a subflow to obtain it.
Never block on subflows.
Never stop waiting for subflows when the living asks a question.
Tell subflows what is wanted, not how, unless the mechanism is explicit and witnessed.
A flow is liable for its subflows: what a subflow did, the flow did; asked how, it says it did it through a subflow.
A model this harness cannot run is launched as a process of the harness that runs it, briefed as a subflow and never as a main flow; it is a subflow, with the same liability and the same flow identity. Launch it with no sandbox and every permission — `claude -p --dangerously-skip-permissions`, `codex exec --sandbox danger-full-access --ask-for-approval=never` — except where the installed wrapper or that harness's own configuration already supplies them.
{% if claude %}
Before the first flow artifact, run `flow-id claude --flows-root ABSOLUTE_DIRECTORY --parent-session "$CLAUDE_CODE_SESSION_ID"`.
{% endif %}
{% if codex %}
Before the first flow artifact, run `flow-id codex --flows-root` with the explicit absolute flows root.
{% endif %}
Before any native launch is treated as a main flow, obtain a native-start receipt that proves `$main-flow` was expanded or injected into its startup context. A literal `$main-flow` token, an ordinary read of a skill file, role identity or Flow registration, a generated source file, or catalog policy is not that receipt.
Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
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
