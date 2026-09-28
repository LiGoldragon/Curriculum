# ethos: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/ethos.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/ethos.md
+++ b/skills/ethos.md
@@ -4,152 +4,109 @@ dependencies: [protos, datom]
 ---
 
-Ethos is the schema language. It specifies the types; datom fills them with data. Ethos generates the Rust.
+Ethos is the schema language: it specifies the types, datom fills them with data, and ethos-zero generates the Rust. In ethos there are no generics, only kinds. Any repetition in ethos syntax is an implementation failure.
 
-## File roots
+## Roots and file shape
 
-Two roots: `Library` and `Signal`. A file is one sweet ethos object; the outer braces are omitted and always implied:
+Three roots: Library, Signal, Sema. Signal gives a Nexus its main types and Sema its database types. An ethos file carries no version; what is versioned is versioned in a manifest.
 
-```
-; A Library file (sweet form). The full form wraps everything in Library.{ ... }.
-Library.{ 0 1 0 }
-[]                                                             ; imports
-[ Scores.Vector<Integer>                                       ; types
-  Record.{ Text Scores } ]
-[ Summarizable.[ summarize.[ Text ] ] ]                        ; kinds
-[ Record.[ Summarizable ] ]                                    ; associations
-```
-```rust
-pub type Scores = Vec<protos::Integer>;
-pub struct Record(pub protos::Text, pub Scores);
-// Association: compile-time assertion; the impl body is hand-written.
-const _: () = {
-    fn assert_record_summarizable<T: Summarizable>() {}
-    let _ = assert_record_summarizable::<Record>;
-};
-impl datomic::Datomic for Record { /* generated from anatomy */ }
-```
+The unit is File: one file, one Rust module, no namespace inside it. A file is written in the sweet form — the root's head, then the sections as siblings, the outer braces omitted — and is converted mechanically to the canonical braced form before it is read as ethos. A Library's sections in order are imports, types, kinds, associations; a Signal's are imports, queries, responses, types; a Sema's are imports, record types. In the Signal and Sema roots the associations of the query, response and record types are implied and never written.
 
 ```
-; A Signal file (sweet form).
-Signal.{ 1 0 0 }
-[]                                                             ; imports
-[ Lock.LockRequest  Release.LockId  Observe.ObserveSelection ]  ; requests
-[ Locked.Lock  LockRejected.LockRejection  Released.Lock  ReleaseRejected.ReleaseRejection  Observed.Observation ]
-[ LockId.Integer  LockName.Text  LockRequest.{ LockName FlowId LockPaths LockReason } ... ]  ; types
+Library                                   ; the sweet form, as a file is written
+[]                                        ; imports
+[ Record.{ String Integer } ]             ; types
+[]                                        ; kinds
+[]                                        ; associations
+
+Library.{ [] [ Record.{ String Integer } ] [] [] }   ; the canonical form the reader sees
 ```
 ```rust
-pub type LockId = protos::Integer;
-pub enum Request { Lock(LockRequest), Release(LockId), Observe(ObserveSelection) }
-pub enum Reply { Locked(Lock), LockRejected(LockRejection), /* ... */ }
+#[derive(datom_codec::Datomizable, datom_codec::Composing, Clone, Debug, PartialEq, Eq, Hash)]
+pub struct Record { pub string: String, pub integer: i64 }
 ```
 
-## Type declarations
+## Declarations
 
-`Name.{ ... }` -- a struct; positions are unnamed; the type says what each position holds:
-```
-Lock.{ LockId LockName FlowId LockPaths LockReason }
-```
-```rust
-pub struct Lock(pub LockId, pub LockName, pub FlowId, pub LockPaths, pub LockReason);
-```
+`Name.Type` is an alias, `Name.{ … }` a struct, `Name.[ … ]` an enum. A field is named after its type in snake case; a constructed type type-first, `string_vector`, `lock_option`; a repeated type as first and second. A variant either names a type already defined, which is then the data it carries, or declares its payload in place — a vector, a struct or an enum, each a full type whose derived name carries `_Data`, recursively.
 
