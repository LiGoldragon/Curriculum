# skill-designing: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/skill-designing.md` is the baseline text with today's rulings applied (see below). Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Today's rulings applied in the recovered text

Regeneration (record 8904b1-10: "I want the skills to be fucking recommitted when they're changed and regenerated"): added "A changed skill is committed and pushed at once; the generated skill trees are then regenerated from it and committed, so every flow receives it."
Kinds of skills (record 8904b1-17): added under "Skill types" four lines: gold skills unprefixed, the living's vision of the desired result, changed only on his word; `operation-` skills deployed on the living's description, reviewed and interpreted by the primary Mind seat, no glance needed; `test-` and `compensation-` skills written by flows. No documentation kind was made (the living is undecided). No skill in the recovered set carries a prefix, so no reference had to follow the renames.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/skill-designing.md
+++ b/skills/skill-designing.md
@@ -51,8 +51,8 @@ Target-specific text in a flat source uses `{% if claude %}`, `{% if codex %}`,
 ## Skill types
 
-`user-only: true` — the skill enters only through the user's
-typed prompt; the flow cannot load it. It deploys as
-`disable-model-invocation: true` in Claude Code and as
-$-name-only injection in Codex.
+`user-only: true` — the skill enters only through the user prompt or a
+launcher's first turn; the flow cannot load it. It deploys as
+`disable-model-invocation: true` in Claude Code, and in Codex as a policy
+sidecar beside the skill that withholds it from the skills catalog.
 
 A role skill carries an aspect's identity and names its
@@ -60,2 +60,6 @@ dependencies. Mark role skills user-only.
 
 A skill's reasoning and concepts live in a parallel <skill>-rationale skill, loaded by psyche-facing flows only.
+
+## Authority by prefix
+
+A skill's prefix says who stands behind it. `testing-` is machine-generated and mostly unreviewed: field-level authority, approved by Mind automatically. `operational-` has been reviewed and approved by the psyche: mind-level authority. No prefix means approved by the living, or by the psyche in words that match the case exactly, which counts as the living's approval. A skill stays within a couple of hundred lines; one that grows past that is split by topic.
````

## Each edit

### 903b2fa, 2026-09-21 13:56 -0600: Add authority-by-prefix and refresh-payload rules

Made by: no model trailer (a Codex seat by the audit reading).

Flow 1b8ac0, on the living's words of 2026-09-21 (flows/1b8ac0/vision/skills.md), which the landed wording goes beyond ("`operational-` has been reviewed and approved by the psyche").
View: superseded. The living settled the kinds of skills on 2026-09-28 (record 8904b1-17); that settlement is applied in the recovered skill-designing as one of today's rulings. The "refresh payload" part: drop.

````diff
--- a/skills/skill-designing.md
+++ b/skills/skill-designing.md
@@ -61 +61,5 @@ dependencies. Mark role skills user-only.
 A skill's reasoning and concepts live in a parallel <skill>-rationale skill, loaded by psyche-facing flows only.
+
+## Authority by prefix
+
+A skill's prefix says who stands behind it. `testing-` is machine-generated and mostly unreviewed: field-level authority, approved by Mind automatically. `operational-` has been reviewed and approved by the psyche: mind-level authority. No prefix means approved by the living, or by the psyche in words that match the case exactly, which counts as the living's approval. A skill stays within a couple of hundred lines; one that grows past that is split by topic.
````

### 3ab0251, 2026-09-24 13:17 -0600: Explain skill visibility and user-prompt activation in the harness skills

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 752e0f, same session.
View: candidate. The Codex half of `user-only` changed from "$-name-only injection" to a policy sidecar; the baseline sentence is no longer true of the generator.

````diff
--- a/skills/skill-designing.md
+++ b/skills/skill-designing.md
@@ -52,6 +52,6 @@ Target-specific text in a flat source uses `{% if claude %}`, `{% if codex %}`,
 
-`user-only: true` — the skill enters only through the user's
-typed prompt; the flow cannot load it. It deploys as
-`disable-model-invocation: true` in Claude Code and as
-$-name-only injection in Codex.
+`user-only: true` — the skill enters only through the user prompt or a
+launcher's first turn; the flow cannot load it. It deploys as
+`disable-model-invocation: true` in Claude Code, and in Codex as a policy
+sidecar beside the skill that withholds it from the skills catalog.
````

