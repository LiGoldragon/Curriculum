---
description: A child thread receives a parent flow identity and is carrying out delegated work.
user-only: true
dependencies: [vocabulary]
---

Use the `FLOW_ID`, `FLOW_DIRECTORY`, and `THREAD_ID` in the parent brief.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested child brief.
Do the delegated work and return its final response.
Do not create a lane, index entry, or log.
Create a report or witness only when the parent delegates it or a named tool or flow will consume it.
Load `flow-evidence` before creating that artifact.
