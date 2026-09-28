# psyche: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/psyche.md` is the baseline text with today's rulings applied (see below). Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Today's rulings applied in the recovered text

Conflicting records (record 8904b1-17, "For conflicting records of yours, you have it"): the baseline's "A later entry supersedes earlier entries on the same subject. Entries conflict only when simultaneous; surface a same-time conflict to the psyche." is replaced by the wording of 51d4922, which the living confirmed today.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/psyche.md
+++ b/skills/psyche.md
@@ -26,4 +26,18 @@ Where psyche lives;
 the living psyche is always called the living psyche, or the living.
 
+Psyche contains Spirit, Intent, Vision, and Notion, in descending authority.
+
+Operational vision skills use the `operational-` prefix and support faster
+iteration with an overview to the living. Testing skills use `testing-`.
+Pure vision skills use neither prefix. Distilled vision preserves references
+to its supporting raw records; archived records retain their original words
+and provenance.
+
+Psyche data belongs in a dedicated repository symlinked into Primary. Primary
+Next begins from Primary's root commit and carries selected repository mounting
+points plus a README and AGENTS.md explaining those relationships. Orchestrate
+coordinates concurrent work across those repositories. This is a target shape,
+not authorization to migrate data or rewrite history.
+
 ## Four levels
 
@@ -65,5 +79,10 @@ raw; Intent and Spirit can only be distilled.
 A topic is a noun subject an agent would guess before knowing any ruling; a statement is an entry heading inside it.
 
-A later entry supersedes earlier entries on the same subject. Entries conflict only when simultaneous; surface a same-time conflict to the psyche.
+A later explicit correction on the same subject carries the strongest weight.
+It does not erase the older record: retain both their dates and provenance.
+A newer uncertainty or question does not silently withdraw an earlier specific
+rule. When records point to incompatible actions, or it is unclear whether the
+newer words correct the earlier rule, surface the tension to the psyche rather
+than choosing by a strict supersession rule.
 
 Any agent can search psyche logs for answers. If a topic is raised
````

## Each edit

### 51d4922, 2026-09-18 08:13 -0600: Preserve and surface psyche-record conflicts in recency reviews

Made by: no model trailer (a Codex seat by the audit reading).

Flow 1ac573 (Terra), after the living's "Always audit against vision and raise conflicts" (flows/908786/vision/vision-led-audit-and-fable.md). The wording is the flow's.
View: settled by the living on 2026-09-28 (record 8904b1-17, "For conflicting records of yours, you have it"): this wording stands. It is applied in the recovered text as one of today's rulings, and psyche-distillation is brought in line.

````diff
--- a/skills/psyche.md
+++ b/skills/psyche.md
@@ -66,3 +66,8 @@ A topic is a noun subject an agent would guess before knowing any ruling; a stat
 
-A later entry supersedes earlier entries on the same subject. Entries conflict only when simultaneous; surface a same-time conflict to the psyche.
+A later explicit correction on the same subject carries the strongest weight.
+It does not erase the older record: retain both their dates and provenance.
+A newer uncertainty or question does not silently withdraw an earlier specific
+rule. When records point to incompatible actions, or it is unclear whether the
+newer words correct the earlier rule, surface the tension to the psyche rather
+than choosing by a strict supersession rule.
````

### 59acfc3, 2026-09-18 11:38 -0600: Define Psyche hierarchy and repository target shape

Made by: no model trailer (a Codex seat by the audit reading).

Flow b05237/108ab0, with the living (flows/b05237/vision/operational-skillTypesTriad.md, operational-skillIsVisionPrefixed.md: "By using a prefix, we can tell the type of skill"; operational-threeDataReposAndPrimaryNext.md).
View: split. The prefix paragraph is superseded by the living's settlement of 2026-09-28, applied in skill-designing. The repository target shape is the living's plan, not a rule: it belongs in Vision, and today's thought that the gold skills may live in their own repository joins it. The first sentence repeats "Four levels": drop.

````diff
--- a/skills/psyche.md
+++ b/skills/psyche.md
@@ -27,2 +27,16 @@ the living psyche is always called the living psyche, or the living.
 
+Psyche contains Spirit, Intent, Vision, and Notion, in descending authority.
+
+Operational vision skills use the `operational-` prefix and support faster
+iteration with an overview to the living. Testing skills use `testing-`.
+Pure vision skills use neither prefix. Distilled vision preserves references
+to its supporting raw records; archived records retain their original words
+and provenance.
+
+Psyche data belongs in a dedicated repository symlinked into Primary. Primary
+Next begins from Primary's root commit and carries selected repository mounting
+points plus a README and AGENTS.md explaining those relationships. Orchestrate
+coordinates concurrent work across those repositories. This is a target shape,
+not authorization to migrate data or rewrite history.
+
 ## Four levels
````

