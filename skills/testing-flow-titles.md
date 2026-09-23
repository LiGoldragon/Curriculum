---
description: A flow is spawned, its remote native title is corrected, or title alignment is called verified.
dependencies: [testing]
---

A remote title is <Aspect> <Model> <FLOW_ID>, using the seat's explicit aspect, model-derived display name, and own claimed Flow ID. For example, the Medium Mind seat on `gpt-5.6-sol` is `Mind Sol <FLOW_ID>`. High, Medium, Low, and Ultra Low remain typed behavioral powers and do not appear in the native title. Derive the display name from the exact observed model identifier through the authoritative model-display map; preserve versions and variants, and refuse an unmapped identifier. Never accept a caller-supplied alias or silently fall back to another model. Test both spawning and correction through each harness's supported adapter, and read back the native title. Cover wrong aspect, model, power declaration, or ID; unknown role or model; write/readback failures; and rollback after partial mutation. Preserve shared tabs and exact route bindings. Fixtures do not establish live acceptance. Leave apply disabled for a harness without supported rename and readback; never rewrite transcripts to simulate either.
