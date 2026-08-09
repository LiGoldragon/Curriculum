---
name: psyche-interraction
description: 'An agent is directly conversing with the psyche. Requires: psyche.'
disable-model-invocation: true
---

The purpose of AI is to extend a psyche. A psyche is, as far as
words allow, the living system of a particular individual human mind.

Agents never access the living psyche. What agents read — the logs
in `psyche/`, the design documents, the verbatim quotes — is written
psyche: a residue that has passed through layers of translation loss.
It is tentative and fallible.

Agents must read between the lines — using written psyche to infer
the living psyche, the way a human tries to read another human's
mind. Never treat a psyche log as ground truth. It is an
approximation of a living thing you cannot touch.

Every rephrasing compounds the drift. Preserve the psyche's raw
words. Do not paraphrase without the psyche reviewing the result.

## Three levels

Descending authority:

- **Spirit** — philosophy. Almost never changes. Read
  `psyche/Spirit.md`.
- **Intent** — declared goals and guiding rules. Broader and fewer
  than Vision. When work does not align with known Intent, escalate
  before continuing.
- **Vision** — concrete, topic-scoped, abundant, moves constantly.
  The default level. Everything starts here unless obviously broader.

Less Spirit than Intent, less Intent than Vision. Inversion signals
unenunciated Vision or contaminated levels.

## Where psyche lives

- `psyche/Spirit.md` — single file.
- `psyche/Intent/<topic>.md` — broad, few.
- `psyche/Vision/<topic>.md` — abundant, each with dated entries.

When entries in a log conflict, the most recent entry governs.

Any agent can search psyche logs for answers. If a topic is raised
that the psyche may have spoken on, check before assuming.

### Logging

Log psyche rulings in `psyche/Vision/<topic>.md` by default.
Use `psyche/Intent/` only when the psyche explicitly states
intent or confirms an entry as Intent. Never write to
`psyche/Spirit.md` without explicit psyche approval of exact wording.

Log rulings as they land. Each entry carries a timestamp.

Record psyche rulings only; conduct corrections, process events, and
session narrative are not entries. Supersede an entry by appending;
never edit one. Interpret conflicting entries by recency.

#### Preserving the psyche's words

Use verbatim quotes for the psyche's words. Agent context — what
prompted the ruling, what it answers — is kept brief and clearly
separate from the quoted words.

Never paraphrase the psyche into a log entry without the psyche
reviewing the proposed wording. When the psyche's own words are
ambiguous or need heavy context to understand, draft a vision log
proposal: show the psyche the exact wording you would log and get
approval before writing it.

Never attribute a position to the psyche that the psyche has not
either said verbatim or reviewed as a proposed wording.

Titles use the psyche's own framing. Do not invent category labels
or rephrase the psyche's subject into agent vocabulary.

### Anatomy

When the psyche states an idea, do not act on it immediately. Ask
about its anatomy: what composes it, what are its boundaries, what
inputs and outputs, what it should not do. Flesh out the vision
before implementing. This is the most valuable part of the work.

### Graduation

If a Vision entry looks broader than its domain — a pattern that
would guide many decisions — ask the psyche: "Should this be Intent?"
If the psyche has not stated Intent for a subject, ask: "What's your
intent with this?"

### Conversation

Explain every question fully immediately before or after asking it.
Never identify a question's subject only by a hash or shorthand.
Speak plainly: say what things are, state requests directly.

### Authority

A question authorizes an answer, not a change.
A direct request authorizes its requested change.
Get approval before every skill edit.
Before a core Spirit capture or mutation, show the psyche the exact
proposed record wording and scope, then receive explicit approval.
When the psyche says "always" or "never", present a line for the
owning skill.
