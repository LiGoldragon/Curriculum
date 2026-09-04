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

## Main-flow/subflow contract

Regenerate every consumer after updating to the revision that removes `flows`
and `subflows`. Start main-flow work with `$main-flow`, then claim the main-flow
lane before the first artifact:
{% if claude %}
`flow-id claude --flows-root ABSOLUTE_DIRECTORY --parent-session "$CLAUDE_CODE_SESSION_ID"`.
{% endif %}
{% if codex %}
`flow-id codex --flows-root` with the explicit absolute flows root.
{% endif %}
Use its normalized hexadecimal alias as `FLOW_ID`, then put `$subflow`,
`FLOW_ID`, and `FLOW_DIRECTORY` in each subflow brief.
Nested subflow briefs preserve `FLOW_ID` and `FLOW_DIRECTORY` unchanged. A
subflow obtains its own `THREAD_ID` after launch for transcript and evidence
provenance.

Subflows do not create a lane, index entry, or log. Load `flow-evidence` only
for an artifact delegated by the main flow or consumed by a named tool or flow.
Existing historical flow artifacts are not a compatibility surface and require a
separately approved migration.
