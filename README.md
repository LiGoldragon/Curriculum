# Curriculum

Curriculum provides the typed skill registry used by the Flow client and the
native harness launchers. The authored skill sources live in the Psyche, Mind,
and Field skill repositories, under each repository's `skills/` directory.

The `curriculum-nexus` process loads those source paths, validates skill names
and dependencies, resolves dependency closures, and regenerates the consumer
workspace through typed signals. The `curriculum` CLI parses one Datom request
and sends the typed request to the Nexus. The Nexus protocol itself contains
no Datom parsing. The CLI also parses `roles.datom` and sends the role packet
plan as typed data. The Nexus assembles each packet with the current standing
skill dependency closure. `CheckSkills` is read-only; `RebuildSkills` projects
the skill trees and role packets from current inputs.

The generated workspace includes all five skill trees, role packet files,
`skills/generated-role-outputs.datom`, and
`tools/standing-skill-selection.mjs`. Book's authored procedure is carried
byte-for-byte after the role and standing skill bodies.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the source, signal, and projection
contracts.
