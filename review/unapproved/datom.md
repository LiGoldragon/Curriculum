# datom: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/datom.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/datom.md
+++ b/skills/datom.md
@@ -1,103 +1,106 @@
 ---
-description: Constructing, reading, or interpreting datom, or implementing Datomic.
+description: Constructing, reading or interpreting datom text, or giving a Rust type its datom kinds.
 dependencies: [protos]
 ---
 
-Datom is the pure-data dialect on the protos substrate. It carries data, strictly typed, and its whole work is serialization and deserialization. Schema-driven and positional: the reader walks the expected type, writing is the exact reverse projection. All naming lives in the type; the text carries only the data.
+Datom is the pure-data dialect on the protos substrate: data, strictly typed, super dense, no field names. Its whole work is carrying data between text and typed form. Schema-driven and positional: the reader walks the expected type, writing is the exact reverse projection. All naming lives in the type; the text carries only the data. The library is datom-codec.
 
-## Text forms
+## A datom is a form at a path
 
-Each form occupies the position its type declares. What a structure means is said by the position, never by the structure alone.
-
-Struct — braces, positional:
-```
-; Person: a struct of name Text, born Integer, address Address, roles Vector<Role>.
-{ Ada 1990 { “12 Rue de la Paix” Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
-```
-
-Vector — brackets:
-```
-[ 0 42 -42 ]
+```rust
+pub struct Datom { pub path: Path, pub form: Form }
+pub enum Form { Struct(Vec<Datom>), Vector(Vec<Datom>), Variant(Symbol, Box<Datom>), Bare(String), String(String), Meaning(Opaque) }
 ```
 
-Map — guillemets, key and value by position:
-```
-« home { “12 Rue de la Paix” Paris 75002 }  work { “1 Place Vendôme” Paris 75001 } »
-```
+## Syntax
 
-Variant — a head alone for a variant carrying nothing; a head, the dot, and a body for a variant carrying data:
-```
-Pending
-Accepted.{ 42 2026-09-03T17:46:20 }
-Observed.Locks.[]
-```
+A brace structure is a struct, a bracket structure is a vector, and a head in front of a structure is a variant carrying it. In datom a head is always a variant, so it is capitalized. A symbol alone, in a position expecting an enum, is a variant carrying nothing; a variant's name is written as the head every time, one carrying nothing included. Guillemets are the string delimiter and parentheses are reserved for Meaning. A datom is not preceded by a Datom root. What a structure means — struct, vector, string, integer, variant — is said by the position it sits in, never by the structure alone.
 
-String — bare when it contains no space and no delimiter; curly-quoted otherwise. In a string position a bare word may contain characters that are syntax elsewhere:
-```
-Ada
-“no such file: { } is content”
-name:first
-```
+A string has two forms: bare, a run with no space and no delimiter glyph, which may be a whole sentence written without spaces in any casing; and guillemets, where every glyph is content until the closing guillemet, which is escaped with a backslash where it is content. Because the position already knows it holds a string, a bare run may carry characters that are syntax elsewhere, the colon among them. An integer is bare ASCII decimal, no leading plus and no leading zero except `0` itself. A decimal is finite and point-mandatory. Today a parenthesized text lands as a plain String, with the Meaning type marked in code.
 
-Integer — bare ASCII decimal, optional leading `-`, no `+`, no leading zero except `0` itself:
-```
-0  42  -42
-```
+There is no map. What a map would hold is a struct when its keys are fixed, and a vector of structs when they are not.
 
-Decimal — finite, point-mandatory:
-```
-3.14  -0.5
 ```
+; datom, in a position expecting Person: a struct of name String, born Integer, address Address, roles Vector<Role>.
+{ Ada 1990 { «12 Rue de la Paix» Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
 
-Boolean:
-```
-True  False
-```
+; Reply: an enum of Accepted.{ id Integer  at String }, Refused.{ reason String  code Integer }, Pending
+Accepted.{ 42 2026-09-03T17:46:20 }          ; the timestamp has no space and no delimiter, so it is bare
+Refused.{ «no such file: { } is content» 2 } ; delimited: the string has spaces and braces; inside the guillemets they are content
+Pending                                      ; a variant carrying nothing
 
-Meaning — parenthesized text, read by balance:
-```
-(The build passed on the third try (after two timeouts))
+[ 0 42 -42 ]                                 ; a vector of Integer
+Observed.Locks.[]                            ; the Observed variant, its Locks variant, the empty vector
+[ Some.42 None ]   Ok.{ Ada 1990 }   Err.«no such lock»   ; Vector, Option and Result read as ordinary variants
 ```
 
-## The CLI
+## The datom composes; the type states its positions
 
-A datom-speaking CLI takes exactly one inline datom value and no flags. Its type system is the only interface.
+The descent into a composition is the datom's act, written once. What only the type can supply, its positions in order, is stated by the type through the derive, so arity, budget and locus live in one place and no type repeats them.
 
-```sh
-orchestrate 'Lock.{ MyLock 6329f1 [ /abs/path ] “why I hold it” }'
-# -> Locked.{ 442 MyLock 6329f1 [ /abs/path ] “why I hold it” }
+```rust
+pub trait Composable {
+    fn compose<T: Composing>(&self, budget: &mut Budget) -> Result<T, Error>;
+    fn compose_positions<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error>;
+}
+pub trait Composing: Sized { fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>; }
+pub trait Compositional: Composing { const ARITY: Integer; fn from_positions(positions: Positions<'_>) -> Result<Self, Error>; }
+pub trait Datomizable { fn datomize(&self, at: Path) -> Datom; }
 ```
 
