---
description: Rust is generated from an ethos file with ethos-zero as it runs today, or what the generator accepts is read.
dependencies: [vision-ethos]
---

As of 16.0.0 ethos-zero reads four roots: Library, Signal, Operation, Memory. The sweet form, root head then sections as siblings, is converted to the canonical braced form before reading. Section order: Library imports, types, kinds, associations; Signal imports, queries, responses, types; Operation imports, operations, outcomes, types, proposed and pending the living's word; Memory imports, record types. A Signal generates `pub enum Query` and `pub enum Response`, an Operation `pub enum Operation` and `pub enum Outcome`, from the two sections after imports. A file headed `Sema`, Memory's head before 15.0.0, is refused as `Conceptual.{ [ 0 ] Renamed.Memory }`.

The canonical print is vertical: a structure with more than one element, one of which has a next layer, opens on its line and its elements hang aligned beneath the first; the closer ends the last line; a space inside every non-empty bracket and brace, `[]` and `{}` empty, angles tight (`Vector<Event>`). protos owns it as `Textualizable::textualize`; the one-line form is `Compactable::compact`, which the command-line replies use.

`Name.Type` is an alias, `Name.{ }` a struct, `Name.[ ]` an enum. A field is named after its type in snake case, `string_vector`, `lock_option`, `first_lock` and `second_lock` when repeated. An inline struct or enum payload generates a Rust type named `<Variant>_Data`; a vector payload is carried as the vector, with no `_Data`. A struct position may declare its type in place, `Brief.String`, `State.[ Running Ended ]`, `Capsule.{ Home.String Login.Vector<String> }`: the type takes that name in the file's namespace and the position holds it by name (`brief: Brief`). Imports are `protos:String` or `protos:[ String Integer ]`; intrinsics need none: String, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self.

A simple kind is `Name.[ capabilities ]`; receivers `.` self, `!` mutable self, `:` none; a capability with inputs is `push!{ [ inputs ] [ yield ] }`; a complex kind is `Name.{ [ superkinds ] [ associated types ] [ CONSTANTS ] [ capabilities ] }`. A kind in an input or yield becomes a method parameter bounded by it, lettered from `N` (`fn resolve<N: Textualizable>(&self, input: N)`), or `Self::Item` where an associated type is already bounded by that kind (`Item<Textualizable>`). Self, the kind's head parameters and a named associated type stay as written. A concrete type in an input (`String`, `Vector<Self>`, a declared type) is refused with `KindWanted`; a yield may name one. An imported name is taken as a kind in an input and as a type in a yield.

protos and datom-codec generate their kinds files from their own ethos, with the kinds Spendable (protos), Branchable, Budgeted, Positional and Composable (datom-codec).

In every root each generated struct and enum derives rkyv's Archive, Serialize and Deserialize, Clone, Debug, PartialEq, Eq and Hash, and its Datomizable and Composing sit behind a `datom` feature that the CLI enables and the Nexus does not. A crate holding generated Rust depends on rkyv 0.8 always and on datom-codec only under `datom`; where a position holds a `Decimal`, a `Meaning` or a datom-codec `Error`, it enables datom-codec 0.32.2's `rkyv` feature, which enables protos's, unconditionally. An alias carries no derive. A generated file opens with `#![allow(dead_code, non_camel_case_types, non_snake_case)]`, every item carries `#[rustfmt::skip]`, names are fully qualified with no `use`. Generated Rust is committed and held fresh by a test. An association generates a compile-time assertion; the interaction body is hand-written.

```
ethos-zero 'Generate.{ /abs/flow.ethos /abs/out }'
Generated.[ /abs/out/flow.rs ]

ethos-zero 'Check./abs/flow.ethos'
Checked./abs/flow.ethos
```

A refused file answers `Rejected.{ file { line column } reason }`, the reason naming the path within the file (`Conceptual.{ [ path ] KindWanted.Path }`), and a rejected Generate writes nothing. With no argument ethos-zero prints its own ethos in the canonical print.
