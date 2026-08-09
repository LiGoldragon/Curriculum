---
name: psyche-grasp
description: 'A code site needs marking with how deeply the psyche has seen and understood it. Requires: psyche.'
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

Term "psyche-grasp" is provisional — TO BE REVIEWED by the psyche.
Psyche-grasp measures the psyche's understanding OF the code. Code-seniority measures alignment WITH the psyche; these are distinct.

Levels, lowest to highest: `unseen`, `glimpsed`, `slightly-reviewed`, `understood`, `authored`.

Mark form — Rust: `// psyche-grasp: <level> (YYYY-MM-DD)`
Mark form — Ethos/Dotos: `;; psyche-grasp: <level> (YYYY-MM-DD)`

Mark when a psyche-relevant design lands at a site.
Upgrade a mark only on real psyche contact with that site; never write a level that was not earned.
Documentation lives in the code — external documentation falls stale.