-`Name.[ ... ]` -- an enum; each variant bare or carrying an inline payload:
 ```
-SinkError.[ Closed Full ]
-LockRejection.[ DuplicateName.Lock  PathOverlap.LockOverlap ]
+Signal
+[]                                                   ; imports
+[ Lock.LockRequest  Release.LockId ]                 ; queries
+[ Locked.Lock  LockRejected.LockRejection ]          ; responses
+[ LockId.Integer                                     ; types
+  LockName.String
+  LockPath.String
+  LockRequest.{ LockName Vector<LockPath> }
+  Lock.{ LockId LockName }
+  LockRejection.[ DuplicateName.Lock                 ;   a variant naming a defined type carries that type
+                  PathOverlap.{ Lock Lock } ] ]      ;   a variant declaring its payload inline
 ```
 ```rust
-pub enum SinkError { Closed, Full }
-pub enum LockRejection { DuplicateName(Lock), PathOverlap(LockOverlap) }
+pub type LockId = i64;
+pub type LockName = String;
+pub type LockPath = String;
+#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
+#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
+pub struct LockRequest { pub lock_name: LockName, pub lock_path_vector: std::vec::Vec<LockPath> }
+// … Lock and PathOverlap_Data likewise
+pub struct PathOverlap_Data { pub first_lock: Lock, pub second_lock: Lock }
+pub enum LockRejection { DuplicateName(Lock), PathOverlap(PathOverlap_Data) }
+pub enum Query    { Lock(LockRequest), Release(LockId) }
+pub enum Response { Locked(Lock), LockRejected(LockRejection) }
 ```
 
-`Name.Type` -- an alias; `Name.« K V »` is a map alias:
-```
-LockId.Integer
-Roles.« Text Integer »
-```
-```rust
-pub type LockId = protos::Integer;
-pub type Roles = BTreeMap<protos::Text, protos::Integer>;
-```
+Ethos Zero emits the datom kinds on every struct and enum it generates; an alias bears them through the type it names and carries no derive. A Signal's types gate them behind a `datom` feature that the CLI and client enable and the Nexus does not, so the Nexus compiles its contract without datom-codec. No tuple in the code we design; where a standard trait or a dependency requires one it is allowed at that contact point only.
+
+## Imports and intrinsics
+
+An import names a source and a type: `protos:String`, or `protos:[ String Integer ]`. Written `Ethos.Rust`, the part after the period is what the generated Rust writes, and nothing checks that the resulting path exists. Intrinsic names known without import: String, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self. The generated code carries no `use` statements; each imported name is written fully qualified.
 
 ## Kinds
 
-A kind is the bearer of capabilities. The `.` receiver takes self, `!` takes mutable self, `:` takes no self.
+Kind is the word for the bearer of capabilities: something that can run is a runner, Runnable is its kind, run is its capability. Kinds are qualifier-named — Runnable, Textualizable, Embodied; Run is not a kind. A kind's identity is its name and its constraints, written as one head, and a constraint is a kind, never a type; angle brackets hold the constraints.
 
-Simple kind -- capabilities in a bracket:
-```
-Summarizable.[ summarize.[ Text ] ]
-```
-```rust
-pub trait Summarizable { fn summarize(&self) -> protos::Text; }
-```
+A simple kind opens with a bracket after the dot, holding its capabilities. The receiver after a capability's head names who is called: `.` takes self, `!` takes mutable self, `:` takes no self. A capability with inputs is a headed brace: inputs in a bracket, yield in a bracket holding one type. A complex kind opens with a brace holding four brackets: superkinds, associated types with their constraints, associated constants — upper case, each the name, a dot, its type — and capabilities.
 
