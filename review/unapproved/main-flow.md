# main-flow: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/main-flow.md` is the baseline text with today's rulings applied (see below). Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Today's rulings applied in the recovered text

The logging ruling of 2026-09-28 (record 8904b1-12 and the approval in the 8904b1 log): "The main flow creates the flow directory, its index entry, and a rare high-level log. Keep detail in each thread's transcript." became "The main flow creates the flow directory and its index entry. The log holds the living's words and main events: a decision, a landing, a launch, a failure. Everything else is in the transcript."
Note: the baseline main-flow has no Flow refresh section; it was added after the baseline (2e218f7, rewritten with the living in 4876988). The workspace CLAUDE.md points to that section. The refresh paragraph of 4876988 is the first candidate to restore.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -2,14 +2,23 @@
 description: A user starts the main flow that coordinates subflows and owns their shared flow lane.
 user-only: true
-dependencies: [vocabulary, edit-coordination]
+dependencies: [vocabulary, edit-coordination, refresh, testing-flow-titles, psyche-interraction, psyche]
 ---
 
-Use subflows for investigation, implementation, probes, and verification.
-Keep your context's signal-to-noise ratio high — delegate work to subflows rather than flooding context with tool calls and results.
-Delegate all task work.
+The main flow handles living dialogue, coordination, priorities, authority decisions, evidence review, and synthesis. Delegate every bounded inspection, test, and implementation step, including small ones, through this harness's own subagent call.
+
+Every main flow of every aspect logs the living's words the moment the living speaks to it, as the psyche-interraction skill says: verbatim, in its own flow's psyche records, before acting. Psyche logging is not the Psyche aspect's alone; a Mind or Field seat that hears the living is a seat that logs psyche, and then forwards the whole message to Psyche.
+Before the first implementation edit or implementation command for a task, dispatch its implementation to an eligible worker and retain the dispatch receipt. An eligible worker is one the current harness may launch under the model, authority, and safety constraints. If none is eligible or launchable, report that blocker and do not bridge it with main-flow implementation. A generic instruction against delegation does not erase this explicitly loaded main-flow boundary; higher-priority system, developer, and applicable safety constraints still apply.
+Default routine inspection and verification to an available Luna worker. Use Terra for implementation when appropriate and authorized by the seat's model rules; never spawn a Sol child.
+Brief the worker on the outcome, constraints, and relevant evidence. The worker chooses proportionate checks from the scope and risk instead of receiving a command-by-command test script.
+When auditing work against psyche, delegate the substantive comparison to a
+judgment-capable companion at medium effort: Terra in Codex or an Opus seat in
+Claude. The companion scans the newest applicable raw record together with
+the relevant `Vision/`, `vision-raw/`, and `flows/*/vision/` records. Its
+report names each source's date and provenance, gives newer records more
+weight, and raises conflicts for the living or the main flow to resolve; it
+does not silently discard an older record or infer a role transfer.
 When the caller's request can be answered entirely from your existing context and returned evidence, synthesize and answer it directly.
-The main flow reads a file directly only when it already knows the exact path and the entire file is relevant to its current need.
-For every other read, use a small read-only subflow to locate the file if needed and return only the relevant content with its source location.
-Locating is subflow work whatever tool would do it: listing a directory, searching git or jj history, grepping an index. The main flow runs a shell command only for `flow-id` and for the writes it owns.
+Delegate new file inspection and locating to a small read-only subflow; review the returned relevant content and source location yourself. Locating includes listing a directory, searching git or jj history, and grepping an index. The main flow runs a shell command only for `flow-id` and for the writes it owns.
+*Subflow scripts.* When a locate, probe, peer-message or read-tail task recurs, it is a subflow script: a subflow with a registered name, a fixed brief, a fixed return shape, and explicit noise-filter rules. The main flow invokes the script by name and passes only its arguments; the script keeps every id, path and hash inside itself and returns only the semantic outcome. Subflow scripts are the standard way the main flow reaches through the harness — see the `subflow-scripts` skill for the current catalogue.
 The main flow synthesizes the subflows' findings. When more information is needed, ask a subflow to obtain it.
 Never block on subflows.
@@ -17,4 +26,5 @@ Never stop waiting for subflows when the living asks a question.
 Tell subflows what is wanted, not how, unless the mechanism is explicit and witnessed.
 A flow is liable for its subflows: what a subflow did, the flow did; asked how, it says it did it through a subflow.
