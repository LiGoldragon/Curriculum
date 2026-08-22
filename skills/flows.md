---
description: A session is starting, or work has produced something with no home repository.
dependencies: [vocabulary]
---

One directory per flow: `flows/<short-id>/`, the short id being the
first 8 hex of the session UUID.

    flows/
      index.md
      <short-id>/
        log.md
        annotations.md
        vision/<topic>.md
        witnesses/<subject>.md
        reports/<subject>.md

Create `log.md` at the first prompt and append the flow's line to
`flows/index.md` as `<aspect>, <short-id>, <description>`; the tail is
the latest. The log opens with a very terse summary, re-edited as the
session goes, and changes further when asked, at handoff, or when the
handoff state materially changes.

A psyche record goes in `vision/<topic>.md`, the psyche's words
verbatim. A witness goes in `witnesses/<subject>.md` with its method:
`Method: probe <command>` or `Method: code read <path>`. A report goes
in `reports/<subject>.md` and ends with `## Sources` written as the
report is made: documents and URLs for carried claims, witness records
for own inference, flows by short id.

Files are named by camelCase subject; the log holds the dates; an
artifact's body holds only its subject. A subflow's artifacts go in its
flow's directory.

Earlier work is continued by remembering the flows concerned: their
psyche records, a high-level awareness of their work from log and
reports, and a light check of the current state of what their topics
touched, with the result shown to the psyche in simple words and
visuals. The log records each remembering as `Remembered: <short-ids>
— depth <n>`. One layer at session start; a stated number when going
deep into a topic; the whole chain only on the psyche's explicit word.

An edit from outside the flow goes in that flow's
`annotations.md`, its entry headed by the editing flow's short id and
the artifact it concerns.