-Complex kind -- a struct of superkinds, associated types with their constraints, associated constants in `« UPPER_CASE Type »`, and capabilities:
 ```
-Streamable.{ [ Fillable ]
-             [ Item<Serializable> ]
-             « CAPACITY Integer »
-             [ next![ Option<Item> ] ] }
+Library
+[ serde:Serializable.Serialize  std:Clonable.Clone  std:Sendable.Send ]  ; imports
+[ SinkError.[ Closed Full ] ]                                 ; types
+[ Fillable.[ push!{ [ String ] [ Result<Integer SinkError> ] } ; kinds
+             create:[ Self ] ]
+  Streamable.{ [ Fillable ]
+               [ Item<Serializable> ]
+               [ CAPACITY.Integer ]
+               [ next![ Option<Item> ] ] }
+  Processable<[Clonable Sendable] Serializable>.[] ]
+[ SinkError.[ Fillable ] ]                                    ; associations
 ```
 ```rust
+pub enum SinkError { Closed, Full }
+pub trait Fillable {
+    fn push(&mut self, input: String) -> std::result::Result<i64, SinkError>;
+    fn create() -> Self where Self: Sized;
+}
 pub trait Streamable: Fillable {
-    type Item: Serializable;
-    const CAPACITY: protos::Integer;
-    fn next(&mut self) -> Option<Self::Item>;
+    type Item: serde::Serialize;
+    const CAPACITY: i64;
+    fn next(&mut self) -> std::option::Option<Self::Item>;
 }
-```
-
-A capability's inputs and yield are each a bracket holding one type:
-```
-push!{ [ Text ] [ Result<Integer SinkError> ] }
-create:[ Self ]
-```
-```rust
-fn push(&mut self, input: protos::Text) -> Result<protos::Integer, SinkError>;
-fn create() -> Self;
-```
-
-Kind identity is the name and the constraints, written as one head:
-```
-Processable<[Clonable Sendable] Serializable>.[ ... ]
-```
-```rust
-pub trait Processable<A: Clone + Send, B: Serialize> { /* ... */ }
-```
-
-## Associations
-
-A type bears a kind. The generated code asserts the type bears the kind at compile time; the impl body is hand-written:
-```
-[ Sink.[ Summarizable Fillable ] ]
-```
-```rust
+pub trait Processable<A: std::clone::Clone + std::marker::Send, B: serde::Serialize> {}
+// An association is a compile-time assertion; the interaction body is hand-written.
 const _: () = {
-    fn assert_sink_summarizable<T: Summarizable>() {}
-    let _ = assert_sink_summarizable::<Sink>;
-    fn assert_sink_fillable<T: Fillable>() {}
-    let _ = assert_sink_fillable::<Sink>;
+    fn assert_sinkerror_fillable<T: Fillable>() {}
+    let _ = assert_sinkerror_fillable::<SinkError>;
 };
 ```
 
-Every ethos-declared type gets `impl datomic::Datomic` generated from its anatomy.
+## Spacing and generation
 
-## Imports and intrinsics
-
-`[ protos:[ Text Textualizable ] ]` imports names from another library. Intrinsic names known without import: Text, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self.
-
-## Generation
-
-`ethos-zero` generates the Rust. Generated Rust is committed; a freshness test asserts the committed output matches a fresh generation. The CLI speaks datom:
+Ethos follows the canonical protos print: a space inside every bracket and brace at both ends when non-empty. Generated Rust is committed and held fresh by a test. Every generated file opens with `#![allow(dead_code, non_camel_case_types, non_snake_case)]` and every item carries `#[rustfmt::skip]`.
 
 ```sh
-ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
-# -> Generated.[ /abs/out-dir/signal.rs ]
+ethos-zero 'Generate.{ /abs/orchestrate.ethos /abs/out }'
+# -> Generated.[ /abs/out/orchestrate.rs ]
 ```
 
-With no argument, `ethos-zero` prints its own ethos.
-
-## Non-repetition
-
-Any repetition in ethos syntax is an implementation failure.
+A datom object's basic CLI help emits the ethos that describes its anatomy: point at the object and its ethos prints.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d. The replacement body was approved; one added sentence on `Ethos.Rust` was the flow's own.
View: candidate for the approved body; drop the `Ethos.Rust` sentence unless the living confirms it.

````diff
--- a/skills/ethos.md
+++ b/skills/ethos.md
@@ -5,71 +5,60 @@ dependencies: [protos, datom]
 
-Ethos is the schema language. It specifies the types; datom fills them with data. Ethos generates the Rust.
+Ethos is the schema language: it specifies the types, datom fills them with data, and ethos-zero generates the Rust. In ethos there are no generics, only kinds. Any repetition in ethos syntax is an implementation failure.
 
-## File roots
+## Roots and file shape
 
