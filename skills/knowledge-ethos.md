---
description: Rust is generated from an ethos file with ethos-zero as it runs today, or what the generator accepts is read.
dependencies: [vision-ethos]
---

ethos-zero reads three roots: Library, Signal, Sema. The sweet form, root head then sections as siblings, is converted to the canonical braced form before reading. Section order: Library imports, types, kinds, associations; Signal imports, queries, responses, types; Sema imports, record types; associations of query, response and record types are implied. The reader accepts the vertical layout with hanging closers; the printer alone writes one line, a space inside every non-empty bracket and brace.

`Name.Type` is an alias, `Name.{ }` a struct, `Name.[ ]` an enum. A field is named after its type in snake case, `string_vector`, `lock_option`, `first_lock` and `second_lock` when repeated. An inline struct or enum payload generates a Rust type named `<Variant>_Data`; a vector payload is carried as the vector, with no `_Data`. Imports are `protos:String` or `protos:[ String Integer ]`; intrinsics need none: String, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self.

A simple kind is `Name.[ capabilities ]`; receivers `.` self, `!` mutable self, `:` none; a capability with inputs is `push!{ [ inputs ] [ yield ] }`; a complex kind is `Name.{ [ superkinds ] [ associated types ] [ CONSTANTS ] [ capabilities ] }`. The generator accepts a concrete type as a capability input and refuses a kind there.

In a Library or Sema every generated struct and enum derives Datomizable and Composing unconditionally. In a Signal every generated struct and enum derives rkyv's Archive, Serialize and Deserialize, and its Datomizable and Composing sit behind a `datom` feature that the CLI enables and the Nexus does not. An alias carries no derive. A generated file opens with `#![allow(dead_code, non_camel_case_types, non_snake_case)]`, every item carries `#[rustfmt::skip]`, names are fully qualified with no `use`. Generated Rust is committed and held fresh by a test. An association generates a compile-time assertion; the interaction body is hand-written.

```
ethos-zero 'Generate.{ /abs/flow.ethos /abs/out }'
Generated.[ /abs/out/flow.rs ]

ethos-zero 'Check./abs/flow.ethos'
Checked./abs/flow.ethos
```

A refused file answers `Rejected.{ file { line column } reason }`, and a rejected Generate writes nothing. With no argument ethos-zero prints its own ethos.
