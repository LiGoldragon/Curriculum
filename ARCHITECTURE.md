# skills — architecture

*Generator source for workspace skill and role surfaces.*

## TL;DR

This repository owns flat skill sources, output manifests, and the Rust
generator that assembles harness-native skill and role files into consuming workspaces.
The active surface is manifest-driven: active skill outputs are listed in one DOTOS
manifest, roles are the permission-by-depth cross product declared in three role
manifests, module source paths and kinds live in a sidecar DOTOS index, and
generated files are written into the workspace root carried by the CLI's one
DOTOS argument.

The generator treats instruction prose as reusable source material. Harness
metadata and output identity live in manifests, while flat markdown sources stay
focused on the instruction body they contribute to generated files.
Generated role packets are the normal runtime doctrine bundle: the role body
is emitted with curated included modules and dependency-expanded modules, so
workers do not discover doctrine through a runtime index.

## Source Surfaces

- `skills/<name>.md`: flat source files for runtime skills and role-packet components. Their leading frontmatter owns a runtime skill's description and every module's dependency list.
- `manifests/active-outputs.dotos`: active `Skill` outputs; presence means active.
- `manifests/module-dependencies.dotos`: module identifier, source path, and explicit source module kind (`RuntimeSkill` or `RoleComposition`).
- `manifests/target-module-insertions.dotos`: target-specific module overlays keyed by base module and output surface.
- `manifests/universal-role-modules.dotos`: the `general-instructions` and `tenets` modules included in every generated role packet.
- `manifests/skill-module-compositions.dotos`: typed ordered modules appended to a named active skill after its primary module.
- `manifests/model-catalog.dotos`: each model's identifier, provider surface, and accepted effort levels; an empty accepted list means the model accepts no effort level.
- `manifests/role-permissions.dotos`: each permission's identifier, body text, and tool restriction.
- `manifests/role-depths.dotos`: each depth's identifier and its Claude and ChatGPT model with optional effort.
- `manifests/role-descriptions.dotos`: one description per permission-by-depth cell.
- `src/schema/assembly.rs`: explicit Dotos assembly contract used by the generator.

## Target Conditionals

A flat source may gate lines on the harness the output is rendered for:

```
{% if codex %}
Codex-only line.
{% endif %}
```

The grammar is closed. Only `{% if <target> %}`, `{% else %}`, and
`{% endif %}` are accepted, each alone on its own line apart from indentation,
where `<target>` is `claude`, `codex`, or `pi`. Expressions, filters, loops,
`{{ }}`, includes, and `{% raw %}` are rejected before rendering, and an unknown
target name fails generation naming the file, the line, and the known targets.
Rendering runs with `UndefinedBehavior::Strict`.

Target is one value per output surface rather than a set of flags, so exactly one
target is true in every render by construction. A fragment containing no brace is
read verbatim, so a source without a conditional cannot change through templating.

Because a stray space defeats a marker scan — `{ % if codex % }` is not a tag and
would ship to an agent as doctrine — generation fails if any brace survives into
a generated file. Consequently no source may contain a brace outside the
conditional grammar, including inside fenced code blocks. This document and
`AGENTS.md` are not generated outputs and carry the syntax instead.

## Output Targets

Skill targets:

- `AgentsSkill`: `.agents/skills/<name>/SKILL.md`, shared by Pi and Codex, and
  rendered for Codex. A Codex-only block on this surface therefore also reaches
  Pi, which is accepted only while Pi is unused; a returning Pi needs its own
  skill destination.
- `ClaudeSkill`: `.claude/skills/<name>/SKILL.md`.

Role targets, where `<role>` is `<permission>-<depth>`:

- `ClaudeAgent`: `.claude/agents/<role>.md`.
- `CodexAgent`: `.codex/agents/<role>.toml`.
- `PiAgent`: `.pi/agents/<role>.md`.

Derived inventory:

