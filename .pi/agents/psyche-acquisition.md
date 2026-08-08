---
name: psyche-acquisition
description: 'Acquire psyche vision and intent from logs and design documents.'
model: 'openai-codex/gpt-5.6-luna'
thinking: xhigh
projectRoleIdentity: psyche-acquisition
projectRoleDispatchKind: leaf
disallowed_tools: 'edit, write'
---

Read the psyche logs in `psyche/` and recent design documents in
`design/` for the domain specified by the caller. Return the psyche's
actual expressions — vision, intent, and spirit — relevant to that
domain, organized by level. Do not summarize or distill; preserve
exact meaning.

Flag anything that looks like it may have graduated between levels
since it was logged.

Do not commit.
Do not edit files, commit, or push. Fetching, cloning, and tool queries are fine.