-With no argument, a CLI prints its contract's ethos.
+## Any Rust type
 
-## Datomic in Rust
-
-A Rust type bears `Datomic` through two capabilities — `incorporate` (static, constructs the value from a `Datom`) and `datomize` (projects the value into a `Datom`):
+Any Rust type bears the two kinds through datom-codec's derive, with no attributes, because datom is structural all the way down: field order is position order, a field's type is the position's type, a bare variant carries nothing, a single-field variant carries its type's own form, a multi-field variant carries an inline struct. Hand-written impls are reserved to the intrinsics.
 
 ```rust
-// Corporal: the kind whose static capability takes a concept and yields Self.
-pub trait Corporal<C: Protosizable>: Embodied {
-    type Fault;
-    fn incorporate(concept: C) -> Result<Self, Self::Fault>;
+#[derive(datom_codec::Datomizable, datom_codec::Composing)]
+pub struct Locus { pub path: Path, pub extent: Extent }
+
+impl Compositional for Locus {                              // generated: the positions, in order
+    const ARITY: Integer = 2;
+    fn from_positions(mut positions: Positions<'_>) -> Result<Self, Error> {
+        Ok(Self { path: positions.position()?, extent: positions.position()? })
+    }
 }
-
-// Datomic: the corporal kind of the datom dialect.
-pub trait Datomic: Corporal<Datom, Fault = Fault> {
-    fn datomize(&self) -> Datom;
+impl Composing for Locus {                                  // generated: the datom reads, spending the budget
+    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
+        datom.compose_positions(budget)
+    }
 }
-
-// Provided for every Datomic: datomize -> protosize -> print.
-pub trait Textualizable {
-    fn textualize(&self) -> protos::Text;
+impl Datomizable for Locus {                                // generated: each child placed as the tree is built
+    fn datomize(&self, at: Path) -> Datom {
+        Datom { path: at.clone(), form: Form::Struct(vec![self.path.datomize(at.child(0)), self.extent.datomize(at.child(1))]) }
+    }
 }
 ```
 
-`Potential<T>::actualize()` chains the whole descent — delineate, conceive, incorporate — and may fault. `Textualizable::textualize()` chains the whole ascent — datomize, protosize, print — and cannot fault:
+## From text and back
 
 ```rust
-let potential = Potential::<Lock>::from(text);
-let lock: Lock = potential.actualize()?;
-let text: protos::Text = lock.textualize();
+let query: Query = Potential::<Query>::from(text).actualize(&mut budget)?;
+let out = response.datomize(Path::new()).protosize().textualize();
+```
+
+## Errors
+
+An error names the layer that raised it and the path of the datom where it arose; the extent is the protos node at that path. An error is itself datomizable.
+
+```
+[ 1 x ]                                  ; read as Vector<Integer>
+Error.{ Composition [ 1 ] Value.{ Integer x } }   ; at path 1, the bare string x is not an integer
 ```
 
