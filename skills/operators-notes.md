---
description: A harness operation is blocked, or operational experience needs preserving for later flows.
dependencies: [behavior]
---

An operator's note is a dated observation kept under its harness, with enough evidence and context for another flow to use it.

## Attention boundary

The authored instruction body defines reusable behavior and needs the living psyche's deep approval.
The `Operators' notes` section holds incident evidence and scoped operational observations; flows compose it for a glance and a "fair enough."
A note's acceptance approves that record only.
A note describes what happened, the current state, and known conditions; directives about authority, permissions, required behavior, or general policy belong in the authored instruction body and follow its approval tier.

## Compose

Append a short entry to the owning harness's `Operators' notes`, or update the same incident.
Place it under the harness session that returned the result, using the blocked command to identify the operation; leave ownership unresolved when that session is unknown.
For a block, record the date and reporting flow, attempted operation, refusing component and known mode/version, category, decisive result or reason, evidence pointer, stopped work and known unblock condition, and attention.
Use a few sentences; label unavailable facts unknown and distinguish a returned reason from an established cause.
Other notes carry the observation, its scope, evidence, and attention.
A source pointer and decisive excerpt suffice; note-taking requires neither a separate report nor a fresh reproduction.

## Block categories

Classify the enforcing boundary independently of the operation.
`classifier-refusal`: a model-based action reviewer refuses the attempted tool action.
`permission-denial`: a harness permission rule, workspace-isolation guard, sandbox, or operating-system access control denies access.
`provider-policy`: the provider explicitly rejects the operation under its policy.
`execution-failure`: the operation is admitted but fails in its program, dependency, or environment.
`unknown-block`: the available evidence does not identify the enforcing boundary.
Record the operation as `launch`, `resume`, `configuration-edit`, `repository-setup`, or a short description for another operation.
A command's subject does not establish the boundary: a settings edit refused by an action classifier is `classifier-refusal / configuration-edit`.

## Carry a block forward

A refusal ends the attempted action, including equivalent variants and delegation of that refused action.
Keep unaffected work moving; record a confirmed change in the blocking condition before considering another attempt.
Attention starts at `pending`; mark it `seen` with evidence the living saw the block.
Note acceptance stays `awaiting-glance` until a "fair enough" or equivalent acceptance is recorded.
Keep pending blocks visible in the next psyche-facing response and relevant handoffs until seen; a sent message alone is not evidence of seeing.
Re-propose the report, not the refused action; seeing a block neither resolves it nor accepts a note.

## Home

Today notes live inside `claude-harness.md`, `codex-harness.md`, or the other owning harness source under `Curriculum skills`, after the authored instruction body.
Regenerate the consumer evidence through the existing Curriculum deployment path.
The intended later home is dedicated data repositories of the Curriculum Nexus; today's notes require no running Nexus.
