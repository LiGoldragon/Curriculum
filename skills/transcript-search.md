---
description: A transcript must be searched — for the psyche's typed words, for what a flow said or did, or for one record by line.
dependencies: []
---

`transcript show <session>` prints every typed message of a session
with its line number and the model responses before it.
`transcript search <pattern> --recent <n>` finds typed messages across
recent transcripts, with file and line. `transcript raw <session>
<lines>` prints the records at those lines. Start from typed messages;
widen to model text only with `--assistant`; never read a transcript
whole. Quote with the transcript and line number.
