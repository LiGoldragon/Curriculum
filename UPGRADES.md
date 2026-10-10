# Deployment

The deployable consists of the `curriculum` CLI, the `curriculum-nexus`
service, the `roles.datom` file, the consumer workspace, and the three authored
skill repositories. Set `CURRICULUM_PSYCHES_REPOSITORY_DIR` to the Psyche
repository root. Set `CURRICULUM_MIND_SKILLS_DIR` and
`CURRICULUM_FIELD_SKILLS_DIR` to the Mind and Field `skills/` directories. Set
`CURRICULUM_ROLES_FILE` to `roles.datom`, `CURRICULUM_WORKSPACE` to the
consumer workspace, and `XDG_RUNTIME_DIR` to the runtime root containing
`curriculum/curriculum.sock`.

Build the CLI with the `datom` feature. The Nexus binary uses typed archived
signals and does not need Datom. Send `RebuildSkills` when the workspace needs
its current skill and role projections. Use path-only `EditSkills` signals for
source changes within the currently configured roots. `CheckSkills` rereads
the authored roots and checks every generated skill tree and role packet
without writing.

Changing a configured repository root is a service configuration change, not
a path edit. Restart the Nexus after changing the root; `SkillMemory::open`
loads the registry from the configured roots. Then send `RebuildSkills` to
project that registry and role data. Do not send `EditSkills` paths from the
previous root after switching roots.
