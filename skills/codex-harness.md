---
description: Invoking, seizing, or reasoning about the OpenAI Codex CLI harness: its base instructions, developer instructions, AGENTS.md, and what persists outside them.
dependencies: [context-strata, operators-notes]
---

Use operators-notes to read or compose the operational records below.

Codex's top stratum is the base instructions, sent as the
instructions field of the Responses API request, above the whole
input array. The stock text is a per-model template served from
the backend model catalog and cached locally, with a compiled-in
default as fallback. The model_instructions_file config key
replaces the base instructions with a file's text and outranks the
instructions config key, which replaces them with a string; the
source discourages both, and we use the file.

A Codex session's model and reasoning effort come from
`~/.codex/config.toml` at launch and change without notice. A
launcher that must pin a model passes `-m <model> -c
model_reasoning_effort=<effort>` rather than inheriting them.

Codex has three strata with a ranking inside the middle: the
developer role outranks the user role within the input array.
developer_instructions is a developer-role message sent beside the
base instructions and never part of them. AGENTS.md files, from
the Codex home and from the repository root down to the working
directory, enter as user-role messages under an AGENTS.md
instructions marker and cannot override base instructions. Tool
results and the machine's own output are bottom stratum.

The living and the machine both read Codex's base instructions:
the model catalog cache and the open source carry the stock text,
and a replacement file is the living's own.

Replacing the base instructions changes only what the main session
is told. The guardian safety layer is a separate model session
with its own prompt, untouched by any base-instruction override.

## Operators' notes

### 2026-09-17 — Flow identity helper arguments

`execution-failure / flow-identity`: in flow 99f9f7, the installed `flow-id` argument parser returned usage and exit 2 when the Codex invocation included `--parent-session`. Its usage assigned that flag to the Claude form. The subsequent Codex invocation with only the explicit flows root succeeded and returned `99f9f7`; flow identity was established. This was a CLI argument failure, with no classifier or permission refusal in either result. Helper version: unknown. Evidence: flow 99f9f7's directly witnessed exec result chunks `dce0bf` and `ff0327`. Attention: pending; note acceptance: awaiting-glance.
