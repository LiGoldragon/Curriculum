# Curriculum

A versioned context registry and Nexus for psyche-belief, mind-belief and field-belief. It resolves module IDs and composes startup and role prompts in memory. See ARCHITECTURE.md for the declared roots, wire contract, placement, hashes and guard boundaries; signal-curriculum-nexus is the wire contract, and curriculum.ethos, memory.ethos and emission.ethos the authoritative schema of Curriculum's own files, its memory and its one external call.


A module is named `kind:topic:number` and stored as `kind/topic/number.md`. Every caller and dependency supplies the explicit version, including `0`; resolution selects exactly that version. Generated skill names retain the complete three-field ID.
There is no caller-selected context file, inline prompt, legacy skills directory, file projection or implicit context source. Flow requests startup composition; launcher adapters do not implement another composer. Consumers must deploy the schema, generated code, immutable source cohort and ID-only role declarations together. Existing live cohorts remain pinned until the replacement is checked.
