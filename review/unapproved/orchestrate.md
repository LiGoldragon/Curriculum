# orchestrate: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/orchestrate.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/orchestrate.md
+++ b/skills/orchestrate.md
@@ -12,7 +12,7 @@ Acquire a Lock. Its four positional fields are `LockName`, `FlowId`, `LockPaths`
 `LockName` names the work; `FlowId` is the owning flow's `FLOW_ID`; `LockPaths` lists the reserved absolute paths; `LockReason` states why. Put the actual owning `FLOW_ID` only in `FlowId`, never concatenated into `LockName` merely for uniqueness or title display. A name such as `FlowIdDocumentation` remains valid when flow ID is the subject of the work.
 
-A reason containing a space or a delimiter is written in Datom curly quotes, “like this”; ASCII double quotes are not Datom string delimiters. A copyable multi-word reason example is:
+A reason containing a space or a delimiter is written in guillemets. A copyable multi-word reason example is:
 
-    orchestrate 'Lock.{ OrchestrateDocs 444e5e [ /absolute/path/to/file ] “Clarify Lock fields” }'
+    orchestrate 'Lock.{ OrchestrateDocs 444e5e [ /absolute/path/to/file ] «Clarify Lock fields» }'
 
 A single-word reason is bare.
@@ -28,5 +28,5 @@ Observe current Locks:
     orchestrate 'Observe.Locks'
 
-`Observed` carries one complete point-in-time Lock snapshot. It is not a subscription.
+`Observed` carries the complete Lock set on open, then again after every Lock or Release — the connection itself is the subscription, with no token and no `Unwatch`. The current `orchestrate` CLI reads one `Observed` frame and exits; it does not yet hold the connection open to receive the later ones, so re-issuing `Observe.Locks` is today's only way to see a change, not the designed one.
 
 Treat a client failure as a failed operation.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d. The Observe subscription sentence matches Vision/nexus.md; the note that the CLI reads one frame and exits was the flow's own.
View: candidate for the first clause; drop the CLI note (a statement about one version).

````diff
--- a/skills/orchestrate.md
+++ b/skills/orchestrate.md
@@ -29,3 +29,3 @@ Observe current Locks:
 
-`Observed` carries one complete point-in-time Lock snapshot. It is not a subscription.
+`Observed` carries the complete Lock set on open, then again after every Lock or Release — the connection itself is the subscription, with no token and no `Unwatch`. The current `orchestrate` CLI reads one `Observed` frame and exits; it does not yet hold the connection open to receive the later ones, so re-issuing `Observe.Locks` is today's only way to see a change, not the designed one.
````

### 987a1e3, 2026-09-12 04:17 -0600: Apply §7: Replace curly quotes with guillemets in orchestrate skill

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d. Guillemets for the reason string: the living's own ("let use the guillemets", Vision/datom.md). Removing the guard against ASCII quotes was the flow's own.
View: candidate for the guillemets. Check against the installed binary first: the trace found it may refuse that form.

````diff
--- a/skills/orchestrate.md
+++ b/skills/orchestrate.md
@@ -13,5 +13,5 @@ Acquire a Lock. Its four positional fields are `LockName`, `FlowId`, `LockPaths`
 
-A reason containing a space or a delimiter is written in Datom curly quotes, “like this”; ASCII double quotes are not Datom string delimiters. A copyable multi-word reason example is:
+A reason containing a space or a delimiter is written in guillemets. A copyable multi-word reason example is:
 
-    orchestrate 'Lock.{ OrchestrateDocs 444e5e [ /absolute/path/to/file ] “Clarify Lock fields” }'
+    orchestrate 'Lock.{ OrchestrateDocs 444e5e [ /absolute/path/to/file ] «Clarify Lock fields» }'
````

