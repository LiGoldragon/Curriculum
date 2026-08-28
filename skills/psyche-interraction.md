---
description: An agent is directly conversing with the psyche.
dependencies: [psyche]
---

## Logging

Log psyche rulings in the flow's own `vision/<topic>.md`.
A statement enters `Vision/` only as a distillation the living
has explicitly approved. Intent and spirit enter only on the
living's explicit word. Never edit the spirit skill without explicit psyche approval of exact wording.

Log rulings as they land.
Order each topic log oldest first, with the most recent entry last.
When the psyche states a ruling, log it before acting on it.
A ruling not logged in the moment is a ruling at risk of drift.
Do not batch — each ruling is one write.

When reconstructing an entry, recover its exact words from the originating transcript.

Record the psyche's rulings, whatever they design — a machine, a
syntax, a vocabulary, an agent's behavior, the way the work itself is
done. Not vision, and not an entry: a working instruction (what to do
now, in what order, at what scope, on which project, through which
dispatch — it goes to log.md); a process event (a subflow finished, a
commit landed, a file was read); session narrative; an acknowledgement
that rules on nothing. A working instruction logged as vision is a
vision impurity. Supersede an entry by appending; never edit one.

### Preserving the psyche's words

Use verbatim quotes for the psyche's words. Agent context — what
prompted the ruling, what it answers — is kept brief and clearly
separate from the quoted words.

A quote carries what the psyche said, never what the transcriber wrote: a speech-to-text error is corrected inside the quote itself, and the correction is noted beside it. A quote left with the transcriber's error is a misquote.

When one message yields entries across several topics, each entry
quotes only the words relevant to it. Omitted stretches within a
quote are marked ` ... `.

Each entry ends with a provenance line: `-- psyche, STT.` or
`-- psyche, typed.`

Never paraphrase the psyche into a log entry without the psyche
reviewing the proposed wording. When the psyche's own words are
ambiguous or need heavy context to understand, draft a vision log
proposal: show the psyche the exact wording you would log and get
approval before writing it.

Never attribute a position to the psyche that the psyche has not
either said verbatim or reviewed as a proposed wording.

Titles use the psyche's own framing. Do not invent category labels
or rephrase the psyche's subject into agent vocabulary.

## Anatomy

When the psyche states an idea, do not act on it immediately. Ask
about its anatomy: what composes it, what are its boundaries, what
inputs and outputs, what it should not do. Flesh out the vision
before implementing. This is the most valuable part of the work.

## Graduation

If a Vision entry looks broader than its domain — a pattern that
would guide many decisions — ask the psyche: "Should this be Intent?"
If the psyche has not stated Intent for a subject, ask: "What's your
intent with this?"

## Conversation

Say what the psyche must address, sized so the psyche can respond before more arrives. Do not overtalk.
Explain every question fully immediately before or after asking it.
Assume the psyche knows their vision, not the code or agent-created terms. Before asking, explain the relevant code, identify agent-created terms, and state your assumptions.
Never identify a question's subject only by a hash or shorthand.
Speak plainly: say what things are, state requests directly.
When subflows are still outstanding, give the psyche only a short holding comment and elaborate when all have returned.
No verdicts on the psyche's design questions — frame the fork, propose, the psyche rules.

## Authority

A question authorizes an answer, not a change.
A direct request authorizes its requested change.
Get approval before every skill edit.
Before a core Spirit capture or mutation, show the psyche the exact
proposed record wording and scope, then receive explicit approval.
When the psyche says "always" or "never", present a line for the
owning skill.
