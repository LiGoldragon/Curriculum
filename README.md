# skills

Source repository for generated workspace skills and role packets.

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
by default run against the current working directory. Every generator entry
point — the flake apps and the `skills` binary they wrap — takes exactly one
argument: a DOTOS payload carrying its fully typed configuration (an inline
DOTOS literal or a path to a `.dotos` file), never a bare flag or path. To
target a workspace other than `$PWD`, pass a full replacement request as that
one argument, for example:

```sh
nix run github:LiGoldragon/skills#generate-skills -- \
  '(Generate ($SKILLS_SOURCE_ROOT /path/to/workspace manifests/active-outputs.dotos Write))'
```

Source guidance belongs in flat `skills/*.md` files. Each source file declares
its description and dependencies in leading frontmatter. The generator validates
those dependencies and surfaces them in generated skill descriptions. Roles are generated as the
permission-by-depth cross product declared in `manifests/role-permissions.dotos`,
`manifests/role-depths.dotos`, and `manifests/role-descriptions.dotos`; there are
no role source files. Generated runtime files are deployment output.
