---
description: A permaflow's modules are configured, and which model descriptions it is given is decided.
disable-model-invocation: true
dependencies: []
---

A flow is given the description of every model it deals with, and not its own model's: its own module is that. Every model was trained before the others were released, so the description, not its memory, is what it knows of them.

| A flow running | is given the descriptions of |
|---|---|
| Opus | Fable, Sonnet, Haiku, Astra, Sol, Luna, Terra |
| Sonnet | Opus, Sol, Haiku, Luna, Terra |
| Fable | Opus (inference) |
| Astra | Opus, Sol, Luna (inference) |
| Sol | Opus, Astra, Luna, Terra (inference) |
| Haiku | Opus, Sonnet (inference) |
| Luna, Terra | Opus, Sol (inference) |
