# Curriculum

A versioned context registry and Nexus for psyche-belief, mind-belief and field-belief. It resolves module IDs and composes startup and role prompts in memory. See ARCHITECTURE.md for the declared roots, wire contract, placement, hashes and guard boundaries; signal-curriculum-nexus is the wire contract, and curriculum.ethos, memory.ethos and emission.ethos the authoritative schema of Curriculum's own files, its memory and its one external call.


A module is named `kind:topic:number` for a singleton or `kind:topic:first-last` for an inclusive increasing span. Declarations use `Single.number` or `Span.{ first last }`. A numeric caller or dependency binding receives the singleton and every declared span containing that number; an explicit span binding selects that exact span. Generated skill names retain the actual complete three-field identity.
There is no caller-selected context file, inline prompt, legacy skills directory, file projection or implicit context source. Flow requests startup composition; launcher adapters do not implement another composer. Consumers must deploy the schema, generated code, immutable source cohort and ID-only role declarations together. Existing live cohorts remain pinned until the replacement is checked.
