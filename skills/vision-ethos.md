---
description: An ethos file is written, or a type, kind or layout is judged against what the living wants ethos to be.
dependencies: [datom, protos]
---

Ethos is the schema language: it writes the mental model and the code in one swoop. Ethos specifies the types; datom fills them with data. Any repetition in ethos syntax is an implementation failure.

Four roots: Library, Signal, Operation, Memory. Signal declares what a Nexus says; Operation what it does, one operation type for every effect; Memory what it remembers; Library what they share. A memory kind carries a standard successful-or-unsuccessful change and, for each version, the upgrade from the previous format; that upgrade is the very edit the type needs. An ethos file carries no version; a version lives in a manifest.

Everything is a type; there is no key-value. A type used once is declared inline where it is used; a type used in more than one place is declared once and named. A variant's payload is written in the variant and bears the variant's name; no second type is invented to hold it. Inline nesting goes about three deep; past that, the type comes from a Library.

Ethos expands vertically: a structure with more than one element opens on its line and its elements hang beneath the first, aligned; the closing delimiter ends the last element's line. Nothing that has a next layer sits on one line.

A kind is the bearer of capabilities and is qualifier-named: Launchable, Streamable. A capability speaks in Self, the kind's own parameters and other kinds; a concrete type in an input is a kind not yet named. In ethos there are no generics, only kinds; a constraint is a kind, never a type.

The Flow Nexus in the four roots:

```
Library
[]
[ FlowId.Integer
  Voice.[ Psyche.Rank
          Mind.Rank
          Field.Rank ]
  Rank.[ Primary Secondary Tertiary ]
  Event.[ Started
          ToolUsed.String
          Stopped ] ]
[]
[]

Signal
[ flow:[ FlowId Voice Event ] ]
[ Launch.{ Voice
           Brief.String }
  Report.{ FlowId Event } ]
[ Launched.FlowId
  Refused.[ NoCapsule
            VoiceBusy.Voice ]
  Reported ]
[]

Operation                                 ; sections proposed, not yet his word
[ flow:[ FlowId Voice Event ] ]
[ Start.{ Voice Capsule }
  Record.{ FlowId Event } ]
[ Started.FlowId
  Recorded
  Failed.String ]
[ Capsule.{ Home.String
            Login.Vector<String> } ]

Memory
[ flow:[ FlowId Voice Event ] ]
[ Flow.{ FlowId
         Voice
         State.[ Running Ended ]
         Vector<Event> } ]
```
