---
description: Rotate stable and Next after consumer migration while preserving running sessions.
---

Treat a channel role and an endpoint identity as different things. An endpoint identity includes its process, service, socket, state root, client route, and running sessions.

Record the predecessor stable endpoint, running Next endpoint, and each consumer's resolved role mapping before migration.

Migrate every consumer in scope to the Next role through its declared configuration. Preserve the predecessor stable endpoint until migration is proved.

Retain an exact migration artifact that identifies every native consumer in scope and its resolved role mapping. It must prove that none resolves predecessor stable. Source declarations, staged paths, successful evaluation, or intended configuration do not prove migration.

After the migration gate, promote by mapping stable to the already-running Next endpoint. Preserve its process, service, socket, state root, client route, and running sessions. Do not restart, recreate, rename, move, copy, or share its mutable state as part of promotion.

Create separate Next only after promotion. Give it a distinct process, state root, socket, and client route.

Name each endpoint's socket by its role and the short hash of its version, so promotion moves the role without renaming the socket.

A Next older than the running stable is never installed.

Preserve default launch behavior by proving that the ordinary launcher resolves stable and the explicit Next launcher resolves separate Next.

Use the component-supported authentication mechanism without exposing credential contents to the flow or retained evidence. If no supported mechanism can authenticate separate Next, stop before creation and report the prerequisite.

Before declaring success, retain an exact promotion artifact with pre- and post-promotion endpoint identities, the migration gate, default-launch result, explicit-Next result, and actual capability and model verification against the running endpoints.

Do not claim success from a source edit, staged artifact, evaluation, build, or unreachable endpoint.
