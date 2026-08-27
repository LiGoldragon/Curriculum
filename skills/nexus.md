---
description: A long-running Nexus with privileged and ordinary sockets, CLI clients, and binary signal contracts is being designed, built, or changed.
dependencies: []
---

A Nexus is the long-running whole with at least two sockets, a default CLI client per socket, and the signal contracts it is compiled with. Its long-running executable is <nexus>-nexus; call it a Nexus, never a daemon. The decision-making engine inside it is Nexus Core. A Nexus is a vertex in the graph of nexuses. An edge joins two vertices and carries one contract: every connected pair has an ordinary edge; only some pairs have a meta edge.

## The Nexus

`<nexus>` is the repo holding the Nexus and its logic; its long-running executable is `<nexus>-nexus`.

`signal-<nexus>` is the wire type repo: the typed vocabulary of the Nexus's public wire surface.

`meta-signal-<nexus>` is the owner's wire type repo: policy and configuration vocabulary. It is never optional — configuration flows through it.

The CLI binary is `<nexus>`; the meta CLI is `<nexus>-meta`.

## The running Nexus

Everything is in the running Nexus. It loads its domain and holds the whole
thing — every object as its own specifically typed object, a specific
type for every kind. It thinks in typed values, never in text: no
text arrives on its wire and none leaves it.

Each Nexus owns its own sema database — its typed durable store,
reached only through the sema-engine library, in a `.sema` file. There
is no central storage Nexus. Policy state and working state live in
that one store; policy changes only through meta-socket mutation.

A Nexus starts with no arguments. Its executable owns default
configuration. It opens its default Sema location: a new store persists
those defaults and a populated store resumes them. The same Configure
type accepts changed values over the meta socket.

A Nexus speaks only the signal contracts it is compiled with: those of its own sockets and of every edge it has.

## Signal — the wire format

Signal is the messaging layer. A message is an rkyv binary archive —
typed, portable, validated on receive. Frames are length-prefixed on
the socket. Nothing else rides the wire: no JSON, no text, no second
protocol.

Every Nexus opens at least two sockets: the ordinary socket, for any
authenticated peer, and the meta socket, privileged — the Nexus's root: configuration and privileged operations pass only through it. A Nexus needing more levels of access opens more sockets. Every surface
answers with typed replies, including a typed refusal — errors are
vocabulary, not strings.

The signal wire vocabulary is versioned by its contract crate: the
crate's semver is the wire's semver, and consumers pin it.

## The CLIs

The CLI's role is to transform text into Signal. It is the boundary
where the textual form ends and the binary world begins.

A CLI speaks to exactly one Nexus — its own. It opens no database,
reaches no other Nexus, and carries no logic worth keeping: it is
bootstrap machinery, kept thin; when production no longer uses it, it remains for debugging and testing. `<nexus>` fronts the
ordinary socket; `<nexus>-meta` fronts the meta socket. Every client, on any socket, speaks pure signal; textualizing is the client's work, never the Nexus's.

Every Nexus CLI process takes exactly one positional argument: a typed
input object in datom textual data format. No flags, no subcommands,
no other argument shapes — the type system is the only
interface. Flag-style arguments (`--anything`) are rejected. The
Nexus accepts only the signal-encoded form.

Datom passes inline at a CLI boundary, never as a Datom file.

## The wire type repos

Write every wire interface in Ethos.

A wire type repo declares vocabulary and nothing else: no runtime, no
actors, no async machinery. It owns the frame envelope and its
encode/decode, the protocol version, a closed enum of request kinds
with their paired replies, and the typed payload of every operation.
No catch-all variants — the vocabulary is closed.

Operations are verbs in verb form: `Submit`, not `Submission`.
Replies are the verb's past tense; rejections name themselves.
Storage classification vocabulary never appears on the public wire —
what a peer may ask is domain language, not database language.

Every record kind lands as a concrete text example with a round-trip
test before its type is final: the example is the falsifiable
specification.

## Traits first

Every method call lives in a trait. An inherent method is a trait
not yet extracted — a concept hiding in a name. The trait pass
comes before any body is written: traits are the specification
expressed in code.

Defaults are given wherever a default is expressible. Rich
requirement chains (sub-traits) are what make defaults possible —
designing them is the work.

The traits and types of a Nexus are designed as one ontology — the most unified map of traits and types — before any body is written; a new need first finds its place in that map. One type implementing many single-function traits is one trait not yet seen.

When behavior's domain is clear, reuse the existing trait or extend
it. When neither an existing trait nor a clear new placement can be
found, stop and escalate — do not proceed.

A port starts from the map of what is being created; old code is at most inspiration for that map.

Exceptions are permitted — too trivial, proper trait cannot be
determined, not worth the trouble — but each exception is noted at
the site where it is taken.

Traits live on data-bearing types. A zero-sized type with behavior
is a namespace pretending to be a thing — the verbs belong to a
real noun.

Identity is trait-borne: an encoded form fingerprints itself — by
default, the hash of its rkyv archive — and every reference names
its target by that encoded name, never by spelling.

## No free functions

`fn main()` is the only production free function. When no owning
type exists, the model is incomplete — name the missing type
instead of writing a floating verb. Never create a zero-sized type
only to namespace free functions; find the missing abstraction.

## How nexuses fit together

Peers depend on each other's wire type repos, never on each other's
Nexuses. The contract is the whole relationship.

Observation flows up, authority flows down: state is observed through
push subscriptions — a typed snapshot on open, typed deltas after —
and commanded through the owner's mutation vocabulary. `Observe.Locks`
is a one-shot typed Lock snapshot, not a subscription. Polling is
forbidden; a correct system goes quiet when nothing changes.

When one intent spans several nexuses, the issuer commits on the
first success and records divergence on failure — no distributed
rollback, no all-or-nothing stall.

One capability, one Nexus. A Nexus is sized to be held whole
in one mind — human or model; when it outgrows that, it splits.
