# codex-harness: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/codex-harness.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/codex-harness.md
+++ b/skills/codex-harness.md
@@ -1,7 +1,9 @@
 ---
 description: Invoking, seizing, or reasoning about the OpenAI Codex CLI harness: its base instructions, developer instructions, AGENTS.md, and what persists outside them.
-dependencies: [context-strata]
+dependencies: [context-strata, operators-notes]
 ---
 
+Use operators-notes to read or compose the operational records below.
+
 Codex's top stratum is the base instructions, sent as the
 instructions field of the Responses API request, above the whole
@@ -27,4 +29,21 @@ instructions marker and cannot override base instructions. Tool
 results and the machine's own output are bottom stratum.
 
+Codex renders a skills catalog into the session instructions: a name, a
+description and a location for each skill it discovers. The catalog is the
+whole of what the machine knows about them. A skill is withheld by an
+`agents/openai.yaml` beside its `SKILL.md` declaring
+`policy: allow_implicit_invocation: false`; the entry then does not appear,
+and unrecognised frontmatter keys change nothing.
+
+A withheld skill still reaches the machine when the client places it there.
+The turn request's input array takes a skill item carrying a name and a
+path, and that item expands into the turn as the file's text ahead of the
+prompt's own words. This is how a launcher seats a skill the catalog does
+not offer.
+
+A subflow renders its own catalog and so cannot see a withheld skill. The
+base instructions also require the main session to read skill instructions
+itself rather than delegate that reading.
+
 The living and the machine both read Codex's base instructions:
 the model catalog cache and the open source carry the stock text,
@@ -34,2 +53,8 @@ Replacing the base instructions changes only what the main session
 is told. The guardian safety layer is a separate model session
 with its own prompt, untouched by any base-instruction override.
+
+## Operators' notes
+
+### 2026-09-17 — Flow identity helper arguments
+
+`execution-failure / flow-identity`: in flow 99f9f7, the installed `flow-id` argument parser returned usage and exit 2 when the Codex invocation included `--parent-session`. Its usage assigned that flag to the Claude form. The subsequent Codex invocation with only the explicit flows root succeeded and returned `99f9f7`; flow identity was established. This was a CLI argument failure, with no classifier or permission refusal in either result. Helper version: unknown. Evidence: flow 99f9f7's directly witnessed exec result chunks `dce0bf` and `ff0327`. Attention: pending; note acceptance: awaiting-glance.
````

## Each edit

### 618f2d7, 2026-09-17 12:02 -0600: Add operators notes with attention tiers and categorized harness blocks

Made by: no model trailer (a Codex seat by the audit reading).

Flow 9993b5.
View: drop. One dated incident of the flow-id argument parser; incident residue.

````diff
--- a/skills/codex-harness.md
+++ b/skills/codex-harness.md
@@ -2,5 +2,7 @@
 description: Invoking, seizing, or reasoning about the OpenAI Codex CLI harness: its base instructions, developer instructions, AGENTS.md, and what persists outside them.
-dependencies: [context-strata]
+dependencies: [context-strata, operators-notes]
 ---
 
+Use operators-notes to read or compose the operational records below.
+
 Codex's top stratum is the base instructions, sent as the
@@ -35 +37,7 @@ is told. The guardian safety layer is a separate model session
 with its own prompt, untouched by any base-instruction override.
+
+## Operators' notes
+
+### 2026-09-17 — Flow identity helper arguments
+
+`execution-failure / flow-identity`: in flow 99f9f7, the installed `flow-id` argument parser returned usage and exit 2 when the Codex invocation included `--parent-session`. Its usage assigned that flag to the Claude form. The subsequent Codex invocation with only the explicit flows root succeeded and returned `99f9f7`; flow identity was established. This was a CLI argument failure, with no classifier or permission refusal in either result. Helper version: unknown. Evidence: flow 99f9f7's directly witnessed exec result chunks `dce0bf` and `ff0327`. Attention: pending; note acceptance: awaiting-glance.
````

### 3ab0251, 2026-09-24 13:17 -0600: Explain skill visibility and user-prompt activation in the harness skills

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 752e0f, on the same words of the living as claude-harness 3ab0251.
View: candidate. How Codex withholds a skill (the `agents/openai.yaml` policy sidecar) and how a launcher seats one; the living asked for it.

````diff
--- a/skills/codex-harness.md
+++ b/skills/codex-harness.md
@@ -30,2 +30,19 @@ results and the machine's own output are bottom stratum.
 
+Codex renders a skills catalog into the session instructions: a name, a
+description and a location for each skill it discovers. The catalog is the
+whole of what the machine knows about them. A skill is withheld by an
+`agents/openai.yaml` beside its `SKILL.md` declaring
+`policy: allow_implicit_invocation: false`; the entry then does not appear,
+and unrecognised frontmatter keys change nothing.
+
+A withheld skill still reaches the machine when the client places it there.
+The turn request's input array takes a skill item carrying a name and a
+path, and that item expands into the turn as the file's text ahead of the
+prompt's own words. This is how a launcher seats a skill the catalog does
+not offer.
+
+A subflow renders its own catalog and so cannot see a withheld skill. The
+base instructions also require the main session to read skill instructions
+itself rather than delegate that reading.
+
 The living and the machine both read Codex's base instructions:
````