-Every ethos-declared type gets its `Datomic` generated. No hand-written Datomic implementations for declared types.
+## The interface shape
+
+A program's configuration surface is the datom's shape itself: a data enum at the root whose variants are the main operations, a variant's data carrying what follows. Output is an enum, always. A datom-speaking CLI takes exactly one inline datom value and no flags; datom passes inline at a CLI boundary, never as a datom file.
+
+```sh
+orchestrate 'Lock.{ MyLock 6329f1 [ /abs/path ] «why I hold it» }'
+# -> Locked.{ 442 MyLock 6329f1 [ /abs/path ] «why I hold it» }
+```
+
+A written datom gives every position; omittable fields are not yet.
+
+## A datom needs a type
+
+A datom is written only against a type that already exists. When none exists, the type is declared first, in Ethos through the ethos skill; there is no ad hoc datom and no field label standing in for a type.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d, applying proposals the living approved in bulk ("go with your suggestions"); the replacement body was approved, several corrections against released code (ARITY, `Path::new()`, the error example) were the flow's own.
View: candidate for the approved body; the flow's own corrections are uncertain and one of them (dropping ARITY) cuts against the living's arity ruling.

````diff
--- a/skills/datom.md
+++ b/skills/datom.md
@@ -1,3 +1,3 @@
 ---
-description: Constructing, reading, or interpreting datom, or implementing Datomic.
+description: Constructing, reading or interpreting datom text, or giving a Rust type its datom kinds.
 dependencies: [protos]
@@ -5,88 +5,63 @@ dependencies: [protos]
 
-Datom is the pure-data dialect on the protos substrate. It carries data, strictly typed, and its whole work is serialization and deserialization. Schema-driven and positional: the reader walks the expected type, writing is the exact reverse projection. All naming lives in the type; the text carries only the data.
+Datom is the pure-data dialect on the protos substrate: data, strictly typed, super dense, no field names. Its whole work is carrying data between text and typed form. Schema-driven and positional: the reader walks the expected type, writing is the exact reverse projection. All naming lives in the type; the text carries only the data. The library is datom-codec.
 
-## Text forms
+## A datom is a form at a path
 
-Each form occupies the position its type declares. What a structure means is said by the position, never by the structure alone.
-
-Struct — braces, positional:
-```
-; Person: a struct of name Text, born Integer, address Address, roles Vector<Role>.
-{ Ada 1990 { “12 Rue de la Paix” Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
+```rust
+pub struct Datom { pub path: Path, pub form: Form }
+pub enum Form { Struct(Vec<Datom>), Vector(Vec<Datom>), Variant(Symbol, Box<Datom>), Bare(String), String(String), Meaning(Opaque) }
 ```
 
-Vector — brackets:
-```
-[ 0 42 -42 ]
-```
+## Syntax
 
-Map — guillemets, key and value by position:
-```
-« home { “12 Rue de la Paix” Paris 75002 }  work { “1 Place Vendôme” Paris 75001 } »
-```
+A brace structure is a struct, a bracket structure is a vector, and a head in front of a structure is a variant carrying it. In datom a head is always a variant, so it is capitalized. A symbol alone, in a position expecting an enum, is a variant carrying nothing; a variant's name is written as the head every time, one carrying nothing included. Guillemets are the string delimiter and parentheses are reserved for Meaning. A datom is not preceded by a Datom root. What a structure means — struct, vector, string, integer, variant — is said by the position it sits in, never by the structure alone.
 
-Variant — a head alone for a variant carrying nothing; a head, the dot, and a body for a variant carrying data:
-```
-Pending
-Accepted.{ 42 2026-09-03T17:46:20 }
-Observed.Locks.[]
-```
+A string has two forms: bare, a run with no space and no delimiter glyph, which may be a whole sentence written without spaces in any casing; and guillemets, where every glyph is content until the closing guillemet, which is escaped with a backslash where it is content. Because the position already knows it holds a string, a bare run may carry characters that are syntax elsewhere, the colon among them. An integer is bare ASCII decimal, no leading plus and no leading zero except `0` itself. A decimal is finite and point-mandatory. Today a parenthesized text lands as a plain String, with the Meaning type marked in code.
 
-String — bare when it contains no space and no delimiter; curly-quoted otherwise. In a string position a bare word may contain characters that are syntax elsewhere:
-```
-Ada
-“no such file: { } is content”
-name:first
-```
-
-Integer — bare ASCII decimal, optional leading `-`, no `+`, no leading zero except `0` itself:
-```
-0  42  -42
-```
+There is no map. What a map would hold is a struct when its keys are fixed, and a vector of structs when they are not.
 
-Decimal — finite, point-mandatory:
-```
-3.14  -0.5
 ```
+; datom, in a position expecting Person: a struct of name String, born Integer, address Address, roles Vector<Role>.
+{ Ada 1990 { «12 Rue de la Paix» Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }
 
-Boolean:
-```
-True  False
-```
+; Reply: an enum of Accepted.{ id Integer  at String }, Refused.{ reason String  code Integer }, Pending
+Accepted.{ 42 2026-09-03T17:46:20 }          ; the timestamp has no space and no delimiter, so it is bare
+Refused.{ «no such file: { } is content» 2 } ; delimited: the string has spaces and braces; inside the guillemets they are content
+Pending                                      ; a variant carrying nothing
 
-Meaning — parenthesized text, read by balance:
-```
-(The build passed on the third try (after two timeouts))
+[ 0 42 -42 ]                                 ; a vector of Integer
+Observed.Locks.[]                            ; the Observed variant, its Locks variant, the empty vector
+[ Some.42 None ]   Ok.{ Ada 1990 }   Err.«no such lock»   ; Vector, Option and Result read as ordinary variants
 ```
 
-## The CLI
+## The datom composes; the type states its positions
 
-A datom-speaking CLI takes exactly one inline datom value and no flags. Its type system is the only interface.
+The descent into a composition is the datom's act, written once. What only the type can supply, its positions in order, is stated by the type through the derive, so arity, budget and locus live in one place and no type repeats them.
 
-```sh
-orchestrate 'Lock.{ MyLock 6329f1 [ /abs/path ] “why I hold it” }'
-# -> Locked.{ 442 MyLock 6329f1 [ /abs/path ] “why I hold it” }
+```rust
+pub trait Composable { fn compose<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error>; }
+pub trait Compositional: Sized { fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>; }
+pub trait Datomizable { type Output; fn datomize(&self, at: Path) -> Self::Output; }
 ```
 
