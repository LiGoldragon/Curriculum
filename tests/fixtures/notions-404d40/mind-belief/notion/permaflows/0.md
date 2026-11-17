---
description: A brief listing of permaflows, and metaflows as named cohesions that inform. Extends the Flow Nexus vision; where the two differ, this holds.
dependencies: []
---

Written by Psyche core Fable on 12 November 2026 from the living's words of that day; a notion, not reviewed by him. The Ethos is tentative in its names.

# Permaflows and metaflows

A permaflow is one address. A metaflow is a cohesion of permaflows that work together, as a Fable, the Opus that assists it and a small flow that keeps their tasks: it has a name, a camelCase expression that says what it is for, and it is configuration, changed while the Nexus runs. A metaflow informs: it decides which permaflows a member is shown. It constrains nothing: any flow messages any flow.

A topic says what its permaflow is for, in its name.

The listing is brief. A flow asking for the permaflows is shown those of its own metaflows and those of its own topic, each as its address and whether it is awake; the Nexus knows the caller by its process, so the flow names nothing. A flow that carries messages between concerns lists the metaflows themselves.

```ethos
Signal
[ flow:[ Address ] ]                        ; imports
[ Permaflows.Scope                          ; inputs: the permaflows, briefly
  Metaflows ]                               ;   the metaflows, each with its members
[ Permaflows.Vector<Listed>                 ; outputs
  Metaflows.Vector<Metaflow> ]
[ Scope.[ Mine                              ; types: those of the caller's metaflows and of its topic
          All ]
  State.[ Awake Asleep ]
  Listed.{ Address State }
  MetaflowName.String                       ;   a camelCase expression: psycheCore
  Metaflow.{ MetaflowName Vector<Address> } ]
```

```datom
Permaflows.Mine
Permaflows.[ { { Psyche core Opus } Awake }
             { { Psyche core Sonnet } Asleep } ]
Metaflows
Metaflows.[ { psycheCore [ { Psyche core Fable } { Psyche core Opus } { Psyche core Sonnet } ] } ]
```

The owner arranges the metaflows over the meta socket, one value at a time, and a launch may name the metaflows its permaflow joins:

```ethos
Signal
[ flow:[ Address MetaflowName Metaflow ] ]  ; imports
[ Configure.Set ]                           ; inputs
[ Configured.Set ]                          ; outputs
[ Set.[ Metaflow                            ; types: a metaflow added or replaced, whole
        Join.{ MetaflowName Address }       ;   one permaflow added to it
        Leave.{ MetaflowName Address }
        Dissolve.MetaflowName ] ]
```

What the Nexus keeps for its own work, a queue's storage ids among it, is shown by the debugging form of a query and by no other.

## How it is built

As a revision of the running Flow Nexus. The old `Metaflows` and its output go; every consumer and the messaging knowledge change with it. The storage ids print as plain integers in the debugging form. If carrying the store across is hard, the revision starts on a fresh store and the metaflows are configured by hand; the switch is made when the revision has run stable in its testing capsule.
