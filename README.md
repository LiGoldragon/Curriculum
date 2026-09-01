# Curriculum

Curriculum is the canonical data root for reusable agent instruction sources
and role definitions. It is consumed by an external runtime; it contains no
runtime, generator, deployment configuration, or generated consumer output.

`skills/*.md` holds the 38 independently described skill sources. Each source
owns its frontmatter description and instruction body.

`roles.datom` is the complete canonical role record. Its positional fields are
role modules, models, permissions, depths, descriptions, aliases, universal
role-module identifiers, and target module insertions. The two universal
instruction bodies are role-module data rather than standalone skill sources.

The user-only `main-flow` role claims its shared flow identity through the
installed `flow-id` harness helper before its first artifact. It then gives
`FLOW_ID` and `FLOW_DIRECTORY` to every child; child threads never claim a
lane.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the data contract and
[UPGRADES.md](UPGRADES.md) for the runtime cutover.
