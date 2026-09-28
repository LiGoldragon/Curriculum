# psyche-acquisition: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/psyche-acquisition.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/psyche-acquisition.md
+++ b/skills/psyche-acquisition.md
@@ -1,4 +1,4 @@
 ---
-description: Reacquiring what the psyche has expressed.
+description: Reacquiring what the psyche has expressed, or auditing user prompts for Psyche that was not recorded.
 dependencies: [psyche]
 ---
@@ -10,5 +10,31 @@ Acquisition is not only a beginning-of-session action. Whenever a
 new topic is raised or touched upon, reacquire for that topic.
 
-## How to report
+## Capture audit
+
+Bound the sessions and time window. Enumerate original user prompts from
+Claude and Codex transcripts, including queued prompts and threads labeled
+as bootstrap or subagent. Deduplicate an enqueue record and its delivered
+user record by payload, preserving both locations. Classify authorship
+from the content and envelope; a `user` record, `human` flag, or queued
+delivery does not by itself distinguish living speech from machine relay.
+
+Search prompt openings for likely Vision or Notion language, such as "I
+want", "I think", "what if", and "we should". Read each candidate in full
+and inspect prompts without those openings for misses. Judge each passage
+as Psyche, working instruction, context, or uncertain; an opening is a
+discovery clue, not a verdict.
+
+For each Psyche passage, compare its words and provenance with the
+originating flow's raw Vision or Notion record, including archived records,
+or an explicitly approved distillation and its source pointer. An explained
+speech-to-text correction can preserve the words; a paraphrase in a flow
+log cannot. Record the prompt location, matching record location, citation
+quality, and whether the words are complete, altered, absent, or uncertain.
+Report separate counts for prompts inspected, passages judged Psyche, and
+passages captured; give coverage limits and gaps by originating flow.
+Route missing records to the flow that heard the living; preserve the
+transcript and prior records.
+
+## Topic acquisition report
 
 Return the psyche's actual expressions organized by level. Preserve
````

## Each edit

### f4d7e5b, 2026-09-21 15:04 -0600: Audit user prompt capture in psyche acquisition

Made by: no model trailer (a Codex seat by the audit reading).

Flow 753e69, with the living: "Get someone to do an audit ... searching all the user prompt for certain signs ... let's make this a standard skill." The counting and citation-grading details are the flow's.
View: candidate, as the living asked; shorten the procedure to the search, the reading of each candidate, and the comparison with the records.

````diff
--- a/skills/psyche-acquisition.md
+++ b/skills/psyche-acquisition.md
@@ -1,3 +1,3 @@
 ---
-description: Reacquiring what the psyche has expressed.
+description: Reacquiring what the psyche has expressed, or auditing user prompts for Psyche that was not recorded.
 dependencies: [psyche]
@@ -11,3 +11,29 @@ new topic is raised or touched upon, reacquire for that topic.
 
-## How to report
+## Capture audit
+
+Bound the sessions and time window. Enumerate original user prompts from
+Claude and Codex transcripts, including queued prompts and threads labeled
+as bootstrap or subagent. Deduplicate an enqueue record and its delivered
+user record by payload, preserving both locations. Classify authorship
+from the content and envelope; a `user` record, `human` flag, or queued
+delivery does not by itself distinguish living speech from machine relay.
+
+Search prompt openings for likely Vision or Notion language, such as "I
+want", "I think", "what if", and "we should". Read each candidate in full
+and inspect prompts without those openings for misses. Judge each passage
+as Psyche, working instruction, context, or uncertain; an opening is a
+discovery clue, not a verdict.
+
+For each Psyche passage, compare its words and provenance with the
+originating flow's raw Vision or Notion record, including archived records,
+or an explicitly approved distillation and its source pointer. An explained
+speech-to-text correction can preserve the words; a paraphrase in a flow
+log cannot. Record the prompt location, matching record location, citation
+quality, and whether the words are complete, altered, absent, or uncertain.
+Report separate counts for prompts inspected, passages judged Psyche, and
+passages captured; give coverage limits and gaps by originating flow.
+Route missing records to the flow that heard the living; preserve the
+transcript and prior records.
+
+## Topic acquisition report
````

