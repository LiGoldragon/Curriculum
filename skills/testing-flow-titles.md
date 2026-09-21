---
description: A flow is spawned, its remote native title is corrected, or title alignment is called verified.
dependencies: [testing]
---

A remote title is <Aspect> <Power> <FLOW_ID>, using the seat's explicit aspect, canonical power, and own claimed Flow ID. Test both spawning and correction through each harness's supported adapter, and read back the native title. Cover wrong aspect, power, or ID; unknown role; write/readback failures; and rollback after partial mutation. Preserve shared tabs and exact route bindings. Fixtures do not establish live acceptance. Leave apply disabled for a harness without supported rename and readback; never rewrite transcripts to simulate either.
