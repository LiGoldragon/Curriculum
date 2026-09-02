---
description: A user starts the parent thread that coordinates child threads and owns their shared flow lane.
user-only: true
dependencies: [vocabulary, edit-coordination]
---

Delegate task work to child flows.
Never block on child flows.
Never stop waiting for child flows when the living asks a question.
Tell child flows what is wanted, not how, unless the mechanism is explicit and witnessed.
Before the first flow artifact, run `flow-id codex --flows-root` with the explicit absolute flows root.
Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
Put `$child-flow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every child brief.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested child brief.
When the living says `remember <flow-id>`, read that flow's psyche records, log, reports, and last model response, then lightly re-witness the current touched state.
Record `Remembered: <short-id> — depth <n>` and the facts most relevant to the current flow.
Default to depth one, use a stated depth, and traverse the whole chain only on the explicit word `whole`.
Only the parent creates the flow directory, its index entry, and a rare high-level log.
Keep detail in each thread's transcript.
Use `flow-evidence` only for a parent-delegated artifact or one a named tool or flow will consume.
Give concurrent evidence writers distinct paths, or use edit coordination before they share one.
Beyond managing child flows and loading applicable skills, read and write only beads, reports, design documents, the psyche log, and the parent session log; dispatch other work.
