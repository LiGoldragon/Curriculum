---
description: Writing or reading an ethos file, or generating Rust from one.
dependencies: [protos, datom]
---

Ethos is the schema language: it specifies the types, datom fills them with data, and ethos-zero generates the Rust. In ethos there are no generics, only kinds. Any repetition in ethos syntax is an implementation failure.

## Roots and file shape

Three roots: Library, Signal, Sema. Signal gives a Nexus its main types and Sema its database types. An ethos file carries no version; what is versioned is versioned in a manifest.

The unit is File: one file, one Rust module, no namespace inside it. A file is written in the sweet form — the root's head, then the sections as siblings, the outer braces omitted — and is converted mechanically to the canonical braced form before it is read as ethos. A Library's sections in order are imports, types, kinds, associations; a Signal's are imports, queries, responses, types; a Sema's are imports, record types. In the Signal and Sema roots the associations of the query, response and record types are implied and never written.

```
Library                                   ; the sweet form, as a file is written
[]                                        ; imports
[ Record.{ String Integer } ]             ; types
[]                                        ; kinds
[]                                        ; associations

Library.{ [] [ Record.{ String Integer } ] [] [] }   ; the canonical form the reader sees
```
```rust
#[derive(datom_codec::Datomizable, datom_codec::Compositional, Clone, Debug, PartialEq)]
pub struct Record { pub string: String, pub integer: i64 }
```

## Declarations

`Name.Type` is an alias, `Name.{ … }` a struct, `Name.[ … ]` an enum. A field is named after its type in snake case; a constructed type type-first, `string_vector`, `lock_option`; a repeated type as first and second. A variant either names a type already defined, which is then the data it carries, or declares its payload in place — a vector, a struct or an enum, each a full type whose derived name carries `_Data`, recursively.

```
Signal
[]                                                   ; imports
[ Lock.LockRequest  Release.LockId ]                 ; queries
[ Locked.Lock  LockRejected.LockRejection ]          ; responses
[ LockId.Integer                                     ; types
  LockName.String
  LockPath.String
  LockRequest.{ LockName Vector<LockPath> }
  Lock.{ LockId LockName }
  LockRejection.[ DuplicateName.Lock                 ;   a variant naming a defined type carries that type
                  PathOverlap.{ Lock Lock } ] ]      ;   a variant declaring its payload inline
```
```rust
pub type LockId = i64;
pub type LockName = String;
pub type LockPath = String;
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Compositional))]
pub struct LockRequest { pub lock_name: LockName, pub lock_path_vector: std::vec::Vec<LockPath> }
// … Lock and PathOverlap_Data likewise
pub struct PathOverlap_Data { pub first_lock: Lock, pub second_lock: Lock }
pub enum LockRejection { DuplicateName(Lock), PathOverlap(PathOverlap_Data) }
pub enum Query    { Lock(LockRequest), Release(LockId) }
pub enum Response { Locked(Lock), LockRejected(LockRejection) }
```

Ethos Zero emits the datom kinds on every struct and enum it generates; an alias bears them through the type it names and carries no derive. A Signal's types gate them behind a `datom` feature that the CLI and client enable and the Nexus does not, so the Nexus compiles its contract without datom-codec. No tuple in the code we design; where a standard trait or a dependency requires one it is allowed at that contact point only.

## Imports and intrinsics

An import names a source and a type: `protos:String`, or `protos:[ String Integer ]`. Written `Ethos.Rust`, the part after the period is what the generated Rust writes, and nothing checks that the resulting path exists. Intrinsic names known without import: String, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self. The generated code carries no `use` statements; each imported name is written fully qualified.

## Kinds

Kind is the word for the bearer of capabilities: something that can run is a runner, Runnable is its kind, run is its capability. Kinds are qualifier-named — Runnable, Textualizable, Embodied; Run is not a kind. A kind's identity is its name and its constraints, written as one head, and a constraint is a kind, never a type; angle brackets hold the constraints.

A simple kind opens with a bracket after the dot, holding its capabilities. The receiver after a capability's head names who is called: `.` takes self, `!` takes mutable self, `:` takes no self. A capability with inputs is a headed brace: inputs in a bracket, yield in a bracket holding one type. A complex kind opens with a brace holding four brackets: superkinds, associated types with their constraints, associated constants — upper case, each the name, a dot, its type — and capabilities.

```
Library
[ serde:Serializable.Serialize  std:Clonable.Clone  std:Sendable.Send ]  ; imports
[ SinkError.[ Closed Full ] ]                                 ; types
[ Fillable.[ push!{ [ String ] [ Result<Integer SinkError> ] } ; kinds
             create:[ Self ] ]
  Streamable.{ [ Fillable ]
               [ Item<Serializable> ]
               [ CAPACITY.Integer ]
               [ next![ Option<Item> ] ] }
  Processable<[Clonable Sendable] Serializable>.[] ]
[ SinkError.[ Fillable ] ]                                    ; associations
```
```rust
pub enum SinkError { Closed, Full }
pub trait Fillable {
    fn push(&mut self, input: String) -> std::result::Result<i64, SinkError>;
    fn create() -> Self where Self: Sized;
}
pub trait Streamable: Fillable {
    type Item: serde::Serialize;
    const CAPACITY: i64;
    fn next(&mut self) -> std::option::Option<Self::Item>;
}
pub trait Processable<A: std::clone::Clone + std::marker::Send, B: serde::Serialize> {}
// An association is a compile-time assertion; the interaction body is hand-written.
const _: () = {
    fn assert_sinkerror_fillable<T: Fillable>() {}
    let _ = assert_sinkerror_fillable::<SinkError>;
};
```

## Spacing and generation

Ethos follows the canonical protos print: a space inside every bracket and brace at both ends when non-empty. Generated Rust is committed and held fresh by a test. Every generated file opens with `#![allow(dead_code, non_camel_case_types, non_snake_case)]` and every item carries `#[rustfmt::skip]`.

```sh
ethos-zero 'Generate.{ /abs/orchestrate.ethos /abs/out }'
# -> Generated.[ /abs/out/orchestrate.rs ]
```

A datom object's basic CLI help emits the ethos that describes its anatomy: point at the object and its ethos prints.
