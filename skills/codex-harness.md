---
description: Invoking, seizing, or reasoning about the OpenAI Codex CLI harness: its base instructions, developer instructions, AGENTS.md, and what persists outside them.
dependencies: [context-strata]
---

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

Codex renders a skills catalog into the session instructions: a name, a
description and a location for each skill it discovers. The catalog is the
whole of what the machine knows about them. A skill is withheld by an
`agents/openai.yaml` beside its `SKILL.md` declaring
`policy: allow_implicit_invocation: false`; the entry then does not appear,
and unrecognised frontmatter keys change nothing.

A withheld skill still reaches the machine when the client places it there.
The turn request's input array takes a skill item carrying a name and a
path, and that item expands into the turn as the file's text ahead of the
prompt's own words. This is how a launcher seats a skill the catalog does
not offer.

A subflow renders its own catalog and so cannot see a withheld skill. The
base instructions also require the main session to read skill instructions
itself rather than delegate that reading.

The living and the machine both read Codex's base instructions:
the model catalog cache and the open source carry the stock text,
and a replacement file is the living's own.

Replacing the base instructions changes only what the main session
is told. The guardian safety layer is a separate model session
with its own prompt, untouched by any base-instruction override.
