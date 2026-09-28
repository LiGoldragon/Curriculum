# subflow: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/subflow.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/subflow.md
+++ b/skills/subflow.md
@@ -8,6 +8,10 @@ Obtain the current `THREAD_ID` from the harness after launch.
 Use `THREAD_ID` only for transcript and evidence provenance.
 Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
+Do not claim a second main identity, upgrade your model or effort, or rebind a
+message route inherited from the main flow. Ask the main flow to make any new
+route or seat decision.
 Do the delegated work and return its final response.
 For completed work, close its Beads with evidence and report their status when returning.
+Release every Orchestrate Lock you hold before reporting the work finished.
 Do not create a lane, index entry, or log.
 Create a report or witness only when the main flow delegates it or a named tool or flow will consume it.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d, mending its own locks left open.
View: candidate, one line: "Release every Orchestrate lock you hold before reporting the work finished." Sensible, but the flow's own.

````diff
--- a/skills/subflow.md
+++ b/skills/subflow.md
@@ -11,2 +11,3 @@ Do the delegated work and return its final response.
 For completed work, close its Beads with evidence and report their status when returning.
+Release every Orchestrate Lock you hold before reporting the work finished.
 Do not create a lane, index entry, or log.
````

### 6c161a0, 2026-09-18 10:15 -0600: Define Field and evidence-disciplined messaging operations

Made by: no model trailer (a Codex seat by the audit reading).

Flow as agent-harness-packaging 6c161a0.
View: drop. "Do not claim a second main identity, upgrade your model, or rebind a route" is grown from one seat's misbehaviour in the multi-seat machinery.

````diff
--- a/skills/subflow.md
+++ b/skills/subflow.md
@@ -9,2 +9,5 @@ Use `THREAD_ID` only for transcript and evidence provenance.
 Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
+Do not claim a second main identity, upgrade your model or effort, or rebind a
+message route inherited from the main flow. Ask the main flow to make any new
+route or seat decision.
 Do the delegated work and return its final response.
````

### 940fed3, 2026-09-24 16:42 -0600: Give subflows a return contract

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 752e0f, on field semantics the living described (relayed by Field Luna e71dab two minutes before): what was asked, what was found, whether it was sent on, what comes next. The datom encoding was the flow's.
View: the datom form stays deleted by the living's ruling of 2026-09-27. Candidate, in plain prose, if the living wants it: "A subflow's last message says what it was asked, what it found, whether it already sent the result on, and what should happen next."

````diff
--- a/skills/subflow.md
+++ b/skills/subflow.md
@@ -18 +18,13 @@ Create a report or witness only when the main flow delegates it or a named tool
 Load `flow-evidence` before creating that artifact.
+
+## Return
+
+A subflow's last message is one datom in the SubflowReturn type, and nothing outside it:
+
+    Type
+    SubflowReturn.{ Request Findings Sent Next }
+    [ Request.Markdown  Findings.Markdown  Markdown.String
+      Sent.[ Nothing  Direct.{ Vector<FlowId> Grade } ]  FlowId.String  Grade.[ Submitted Transported Presented Read ]
+      Next.[ None  Message.Reason  Act.Reason  Ask.Reason ]  Reason.Markdown ]
+
+Request restates what the subflow was launched for, in one line. Findings carry the result. Sent says whether the subflow already delivered the result itself: when the result is for another Flow, the subflow sends it directly with the main flow's identity (`FLOW_ID=<main> hm-send <FLOW> "<body>"`) and reports the recipients and the transport grade, so the main flow need not send it again. Next tells the main flow whether anything remains for it: None when the subflow's send closed the matter, Message when the main flow must still send something and why, Act when it must do something else and why, Ask when a ruling from above is needed and why. The main flow acts on Next and nothing else; it does not repeat a result the subflow already sent.
````

### 1f5a551, 2026-09-27 17:43 -0600: Delete datom response instructions: operational-final-response, testing-datom-messaging, subflow Return section, flow-communication datom wire rule

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 8904b1, carrying out the living's ruling on datom. Nothing to restore.

````diff
--- a/skills/subflow.md
+++ b/skills/subflow.md
@@ -18,13 +18 @@ Create a report or witness only when the main flow delegates it or a named tool
 Load `flow-evidence` before creating that artifact.
-
-## Return
-
-A subflow's last message is one datom in the SubflowReturn type, and nothing outside it:
-
-    Type
-    SubflowReturn.{ Request Findings Sent Next }
-    [ Request.Markdown  Findings.Markdown  Markdown.String
-      Sent.[ Nothing  Direct.{ Vector<FlowId> Grade } ]  FlowId.String  Grade.[ Submitted Transported Presented Read ]
-      Next.[ None  Message.Reason  Act.Reason  Ask.Reason ]  Reason.Markdown ]
-
-Request restates what the subflow was launched for, in one line. Findings carry the result. Sent says whether the subflow already delivered the result itself: when the result is for another Flow, the subflow sends it directly with the main flow's identity (`FLOW_ID=<main> hm-send <FLOW> "<body>"`) and reports the recipients and the transport grade, so the main flow need not send it again. Next tells the main flow whether anything remains for it: None when the subflow's send closed the matter, Message when the main flow must still send something and why, Act when it must do something else and why, Ask when a ruling from above is needed and why. The main flow acts on Next and nothing else; it does not repeat a result the subflow already sent.
````

