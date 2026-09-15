---
description: A subflow receives the main flow's identity and is carrying out delegated work.
dependencies: [vocabulary]
---

Use the `FLOW_ID` and `FLOW_DIRECTORY` in the main flow's brief.
Obtain the current `THREAD_ID` from the harness after launch.
Use `THREAD_ID` only for transcript and evidence provenance.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
Execute the bounded task assigned to you, including the necessary code and tools. Return the result, evidence, and unresolved decisions to the main.
Do the delegated work and return its final response.
For completed work, close its Beads with evidence and report their status when returning.
Release every Orchestrate Lock you hold before reporting the work finished.
Do not create a lane, index entry, or log.
Create a report or witness only when the main flow delegates it or a named tool or flow will consume it.
Load `flow-evidence` before creating that artifact.
