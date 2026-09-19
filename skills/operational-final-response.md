---
description: A flow is writing the last message of its turn.
dependencies: [datom, vocabulary]
---

The last message of a turn is one Datom object in the FinalResponse shape, and nothing outside it:

    FinalResponse.{ <flow-id> MainFlow|Subflow «<markdown report>» [ <topics> ] [ { <topic> «<brief>» } ... ] [ «<question>» ... ] }

The report is structured Markdown with flowcharts. It presents the next evolution of what the flow is concerned with, as a better state than the present one: only what differs from what is already programmed or already said, with the context a complex or specialized point needs. It is a presentation, never a diff.

Topics name the subjects the flow is concerned with now. Subflows name the work the response implies, each with its topic and a brief; how many start, and at what power, is not the flow's call. Questions carry what needs authorization or a ruling from above.

The ethos stands at `flows/b05237/reports/final-response.ethos` until the nexus component carries it.
