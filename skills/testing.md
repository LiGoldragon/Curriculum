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

A new test is seen failing once before it is trusted.
The expected value comes from outside the code under test; a test
that computes it through the tested path confirms nothing.
A test waits on the tested event, never on the clock.
Tests share no mutable state — no process environment, no working
directory, no order between them.
A run that may exhaust memory or time is bounded (a memory cap and a timeout) so that it cannot take the harness down with it.
Stop a process a test started by the PID that test holds, never by a process-name or path pattern — a scratch and a production instance of the same build share that pattern.

Live acceptance has a boundary. A fixture or generated-output test proves its
own contract; an isolated transport test proves only its named receipt grade;
an end-to-end live acceptance needs the actual selected identity, binding, and
target-side observation. Report an unavailable native route as unavailable,
not as a failed simulation or a passing deployment test.

When assigned as a testing worker, accept a bounded target, immutable revision, authority limits, and acceptance contract. Choose the test procedure, fixtures, negative cases, and independent oracle yourself; do not mirror the implementation or a main flow's assertion. Run the smallest test that can distinguish acceptance from a plausible failure, including a rejected or failing case before trusting a new test. Keep source/projection ownership, native binding and receipt, deployment parity and rollback, transport versus target read, passive no-wake observation, and context metric freshness distinct when those boundaries matter. Report what each witness proves, the exact revision and scope tested, and what remains unavailable. Do not wake a production flow merely to test status or delivery.
