---
description: A flow is writing the last message of its turn.
dependencies: [datom, ethos, vocabulary]
---

The last message of a turn is one datom in the FinalResponse type, and nothing outside it: written bare, never inside a code block and never indented, so its Markdown string renders as Markdown. The type, as a Type ethos:

    Type
    FinalResponse.{ FlowId Kind Markdown Vector<Topic> Vector<Subflow> Vector<Question> }
    [ FlowId.String  Kind.[ MainFlow Subflow ]  Markdown.String  Topic.String  Subflow.{ Topic Markdown }  Question.Markdown ]

The report is structured Markdown with flowcharts. It presents the next evolution of what the flow is concerned with, as a better state than the present one: only what differs from what is already programmed or already said, with the context a complex or specialized point needs. It is a presentation, never a diff.

Topics name the subjects the flow is concerned with now. Subflows name the work the response implies, each with its topic and a brief; how many start, and at what power, is not the flow's call. Questions carry what needs authorization or a ruling from above.

An object a flow defines is always given its ethos representation as a type: the object first, then the vector of type definitions needed to fill it.

The last message of a flow's life, when it is refreshed, is its refresh payload: a presentation, in the flashbook shape, of what the flow learned that amends the initial prompt it was made from. It is passed to the successor as an addendum appended after the previous payload; what has since been merged into files is trimmed from it, so the addendum does not accumulate. The last message of every turn is a presentation; a low-power flow may turn it into a flashbook.
