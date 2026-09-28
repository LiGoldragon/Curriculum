# testing: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/testing.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/testing.md
+++ b/skills/testing.md
@@ -23,2 +23,11 @@ Tests share no mutable state — no process environment, no working
 directory, no order between them.
 A run that may exhaust memory or time is bounded (a memory cap and a timeout) so that it cannot take the harness down with it.
+Stop a process a test started by the PID that test holds, never by a process-name or path pattern — a scratch and a production instance of the same build share that pattern.
+
+Live acceptance has a boundary. A fixture or generated-output test proves its
+own contract; an isolated transport test proves only its named receipt grade;
+an end-to-end live acceptance needs the actual selected identity, binding, and
+target-side observation. Report an unavailable native route as unavailable,
+not as a failed simulation or a passing deployment test.
+
+When assigned as a testing worker, accept a bounded target, immutable revision, authority limits, and acceptance contract. Choose the test procedure, fixtures, negative cases, and independent oracle yourself; do not mirror the implementation or a main flow's assertion. Run the smallest test that can distinguish acceptance from a plausible failure, including a rejected or failing case before trusting a new test. Keep source/projection ownership, native binding and receipt, deployment parity and rollback, transport versus target read, passive no-wake observation, and context metric freshness distinct when those boundaries matter. Report what each witness proves, the exact revision and scope tested, and what remains unavailable. Do not wake a production flow merely to test status or delivery.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d, mending a 21-second outage it caused by killing a production process by pattern.
View: candidate. "Stop a process by the PID you hold, never by a name pattern" is true, cheap and cost a real outage; but it is a flow's own rule, so the living's to take or leave.

````diff
--- a/skills/testing.md
+++ b/skills/testing.md
@@ -24 +24,2 @@ directory, no order between them.
 A run that may exhaust memory or time is bounded (a memory cap and a timeout) so that it cannot take the harness down with it.
+Stop a process a test started by the PID that test holds, never by a process-name or path pattern — a scratch and a production instance of the same build share that pattern.
````

### 6c161a0, 2026-09-18 10:15 -0600: Define Field and evidence-disciplined messaging operations

Made by: no model trailer (a Codex seat by the audit reading).

Flow as agent-harness-packaging 6c161a0.
View: drop. Messaging receipt boundaries; not testing doctrine.

````diff
--- a/skills/testing.md
+++ b/skills/testing.md
@@ -25 +25,7 @@ A run that may exhaust memory or time is bounded (a memory cap and a timeout) so
 Stop a process a test started by the PID that test holds, never by a process-name or path pattern — a scratch and a production instance of the same build share that pattern.
+
+Live acceptance has a boundary. A fixture or generated-output test proves its
+own contract; an isolated transport test proves only its named receipt grade;
+an end-to-end live acceptance needs the actual selected identity, binding, and
+target-side observation. Report an unavailable native route as unavailable,
+not as a failed simulation or a passing deployment test.
````

### c9c3954, 2026-09-21 15:22 -0600: Define bounded independent testing worker role

Made by: no model trailer (a Codex seat by the audit reading).

Flow not identified (no trailer). Rests on a quote relayed by a Field Luna with no transcript citation (flows/1b8ac0/vision/skills.md marks it unconfirmed).
View: drop here. The testing worker's role lives in roles.datom (`tester`); one home.

````diff
--- a/skills/testing.md
+++ b/skills/testing.md
@@ -31 +31,3 @@ target-side observation. Report an unavailable native route as unavailable,
 not as a failed simulation or a passing deployment test.
+
+When assigned as a testing worker, accept a bounded target, immutable revision, authority limits, and acceptance contract. Choose the test procedure, fixtures, negative cases, and independent oracle yourself; do not mirror the implementation or a main flow's assertion. Run the smallest test that can distinguish acceptance from a plausible failure, including a rejected or failing case before trusting a new test. Keep source/projection ownership, native binding and receipt, deployment parity and rollback, transport versus target read, passive no-wake observation, and context metric freshness distinct when those boundaries matter. Report what each witness proves, the exact revision and scope tested, and what remains unavailable. Do not wake a production flow merely to test status or delivery.
````

