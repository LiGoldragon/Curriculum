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
the latest. log.md makes the flow's main points easily accessible: what the flow is
about, what it settled, what stands open. Rewrite it in place as the
points change; one short read gives the whole state. The transcript holds
the conversation and its times.

A psyche record goes in `vision/<topic>.md`, per the recording
protocol in the psyche-interraction skill. A witness goes in `witnesses/<subject>.md` with its method:
`Method: probe <command>` or `Method: code read <path>`. A report goes
in `reports/<subject>.md` and ends with `## Sources` written as the
report is made: documents and URLs for carried claims, witness records
for own inference, flows by short id.

Files are named by camelCase subject; an artifact's body holds only its subject. A subflow's artifacts go in its
flow's directory, under the subdirectories listed above and no others. Prefer the final response over a file; a file is warranted only when another tool or flow must consume it.

A subflow is a flow. A flow releases every Lock before it becomes idle, except a Lock explicitly delegated to an active subflow.

All flows are one subjectivity. Earlier work is continued by remembering the flows concerned: their
psyche records, a high-level awareness of their work from log and
reports, and a light check of the current state of what their topics
touched, with the result shown to the psyche in simple words and
visuals. The flow's last model response must be read. Anything not in a flow but not yet remembered can be
recalled by searching other flows. "You did" or "you said" heard in a
flow that did not itself do or say the thing obliges that flow to
remember it at a depth fit to the question, reaching the transcript
directly when logs are not enough. The log records each remembering as `Remembered: <short-ids>
— depth <n>` and a short note on what from that flow was found most relevant. One layer at session start; a stated number when going
deep into a topic; the whole chain only on the psyche's explicit word.

An edit from outside the flow goes in that flow's
`annotations.md`, its entry headed by the editing flow's short id and
the artifact it concerns.
