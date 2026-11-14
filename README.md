# Curriculum

A versioned context registry and Nexus for psyche-belief, mind-belief and field-belief. It resolves module IDs and composes startup and role prompts in memory. See ARCHITECTURE.md for the declared roots, wire contract, placement, hashes and guard boundaries; signal-curriculum-nexus is the wire contract, and curriculum.ethos and memory.ethos the authoritative schema of Curriculum's own files and memory.


A module is named `kind:topic:number` and stored as `kind/topic/number.md`. Every caller and dependency supplies the explicit version, including `0`; resolution selects exactly that version. Generated skill names retain the complete three-field ID.
There is no caller-selected context file, inline prompt, legacy skills directory, file projection or implicit context source. Flow requests startup composition; launcher adapters do not implement another composer. Consumers must deploy the schema, generated code, immutable source cohort and ID-only role declarations together. Existing live cohorts remain pinned until the replacement is checked.
# Claude agent definitions

The producer renders any registry-composed Claude role to stdout:

    curriculum --claude-agent-markdown 'ComposeRole.{ comment-relay ClaudeAgent }'

The deployment owner installs this output as generated
`.claude/agents/<role>.md` within its existing transaction. The producer writes
no files and accepts no prompt path or inline prose. It checks returned payload
hashes and renders the model, effort and tool list the role definition declares;
an `Inherit` role carries no tools line. Native permission mode remains the
caller's; this output grants no permission bypass.

The renderer refuses other surfaces, a restricted role that inherits tools, and
a role, model or tool name outside ASCII letters, digits, `-`, `_` and `.`. A
caller policy is not carried into the native file: a native definition does not
authenticate its caller. The deployment owner must verify native loading and
tool resolution; rendering alone is not dispatch evidence.
