---
description: The main flow needs to reach into the harness or the cluster to locate, probe, message, or read — recurring mechanical work.
dependencies: [vocabulary, subflow]
---

A subflow script is a subflow with a registered name, a fixed brief, a fixed return shape, and explicit noise-filter rules.

The main flow never sees a UUID, a session id, a rollout path, or a long hash. Every subflow script keeps those inside itself and returns only semantic outcomes — a word for what happened, a count, a role name, a yes/no. When a downstream call needs an id the script found, the same script makes the call.

A script's brief lives here, in this skill. New scripts are proposed to the living before landing.

## Catalogue

### find-codex-session
Argument: a name substring or a hint about which Codex thread is wanted.
Task: identify the matching Codex root thread on the local app-server.
Return shape: `{ found: yes|no|ambiguous, name: <session name>, note?: <one line if ambiguous> }`. Never returns the UUID or rollout path.

### queue-to-codex
Arguments: a name substring or hint identifying the target thread, and the message text.
Task: resolve the target, `codex queue --thread <UUID> --message <TEXT>`, verify the enqueue landed as a user-stratum record (rollout tail or queue DB payload typed `UserInput`).
Return shape: `{ ok: yes|no, stratum: user|other, delivered: yes|queued, reason?: <one line if not ok> }`. Never returns the message id or thread id.

### read-transcript-tail
Arguments: a hint identifying the thread, and how many recent records to consider.
Task: read the tail of the matching rollout, skip tool-call/tool-result records, and return the most recent semantic events.
Return shape: `{ role: user|assistant|system|other, kind: <event kind>, excerpt: <=200 chars }` per record, oldest last. Never returns file paths, offsets, or ids.

## Noise filter

Every subflow script strips these from its return:
- Full UUIDs (`01a0…`, `xxxxxxxx-xxxx-…`). Report presence only ("resolved", "not found"), not the value.
- Absolute filesystem paths under `~/.codex`, `~/.claude`, `/tmp/…`, `/nix/store/…`.
- SHA-256, SHA-1, or any hex run of 16 or more characters.
- Base64 blobs, jsonl offsets, byte counts.

When a caller genuinely needs one of these, the script makes the call itself and returns the outcome word.

## Permission

Subflow scripts do not automatically inherit the main flow's allow-list. Each script that needs a shape not covered by the default subflow profile must declare it, and the living must approve the profile before the script runs on that shape.
