---
description: Constructing, reading, or interpreting datom, or implementing Datomic.
dependencies: [protos]
---

Datom is the pure-data dialect on the protos substrate. It carries data, strictly typed, and its whole work is serialization and deserialization. Schema-driven and positional: the reader walks the expected type, writing is the exact reverse projection. All naming lives in the type; the text carries only the data.

## Text forms

Each form occupies the position its type declares. What a structure means is said by the position, never by the structure alone.

Struct — braces, positional:
```
; Person: a struct of name Text, born Integer, address Address, roles Vector<Role>.
{ Ada 1990 { “12 Rue de la Paix” Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
```

Vector — brackets:
```
[ 0 42 -42 ]
```

Map — guillemets, key and value by position:
```
« home { “12 Rue de la Paix” Paris 75002 }  work { “1 Place Vendôme” Paris 75001 } »
```

Variant — a head alone for a variant carrying nothing; a head, the dot, and a body for a variant carrying data:
```
Pending
Accepted.{ 42 2026-09-03T17:46:20 }
Observed.Locks.[]
```

String — bare when it contains no space and no delimiter; curly-quoted otherwise. In a string position a bare word may contain characters that are syntax elsewhere:
```
Ada
“no such file: { } is content”
name:first
```

Integer — bare ASCII decimal, optional leading `-`, no `+`, no leading zero except `0` itself:
```
0  42  -42
```

Decimal — finite, point-mandatory:
```
3.14  -0.5
```

Boolean:
```
True  False
```

Meaning — parenthesized text, read by balance:
```
(The build passed on the third try (after two timeouts))
```

## The CLI

A datom-speaking CLI takes exactly one inline datom value and no flags. Its type system is the only interface.

```sh
orchestrate 'Lock.{ MyLock 6329f1 [ /abs/path ] "why I hold it" }'
# -> Locked.{ 442 MyLock 6329f1 [ /abs/path ] “why I hold it” }
```

With no argument, a CLI prints its contract's ethos.

## Datomic in Rust

A Rust type bears `Datomic` through two capabilities — `incorporate` (static, constructs the value from a `Datom`) and `datomize` (projects the value into a `Datom`):

```rust
// Corporal: the kind whose static capability takes a concept and yields Self.
pub trait Corporal<C: Protosizable>: Embodied {
    type Fault;
    fn incorporate(concept: C) -> Result<Self, Self::Fault>;
}

// Datomic: the corporal kind of the datom dialect.
pub trait Datomic: Corporal<Datom, Fault = Fault> {
    fn datomize(&self) -> Datom;
}

// Provided for every Datomic: datomize -> protosize -> print.
pub trait Textualizable {
    fn textualize(&self) -> protos::Text;
}
```

`Potential<T>::actualize()` chains the whole descent — delineate, conceive, incorporate — and may fault. `Textualizable::textualize()` chains the whole ascent — datomize, protosize, print — and cannot fault:

```rust
let potential = Potential::<Lock>::from(text);
let lock: Lock = potential.actualize()?;
let text: protos::Text = lock.textualize();
```

Every ethos-declared type gets its `Datomic` generated. No hand-written Datomic implementations for declared types.
