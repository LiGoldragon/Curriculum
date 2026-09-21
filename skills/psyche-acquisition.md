---
description: Reacquiring what the psyche has expressed, or auditing user prompts for Psyche that was not recorded.
dependencies: [psyche]
---

Search psyche logs for the topics specified by the caller.
The psyche's typed words live in transcripts before any log; search them too.

Acquisition is not only a beginning-of-session action. Whenever a
new topic is raised or touched upon, reacquire for that topic.

## Capture audit

Bound the sessions and time window. Enumerate original user prompts from
Claude and Codex transcripts, including queued prompts and threads labeled
as bootstrap or subagent. Deduplicate an enqueue record and its delivered
user record by payload, preserving both locations. Classify authorship
from the content and envelope; a `user` record, `human` flag, or queued
delivery does not by itself distinguish living speech from machine relay.

Search prompt openings for likely Vision or Notion language, such as "I
want", "I think", "what if", and "we should". Read each candidate in full
and inspect prompts without those openings for misses. Judge each passage
as Psyche, working instruction, context, or uncertain; an opening is a
discovery clue, not a verdict.

For each Psyche passage, compare its words and provenance with the
originating flow's raw Vision or Notion record, including archived records,
or an explicitly approved distillation and its source pointer. An explained
speech-to-text correction can preserve the words; a paraphrase in a flow
log cannot. Record the prompt location, matching record location, citation
quality, and whether the words are complete, altered, absent, or uncertain.
Report separate counts for prompts inspected, passages judged Psyche, and
passages captured; give coverage limits and gaps by originating flow.
Route missing records to the flow that heard the living; preserve the
transcript and prior records.

## Topic acquisition report

Return the psyche's actual expressions organized by level. Preserve
exact meaning — do not summarize or distill. Use verbatim quotes.
