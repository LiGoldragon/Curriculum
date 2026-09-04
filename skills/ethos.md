---
description: Writing or reading an ethos file, or generating Rust from one.
dependencies: [protos, datom]
---

Ethos is the schema language. It specifies the types; datom fills them with data. Ethos generates the Rust.

## File roots

Two roots: `Library` and `Signal`. A file is one sweet ethos object; the outer braces are omitted and always implied:

```
; A Library file (sweet form). The full form wraps everything in Library.{ ... }.
Library.{ 0 1 0 }
[ protos:[ Text ] ]                                            ; imports
[ Scores.Vector<Integer>                                       ; types
  Record.{ Text Scores } ]
[ Summarizable.[ summarize.[ Text ] ] ]                        ; kinds
[ Record.[ Summarizable ] ]                                    ; associations
```
```rust
pub type Scores = Vec<protos::Integer>;
pub struct Record { pub text: protos::Text, pub scores: Scores }
// Association: compile-time assertion; the impl body is hand-written.
const _: fn() = || { fn carries<T>() where T: Summarizable {} carries::<Record>(); };
impl datomic::Datomic for Record { /* generated from anatomy */ }
```

```
; A Signal file (sweet form).
Signal.{ 1 0 0 }
[ ]                                                            ; imports
[ Lock.LockRequest  Release.LockId  Observe.ObserveSelection ]  ; requests
[ Locked.Lock  LockRejected.LockRejection  Released.Lock  ReleaseRejected.ReleaseRejection  Observed.Observation ]
[ LockId.Integer  LockName.Text  LockRequest.{ LockName FlowId LockPaths LockReason } ... ]  ; types
```
```rust
pub type LockId = protos::Integer;
pub enum Request { Lock(LockRequest), Release(LockId), Observe(ObserveSelection) }
pub enum Reply { Locked(Lock), LockRejected(LockRejection), /* ... */ }
```

## Type declarations

`Name.{ ... }` -- a struct; fields are positional, named by their type:
```
Lock.{ LockId LockName FlowId LockPaths LockReason }
```
```rust
pub struct Lock { pub lock_id: LockId, pub lock_name: LockName, /* ... */ }
```

`Name.[ ... ]` -- an enum; each variant bare or carrying an inline payload:
```
SinkError.[ Closed Full ]
LockRejection.[ DuplicateName.Lock  PathOverlap.LockOverlap ]
```
```rust
pub enum SinkError { Closed, Full }
pub enum LockRejection { DuplicateName(Lock), PathOverlap(LockOverlap) }
```

`Name.Type` -- an alias; `Name.« K V »` is a map alias:
```
LockId.Integer
Roles.« Text Integer »
```
```rust
pub type LockId = protos::Integer;
pub type Roles = BTreeMap<protos::Text, protos::Integer>;
```

## Kinds

A kind is the bearer of capabilities. The `.` receiver takes self, `!` takes mutable self, `:` takes no self.

Simple kind -- capabilities in a bracket:
```
Summarizable.[ summarize.[ Text ] ]
```
```rust
pub trait Summarizable { fn summarize(&self) -> protos::Text; }
```

Complex kind -- a struct of superkinds, associated types with their constraints, associated constants in `« UPPER_CASE Type »`, and capabilities:
```
Streamable.{ [ Fillable ]
             [ Item<Serializable> ]
             « CAPACITY Integer »
             [ next![ Option<Item> ] ] }
```
```rust
pub trait Streamable: Fillable {
    type Item: Serializable;
    const CAPACITY: protos::Integer;
    fn next(&mut self) -> Option<Self::Item>;
}
```

A capability's inputs and yield are each a bracket holding one type:
```
push!{ [ Text ] [ Result<Integer SinkError> ] }
create:[ Self ]
```
```rust
fn push(&mut self, input: protos::Text) -> Result<protos::Integer, SinkError>;
fn create() -> Self;
```

Kind identity is the name and the constraints, written as one head:
```
Processable<[Clonable Sendable] Serializable>.[ ... ]
```
```rust
pub trait Processable<A: Clone + Send, B: Serialize> { /* ... */ }
```

## Associations

A type bears a kind. The generated code asserts the type bears the kind at compile time; the impl body is hand-written:
```
[ Sink.[ Summarizable Fillable ] ]
```
```rust
const _: fn() = || { fn carries<T>() where T: Summarizable + Fillable {} carries::<Sink>(); };
```

Every ethos-declared type gets `impl datomic::Datomic` generated from its anatomy.

## Imports and intrinsics

`[ protos:[ Text Textualizable ] ]` imports names from another library. Intrinsic names known without import: Text, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self.

## Generation

`ethos-zero` generates the Rust. Generated Rust is committed; a freshness test asserts the committed output matches a fresh generation. The CLI speaks datom:

```sh
ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
# -> Generated.[ /abs/out-dir/signal.rs ]
```

With no argument, `ethos-zero` prints its own ethos.

## Non-repetition

Any repetition in ethos syntax is an implementation failure.
