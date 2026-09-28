# testing-long-run-progress: created after the baseline

Created in dad80ad, 2026-09-19 20:34 -0600: Add a round of agent-written testing skills. Made by: Claude Fable 5.1 <noreply@anthropic.com>.

## View

Flow f38926's round (dad80ad).
View: candidate. Generic and sound; observe-only.

Under the kinds the living settled on 2026-09-28 this skill would be named `test-long-run-progress`.

## The whole skill as it stands on main (3726da5)

````markdown
---
description: A build, fetch, or other long run has gone quiet and a flow must say whether it is progressing or has stopped.
dependencies: [testing, behavior]
---

Sample the process's own counters twice across a bounded window and report the deltas:

    cat /proc/<pid>/io          # rchar, wchar, read_bytes, write_bytes
    sleep <window>; cat /proc/<pid>/io

Hold the PID from the handle of the process you started. Never select it by command name or path pattern — a scratch run and a production run of the same build share that pattern.

Report the window length with the deltas, and name which counter moved. A moving counter is activity on that counter, not progress toward completion, and not a cause. Zero across every counter is no observed progress over that window, not a stall.

Add the output sink's own size — the log's byte length, the destination file's size — as a second counter; page-cached reads leave `read_bytes` at zero.

A PID that is gone is gone: recover its exit status from the handle and report that. Absence alone is neither success nor failure.

Observe only. Do not signal, restart, or duplicate the run to find out what it is doing.
````
