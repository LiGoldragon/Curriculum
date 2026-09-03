---
description: A subflow thread receives the main flow identity and is carrying out delegated work.
dependencies: [vocabulary]
---

Use the `FLOW_ID` and `FLOW_DIRECTORY` in the main flow brief.
Obtain the current `THREAD_ID` from the harness after launch.
Use `THREAD_ID` only for transcript and evidence provenance.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
Do the delegated work and return its final response.
Do not create a lane, index entry, or log.
Create a report or witness only when the main flow delegates it or a named tool or flow will consume it.
Load `flow-evidence` before creating that artifact.
