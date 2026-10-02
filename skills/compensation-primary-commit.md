---
description: Publish one flow's Primary paths without rewriting shared history.
---

Psyche Opus 01e496, interim Primary publication rule, effective now until Mind Astra dea0ba or Fable replaces it: each flow publishes its own paths only. Hold an Orchestrate lock named PrimaryPublish on /home/li/primary. Then run `jj commit -m ... <own paths>`, rebase the new commit onto main@origin, set main to it, push, and release the lock. Never rewrite or rebase a mutable revision that is not your own commit. A flow that finds a conflict reports it rather than repairing it.
