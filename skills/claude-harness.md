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
