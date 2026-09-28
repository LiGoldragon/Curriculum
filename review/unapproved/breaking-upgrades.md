# breaking-upgrades: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/breaking-upgrades.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/breaking-upgrades.md
+++ b/skills/breaking-upgrades.md
@@ -7,2 +7,6 @@ Document how to deploy each breaking change in the repository's `UPGRADES.md`.
 Land the documentation with the breaking change.
 If deployment fails or partially fails, correct the documentation before continuing.
+Before deploying a breaking change to production, arm an automatic countdown rollback to the last known working stack.
+Cancel the countdown only after a witness confirms that both network connectivity and remote access work on the new stack, from the host itself or a watch flow on another host.
+Do not deploy a change that can lose remote access without that timeout.
+Record the rollback target, timeout, cancellation authority, and witness in `UPGRADES.md`.
````

## Each edit

### 0a62275, 2026-09-15 14:22 -0600: Add production rollback countdown guidance

Made by: no model trailer (a Codex seat by the audit reading).

Flow fd0f97/05c604. The living: "I approve the countback rollback skill" (relayed by a secondary flow).
View: candidate. An automatic countdown rollback before a breaking production deploy; the living's approved lines.

````diff
--- a/skills/breaking-upgrades.md
+++ b/skills/breaking-upgrades.md
@@ -8 +8,5 @@ Land the documentation with the breaking change.
 If deployment fails or partially fails, correct the documentation before continuing.
+Before deploying a breaking change to production, arm an automatic countdown rollback to the last known working stack.
+Cancel the countdown only after a witness confirms that both network connectivity and remote access work on the new stack, from the host itself or a watch flow on another host.
+Do not deploy a change that can lose remote access without that timeout.
+Record the rollback target, timeout, cancellation authority, and witness in `UPGRADES.md`.
````