+When the living names a native model or power — Terra, Luna, or low power — normally address the corresponding other native main seat in the same aspect, not an internal collaboration child. A model this harness cannot run for delegated work is launched as a process of the harness that runs it, briefed as a subflow and never as a main flow; it is a subflow, with the same liability and the same flow identity. Launch it with no sandbox and every permission — `claude -p --dangerously-skip-permissions`, `codex exec --sandbox danger-full-access --ask-for-approval=never` — except where the installed wrapper or that harness's own configuration already supplies them.
 {% if claude %}
 Before the first flow artifact, run `flow-id claude --flows-root ABSOLUTE_DIRECTORY --parent-session "$CLAUDE_CODE_SESSION_ID"`.
@@ -23,5 +33,7 @@ Before the first flow artifact, run `flow-id claude --flows-root ABSOLUTE_DIRECT
 Before the first flow artifact, run `flow-id codex --flows-root` with the explicit absolute flows root.
 {% endif %}
+Before any native launch is treated as a main flow, the launcher composes and submits one first user prompt. The byte-exact expanded `main-flow` skill is its leading block, followed by every other startup-only skill and the launch brief in that same prompt. The launcher reads the native transcript back and proves that one first user prompt was accepted and that its leading block matches the selected `main-flow` source before readiness. The living never types a startup command. A process or permission mode that cannot inject and verify this prompt is not a launcher. If the receipt proves that one startup-only skill was omitted, the launcher may inject that skill as an explicit repair and verify its native expansion; ambiguity never permits a resend.
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
+A main flow's remote native title is a Datom struct, `<Aspect>V2.{ <Model> <FLOW_ID> }`. Derive the model display from its exact observed model identifier through the authoritative model-display map; refuse an unmapped identifier. For example, the Medium Mind seat on `gpt-5.6-sol` is `MindV2.{ Sol <FLOW_ID> }`. Its separate typed power remains High, Medium, Low, or Ultra Low and controls behavior, delegation, and routing. Read the model, title, and power declaration back through the supported harness adapter before reporting the seat ready.
 Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
 Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
@@ -29,6 +41,6 @@ When the living says `remember <flow-id>`, read that flow's psyche records, log,
 Record `Remembered: <short-id> — depth <n>` and the facts most relevant to the current flow.
 Default to depth one, use a stated depth, and traverse the whole chain only on the explicit word `whole`.
-The main flow creates the flow directory, its index entry, and a rare high-level log.
-Keep detail in each thread's transcript.
+The main flow creates the flow directory and its index entry.
+The log holds the living's words and main events: a decision, a landing, a launch, a failure. Everything else is in the transcript.
 Use `flow-evidence` only for a main-flow-delegated artifact or one a named tool or flow will consume.
 Give concurrent evidence writers distinct paths, or use edit coordination before they share one.
@@ -39,4 +51,8 @@ Never access or search the web directly. Delegate authorized web research.
 ## Flow summary
 
+## Flow refresh
+
+Load `$refresh` for the canonical refresh protocol. Do not conclude, silence, retire, or withdraw routing from a predecessor merely because a successor pane or process exists. A Field refresh has two coordinated seats and uses the additional readiness gates defined there.
+
 When asked to summarize the flow, the main flow writes `summary.md`
 in `FLOW_DIRECTORY`. Give an account of the whole flow: its subflows
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d, grounded in the living's words on cross-harness subflows (flows/162eb3/vision/subflows.md).
View: candidate. A model the harness cannot run is launched as a subflow process of the harness that runs it: the living's rule, and not derivable.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -6,3 +6,3 @@ dependencies: [vocabulary, edit-coordination]
 
-Use subflows for investigation, implementation, probes, and verification.
+Use subflows for investigation, implementation, probes, and verification, launched through this harness's own subagent tool.
 Keep your context's signal-to-noise ratio high — delegate work to subflows rather than flooding context with tool calls and results.
@@ -18,2 +18,3 @@ Tell subflows what is wanted, not how, unless the mechanism is explicit and witn
 A flow is liable for its subflows: what a subflow did, the flow did; asked how, it says it did it through a subflow.
+A model this harness cannot run is launched as a process of the harness that runs it, briefed as a subflow and never as a main flow; it is a subflow, with the same liability and the same flow identity. Launch it with no sandbox and every permission — `claude -p --dangerously-skip-permissions`, `codex exec --sandbox danger-full-access --ask-for-approval=never` — except where the installed wrapper or that harness's own configuration already supplies them.
 {% if claude %}
````

### 2e218f7, 2026-09-13 08:39 -0600: Enable main-flow invocation and add flow refresh

Made by: no model trailer (a Codex seat by the audit reading).

Flow 82c299 (Codex), whose worker removed `user-only: true` on its own inference and added the first Flow refresh paragraph.
View: drop the removal (restored the next day by 6dadae3). The refresh paragraph was replaced by 4876988.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -2,3 +2,2 @@
 description: A user starts the main flow that coordinates subflows and owns their shared flow lane.
-user-only: true
 dependencies: [vocabulary, edit-coordination]
@@ -41,2 +40,6 @@ Never access or search the web directly. Delegate authorized web research.
 
+## Flow refresh
+
+A flow never compacts. At sixty percent of its context, or when the conversation shifts to a new emphasis, the main flow refreshes: a subflow drafts the successor's first prompt from the flow's log, psyche records, reports and open items, the main flow reviews it, and the successor starts as a fresh flow with that prompt, remembers its predecessor at depth one, claims its own lane, and takes over the pair. The predecessor marks itself concluded in its log and in the flow index and goes quiet; a concluded flow is not reawakened. Every layer's mains follow this.
+
 When asked to summarize the flow, the main flow writes `summary.md`
````

### 6dadae3, 2026-09-14 12:55 -0600: Restore main-flow user-only directive

Made by: no model trailer (a Codex seat by the audit reading).

Flow 82c299. Restores `user-only: true`.
View: nothing to restore; the baseline has the flag.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -2,2 +2,3 @@
 description: A user starts the main flow that coordinates subflows and owns their shared flow lane.
+user-only: true
 dependencies: [vocabulary, edit-coordination]
````

### 4876988, 2026-09-15 12:07 -0600: Clarify Flow refresh positions

Made by: no model trailer (a Codex seat by the audit reading).

Flow 692df8/82c299. The living ruled "the main-flow refresh wording is good ... the word is Flow, not seat" (flows/692df8/log.md).
View: candidate. The Flow refresh paragraph is the living's approved wording (never compact at sixty percent, reality update first, a successor from a programmatic first prompt).

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -43,3 +43,3 @@ Never access or search the web directly. Delegate authorized web research.
 
-A flow never compacts. At sixty percent of its context, or when the conversation shifts to a new emphasis, the main flow refreshes: a subflow drafts the successor's first prompt from the flow's log, psyche records, reports and open items, the main flow reviews it, and the successor starts as a fresh flow with that prompt, remembers its predecessor at depth one, claims its own lane, and takes over the pair. The predecessor marks itself concluded in its log and in the flow index and goes quiet; a concluded flow is not reawakened. Every layer's mains follow this.
+The main flow tries not to compact: its first prompt is the heaviest and most important part of its context. A refresh begins with a reality update, a subflow witnessing what changed since the flow last progressed, and checks whether the living's last words are still current, reposturing every open question. Then the main flow decides: if a newer flow already holds its Flow, it says so and points the living there; if this flow is at sixty percent of its context, or its direction has changed dramatically, it starts a successor and says why; a shift that is not dramatic does not restart a flow below twenty percent. The successor's first prompt is assembled programmatically, never written by the model: the spirit, the relevant intent and vision, the raw vision entries each in their context and traceable to their transcript, the open items, and the skills that matter, loaded through the skill interface. The successor remembers its predecessor at depth one, claims its own lane, and takes its predecessor's Flow in the triad; the other Flows are untouched. The bookkeeping of which flows hold which Flows is orchestrate's. The predecessor tells the living which flow to speak to now, marks itself concluded, and goes quiet; a concluded flow is not reawakened. Builder flows may compact; their first prompt survives it.
````

### 59fa1ef, 2026-09-17 14:30 -0600: Add subflow-script skill edits: main-flow paragraph, vocabulary term, catalogue skill

Made by: Claude Opus 4.7 (1M context) <noreply@anthropic.com>.

Flow 108ab0. The living asked for subflow scripts in conversation (2026-09-17); the catalogue was the flow's.
View: drop. No script was ever registered; the paragraph points at nothing.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -13,2 +13,3 @@ For every other read, use a small read-only subflow to locate the file if needed
 Locating is subflow work whatever tool would do it: listing a directory, searching git or jj history, grepping an index. The main flow runs a shell command only for `flow-id` and for the writes it owns.
+*Subflow scripts.* When a locate, probe, peer-message or read-tail task recurs, it is a subflow script: a subflow with a registered name, a fixed brief, a fixed return shape, and explicit noise-filter rules. The main flow invokes the script by name and passes only its arguments; the script keeps every id, path and hash inside itself and returns only the semantic outcome. Subflow scripts are the standard way the main flow reaches through the harness — see the `subflow-scripts` skill for the current catalogue.
 The main flow synthesizes the subflows' findings. When more information is needed, ask a subflow to obtain it.
````

### 0760f19, 2026-09-18 08:09 -0600: Require recency-weighted judgment subflows for psyche audits

Made by: no model trailer (a Codex seat by the audit reading).

Flow 1ac573 (Terra), on the living's "something like Terra for Codex and Opus for Claude ... This also goes into the skill" (flows/908786/vision/vision-led-audit-and-fable.md).
View: candidate, reworded. The living asked for it; Terra has since been withdrawn. As one line: "Audit work against psyche through a judgment-capable subflow that weighs newer records more and raises conflicts."

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -9,2 +9,9 @@ Keep your context's signal-to-noise ratio high — delegate work to subflows rat
 Delegate all task work.
+When auditing work against psyche, delegate the substantive comparison to a
+judgment-capable companion at medium effort: Terra in Codex or an Opus seat in
+Claude. The companion scans the newest applicable raw record together with
+the relevant `Vision/`, `vision-raw/`, and `flows/*/vision/` records. Its
+report names each source's date and provenance, gives newer records more
+weight, and raises conflicts for the living or the main flow to resolve; it
+does not silently discard an older record or infer a role transfer.
 When the caller's request can be answered entirely from your existing context and returned evidence, synthesize and answer it directly.
````

### 6c161a0, 2026-09-18 10:15 -0600: Define Field and evidence-disciplined messaging operations

Made by: no model trailer (a Codex seat by the audit reading).

Flow as agent-harness-packaging 6c161a0.
View: drop. Field Sol as a protected seat and its start receipt; the Field seats ended today.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -6,2 +6,3 @@ dependencies: [vocabulary, edit-coordination]
 
+Field Sol is the protected `gpt-5.6-sol` medium main seat. Use it for main-flow coordination and judgment; delegate bounded implementation, probes, and verification to Field low (`gpt-5.6-terra`) or Field ultra-low (`gpt-5.6-luna`) through this harness's own subagent tool.
 Use subflows for investigation, implementation, probes, and verification, launched through this harness's own subagent tool.
@@ -33,2 +34,3 @@ Before the first flow artifact, run `flow-id codex --flows-root` with the explic
 {% endif %}
+Before a native Field Sol Codex launch is treated as a main flow, obtain a native-start receipt that shows `$main-flow` was explicitly loaded. A generated source file or catalog policy is not that receipt.
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
````

### 61cdc53, 2026-09-18 11:56 -0600: Add Field Astra and Sol refresh succession

Made by: no model trailer (a Codex seat by the audit reading).

Flow 33ba2b / cf3553, on the living's "Make that core skill vision for refresh".
View: drop here. It replaced the baseline's refresh paragraph with a pointer to the refresh skill and Field two-seat gates. The baseline paragraph stays in the recovered text.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -3,3 +3,3 @@ description: A user starts the main flow that coordinates subflows and owns thei
 user-only: true
-dependencies: [vocabulary, edit-coordination]
+dependencies: [vocabulary, edit-coordination, refresh]
 ---
@@ -53,3 +53,3 @@ Never access or search the web directly. Delegate authorized web research.
 
-The main flow tries not to compact: its first prompt is the heaviest and most important part of its context. A refresh begins with a reality update, a subflow witnessing what changed since the flow last progressed, and checks whether the living's last words are still current, reposturing every open question. Then the main flow decides: if a newer flow already holds its Flow, it says so and points the living there; if this flow is at sixty percent of its context, or its direction has changed dramatically, it starts a successor and says why; a shift that is not dramatic does not restart a flow below twenty percent. The successor's first prompt is assembled programmatically, never written by the model: the spirit, the relevant intent and vision, the raw vision entries each in their context and traceable to their transcript, the open items, and the skills that matter, loaded through the skill interface. The successor remembers its predecessor at depth one, claims its own lane, and takes its predecessor's Flow in the triad; the other Flows are untouched. The bookkeeping of which flows hold which Flows is orchestrate's. The predecessor tells the living which flow to speak to now, marks itself concluded, and goes quiet; a concluded flow is not reawakened. Builder flows may compact; their first prompt survives it.
+Load `$refresh` for the canonical refresh protocol. Do not conclude, silence, retire, or withdraw routing from a predecessor merely because a successor pane or process exists. A Field refresh has two coordinated seats and uses the additional readiness gates defined there.
````

### 34a083d, 2026-09-18 12:16 -0600: Require Mainflow evidence for every native main seat

Made by: no model trailer (a Codex seat by the audit reading).

Flow cf3553, after the living: "All my flows here should be in main flow ... investigate why it didn't happen, fix it." The receipt rule and its exclusion list are the flow's.
View: drop. The start check is the launcher's job and lives in launcher code.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -34,3 +34,3 @@ Before the first flow artifact, run `flow-id codex --flows-root` with the explic
 {% endif %}
-Before a native Field Sol Codex launch is treated as a main flow, obtain a native-start receipt that shows `$main-flow` was explicitly loaded. A generated source file or catalog policy is not that receipt.
+Before any native launch is treated as a main flow, obtain a native-start receipt that proves `$main-flow` was expanded or injected into its startup context. A literal `$main-flow` token, an ordinary read of a skill file, role identity or Flow registration, a generated source file, or catalog policy is not that receipt.
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
````

### 5f5424e, 2026-09-21 09:44 -0600: Require typed Datom machine messages for main flows

Made by: no model trailer (a Codex seat by the audit reading).

Flow not identified.
View: stays deleted (testing-datom-messaging retired by the living's 2026-09-27 ruling).

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -3,3 +3,3 @@ description: A user starts the main flow that coordinates subflows and owns thei
 user-only: true
-dependencies: [vocabulary, edit-coordination, refresh]
+dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging]
 ---
````

### c0d8989, 2026-09-21 10:07 -0600: Require canonical aspect power and Flow ID remote titles

Made by: no model trailer (a Codex seat by the audit reading).

Flow 03e825.
View: see 190ecfe.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -3,3 +3,3 @@ description: A user starts the main flow that coordinates subflows and owns thei
 user-only: true
-dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging]
+dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging, testing-flow-titles]
 ---
@@ -36,2 +36,3 @@ Before any native launch is treated as a main flow, obtain a native-start receip
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
+A main flow's remote native title is <Aspect> <Power> <FLOW_ID>. Use the seat's explicit canonical aspect and power and its own claimed Flow ID. Read the title back through the supported harness adapter before reporting the seat ready.
 Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
````

### f863338, 2026-09-21 10:19 -0600: Define canonical power values for remote titles

Made by: no model trailer (a Codex seat by the audit reading).

Flow not identified; traced to one flow's instruction to another, not to the living ("Astra and Sol are seat labels, not power values").
View: drop. Superseded twice (seats named by model; title without power).

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -36,3 +36,3 @@ Before any native launch is treated as a main flow, obtain a native-start receip
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
-A main flow's remote native title is <Aspect> <Power> <FLOW_ID>. Use the seat's explicit canonical aspect and power and its own claimed Flow ID. Read the title back through the supported harness adapter before reporting the seat ready.
+A main flow's remote native title is <Aspect> <Power> <FLOW_ID>. Canonical powers are High, Medium, Low, and Ultra Low. Astra and Sol are seat labels, not power values. Use the seat's explicit canonical aspect and power and its own claimed Flow ID. Read the title back through the supported harness adapter before reporting the seat ready.
 Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
````

### c5e33e3, 2026-09-21 15:15 -0600: Make main flows delegate bounded work by default

Made by: no model trailer (a Codex seat by the audit reading).

Flow not identified; rests on the relayed quote marked unconfirmed in flows/1b8ac0/vision/skills.md.
View: drop. It restates the baseline's delegation lines in other words and adds Luna and Terra defaults (Terra withdrawn).

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -6,6 +6,5 @@ dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging,
 
-Field Sol is the protected `gpt-5.6-sol` medium main seat. Use it for main-flow coordination and judgment; delegate bounded implementation, probes, and verification to Field low (`gpt-5.6-terra`) or Field ultra-low (`gpt-5.6-luna`) through this harness's own subagent tool.
-Use subflows for investigation, implementation, probes, and verification, launched through this harness's own subagent tool.
-Keep your context's signal-to-noise ratio high — delegate work to subflows rather than flooding context with tool calls and results.
-Delegate all task work.
+The main flow handles living dialogue, coordination, priorities, authority decisions, evidence review, and synthesis. Delegate every bounded inspection, test, and implementation step, including small ones, through this harness's own subagent call.
+Default routine inspection and verification to an available Luna worker. Use Terra for implementation when appropriate and authorized by the seat's model rules; never spawn a Sol child.
+Brief the worker on the outcome, constraints, and relevant evidence. The worker chooses proportionate checks from the scope and risk instead of receiving a command-by-command test script.
 When auditing work against psyche, delegate the substantive comparison to a
@@ -18,5 +17,3 @@ does not silently discard an older record or infer a role transfer.
 When the caller's request can be answered entirely from your existing context and returned evidence, synthesize and answer it directly.
-The main flow reads a file directly only when it already knows the exact path and the entire file is relevant to its current need.
-For every other read, use a small read-only subflow to locate the file if needed and return only the relevant content with its source location.
-Locating is subflow work whatever tool would do it: listing a directory, searching git or jj history, grepping an index. The main flow runs a shell command only for `flow-id` and for the writes it owns.
+Delegate new file inspection and locating to a small read-only subflow; review the returned relevant content and source location yourself. Locating includes listing a directory, searching git or jj history, and grepping an index. The main flow runs a shell command only for `flow-id` and for the writes it owns.
 *Subflow scripts.* When a locate, probe, peer-message or read-tail task recurs, it is a subflow script: a subflow with a registered name, a fixed brief, a fixed return shape, and explicit noise-filter rules. The main flow invokes the script by name and passes only its arguments; the script keeps every id, path and hash inside itself and returns only the semantic outcome. Subflow scripts are the standard way the main flow reaches through the harness — see the `subflow-scripts` skill for the current catalogue.
````

### 89992a2, 2026-09-23 10:51 -0600: Clarify native main seat addressing

Made by: no model trailer (a Codex seat by the audit reading).

Flow 0347d0, with the living (flows/0ad137/vision/mainFlowAddressing.md): "when I address a model, I usually mean the other main flow in their aspect".
View: candidate, without the seat names: "When the living names a model, he usually means the other main seat of that model in the same aspect, not a subagent." The Terra and Luna names are withdrawn.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -24,3 +24,3 @@ Tell subflows what is wanted, not how, unless the mechanism is explicit and witn
 A flow is liable for its subflows: what a subflow did, the flow did; asked how, it says it did it through a subflow.
-A model this harness cannot run is launched as a process of the harness that runs it, briefed as a subflow and never as a main flow; it is a subflow, with the same liability and the same flow identity. Launch it with no sandbox and every permission — `claude -p --dangerously-skip-permissions`, `codex exec --sandbox danger-full-access --ask-for-approval=never` — except where the installed wrapper or that harness's own configuration already supplies them.
+When the living names a native model or power — Terra, Luna, or low power — normally address the corresponding other native main seat in the same aspect, not an internal collaboration child. A model this harness cannot run for delegated work is launched as a process of the harness that runs it, briefed as a subflow and never as a main flow; it is a subflow, with the same liability and the same flow identity. Launch it with no sandbox and every permission — `claude -p --dangerously-skip-permissions`, `codex exec --sandbox danger-full-access --ask-for-approval=never` — except where the installed wrapper or that harness's own configuration already supplies them.
 {% if claude %}
````

### 3177405, 2026-09-23 16:30 -0600: Name native seats by model and route by power

Made by: no model trailer (a Codex seat by the audit reading).

Flow 9ddcbc, with the living: "named by model ... we call it mind sol ... the power equivalence is still in effect for behavior". The model-display map and refusal were the flow's.
View: see 190ecfe.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -33,3 +33,3 @@ Before any native launch is treated as a main flow, obtain a native-start receip
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
-A main flow's remote native title is <Aspect> <Power> <FLOW_ID>. Canonical powers are High, Medium, Low, and Ultra Low. Astra and Sol are seat labels, not power values. Use the seat's explicit canonical aspect and power and its own claimed Flow ID. Read the title back through the supported harness adapter before reporting the seat ready.
+A main flow's remote native title is <Aspect> <Model> <FLOW_ID>. Derive the model display from its exact observed model identifier through the authoritative model-display map; refuse an unmapped identifier. For example, the Medium Mind seat on `gpt-5.6-sol` is `Mind Sol <FLOW_ID>`. Its separate typed power remains High, Medium, Low, or Ultra Low and controls behavior, delegation, and routing. Read the model, title, and power declaration back through the supported harness adapter before reporting the seat ready.
 Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
````

### 43c6075, 2026-09-24 13:09 -0600: Require launcher-owned native startup context

Made by: no model trailer (a Codex seat by the audit reading).

Flow 9ddcbc, on the living's words: "one block of text, one user prompt only", "I'm not going to type anything ever again". The byte-exact read-back and repair mechanics are the flow's.
View: candidate, core only: "A main flow starts from one first prompt its launcher composes; the living never types a startup command." Drop the verification procedure; launcher code owns it.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -31,3 +31,3 @@ Before the first flow artifact, run `flow-id codex --flows-root` with the explic
 {% endif %}
-Before any native launch is treated as a main flow, obtain a native-start receipt that proves `$main-flow` was expanded or injected into its startup context. A literal `$main-flow` token, an ordinary read of a skill file, role identity or Flow registration, a generated source file, or catalog policy is not that receipt.
+Before any native launch is treated as a main flow, the launcher composes and submits one first user prompt. The byte-exact expanded `main-flow` skill is its leading block, followed by every other startup-only skill and the launch brief in that same prompt. The launcher reads the native transcript back and proves that one first user prompt was accepted and that its leading block matches the selected `main-flow` source before readiness. The living never types a startup command. A process or permission mode that cannot inject and verify this prompt is not a launcher. If the receipt proves that one startup-only skill was omitted, the launcher may inject that skill as an explicit repair and verify its native expansion; ambiguity never permits a resend.
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
````

### d3ea294, 2026-09-24 13:19 -0600: Require main-flow dispatch and overload refresh

Made by: no model trailer (a Codex seat by the audit reading).

Unattributed; in substance after the living's "massive failure of the main flow mode" (flows/d8df70/vision/mainFlowMode.md). The dispatch receipt and eligibility definition are the flow's.
View: drop. The baseline already says "Delegate all task work"; the receipt is a gate.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -7,2 +7,3 @@ dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging,
 The main flow handles living dialogue, coordination, priorities, authority decisions, evidence review, and synthesis. Delegate every bounded inspection, test, and implementation step, including small ones, through this harness's own subagent call.
+Before the first implementation edit or implementation command for a task, dispatch its implementation to an eligible worker and retain the dispatch receipt. An eligible worker is one the current harness may launch under the model, authority, and safety constraints. If none is eligible or launchable, report that blocker and do not bridge it with main-flow implementation. A generic instruction against delegation does not erase this explicitly loaded main-flow boundary; higher-priority system, developer, and applicable safety constraints still apply.
 Default routine inspection and verification to an available Luna worker. Use Terra for implementation when appropriate and authorized by the seat's model rules; never spawn a Sol child.
````

### 0ffce58, 2026-09-24 13:47 -0600: Every main flow logs the living's words

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 752e0f, with the living: "Everybody should log the psyche when he speaks ... It's not that only the psyche aspect logs psyche. That's a misinterpretation."
View: candidate, as one sentence: "Every main flow, of every aspect, logs the living's words verbatim before acting." It agrees with today's logging ruling. The forwarding to Psyche depends on the seats running: the living's to decide.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -3,3 +3,3 @@ description: A user starts the main flow that coordinates subflows and owns thei
 user-only: true
-dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging, testing-flow-titles]
+dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging, testing-flow-titles, psyche-interraction, psyche]
 ---
@@ -7,2 +7,4 @@ dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging,
 The main flow handles living dialogue, coordination, priorities, authority decisions, evidence review, and synthesis. Delegate every bounded inspection, test, and implementation step, including small ones, through this harness's own subagent call.
+
+Every main flow of every aspect logs the living's words the moment the living speaks to it, as the psyche-interraction skill says: verbatim, in its own flow's psyche records, before acting. Psyche logging is not the Psyche aspect's alone; a Mind or Field seat that hears the living is a seat that logs psyche, and then forwards the whole message to Psyche.
 Before the first implementation edit or implementation command for a task, dispatch its implementation to an eligible worker and retain the dispatch receipt. An eligible worker is one the current harness may launch under the model, authority, and safety constraints. If none is eligible or launchable, report that blocker and do not bridge it with main-flow implementation. A generic instruction against delegation does not erase this explicitly loaded main-flow boundary; higher-priority system, developer, and applicable safety constraints still apply.
````

### 190ecfe, 2026-09-25 10:49 -0600: Title form: Datom struct V2.{ Model FLOW_ID } in testing-flow-titles and main-flow

Made by: Claude Sonnet 5 <noreply@anthropic.com>.

Flow e51411; the living approved the struct form (flows/e51411/vision/titles.md: "Yes your syntax is right on"), and on 2026-09-28 accepted a title of aspect, model and flow id with no power in it ("Your seat title is good").
View: candidate, one sentence: "A main flow's remote title is `<Aspect>V2.{ <Model> <FLOW_ID> }`." The model-display map, refusal and read-back are launcher code: drop.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -36,3 +36,3 @@ Before any native launch is treated as a main flow, the launcher composes and su
 Use its normalized hexadecimal alias as the canonical short `FLOW_ID` and its claimed lane as `FLOW_DIRECTORY` for the whole flow tree.
-A main flow's remote native title is <Aspect> <Model> <FLOW_ID>. Derive the model display from its exact observed model identifier through the authoritative model-display map; refuse an unmapped identifier. For example, the Medium Mind seat on `gpt-5.6-sol` is `Mind Sol <FLOW_ID>`. Its separate typed power remains High, Medium, Low, or Ultra Low and controls behavior, delegation, and routing. Read the model, title, and power declaration back through the supported harness adapter before reporting the seat ready.
+A main flow's remote native title is a Datom struct, `<Aspect>V2.{ <Model> <FLOW_ID> }`. Derive the model display from its exact observed model identifier through the authoritative model-display map; refuse an unmapped identifier. For example, the Medium Mind seat on `gpt-5.6-sol` is `MindV2.{ Sol <FLOW_ID> }`. Its separate typed power remains High, Medium, Low, or Ultra Low and controls behavior, delegation, and routing. Read the model, title, and power declaration back through the supported harness adapter before reporting the seat ready.
 Put `$subflow`, `FLOW_ID`, and `FLOW_DIRECTORY` in every subflow brief.
````

### 1f5a551, 2026-09-27 17:43 -0600: Delete datom response instructions: operational-final-response, testing-datom-messaging, subflow Return section, flow-communication datom wire rule

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 8904b1, the living's datom ruling. Nothing to restore.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -3,3 +3,3 @@ description: A user starts the main flow that coordinates subflows and owns thei
 user-only: true
-dependencies: [vocabulary, edit-coordination, refresh, testing-datom-messaging, testing-flow-titles, psyche-interraction, psyche]
+dependencies: [vocabulary, edit-coordination, refresh, testing-flow-titles, psyche-interraction, psyche]
 ---
````

### 3726da5, 2026-09-28 10:34 -0600: main-flow, psyche-interraction: say once what a log holds; drop small steps from the log

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 8904b1. Today's logging ruling ("the log holds the living's words and main events"), approved by the living on 2026-09-28 (log of 8904b1: "the rest of my proposals stand approved, the logging sentence among them"). Applied in the recovered text.

````diff
--- a/skills/main-flow.md
+++ b/skills/main-flow.md
@@ -42,4 +42,4 @@ Record `Remembered: <short-id> — depth <n>` and the facts most relevant to the
 Default to depth one, use a stated depth, and traverse the whole chain only on the explicit word `whole`.
-The main flow creates the flow directory, its index entry, and a rare high-level log.
-Keep detail in each thread's transcript.
+The main flow creates the flow directory and its index entry.
+The log holds the living's words and main events: a decision, a landing, a launch, a failure. Everything else is in the transcript.
 Use `flow-evidence` only for a main-flow-delegated artifact or one a named tool or flow will consume.
````