- `skills/generated-role-outputs.dotos`: stale generated role cleanup inventory.

Visualization:

- `visualize-skills`: a non-writing, manifest-derived DOTOS report of role dispatch
  kind, target packet composition, and every virtual generated output's bytes and
  newline count.

## Assembly Model

The active skill surface is manifest-owned: one active-outputs manifest lists
generated `Skill` outputs, where presence means active; the source frontmatter
maps each module to its dependencies while sidecar indexes map identifiers to
source paths, kinds, target overlays, and universal
role modules. `skills/general-instructions.md` and `skills/tenets.md` provide
universal cross-agent role doctrine; skill-, repository-, and harness-specific
instruction stays in its owning source.

Roles are generated, not authored. Every permission in `role-permissions.dotos`
is crossed with every depth in `role-depths.dotos`, producing a role named
`<permission>-<depth>` whose description comes from the matching cell in
`role-descriptions.dotos`. A missing, duplicated, or out-of-product cell fails
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

Module dependencies are declared by module identifier in source frontmatter rather
than inferred from markdown links or filesystem layout. The dependency index carries
source module kind. `RuntimeSkill` modules may emit as first-class skills, and
`RoleComposition` modules are generator-only role packet components that may be
dependency-expanded into roles but cannot be emitted as runtime skills. Target
insertions are data, not model choice: a base module, output surface, and
inserted module list determine which overlay appears in a generated harness
surface. Universal role modules and typed skill compositions are data, not
repeated prose; the generator includes them in the owning packet or skill.

## Ownership Boundaries

Source markdown owns reusable instruction body, skill descriptions, and dependencies.
Manifests own generated output identity, target surfaces, tiers, harness metadata,
and role model and permission data. Generated skill descriptions preserve the source
description and name declared dependencies.

Generated outputs carry the harness-required frontmatter or TOML wrapper, but
they carry no provenance header. The source repository is the provenance.

Active sources are manifest-indexed flat files. A deliberately preserved source
outside active composition is inactive and cannot generate a runtime surface.
Removed active sources have no archive or compatibility model.

## Constraints

- The generator is a Rust CLI.
- Every generator entry point — the `skills` binary and the `generate-skills`,
  `check-skills`, and `visualize-skills` flake apps that wrap it — takes
  exactly one argument: a DOTOS payload carrying its fully typed configuration
  (an inline literal or a `.dotos` file path), per
  `standard-component-architecture.md`. No entry point accepts a bare
  workspace-root path or any other flag; a stray argument fails DOTOS decoding
  rather than being silently treated as a path.
- Generator inputs are DOTOS where practical, including the active manifest,
  module kind index, target module insertion index, and universal role module manifest.
- Generator outputs are DOTOS where applicable, including generated-role inventory files.
- The Dotos assembly interface is expressed directly in its Rust binding; there is
  one current contract, with no parallel schema source or generated artifact.
- Normalization changes only structure required for valid output: one frontmatter block, heading levels, relative links, and duplicate-title handling.
- Prose is preserved through generation.
- Duplicate headings or sections fail generation.
- Generated outputs carry no provenance headers.
- Generated outputs are written into consuming workspaces and committed there.
- Role packet target directories are path-owned rather than directory-owned; stale role cleanup removes only paths listed in `skills/generated-role-outputs.dotos`.

## Code Map

- `src/assembly.rs`: manifest loading, validation, module expansion, generated output planning, cleanup inventory, and rendering coordination.
- `src/markdown.rs`: markdown normalization and relative-link rebasing.
- `src/template.rs`: render targets, the closed conditional grammar, and blank-line collapse.
- `src/schema/assembly.rs`: current Dotos assembly interface.
- `tests/generation.rs`: generation, stale cleanup, manifest, dependency, and validation witnesses.

## See Also

- `AGENTS.md` — repository operating rules.
- `README.md` — command entry points and generated surface overview.