-With no argument, a CLI prints its contract's ethos.
+## Any Rust type
 
-## Datomic in Rust
-
-A Rust type bears `Datomic` through two capabilities — `incorporate` (static, constructs the value from a `Datom`) and `datomize` (projects the value into a `Datom`):
+Any Rust type bears the two kinds through datom-codec's derive, with no attributes, because datom is structural all the way down: field order is position order, a field's type is the position's type, a bare variant carries nothing, a single-field variant carries its type's own form, a multi-field variant carries an inline struct. Hand-written impls are reserved to the intrinsics.
 
 ```rust
-// Corporal: the kind whose static capability takes a concept and yields Self.
-pub trait Corporal<C: Protosizable>: Embodied {
-    type Fault;
-    fn incorporate(concept: C) -> Result<Self, Self::Fault>;
+#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
+pub struct Locus { pub path: Path, pub extent: Extent }
+
+impl Compositional for Locus {                              // generated
+    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
+        budget.spend(&datom.path)?;                         // the arity the type knows, asked of the datom
+        let mut positions = datom.positions(2)?;
+        Ok(Self { path: positions.position(budget)?, extent: positions.position(budget)? })
+    }
 }
-
-// Datomic: the corporal kind of the datom dialect.
-pub trait Datomic: Corporal<Datom, Fault = Fault> {
-    fn datomize(&self) -> Datom;
-}
-
-// Provided for every Datomic: datomize -> protosize -> print.
-pub trait Textualizable {
-    fn textualize(&self) -> protos::Text;
+impl Datomizable for Locus {                                // generated: each child placed as the tree is built
+    type Output = Datom;
+    fn datomize(&self, at: Path) -> Datom {
+        Datom { path: at.clone(), form: Form::Struct(vec![self.path.datomize(at.child(0)), self.extent.datomize(at.child(1))]) }
+    }
 }
@@ -94,10 +69,27 @@ pub trait Textualizable {
 
-`Potential<T>::actualize()` chains the whole descent — delineate, conceive, incorporate — and may fault. `Textualizable::textualize()` chains the whole ascent — datomize, protosize, print — and cannot fault:
+## From text and back
 
 ```rust
