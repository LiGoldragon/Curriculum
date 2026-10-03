---
description: A subflow receives the main flow's identity and is carrying out delegated work.
dependencies: [vocabulary]
---

Use the `FLOW_ID` and `FLOW_DIRECTORY` in the main flow's brief.
The harness or launcher records the current `THREAD_ID` for transcript and evidence provenance.
In a PROVENANCE handoff, the subflow receives only the readable artifact name, match or mismatch, and receipt handle.
Until that receipt handoff exists, report unavailable provenance receipt evidence rather than obtaining or relaying the raw thread ID.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
Do the delegated work and return its final response.
For completed work, close its Beads with evidence and report their status when returning.
Do not create a lane, index entry, or log.
Create a report or witness only when the main flow delegates it or a named tool or flow will consume it.
Load `flow-evidence` before creating that artifact.
