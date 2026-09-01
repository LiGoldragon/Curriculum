# Curriculum — architecture

Curriculum is a pure data repository. Its canonical surface is 37 described
skill sources and one complete Datom role record. A runtime outside this
repository reads these sources and produces any harness-specific output.

## Canonical data

`skills/<name>.md` is an independently described skill source. The file's
frontmatter owns its description and its body owns its instructions.

`roles.datom` is a `Roles` record with these positional fields: role modules,
models, permissions, depths, descriptions, aliases, universal module
identifiers, and target module insertions.

A role module is `{identifier body}`. The general instruction and Codex
skill-loading bodies are role modules because they compose roles rather than
describe independently invocable skills.

`main-flow` and `child-flow` are user-only roles. The parent role owns one
shared flow identity and directory, passes them through every child brief, and
alone makes a rare flow log. The child role returns its delegated work without
creating a lane, index, or log. `flow-evidence` is loaded only when an artifact
is delegated or will be consumed; concurrent writers use separate paths or the
standard edit coordination contract.

The record keeps every role decision together: model availability, permission
policy, effort choices, role descriptions, aliases, and ordered module
composition. Its data is positional because Datom is schema-driven.

## Boundary

Curriculum does not contain a CLI, Rust code, Cargo or Nix configuration,
request fixtures, assembly manifests, templates, tests, or generated consumer
trees. Those belong to the runtime and consuming workspaces. This repository
does not maintain a parallel legacy-DOTOS representation or a generated-output
inventory.

The current harness deployment interface carries a child brief but has no
owned automatic parent-identity injection. It must therefore pass `FLOW_ID`,
`FLOW_DIRECTORY`, and `THREAD_ID` explicitly. A harness that gains a verified
injection surface may implement this contract there without changing the data
shape.

## See also

`README.md` describes the repository surface.

`UPGRADES.md` records the data-root cutover for runtime maintainers.
