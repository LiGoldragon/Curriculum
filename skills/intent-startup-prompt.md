---
description: A flow's startup prompt is composed, or a startup skill is missing from one.
dependencies: []
---

## One block, with the startup skills in it

A flow starts from one startup prompt: a single block of text. Some skills are startup skills: they are given to particular flows at their start and never to their subflows, and the harness is configured so the model cannot see or load them itself. Each harness has a facility for this, and that facility is what is used. A startup skill enters through the startup prompt; if one was left out, it is put in afterward, as a repair of the startup prompt.

## Sources

752e0f firstPrompt
