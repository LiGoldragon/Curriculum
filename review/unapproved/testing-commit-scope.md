# testing-commit-scope: created after the baseline

Created in dad80ad, 2026-09-19 20:34 -0600: Add a round of agent-written testing skills. Made by: Claude Fable 5.1 <noreply@anthropic.com>.

## View

Flow f38926's round of testing skills (dad80ad), allowed by the living's 09-18 rule that flows may add testing skills on their own judgment.
View: drop. file-editing holds the rule and field-clj performs the check.

Under the kinds the living settled on 2026-09-28 this skill would be named `test-commit-scope`.

## The whole skill as it stands on main (3726da5)

````markdown
---
description: A commit has been made in a working copy other flows also write, and its file list must be shown to hold only this flow's files.
dependencies: [file-editing, edit-coordination]
---

Prove the commit's scope by listing what it actually landed:

    jj diff -r @- --name-only

Every path returned must be one this flow edited. A path this flow did not touch means another flow's work was swept in: say so, name the path, and do not report the commit as clean.

Do this after every commit in a shared working copy, including one made with explicit paths — a path argument that names a directory takes everything under it.

Run it again before setting the bookmark, so a swept file is found before it is pushed.
````