-Two roots: `Library` and `Signal`. A file is one sweet ethos object; the outer braces are omitted and always implied:
+Three roots: Library, Signal, Sema. Signal gives a Nexus its main types and Sema its database types. An ethos file carries no version; what is versioned is versioned in a manifest.
 
-```
-; A Library file (sweet form). The full form wraps everything in Library.{ ... }.
-Library.{ 0 1 0 }
-[]                                                             ; imports
-[ Scores.Vector<Integer>                                       ; types
-  Record.{ Text Scores } ]
-[ Summarizable.[ summarize.[ Text ] ] ]                        ; kinds
-[ Record.[ Summarizable ] ]                                    ; associations
-```
-```rust
-pub type Scores = Vec<protos::Integer>;
-pub struct Record(pub protos::Text, pub Scores);
-// Association: compile-time assertion; the impl body is hand-written.
-const _: () = {
-    fn assert_record_summarizable<T: Summarizable>() {}
-    let _ = assert_record_summarizable::<Record>;
-};
-impl datomic::Datomic for Record { /* generated from anatomy */ }
-```
+The unit is File: one file, one Rust module, no namespace inside it. A file is written in the sweet form — the root's head, then the sections as siblings, the outer braces omitted — and is converted mechanically to the canonical braced form before it is read as ethos. A Library's sections in order are imports, types, kinds, associations; a Signal's are imports, queries, responses, types; a Sema's are imports, record types. In the Signal and Sema roots the associations of the query, response and record types are implied and never written.
 
 ```
-; A Signal file (sweet form).
-Signal.{ 1 0 0 }
-[]                                                             ; imports
-[ Lock.LockRequest  Release.LockId  Observe.ObserveSelection ]  ; requests
-[ Locked.Lock  LockRejected.LockRejection  Released.Lock  ReleaseRejected.ReleaseRejection  Observed.Observation ]
-[ LockId.Integer  LockName.Text  LockRequest.{ LockName FlowId LockPaths LockReason } ... ]  ; types
+Library                                   ; the sweet form, as a file is written
+[]                                        ; imports
+[ Record.{ String Integer } ]             ; types
+[]                                        ; kinds
+[]                                        ; associations
+
+Library.{ [] [ Record.{ String Integer } ] [] [] }   ; the canonical form the reader sees
 ```
 ```rust
-pub type LockId = protos::Integer;
-pub enum Request { Lock(LockRequest), Release(LockId), Observe(ObserveSelection) }
-pub enum Reply { Locked(Lock), LockRejected(LockRejection), /* ... */ }
+#[derive(datom_codec::Datomizable, datom_codec::Compositional, Clone, Debug, PartialEq)]
+pub struct Record { pub string: String, pub integer: i64 }
 ```
 
-## Type declarations
+## Declarations
 
-`Name.{ ... }` -- a struct; positions are unnamed; the type says what each position holds:
-```
-Lock.{ LockId LockName FlowId LockPaths LockReason }
-```
-```rust
-pub struct Lock(pub LockId, pub LockName, pub FlowId, pub LockPaths, pub LockReason);
-```
+`Name.Type` is an alias, `Name.{ … }` a struct, `Name.[ … ]` an enum. A field is named after its type in snake case; a constructed type type-first, `string_vector`, `lock_option`; a repeated type as first and second. A variant either names a type already defined, which is then the data it carries, or declares its payload in place — a vector, a struct or an enum, each a full type whose derived name carries `_Data`, recursively.
 
