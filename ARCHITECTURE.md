# Curriculum architecture

Curriculum owns the skill registry implementation and its CLI/Nexus boundary.
Skill bodies remain authored Markdown in three repositories:

- Psyche: `psyche-skills/skills/*.md`
- Mind: `mind-skills/skills/*.md`
- Field: `field-skills/skills/*.md`

Each filename is the skill name. Frontmatter `dependencies` names other skills.
The registry is keyed by canonical source path, reads the current file bodies,
and validates unique names and complete acyclic dependency graphs. Source data
is not compiled into the Rust binary.

## Requests

`curriculum 'ResolveSkills.[ roots ]'` parses one Datom query at the CLI
boundary, sends the typed query to the local Nexus, and prints one Datom
response. A successful resolution returns a dependency-first list with each
skill once. Launcher callers put `operation-main-flow` first after resolution.

An `EditSkills` signal carries three vectors of source paths: new, edited, and
deleted, plus a typed role packet plan parsed by the CLI from `roles.datom`.
The Nexus rereads the current skill files, validates the resulting registry,
computes changed/new/deleted skill names, and regenerates the workspace. Skill
bodies do not travel in the signal. The role plan carries standing names and
role module metadata; the Nexus resolves each selected name through the live
registry and appends every body in dependency-first order, once. A rejected
projection keeps the previous registry active and reports its reason on CLI
stderr.

`RebuildSkills` projects the loaded registry and role plan without changing
their source data. `CheckSkills` rereads all three authored repositories and
compares the current sources, five generated skill trees, role packet files,
role manifest, and standing selection without writing. It rejects when a
skill source changed without an `EditSkills` signal or when any projection is
stale.

The Nexus handles only typed archived signals. It does not parse Datom. The
CLI reads `roles.datom`, parses the previous generated-role manifest, and
passes the resulting plan to the Nexus as typed fields.

## Projections

The workspace receives the registry in five generated trees:

- `.agents/skills`
- `.claude/skills`
- `.codex/skills`
- `.pi/skills`
- `.opencode/skills`

Projection renders harness-specific frontmatter and user-only invocation
policy. It stages the complete output before replacing the five skill trees
and role output files. The regular immutable check compares generated files
and names with current authored sources.

## Role data

`roles.datom` remains the role configuration record. Its positional fields are
role modules, models, permissions, depths, descriptions, aliases, universal
module identifiers, target module insertions, and standing skill names. The
CLI converts that record to a typed role plan. The Nexus validates generated
role paths, composes each packet, and replaces stale files listed by the prior
manifest. The authored `subagents/book.md` procedure is copied without
reflowing its contents.
