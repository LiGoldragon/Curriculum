---
description: Invoking, seizing, or reasoning about the Claude Code harness: its system prompt flags, what they replace, what persists, and where its entry files land.
dependencies: [context-strata, operators-notes]
---

Use operators-notes to read or compose the operational records below.

Claude Code's top stratum is the system prompt. The
--system-prompt and --system-prompt-file flags replace the whole
of it; --append-system-prompt and --append-system-prompt-file add
to the end of the stock one; an output style rewrites it, layering
over the coding instructions when it says so; the SDK's system
prompt setting chooses between the minimal default, the Claude
Code preset with an optional append, and a custom text.
--exclude-dynamic-system-prompt-sections moves the per-machine
sections (working directory, environment, git status) out of the
system prompt into the first user message. --bare skips CLAUDE.md
discovery; --safe-mode disables every customization.

Claude Code has three strata. CLAUDE.md and the other entry files
are delivered as a user message after the system prompt, never
inside it: they are middle stratum, as are system-reminder
injections, skills loaded through the skill interface, and subflow
briefs. Tool results and the machine's own output are bottom
stratum.

A skill's frontmatter says who may invoke it.
`disable-model-invocation: true` withholds it from the model: the name is
absent from the available-skills listing, and the skill interface refuses
it. `user-invocable: false` withholds it from the typed command list.

A withheld skill enters through the user prompt. The harness reads a
leading `/name` from that prompt, expands the skill body itself, and
delivers it as a middle-stratum message; two records mark the turn, one
naming the command and one carrying the rest of the prompt as its
argument. The command is read only at the head of the prompt, up to five
may stack there, and the first text that is not a command ends the parse,
so a command written further down a block stays literal. One block of
startup text carries a skill only when the command is its first token.

A launcher has two other routes into the first turn: a SessionStart hook
returns `initialUserMessage` or `additionalContext`, or the launcher reads
the skill file and writes its body into the first prompt.

A subflow receives no startup prompt of its own. It cannot see or load a
withheld skill; what it must carry belongs in its brief.

The machine reads its system prompt; the living cannot, through
any channel the harness offers: debug logs, session transcripts,
JSON output, and verbose mode all omit it. The living witnesses
the stock system prompt only through the machine's transcription
of it.

Replacing the system prompt removes its behavioral guidance and
nothing else. Tool schemas travel in the API's tools parameter and
remain; the permission system, hooks, the scanning of subflow
output, entry-file injection, and the model's training persist
outside the prompt.

Claude Code stops its background tasks when the host's free memory
looks low, judging by free rather than available memory, so another
process's build can end a long task that is nowhere near its own
limit. A long-running process launched from the harness runs
detached, as a transient systemd user service or scope with its own
memory cap, and the harness watches for its end.

## Operators' notes

Flow 99f9f7 recorded these entries from saved tool results inspected by its incident-evidence subflow. References identify transcripts under `Claude transcript root` by session and result record. Harness versions and later unblock conditions are unknown. Each refused call left its requested work unperformed; later resolution is not established by these excerpts. Attention: pending per-incident acknowledgement; note acceptance: awaiting-glance.

### 2026-09-17 — Background launches

`classifier-refusal / launch`: the Claude Code auto-mode classifier refused two `claude --bg` launches at 15:06 UTC. The returned reason was "Permission for this action was denied by the Claude Code auto mode classifier. Reason: Blocked by classifier." This identifies the refusing component; its internal rationale is unknown.
Evidence: session `f55ec8ce-4aa1-45d6-9a3e-dc5bc4ed0764`, result records `01c34d77-b3a7-43a2-b077-6a111bb3b886` and `5ea58eb3-d249-4d33-9856-f546d1c3a3a1`.

### 2026-09-17 — Resume

`classifier-refusal / resume`: the Claude Code auto-mode classifier refused a `claude --bg --resume` call at 15:17 UTC with the same classifier message as the launch entries.
Evidence: parent session `f55ec8ce-4aa1-45d6-9a3e-dc5bc4ed0764`, subagent `a9f5e3c5ed9c97e76`, result record `edc5c780-6732-4395-b990-4e102befa6c0`.

### 2026-09-17 — Settings JSON

`classifier-refusal / configuration-edit`: the Claude Code auto-mode classifier refused an Edit of user settings JSON at 16:52 UTC and a Write of local settings JSON at 16:53 UTC. Both returned the classifier message above. The permission settings were the target of the rejected edits; this does not establish a permission-rule denial or provider-policy refusal.
Evidence: session `9993b5f1-d646-41a7-921a-ffaf3d02f3fe`, result records `a406a5dc-8442-47fb-898f-5b345e4d0c82` and `e649fce7-af73-4cc3-abe1-eb7c122e8b47`.

### 2026-09-16 — Repository setup, carried in the September 17 recovery record

`permission-denial / repository-setup`: Claude's worktree-isolation guard refused `jj git clone --colocate` at 18:38 UTC. The decisive returned text was "a worktree-isolated session's git operations must target its own worktree." The message said it could not determine the `jj git` operation's target. This is a workspace guard refusal; the direct result neither names the auto-mode classifier nor demonstrates that a clone started.
Evidence: parent session `f55ec8ce-4aa1-45d6-9a3e-dc5bc4ed0764`, subagent `a7ef196baa4490aa0`, result record `74f5214d-9494-4e64-8ab9-4df1e14c012e`. The direct timestamp is September 16; the September 17 recovery report carries it forward.
