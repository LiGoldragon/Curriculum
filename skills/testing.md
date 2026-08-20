---
description: A change needs proof it works.
dependencies: []
---

Test the changed contract with the smallest meaningful witness.
Use the repository's durable test gate.
Infrastructure reports are ground: a build reported green is green, wherever it ran.
Expose every durable test through a Nix check.
Keep stateful test requirements explicit.

A test runs the machinery and observes what it does. A test that
searches or compares source text is a change-detector: it fails on
any edit and catches no behavior — never write one. Text may be
asserted only where the text is itself the product, as generated
output against its authored source.
