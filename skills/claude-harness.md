---
description: Invoking, seizing, or reasoning about the Claude Code harness: its system prompt flags, what they replace, what persists, and where its entry files land.
dependencies: [context-strata]
---

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
argument. The command is read only at the head of the prompt, and the
first text that is not a command ends the parse, so a command written
further down a block stays literal. One block of
startup text carries a skill only when the command is its first token.

A launcher has two other routes into the first turn: a SessionStart hook
returns `initialUserMessage` or `additionalContext`, or the launcher reads
the skill file and writes its body into the first prompt.

A subflow receives no startup prompt of its own. It cannot see or load a
withheld skill; what it must carry belongs in its brief.

A request to update spawned models does not authorize changing the main-flow model.

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
