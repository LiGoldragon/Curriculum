---
description: A long-running Nexus — its sockets, clients, wire contracts and store — is being designed or judged against what the living wants it to be.
dependencies: [vision-ethos, datom]
---

A Nexus is the long-running whole: its executable `<nexus>-nexus`, at least two sockets, one default CLI client per socket, and the signal contracts it is compiled with. It is a Nexus, never a daemon. Its three parts are the three layers of ethos: Signal is what it says, Operation is what it does, Memory is what it remembers. A Nexus is a vertex in the graph of nexuses; an edge joins two vertices and carries one contract; every connected pair has an ordinary edge, only some a meta edge.

`<nexus>` is the repository holding the Nexus; `signal-<nexus>` its wire vocabulary; `meta-signal-<nexus>` the owner's vocabulary, never optional, since configuration flows through it. The CLI is `<nexus>`, the meta CLI `<nexus>-meta`.

The running Nexus holds its whole domain as typed values, a specific type for every kind; no text arrives on its wire and none leaves it. Its Memory is its own typed store, reached only through the memory engine; there is no central store; policy state and working state live in that one store, and policy changes only over the meta socket. A Nexus starts with no arguments: its executable owns the defaults, a new store persists them, a populated store resumes them, and the same Configure type accepts changed values over the meta socket. A Nexus speaks only the contracts it is compiled with: its own sockets' and every edge's.

Signal is the messaging layer: an rkyv binary archive, typed, validated on receive, length-prefixed on the socket; nothing else rides the wire. The ordinary socket serves any authenticated peer; the meta socket is the Nexus's root, where configuration and privileged operations pass; more levels of access open more sockets. Every reply is typed, refusals included: errors are vocabulary, never strings. The wire vocabulary's version is its contract crate's semver.

A CLI turns text into Signal and nothing more: it takes one inline datom, no flags, no subcommands; it identifies the process that called it and carries that identity in the message, so a Nexus knows its caller by the process, never by a claim. A CLI speaks to one Nexus, opens no store, stays thin; datom and all text handling are compiled out of the Nexus, which decodes only known types in their rkyv form and so stays small, since it keeps running and there may be many. The CLI is bootstrap and remains for debugging and testing when production no longer uses it.

A wire type repository is written in ethos and declares vocabulary only: the frame envelope, the protocol version, a closed enum of operations with their paired replies, the typed payload of each; no catch-all. Operations are verbs, `Submit`; replies the past tense, `Submitted`; rejections name themselves. Storage vocabulary never appears on the public wire.

Every method lives in a trait; an inherent method is a trait not yet extracted. The traits and types of a Nexus are one ontology designed before any body is written; defaults wherever expressible, through rich sub-trait chains. Traits live on data-bearing types; a zero-sized type with behaviour is a namespace pretending. Identity is trait-borne: an encoded form fingerprints itself, by default the hash of its rkyv archive, and every reference names its target by that name. `fn main()` is the only free function; a missing owner is a missing type.

Peers depend on each other's wire repositories, never on each other's Nexuses. State is observed by subscription: the state on open, then each change; polling is forbidden, and a correct system goes quiet when nothing changes. A Nexus deals with one domain; grown too large, it splits. Each Nexus runs on its own and is recompiled on its own, toward zero-downtime self-update, one problem at a time.

The Flow Nexus's four ethos files in vision-ethos are the example.
