---
description: A long-running Nexus — its sockets, clients, wire contracts and store — is being designed or judged against what the living wants it to be.
dependencies: [vision-ethos, datom]
---

## A Nexus is the whole

A Nexus is the whole long-running component: the process, its sockets, and the signal contracts it is compiled with. Nexus is its name; daemon is not. A Nexus is like a daemon, said only so that a thinking machine which thinks in daemons understands what a Nexus is. Every Nexus is named component-nexus — orchestrate-nexus, ethos-nexus — and in everyday speech orchestrate-nexus is called orchestrate.

## A kind of thing

Nexus is our word for the style of component that speaks signal and uses a similar database.

## Library and daemon

Every component built from now on is a Nexus. The nexus repository is
the library that defines the core of a Nexus component.

## Universal traits first

The basic ontology of an actor and dataflow system is designed before
implementation; signal and sema are designed against it as if new, the
old code at most inspiration.

Every method lives in a trait; an inherent method is a trait not yet extracted. The traits and types of a Nexus are one ontology designed before any body is written; defaults wherever expressible, through rich sub-trait chains. Traits live on data-bearing types; a zero-sized type with behaviour is a namespace pretending. Identity is trait-borne: an encoded form fingerprints itself, by default the hash of its rkyv archive, and every reference names its target by that name. `fn main()` is the only free function; a missing owner is a missing type.

## Processing is for the effect

An object enters a Nexus for the effect; the response follows as an
effect of it. Conversion is the wrong frame for it. The name is open,
Apply liked.

## Documents

The nexus and sema documents are undesigned; when they are designed
they live in the Nexus's main repository.

## Sockets

A Nexus opens at least two sockets. The ordinary socket serves
ordinary peers. The meta socket is privileged — the root user of the
Nexus — and configuration and privileged operations pass through it;
every Nexus has one, since without it nothing could configure the
Nexus. A Nexus that needs more levels of access opens more sockets.

## Default clients

A client is a separate program from the Nexus. For now the default
clients are packaged with the Nexus as separate crates of its
repository, which is a multi-crate repository: one datom-converting
CLI per socket, however many sockets the Nexus has, at least two. A
default client serves bootstrap first, then debugging and testing,
long after production has stopped using it. The meta CLI is named
component-meta.

A CLI turns text into Signal and nothing more: it takes one inline datom, no flags, no subcommands; it identifies the process that called it and carries that identity in the message, so a Nexus knows its caller by the process, never by a claim. A CLI speaks to one Nexus, opens no store, stays thin; datom and all text handling are compiled out of the Nexus, which decodes only known types in their rkyv form and so stays small, since it keeps running and there may be many.

## Signal only

Every client speaks to a Nexus in pure signal, fully binary. A Nexus
speaks only the signal contracts it is compiled with; two of these
are its own, one per socket. A Nexus thinks in typed values — enums,
structs, scalars — and the string fields it still carries are
records on the way to a fully typed form.

The running Nexus holds its whole domain as typed values, a specific type for every kind; no text arrives on its wire and none leaves it. Its Memory is its own typed store, reached only through the memory engine; there is no central store; policy state and working state live in that one store, and policy changes only over the meta socket.

Signal is the messaging layer: an rkyv binary archive, typed, validated on receive, length-prefixed on the socket; nothing else rides the wire. Every reply is typed, refusals included: errors are vocabulary, never strings. The wire vocabulary's version is its contract crate's semver.

## The graph

A Nexus is a vertex in the graph of nexuses. An edge joins two
vertices and carries one contract. Every connected pair has an
ordinary edge; only some pairs have a meta edge. A Nexus is compiled
with the contracts of its own sockets and of every edge it has.

Peers depend on each other's wire repositories, never on each other's Nexuses.

## Routing

Signals cross the network through a router. The router tells signal
types apart by an enum that wraps the objects, held in the signal
repository, which every component depends on. That repository also
holds what every signal needs in common — the handshake payload
among it.

## Configuration

A Nexus starts with no arguments and there is no bootstrap binary.
Its executable holds a default configuration as a constant. On start
it looks for its Sema database at the default location: a database
that exists holds the configuration; a database created new is
seeded with the defaults. The meta socket carries a Configure
interface, and changed values are accepted through it.

## First configuration

A Nexus keeps a standard metadata tree. In it a type records whether
the meta Configure was ever done; that record is reversed only on the
meta socket, and while it is unset Configure is accessible on the
ordinary socket. The tree holds everything standard about the Nexus:
its socket paths — its own and those of every edge-socket it connects
to — and whatever else comes up as standard nexus configuration data.
The built-in default configuration is independent of this and is
what gives the socket path on which the Configure signal arrives.

