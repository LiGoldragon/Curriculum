# skills — architecture

*Generator source for workspace skill and role surfaces.*

## TL;DR

This repository owns flat skill sources, output manifests, and the Rust
generator that assembles harness-native skill and role files into consuming workspaces.
The active surface is manifest-driven: active skill outputs are listed in one NOTA
manifest, roles are the permission-by-depth cross product declared in three role
manifests, module source paths and dependencies live in sidecar NOTA indexes, and
generated files are written into the workspace root passed to the CLI.

The generator treats instruction prose as reusable source material. Harness
metadata and output identity live in manifests, while flat markdown sources stay
focused on the instruction body they contribute to generated files.
Generated role packets are the normal runtime doctrine bundle: the role body
is emitted with curated included modules and dependency-expanded modules, so
workers do not discover doctrine through a runtime index.

## Source Surfaces

- `skills/<name>.md`: flat source files for runtime skills and role-packet components.
- `manifests/active-outputs.nota`: active `Skill` outputs; presence means active.
- `manifests/module-dependencies.nota`: module identifier, source path, dependency module identifiers, and explicit source module kind (`RuntimeSkill` or `RoleComposition`).
- `manifests/target-module-insertions.nota`: target-specific module overlays keyed by base module and output surface.
- `manifests/universal-role-modules.nota`: the `general-instructions` and `tenets` modules included in every generated role packet.
- `manifests/skill-module-compositions.nota`: typed ordered modules appended to a named active skill after its primary module.
- `manifests/model-catalog.nota`: each model's identifier, provider surface, and accepted effort levels; an empty accepted list means the model accepts no effort level.
- `manifests/role-permissions.nota`: each permission's identifier, body text, and tool restriction.
- `manifests/role-depths.nota`: each depth's identifier and its Claude and ChatGPT model with optional effort.
- `manifests/role-descriptions.nota`: one description per permission-by-depth cell.
- `schema/assembly.schema`: schema-authored generator interface source.
- `src/schema/assembly.rs`: generated Rust interface from `schema/assembly.schema`.

## Output Targets

Skill targets:

- `AgentsSkill`: `.agents/skills/<name>/SKILL.md`, shared by Pi and Codex.
- `ClaudeSkill`: `.claude/skills/<name>/SKILL.md`.

Role targets, where `<role>` is `<permission>-<depth>`:

- `ClaudeAgent`: `.claude/agents/<role>.md`.
- `CodexAgent`: `.codex/agents/<role>.toml`.
- `PiAgent`: `.pi/agents/<role>.md`.

Derived inventory:

- `skills/generated-role-outputs.nota`: stale generated role cleanup inventory.

Visualization:

- `visualize-skills`: a non-writing, manifest-derived NOTA report of role dispatch
  kind, target packet composition, and every virtual generated output's bytes and
  newline count.

## Assembly Model

The active skill surface is manifest-owned: one active-outputs manifest lists
generated `Skill` outputs, where presence means active; sidecar indexes map
module identifiers to source paths, dependencies, target overlays, and universal
role modules. `skills/general-instructions.md` and `skills/tenets.md` provide
universal cross-agent role doctrine; skill-, repository-, and harness-specific
instruction stays in its owning source.

Roles are generated, not authored. Every permission in `role-permissions.nota`
is crossed with every depth in `role-depths.nota`, producing a role named
`<permission>-<depth>` whose description comes from the matching cell in
`role-descriptions.nota`. A missing, duplicated, or out-of-product cell fails
generation. Each role packet opens with its own generated body: a permission
that carries body text places that text before the shared closing body, and a
permission with no body text emits the shared body alone. Universal role modules
and their target insertions follow.

A depth names one Claude model and one ChatGPT model, each with an optional
effort level. The model catalog decides validity: a model that accepts no effort
level must be paired with none, a model that accepts effort levels must be paired
with one it accepts, and a model assigned to the wrong provider surface fails
generation. `ClaudeAgent` renders the Claude model, `CodexAgent` renders the
ChatGPT model bare, and `PiAgent` renders the ChatGPT model provider-qualified.
A restricted permission blocks the editing tools by the name each harness uses;
Codex role files carry no tool field, so their restriction is not expressible.

Assembly is ordered concatenation of source modules after manifest expansion.
For skills, the active skill's module expands through the dependency index,
target-specific insertions, and any typed ordered skill composition.

Module dependencies are typed by module identifier rather than inferred from
markdown links or filesystem layout. The dependency index also carries source
module kind. `RuntimeSkill` modules may emit as first-class skills, and
`RoleComposition` modules are generator-only role packet components that may be
dependency-expanded into roles but cannot be emitted as runtime skills. Target
insertions are data, not model choice: a base module, output surface, and
inserted module list determine which overlay appears in a generated harness
surface. Universal role modules and typed skill compositions are data, not
repeated prose; the generator includes them in the owning packet or skill.

## Ownership Boundaries

Source markdown owns reusable instruction body. Manifests own generated output
identity, target surfaces, descriptions, tiers, harness metadata, and role
model and permission data.

Generated outputs carry the harness-required frontmatter or TOML wrapper, but
they carry no provenance header. The source repository is the provenance.

Active sources are manifest-indexed flat files. A deliberately preserved source
outside active composition is inactive and cannot generate a runtime surface.
Removed active sources have no archive or compatibility model.

## Constraints

- The generator is a Rust CLI.
- Generator inputs are NOTA where practical, including the active manifest,
  module dependency index, target module insertion index, and universal role module manifest.
- Generator outputs are NOTA where applicable, including generated-role inventory files.
- Interfaces are schema-authored in `schema/assembly.schema`; Rust schema types are generated, not hand-authored in parallel.
- Normalization changes only structure required for valid output: one frontmatter block, heading levels, relative links, and duplicate-title handling.
- Prose is preserved through generation.
- Duplicate headings or sections fail generation.
- Generated outputs carry no provenance headers.
- Generated outputs are written into consuming workspaces and committed there.
- Role packet target directories are path-owned rather than directory-owned; stale role cleanup removes only paths listed in `skills/generated-role-outputs.nota`.

## Code Map

- `src/assembly.rs`: manifest loading, validation, module expansion, generated output planning, cleanup inventory, and rendering coordination.
- `src/markdown.rs`: markdown normalization and relative-link rebasing.
- `src/schema/assembly.rs`: generated Rust schema interface.
- `tests/generation.rs`: generation, stale cleanup, manifest, dependency, and validation witnesses.

## See Also

- `AGENTS.md` — repository operating rules.
- `README.md` — command entry points and generated surface overview.
