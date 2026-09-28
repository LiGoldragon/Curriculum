# file-editing: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/file-editing.md` is the baseline text with today's rulings applied (see below). Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Today's rulings applied in the recovered text

One Primary workspace (records 8904b1-7, -9, -10) and unsaved changes (8904b1-17: "On the primary workspace we just commit everything unless it looks like fucking nonsense"): added "All flows work in one Primary workspace. Commit each change there as soon as it is made. Changes found unsaved in Primary are committed too, unless they look like nonsense. In any other repository, a commit names only the paths this flow edited: `jj commit -m 'message' path ...`."

## Net change, baseline to main (3726da5)

````diff
--- a/skills/file-editing.md
+++ b/skills/file-editing.md
@@ -11,5 +11,5 @@ before starting new work.
 The sequence for landing work:
 
-    jj commit -m 'short imperative message'
+    jj commit -m 'short imperative message' path ...
     jj bookmark set main -r @-
     jj git push --bookmark main
@@ -19,6 +19,14 @@ commit. `jj bookmark set main -r @-` advances main to it. Then
 push.
 
+A commit names the files it lands: `jj commit -m 'message' path ...`, and only the files this flow edited, usually inside its own flow directory. A commit without paths takes the whole working copy and is made only while the whole repository is locked, when nobody else may be editing.
+
+FLOW_ID=<id> field-clj '#commit ["message" ["path" ...]]' runs this landing under one rule: it commits exactly the named repository-relative paths with a `Flow: <id>` trailer, leaves other dirty paths uncommitted, refuses when `FLOW_ID` is unset, when a named path is clean, or when `jj diff -r @- --name-only` differs from the named set, and prints one positional variant, `#success [flow commit [path ...] :main :pushed :present]` or `#refused …`.
+
+field-clj 'observe []' reads only the current flow-nexus user-service state, durable Flow rows, and Herdr route snapshot. It reports each unavailable surface and never retries, changes runtime state, or submits a message.
+
 Every `jj` command that takes a description uses `-m`. Never open
 an editor. Never use raw `git`.
 
+Clone a working copy from its real remote URL, never from another local checkout (`git clone --shared <local-path>` repoints `origin` at that checkout, and a push there never reaches the real remote). Before reporting a push landed, confirm the pushed revision against the real remote directly — `git ls-remote <real-remote-url>` — not merely against the checkout's configured `origin`, which some checkouts point at a mirror (gitolite, or another local clone) distinct from it.
+
 A source file is written in pieces of a few hundred lines; a module that would exceed that is split.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d, mending its own false "pushed" reports (a clone from a local checkout pushed to that checkout, not the real remote).
View: candidate. A push confirmed only against a local `origin` never reached the real remote; a flow cannot guess this.

````diff
--- a/skills/file-editing.md
+++ b/skills/file-editing.md
@@ -23,2 +23,4 @@ an editor. Never use raw `git`.
 
+Clone a working copy from its real remote URL, never from another local checkout (`git clone --shared <local-path>` repoints `origin` at that checkout, and a push there never reaches the real remote). Before reporting a push landed, confirm the pushed revision against the real remote directly — `git ls-remote <real-remote-url>` — not merely against the checkout's configured `origin`, which some checkouts point at a mirror (gitolite, or another local clone) distinct from it.
+
 A source file is written in pieces of a few hundred lines; a module that would exceed that is split.
````

### de8e8cc, 2026-09-19 16:31 -0600: Land approved commit-path rule for jj commit in file-editing skill

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f38926, with the living (flows/f38926/vision/archive-committing.md): "The call has to be explicit with file paths. Unless the whole repo is locked ..."
View: narrowed by the living's ruling of 2026-09-28 ("On the primary workspace we just commit everything unless it looks like fucking nonsense"): on Primary it no longer holds. Candidate for other repositories, where another flow's uncommitted work may sit: "Outside Primary, a commit names the paths this flow edited."

````diff
--- a/skills/file-editing.md
+++ b/skills/file-editing.md
@@ -12,3 +12,3 @@ The sequence for landing work:
 
-    jj commit -m 'short imperative message'
+    jj commit -m 'short imperative message' path ...
     jj bookmark set main -r @-
@@ -20,2 +20,4 @@ push.
 
+A commit names the files it lands: `jj commit -m 'message' path ...`, and only the files this flow edited, usually inside its own flow directory. A commit without paths takes the whole working copy and is made only while the whole repository is locked, when nobody else may be editing.
+
 Every `jj` command that takes a description uses `-m`. Never open
````

### c455e38, 2026-09-25 14:51 -0600: State field-clj path rule in file-editing

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 38de5b, under e51411's audit, whose own report says of this path rule: "The living has not ruled on it."
View: fold into a compensation skill for field-clj if field-clj stays in use; otherwise drop.

````diff
--- a/skills/file-editing.md
+++ b/skills/file-editing.md
@@ -22,2 +22,4 @@ A commit names the files it lands: `jj commit -m 'message' path ...`, and only t
 
+`field-clj 'commit ["message" ["path" ...]]'` runs this landing under one rule: it commits exactly the named paths, leaves other dirty paths uncommitted, and refuses when a named path is clean or when `jj diff -r @- --name-only` differs from the named set.
+
 Every `jj` command that takes a description uses `-m`. Never open
````

### d26a80b, 2026-09-25 15:18 -0600: file-editing: field-clj #commit form

Made by: Claude Fable 5.1 <noreply@anthropic.com>; Flow trailer 38de5b.

Flow 38de5b.
View: as c455e38.

````diff
--- a/skills/file-editing.md
+++ b/skills/file-editing.md
@@ -22,3 +22,3 @@ A commit names the files it lands: `jj commit -m 'message' path ...`, and only t
 
-`field-clj 'commit ["message" ["path" ...]]'` runs this landing under one rule: it commits exactly the named paths, leaves other dirty paths uncommitted, and refuses when a named path is clean or when `jj diff -r @- --name-only` differs from the named set.
+FLOW_ID=<id> field-clj '#commit ["message" ["path" ...]]' runs this landing under one rule: it commits exactly the named repository-relative paths with a `Flow: <id>` trailer, leaves other dirty paths uncommitted, refuses when `FLOW_ID` is unset, when a named path is clean, or when `jj diff -r @- --name-only` differs from the named set, and prints one positional variant, `#success [flow commit [path ...] :main :pushed :present]` or `#refused …`.
````

### 8a814be, 2026-09-25 20:05 -0600: Document field-clj observation

Made by: no model trailer (a Codex seat by the audit reading).

Flow b860be (subflow a676b3), documenting a tool it built.
View: drop. It reads Flow Nexus state, which is not used now.

````diff
--- a/skills/file-editing.md
+++ b/skills/file-editing.md
@@ -24,2 +24,4 @@ FLOW_ID=<id> field-clj '#commit ["message" ["path" ...]]' runs this landing unde
 
+field-clj 'observe []' reads only the current flow-nexus user-service state, durable Flow rows, and Herdr route snapshot. It reports each unavailable surface and never retries, changes runtime state, or submits a message.
+
 Every `jj` command that takes a description uses `-m`. Never open
````

