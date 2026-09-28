# nexus: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/nexus.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/nexus.md
+++ b/skills/nexus.md
@@ -6,4 +6,6 @@ dependencies: []
 A Nexus is the long-running whole with at least two sockets, a default CLI client per socket, and the signal contracts it is compiled with. Its long-running executable is <nexus>-nexus; call it a Nexus, never a daemon. The decision-making engine inside it is Nexus Core. A Nexus is a vertex in the graph of nexuses. An edge joins two vertices and carries one contract: every connected pair has an ordinary edge; only some pairs have a meta edge.
 
+Do not call a collaboration-harness subagent a Flow Nexus flow. A Flow Nexus is a named component that resolves and exactly binds a logical flow identity to a live endpoint; Message Nexus owns durable messaging attempts and receipts when deployed.
+
 ## The Nexus
 
@@ -48,5 +50,7 @@ vocabulary, not strings.
 
 The signal wire vocabulary is versioned by its contract crate: the
-crate's semver is the wire's semver, and consumers pin it.
+crate's semver is the wire's semver, and consumers pin it. A contract
+crate's version reflects only its own wire text; it is never raised to
+match another crate's version.
 
 ## The CLIs
@@ -55,4 +59,6 @@ The CLI's role is to transform text into Signal. It is the boundary
 where the textual form ends and the binary world begins.
 
+A CLI takes one inline datom value and translates it into Signal; a Nexus receives only Signal and never sees datom.
+
 A CLI speaks to exactly one Nexus — its own. It opens no database,
 reaches no other Nexus, and carries no logic worth keeping: it is
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d. The semantic-versioning contract rule was the flow's own mend of a break it caused; the living was not shown it.
View: drop, or ask. A contract-version rule may be sound design, but it is procedure grown from one incident.

````diff
--- a/skills/nexus.md
+++ b/skills/nexus.md
@@ -49,3 +49,5 @@ vocabulary, not strings.
 The signal wire vocabulary is versioned by its contract crate: the
-crate's semver is the wire's semver, and consumers pin it.
+crate's semver is the wire's semver, and consumers pin it. A contract
+crate's version reflects only its own wire text; it is never raised to
+match another crate's version.
````

### fd99d0e, 2026-09-15 12:32 -0600: Land item 19: CLI and Nexus Signal/datom boundary clarification

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow fd0f97/5f4fea. Matches the living's correction "the Nexus only gets signal ... has to be clear everywhere". The subflow that landed it went beyond its brief.
View: candidate. A CLI takes one inline datom and translates it to Signal; a Nexus sees only Signal: the living's rule.

````diff
--- a/skills/nexus.md
+++ b/skills/nexus.md
@@ -58,2 +58,4 @@ where the textual form ends and the binary world begins.
 
+A CLI takes one inline datom value and translates it into Signal; a Nexus receives only Signal and never sees datom.
+
 A CLI speaks to exactly one Nexus — its own. It opens no database,
````

### 6c161a0, 2026-09-18 10:15 -0600: Define Field and evidence-disciplined messaging operations

Made by: no model trailer (a Codex seat by the audit reading).

Flow as agent-harness-packaging 6c161a0.
View: drop. Flow Nexus and Message Nexus as they were then; dead references today.

````diff
--- a/skills/nexus.md
+++ b/skills/nexus.md
@@ -7,2 +7,4 @@ A Nexus is the long-running whole with at least two sockets, a default CLI clien
 
+Do not call a collaboration-harness subagent a Flow Nexus flow. A Flow Nexus is a named component that resolves and exactly binds a logical flow identity to a live endpoint; Message Nexus owns durable messaging attempts and receipts when deployed.
+
 ## The Nexus
````

