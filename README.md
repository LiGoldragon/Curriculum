# skills

Source repository for generated workspace skills and role packets. Its Dotos
assembly contract is maintained directly in Rust alongside the generator.

Run the generator or checker against a consuming workspace through the repository flake.

Inspect the assembled repository without writing workspace output:

```sh
nix run github:LiGoldragon/skills#visualize-skills
```

The deterministic DOTOS report lists each generated role, shows each target
packet's ordered module composition, and lists every virtual generated output by
relative path, UTF-8 byte count, and newline count (the same line measure as
`wc -l`). The command renders from canonical manifests and sources but does not
read or write the workspace.

`generate-skills` and `check-skills` follow the same entry-point contract and
require `SKILLS_WORKSPACE_ROOT` to name an explicit consuming workspace; they
never default to the source checkout. Every generator entry
point — the flake apps and the `skills` binary they wrap — takes exactly one
argument: a DOTOS payload carrying its fully typed configuration (an inline
DOTOS literal or a path to a `.dotos` file), never a bare flag or path. For
the default request, set the workspace explicitly:

```sh
SKILLS_WORKSPACE_ROOT=/path/to/consumer nix run github:LiGoldragon/skills#generate-skills
```

`visualize-skills` is read-only and may be run from the source checkout without
setting `SKILLS_WORKSPACE_ROOT`.

To target a workspace other than the explicitly selected consumer workspace,
pass a full replacement request as the one argument, for example:

```sh
SKILLS_WORKSPACE_ROOT=/path/to/consumer nix run github:LiGoldragon/skills#generate-skills -- \
  '(Generate ($SKILLS_SOURCE_ROOT /path/to/workspace manifests/active-outputs.dotos Write))'
```

Source guidance belongs in flat `skills/*.md` files. Each source file declares
its description and dependencies in leading frontmatter. The generator validates
those dependencies and surfaces them in generated skill descriptions. Roles are generated as the
permission-by-depth cross product declared in `manifests/role-permissions.dotos`,
`manifests/role-depths.dotos`, and `manifests/role-descriptions.dotos`; there are
no role source files. Generated runtime files are deployment output.