## Repositories

A component has three repositories: its main repository, holding all
its code, and two signal repositories — one for the ordinary
socket's contract, one for the meta socket's. Shared kinds go into
reusable libraries, which are encouraged.

`<nexus>` is the repository holding the Nexus; `signal-<nexus>` its wire vocabulary; `meta-signal-<nexus>` the owner's vocabulary, never optional, since configuration flows through it. The CLI is `<nexus>`.

A wire type repository is written in ethos and declares vocabulary only: the frame envelope, the protocol version, a closed enum of operations with their paired replies, the typed payload of each; no catch-all. Operations are verbs, `Submit`; replies the past tense, `Submitted`; rejections name themselves. Storage vocabulary never appears on the public wire.

## Why everything is a Nexus

Everything built from now on is a Nexus, and what was built in
another shape is rewritten as one. The consistency creates
reliability, quality, and clarity.

## Actors

The engine inside a Nexus is driven by Kameo actors. The standards
of their use are still to be designed. Arc-Mutex is permitted.

## Splitting a Nexus

A Nexus deals with a domain. When its features grow too many,
splitting one or more nexuses out of it is considered.

Each Nexus runs on its own and is recompiled on its own, toward zero-downtime self-update, one problem at a time.

## Observation by subscription

State is observed by subscription: the subscriber receives the state
on open, then each change as it happens.

## Polling is forbidden

Polling is forbidden; a correct system goes quiet when nothing
changes.

## Three parts and one path

A Nexus has three parts, each its own ethos specification: Signal is what it says, Operation is what it does, Memory is what it remembers. Nexus always names the whole; the part that does is Operation. Every effect has a matching operation type. A signal reaches memory only through operation; memory answers operation that the change succeeded or failed; the answer returns through operation and leaves as a signal. Reading a Nexus's ethos alone shows every object and process on that path.

```
Library                                  ; what the three parts share
[]                                       ; imports: none
[ FlowId.Integer                         ; types
  Voice.{ Aspect Layer }                 ;   who a flow speaks for, and at which layer
  Aspect.[ Psyche Mind Field ]
  Layer.[ Primary Secondary Tertiary Quaternary ]
  Event.[ Started Stopped ] ]
[]                                       ; kinds
[]                                       ; associations

Signal                                   ; what Flow says, on its ordinary socket
[ flow:[ FlowId Voice Event ] ]          ; imports from the Library
[ Launch.{ Voice                         ; queries
           Brief.String }
  Report.{ FlowId Event } ]
[ Launched.FlowId                        ; responses
  Refused.[ NoCapsule
            VoiceBusy.Voice ]
  Reported ]
[]                                       ; types

Operation                                ; what Flow does: one operation for every effect
[ flow:[ FlowId Voice Event ] ]          ; imports
[ Start.{ Voice Capsule }                ; operations
  Record.{ FlowId Event } ]
[ Started.FlowId                         ; outcomes
  Recorded
  Failed.[ CapsuleRefused StoreRefused ] ]
[ Capsule.{ Home.String                  ; types
            Login.Vector<String> } ]

Memory                                   ; what Flow remembers
[ flow:[ FlowId Voice Event ] ]          ; imports
[ Flow.{ FlowId                          ; record types
         Voice
         State.[ Running Ended ]
         Vector<Event> } ]
```

```
Launch.{ { Psyche Primary } «Draft the Nexus book» }      ; 1 Query, typed at the CLI, sent as signal
Start.{ { Psyche Primary } { /home/li/primary [] } }      ; 2 Operation, from Signal's intend
Flow.{ 7 { Psyche Primary } Running [ Started ] }         ; 3 the Change Operation hands Memory
Succeeded                                             ; 4 Memory's answer to Operation
Started.7                                             ; 5 Outcome
Launched.7                                            ; 6 Response, from Signal's answer, back to the CLI
```

## Sources

e06e4c07 nexus
01a03d6e nexus
acbb6006 nexus
98fbfa47 metaCliIsComponentDashMeta
012fbf07 threeStacks
15b67974 actorLibrary
564f55 nexus
01a05487 nexus
db97561c nexus
fd301d9a nexusTraits
f426777b nexusTraits
f426777b ethosSourceFiles
b675f3d9 ethosMonolith
fe34eb nexus
aa887c nexus
aa887c ethos
bad807 nexus
bad807 ethos
