---
description: A Rust method is written outside a trait, or the trait surface of a crate is designed.
dependencies: []
---

## 2026-08-13 — approved

> Every method call in our Rust code lives under a trait, because
> traits are the comprehension surface — the layer where concepts
> become visible and implementations are constrained to think within
> them. Rust is the new assembly language: no serious engineer reads
> all the assembly, and the same is happening to Rust. Traits and
> main types are what the psyche reads; everything else is
> implementation detail that Ethos will eventually generate.

— psyche-approved wording, 2026-08-13 (Steward session d2bb5f5f).
Proposed by Steward, approved with "otherwise its good, implement
commit and deploy."