-`Name.[ ... ]` -- an enum; each variant bare or carrying an inline payload:
 ```
-SinkError.[ Closed Full ]
-LockRejection.[ DuplicateName.Lock  PathOverlap.LockOverlap ]
+Signal
+[]                                                   ; imports
+[ Lock.LockRequest  Release.LockId ]                 ; queries
+[ Locked.Lock  LockRejected.LockRejection ]          ; responses
+[ LockId.Integer                                     ; types
+  LockName.String
+  LockPath.String
+  LockRequest.{ LockName Vector<LockPath> }
+  Lock.{ LockId LockName }
+  LockRejection.[ DuplicateName.Lock                 ;   a variant naming a defined type carries that type
+                  PathOverlap.{ Lock Lock } ] ]      ;   a variant declaring its payload inline
 ```
 ```rust
-pub enum SinkError { Closed, Full }
-pub enum LockRejection { DuplicateName(Lock), PathOverlap(LockOverlap) }
+pub type LockId = i64;
+pub type LockName = String;
+pub type LockPath = String;
+#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
+#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Compositional))]
+pub struct LockRequest { pub lock_name: LockName, pub lock_path_vector: std::vec::Vec<LockPath> }
+// … Lock and PathOverlap_Data likewise
+pub struct PathOverlap_Data { pub first_lock: Lock, pub second_lock: Lock }
+pub enum LockRejection { DuplicateName(Lock), PathOverlap(PathOverlap_Data) }
+pub enum Query    { Lock(LockRequest), Release(LockId) }
+pub enum Response { Locked(Lock), LockRejected(LockRejection) }
 ```
 
-`Name.Type` -- an alias; `Name.« K V »` is a map alias:
-```
-LockId.Integer
-Roles.« Text Integer »
-```
-```rust
-pub type LockId = protos::Integer;
-pub type Roles = BTreeMap<protos::Text, protos::Integer>;
-```
+Ethos Zero emits the datom kinds on every struct and enum it generates; an alias bears them through the type it names and carries no derive. A Signal's types gate them behind a `datom` feature that the CLI and client enable and the Nexus does not, so the Nexus compiles its contract without datom-codec. No tuple in the code we design; where a standard trait or a dependency requires one it is allowed at that contact point only.
+
+## Imports and intrinsics
+
+An import names a source and a type: `protos:String`, or `protos:[ String Integer ]`. Written `Ethos.Rust`, the part after the period is what the generated Rust writes, and nothing checks that the resulting path exists. Intrinsic names known without import: String, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self. The generated code carries no `use` statements; each imported name is written fully qualified.
 
@@ -77,57 +66,35 @@ pub type Roles = BTreeMap<protos::Text, protos::Integer>;
 
-A kind is the bearer of capabilities. The `.` receiver takes self, `!` takes mutable self, `:` takes no self.
+Kind is the word for the bearer of capabilities: something that can run is a runner, Runnable is its kind, run is its capability. Kinds are qualifier-named — Runnable, Textualizable, Embodied; Run is not a kind. A kind's identity is its name and its constraints, written as one head, and a constraint is a kind, never a type; angle brackets hold the constraints.
 
-Simple kind -- capabilities in a bracket:
-```
-Summarizable.[ summarize.[ Text ] ]
-```
-```rust
-pub trait Summarizable { fn summarize(&self) -> protos::Text; }
-```
+A simple kind opens with a bracket after the dot, holding its capabilities. The receiver after a capability's head names who is called: `.` takes self, `!` takes mutable self, `:` takes no self. A capability with inputs is a headed brace: inputs in a bracket, yield in a bracket holding one type. A complex kind opens with a brace holding four brackets: superkinds, associated types with their constraints, associated constants — upper case, each the name, a dot, its type — and capabilities.
 
