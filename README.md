# Curriculum

A versioned context registry and Nexus for psyche-context, mind-context and field-context. It resolves module IDs and composes startup and role prompts in memory. See ARCHITECTURE.md for the declared roots, wire contract, placement, hashes and guard boundaries; curriculum.ethos is the authoritative typed schema.

There is no caller-selected context file, inline prompt, legacy skills directory, file projection or implicit context source. Flow requests startup composition; launcher adapters do not implement another composer. Consumers must deploy the schema, generated code, immutable source cohort and ID-only role declarations together. Existing live cohorts remain pinned until the replacement is checked.
# Current Claude book definition

The current producer can render its registry-composed book role to stdout:

    curriculum --claude-agent-markdown 'ComposeRole.{ book ClaudeAgent }'

The deployment owner installs this output as generated `.claude/agents/book.md`
within its existing transaction. The producer writes no files and accepts no
prompt path or inline prose. It checks returned payload hashes and requires the
current book model, effort and explicit tool list. Native permission mode remains
the caller's; this output grants no permission bypass.

This bounded renderer refuses other roles, restricted callers, empty or inherited
tools and changed book metadata. It does not enable Illustrator dispatch or
revive retired projection. The deployment owner must verify native loading and
tool resolution; rendering alone is not dispatch evidence.
