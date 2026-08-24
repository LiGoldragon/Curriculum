# Upgrades

## Data-only cutover

Curriculum no longer carries its generator runtime or the legacy DOTOS assembly
manifests. Its canonical data root is `skills/*.md` and `roles.datom`.

The replacement runtime is the public `curriculum-deploy` repository. The
cutover was validated against this exact data root at public revision
`c64223acb7d38b53968b55701f2ded93e82587c1`. Runtime maintainers must use a
runtime revision that reads this Datom root before pointing a deployment at it.
Do not use the retired Cargo, Nix, CLI, DOTOS request, manifest, or
generated-inventory surfaces from this repository.

The two former role-composition instruction files are now embedded role modules
in `roles.datom`; consumers must read them from the `role_modules` field.