-Complex kind -- a struct of superkinds, associated types with their constraints, associated constants in `« UPPER_CASE Type »`, and capabilities:
 ```
-Streamable.{ [ Fillable ]
-             [ Item<Serializable> ]
-             « CAPACITY Integer »
-             [ next![ Option<Item> ] ] }
+Library
+[ serde:Serializable.Serialize  std:Clonable.Clone  std:Sendable.Send ]  ; imports
+[ SinkError.[ Closed Full ] ]                                 ; types
+[ Fillable.[ push!{ [ String ] [ Result<Integer SinkError> ] } ; kinds
+             create:[ Self ] ]
+  Streamable.{ [ Fillable ]
+               [ Item<Serializable> ]
+               [ CAPACITY.Integer ]
+               [ next![ Option<Item> ] ] }
+  Processable<[Clonable Sendable] Serializable>.[] ]
+[ SinkError.[ Fillable ] ]                                    ; associations
 ```
 ```rust
+pub enum SinkError { Closed, Full }
+pub trait Fillable {
+    fn push(&mut self, input: String) -> std::result::Result<i64, SinkError>;
+    fn create() -> Self where Self: Sized;
+}
 pub trait Streamable: Fillable {
-    type Item: Serializable;
-    const CAPACITY: protos::Integer;
-    fn next(&mut self) -> Option<Self::Item>;
+    type Item: serde::Serialize;
+    const CAPACITY: i64;
+    fn next(&mut self) -> std::option::Option<Self::Item>;
 }
-```
-
-A capability's inputs and yield are each a bracket holding one type:
-```
-push!{ [ Text ] [ Result<Integer SinkError> ] }
-create:[ Self ]
-```
-```rust
-fn push(&mut self, input: protos::Text) -> Result<protos::Integer, SinkError>;
-fn create() -> Self;
-```
-
-Kind identity is the name and the constraints, written as one head:
-```
-Processable<[Clonable Sendable] Serializable>.[ ... ]
-```
-```rust
-pub trait Processable<A: Clone + Send, B: Serialize> { /* ... */ }
-```
-
-## Associations
-
-A type bears a kind. The generated code asserts the type bears the kind at compile time; the impl body is hand-written:
-```
-[ Sink.[ Summarizable Fillable ] ]
-```
-```rust
+pub trait Processable<A: std::clone::Clone + std::marker::Send, B: serde::Serialize> {}
+// An association is a compile-time assertion; the interaction body is hand-written.
 const _: () = {
-    fn assert_sink_summarizable<T: Summarizable>() {}
-    let _ = assert_sink_summarizable::<Sink>;
-    fn assert_sink_fillable<T: Fillable>() {}
-    let _ = assert_sink_fillable::<Sink>;
+    fn assert_sinkerror_fillable<T: Fillable>() {}
+    let _ = assert_sinkerror_fillable::<SinkError>;
 };
@@ -135,21 +102,11 @@ const _: () = {
 
-Every ethos-declared type gets `impl datomic::Datomic` generated from its anatomy.
+## Spacing and generation
 
-## Imports and intrinsics
-
-`[ protos:[ Text Textualizable ] ]` imports names from another library. Intrinsic names known without import: Text, Integer, Decimal, Boolean, Meaning, Vector, Option, Result, Self.
-
-## Generation
-
-`ethos-zero` generates the Rust. Generated Rust is committed; a freshness test asserts the committed output matches a fresh generation. The CLI speaks datom:
+Ethos follows the canonical protos print: a space inside every bracket and brace at both ends when non-empty. Generated Rust is committed and held fresh by a test. Every generated file opens with `#![allow(dead_code, non_camel_case_types, non_snake_case)]` and every item carries `#[rustfmt::skip]`.
 
 ```sh
-ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
-# -> Generated.[ /abs/out-dir/signal.rs ]
+ethos-zero 'Generate.{ /abs/orchestrate.ethos /abs/out }'
+# -> Generated.[ /abs/out/orchestrate.rs ]
 ```
 
-With no argument, `ethos-zero` prints its own ethos.
-
-## Non-repetition
-
-Any repetition in ethos syntax is an implementation failure.
+A datom object's basic CLI help emits the ethos that describes its anatomy: point at the object and its ethos prints.
````

### 02a3770, 2026-09-25 15:36 -0600: Name the Composing derive in the ethos and datom skills

Made by: Claude Opus 5.5 (1M context) <noreply@anthropic.com>.

Flow 38de5b, as for datom.
View: candidate, same reason as datom 02a3770.

````diff
--- a/skills/ethos.md
+++ b/skills/ethos.md
@@ -23,3 +23,3 @@ Library.{ [] [ Record.{ String Integer } ] [] [] }   ; the canonical form the re
 ```rust
-#[derive(datom_codec::Datomizable, datom_codec::Compositional, Clone, Debug, PartialEq)]
+#[derive(datom_codec::Datomizable, datom_codec::Composing, Clone, Debug, PartialEq, Eq, Hash)]
 pub struct Record { pub string: String, pub integer: i64 }
@@ -48,4 +48,4 @@ pub type LockName = String;
 pub type LockPath = String;
-#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
-#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Compositional))]
+#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
+#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
 pub struct LockRequest { pub lock_name: LockName, pub lock_path_vector: std::vec::Vec<LockPath> }
````

