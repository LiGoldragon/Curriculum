# Curriculum

A versioned context registry and Nexus for psyche-belief, mind-belief and field-belief. It resolves module IDs and composes startup and role prompts in memory. See ARCHITECTURE.md for the declared roots, wire contract, placement, hashes and guard boundaries; signal-curriculum-nexus is the wire contract, and curriculum.ethos, memory.ethos and emission.ethos the authoritative schema of Curriculum's own files, its memory and its one external call.


A module is named `kind:topic:number` for a singleton or `kind:topic:first-last` for an inclusive increasing span. Its source file is named `number.md` or `first-last.md`, respectively, and exactly one declaration owns that file. Declarations use `Single.number` or `Span.{ first last }`. A numeric caller or dependency binding receives the singleton and every declared span containing that number; an explicit span binding selects that exact span. Generated skill names retain the actual complete three-field identity.
The Nexus accepts no caller-selected context file or inline prompt, and performs no legacy skills-directory lookup or file projection. There is no implicit context source. Flow requests startup composition; launcher adapters do not implement another composer. Consumers must deploy the schema, generated code, immutable source cohort and ID-only role declarations together. Existing live cohorts remain pinned until the replacement is checked.

## Publication admission

The optional `curriculum-publication` binary accepts a local Git publication
announcement. Build it with the `datom` feature. Its one inline configuration
Datom is `{ /path/to/socket /path/to/admissions 1001 5000 None }`: the socket,
private admission directory, authorized receiver UID, and exchange timeout in
milliseconds, and an optional readiness marker (`Some./path/to/ready`).
The listener creates the marker only after successfully binding the socket.
The deployment supplies the directories and owns socket
lifecycle. The binary acknowledges an exact publication by digest after
durable admission. Replaying the same announcement returns the same receipt.
Admission is the handoff only; it does not project skills or role packets.

## Role packet projection

`curriculum-project` takes one Datom request:
`ProjectRoles.{ /path/to/curriculum.sock /path/to/workspace [ { { read-trivial CodexAgent } .codex/agents/read-trivial.toml } ] }`.
Build it with the `datom` feature. It requests each `ComposeRole` over the
same Signal contract as `curriculum`, then writes the configured role
packets and their inventory and provenance under `skills/`. The request
provides the full set of destinations owned by the projector; omitted
previously owned outputs are retired. A composition refusal leaves the
workspace untouched. A successful run returns `RolesProjected.<count>`.
