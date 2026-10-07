# Deployment

The deployable consists of the `curriculum` CLI, the `curriculum-nexus`
service, the `roles.datom` file, the consumer workspace, and the three authored skill repositories. Configure their absolute
`skills/` paths with `CURRICULUM_PSYCHES_SKILLS_DIR`,
`CURRICULUM_MIND_SKILLS_DIR`, and `CURRICULUM_FIELD_SKILLS_DIR`. Set
`CURRICULUM_ROLES_FILE` to `roles.datom`, `CURRICULUM_WORKSPACE` to the
consumer workspace, and `XDG_RUNTIME_DIR` to the runtime root containing
`curriculum/curriculum.sock`.

Build the CLI with the `datom` feature. The Nexus binary uses typed archived
signals and does not need Datom. Send `RebuildSkills` when the workspace needs
its current skill and role projections, or path-only `EditSkills` signals
after skill-source changes. `CheckSkills` rereads the authored roots and checks
every generated skill tree and role packet without writing. Restarting the
Nexus reloads all three repositories; the next `RebuildSkills` recreates every
projection from those current sources and role data.
