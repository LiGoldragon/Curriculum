# skills

Source repository for generated workspace skills and role packets.

Run the generator or checker against a consuming workspace through the repository flake.

Inspect the assembled repository without writing workspace output:

```sh
nix run github:LiGoldragon/skills#visualize-skills -- <workspace-root>
```

The deterministic NOTA report lists each generated role, shows each target
packet's ordered module composition, and lists every virtual generated output by
relative path, UTF-8 byte count, and newline count (the same line measure as
`wc -l`). The command renders from canonical manifests and sources but does not
read or write `<workspace-root>`.

Source guidance belongs in flat `skills/*.md` files. Roles are generated as the
permission-by-depth cross product declared in `manifests/role-permissions.nota`,
`manifests/role-depths.nota`, and `manifests/role-descriptions.nota`; there are
no role source files. Generated runtime files are deployment output.
