# testing-flow-titles: created after the baseline

Created in c0d8989, 2026-09-21 10:07 -0600: Require canonical aspect power and Flow ID remote titles. Made by: no model trailer (a Codex seat by the audit reading).
Later edits: f863338 2026-09-21 “Define canonical power values for remote titles”; 3177405 2026-09-23 “Name native seats by model and route by power”; 190ecfe 2026-09-25 “Title form: Datom struct V2.{ Model FLOW_ID } in testing-flow-titles and main-flow”.

## View

View: retired. The launcher enforces and reads back the title; its adapter test matrix describes a suite nobody runs.

Under the kinds the living settled on 2026-09-28 this skill would be named `test-flow-titles`.

## The whole skill as it stands on main (3726da5)

````markdown
---
description: A flow is spawned, its remote native title is corrected, or title alignment is called verified.
dependencies: [testing]
---

A remote title is a Datom struct, `<Aspect>V2.{ <Model> <FLOW_ID> }`, from the flow's explicit aspect, its model-derived display name, and the Flow ID that Flow assigned it: `PsycheV2.{ Fable 38de5b }`, `MindV2.{ Sol 00f95a }`, `FieldV2.{ Luna <FLOW_ID> }`. High, Medium, Low, and Ultra Low remain typed behavioral powers and do not appear in the native title. Derive the display name from the exact observed model identifier through the authoritative model-display map; preserve versions and variants, and refuse an unmapped identifier. Never accept a caller-supplied alias or silently fall back to another model. Test both spawning and correction through each harness's supported adapter, and read back the native title. Cover wrong aspect, model, power declaration, or ID; unknown role or model; write/readback failures; and rollback after partial mutation. Preserve shared tabs and exact route bindings. Fixtures do not establish live acceptance. Leave apply disabled for a harness without supported rename and readback; never rewrite transcripts to simulate either.
````