-let potential = Potential::<Lock>::from(text);
-let lock: Lock = potential.actualize()?;
-let text: protos::Text = lock.textualize();
+let query: Query = Potential::<Query>::from(text).actualize(&mut budget)?;
+let out = response.datomize(Path::new()).protosize().textualize();
+```
+
+## Errors
+
+An error names the layer that raised it and the path of the datom where it arose; the extent is the protos node at that path. An error is itself datomizable.
+
+```
+[ 1 x ]                                  ; read as Vector<Integer>
+Error.{ Composition [ 1 ] Value.{ Integer x } }   ; at path 1, the bare string x is not an integer
+```
+
+## The interface shape
+
+A program's configuration surface is the datom's shape itself: a data enum at the root whose variants are the main operations, a variant's data carrying what follows. Output is an enum, always. A datom-speaking CLI takes exactly one inline datom value and no flags; datom passes inline at a CLI boundary, never as a datom file.
+
+```sh
+orchestrate 'Lock.{ MyLock 6329f1 [ /abs/path ] «why I hold it» }'
+# -> Locked.{ 442 MyLock 6329f1 [ /abs/path ] «why I hold it» }
 ```
 
-Every ethos-declared type gets its `Datomic` generated. No hand-written Datomic implementations for declared types.
+A written datom gives every position; omittable fields are not yet.
````

### 1cce102, 2026-09-21 09:14 -0600: Add datom type rule and correction cause rule

Made by: no model trailer (a Codex seat by the audit reading).

Flow 1b8ac0, with the living (same approval as correction 1cce102).
View: candidate. "A datom is written only against a type that already exists; declare the type first in Ethos" is doctrine of the datom language, approved by the living.

````diff
--- a/skills/datom.md
+++ b/skills/datom.md
@@ -95 +95,5 @@ orchestrate 'Lock.{ MyLock 6329f1 [ /abs/path ] «why I hold it» }'
 A written datom gives every position; omittable fields are not yet.
+
+## A datom needs a type
+
+A datom is written only against a type that already exists. When none exists, the type is declared first, in Ethos through the ethos skill; there is no ad hoc datom and no field label standing in for a type.
````

### 02a3770, 2026-09-25 15:36 -0600: Name the Composing derive in the ethos and datom skills

Made by: Claude Opus 5.5 (1M context) <noreply@anthropic.com>.

Flow 38de5b, under e51411's ethos audit (fix 10). No words of the living found; a fidelity correction to released code.
View: candidate. The skill's trait examples otherwise name `Compositional` where the released codec derives `Composing`; a skill that teaches code which no longer compiles misleads every flow.

````diff
--- a/skills/datom.md
+++ b/skills/datom.md
@@ -41,5 +41,9 @@ The descent into a composition is the datom's act, written once. What only the t
 ```rust
-pub trait Composable { fn compose<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error>; }
-pub trait Compositional: Sized { fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>; }
-pub trait Datomizable { type Output; fn datomize(&self, at: Path) -> Self::Output; }
+pub trait Composable {
+    fn compose<T: Composing>(&self, budget: &mut Budget) -> Result<T, Error>;
+    fn compose_positions<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error>;
+}
+pub trait Composing: Sized { fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>; }
+pub trait Compositional: Composing { const ARITY: Integer; fn from_positions(positions: Positions<'_>) -> Result<Self, Error>; }
+pub trait Datomizable { fn datomize(&self, at: Path) -> Datom; }
 ```
@@ -51,10 +55,14 @@ Any Rust type bears the two kinds through datom-codec's derive, with no attribut
 ```rust
-#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
+#[derive(datom_codec::Datomizable, datom_codec::Composing)]
 pub struct Locus { pub path: Path, pub extent: Extent }
 
-impl Compositional for Locus {                              // generated
+impl Compositional for Locus {                              // generated: the positions, in order
+    const ARITY: Integer = 2;
+    fn from_positions(mut positions: Positions<'_>) -> Result<Self, Error> {
+        Ok(Self { path: positions.position()?, extent: positions.position()? })
+    }
+}
+impl Composing for Locus {                                  // generated: the datom reads, spending the budget
     fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
-        budget.spend(&datom.path)?;                         // the arity the type knows, asked of the datom
-        let mut positions = datom.positions(2)?;
-        Ok(Self { path: positions.position(budget)?, extent: positions.position(budget)? })
+        datom.compose_positions(budget)
     }
@@ -62,3 +70,2 @@ impl Compositional for Locus {                              // generated
 impl Datomizable for Locus {                                // generated: each child placed as the tree is built
-    type Output = Datom;
     fn datomize(&self, at: Path) -> Datom {
````

