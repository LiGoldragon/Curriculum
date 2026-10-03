---
description: Rust is generated from an ethos file with ethos-zero as it runs today, or what the generator accepts is read.
dependencies: [vision-ethos]
---

As of 14.2.0 ethos-zero reads three roots: Library, Signal, Sema. The sweet form, root head then sections as siblings, is converted to the canonical braced form before reading. Section order: Library imports, types, kinds, associations; Signal imports, queries, responses, types; Sema imports, record types; associations of query, response and record types are implied.

The canonical print is vertical: a structure with more than one element, one of which has a next layer, opens on its line and its elements hang aligned beneath the first; the closer ends the last line; a space inside every non-empty bracket and brace, `[]` and `{}` empty, angles tight (`Vector<Event>`). protos owns it as `Textualizable::textualize`; the one-line form is `Compactable::compact`, which the command-line replies use.

`Name.Type` is an alias, `Name.{ }` a struct, `Name.[ ]` an enum. A field is named after its type in snake case, `string_vector`, `lock_option`, `first_lock` and `second_lock` when repeated. An inline struct or enum payload generates a Rust type named `<Variant>_Data`; a vector payload is carried as the vector, with no `_Data`. Imports are `protos:String` or `protos:[ String Integer ]`; intrinsics need none: String, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self.

A simple kind is `Name.[ capabilities ]`; receivers `.` self, `!` mutable self, `:` none; a capability with inputs is `push!{ [ inputs ] [ yield ] }`; a complex kind is `Name.{ [ superkinds ] [ associated types ] [ CONSTANTS ] [ capabilities ] }`. A kind in an input or yield becomes a method parameter bounded by it, lettered from `N` (`fn resolve<N: Textualizable>(&self, input: N)`), or `Self::Item` where an associated type is already bounded by that kind (`Item<Textualizable>`). Self, the kind's head parameters and a named associated type stay as written. A concrete type in an input (`String`, `Vector<Self>`, a declared type) is refused with `KindWanted`; a yield may name one. An imported name is taken as a kind in an input and as a type in a yield.

protos and datom-codec generate their kinds files from their own ethos, with the kinds Spendable (protos), Branchable, Budgeted, Positional and Composable (datom-codec).

In a Library or Sema every generated struct and enum derives Datomizable and Composing unconditionally. In a Signal every generated struct and enum derives rkyv's Archive, Serialize and Deserialize, and its Datomizable and Composing sit behind a `datom` feature that the CLI enables and the Nexus does not. An alias carries no derive. A generated file opens with `#![allow(dead_code, non_camel_case_types, non_snake_case)]`, every item carries `#[rustfmt::skip]`, names are fully qualified with no `use`. Generated Rust is committed and held fresh by a test. An association generates a compile-time assertion; the interaction body is hand-written.

```
ethos-zero 'Generate.{ /abs/flow.ethos /abs/out }'
Generated.[ /abs/out/flow.rs ]

ethos-zero 'Check./abs/flow.ethos'
Checked./abs/flow.ethos
```

A refused file answers `Rejected.{ file { line column } reason }`, the reason naming the path within the file (`Conceptual.{ [ path ] KindWanted.Path }`), and a rejected Generate writes nothing. With no argument ethos-zero prints its own ethos in the canonical print.
