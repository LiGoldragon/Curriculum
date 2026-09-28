# protos: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/protos.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/protos.md
+++ b/skills/protos.md
@@ -4,63 +4,83 @@ dependencies: []
 ---
 
-Protos is the universal textual structure every dialect shares. It owns the only character reader and the only character writer. A dialect receives already-delineated structure and supplies its own types.
+Protos is the style every dialect shares: the context-switching parse, the delimiters, the heads, the recursive structure. It owns the only character reader and the only character writer. It knows form and nothing else: what a structure means — struct, vector, string, integer, variant — is said by the conceptual layer that reads it, never by protos.
+
+## Four layers
+
+Textual, protosic, conceptual, compositional. Going down, the information gains density and strictness; going up, visibility. Each step converts into a wholly different type, and nothing of the previous step is used after it. Implement the descent as multiple passes; a single pass is not an option.
+
+```
+; textual: these characters, uninterpreted
+{ Ada 1990 }
+```
+```rust
+// protosic: an enclosure of two bare runs, each structure at its extent in the text
+Protos::Enclosed { extent: Extent { start: 0, end: 12 }, enclosure: Braced, children: vec![
+    Protos::Bare { extent: Extent { start: 2, end: 5 }, text: "Ada".to_owned() },
+    Protos::Bare { extent: Extent { start: 6, end: 10 }, text: "1990".to_owned() } ] }
+// conceptual, here datomic: a struct of two positions, each datom at its path
+Datom { path: vec![], form: Form::Struct(vec![
+    Datom { path: vec![0], form: Form::Bare("Ada".to_owned()) },
+    Datom { path: vec![1], form: Form::Bare("1990".to_owned()) } ]) }
+// compositional: the meaning fully absorbed; no position, because the tree is consumed
+Person { name: String::from("Ada"), born: 1990 }
+```
 
 ## Delimiters
 
-Six delimiter pairs, four structural and two opaque:
+Five pairs. `{ }`, `[ ]`, `< >` structural; `« »` opaque, every glyph content; `( )` reserved for meaning, its type unspecified, read by balance as opaque until it is. The curly quotes are not delimiters. There is no key-value map in protos or in any dialect. A brace enclosure's arity is anatomical; a bracket enclosure's is not.
 
-| glyph | name | encloses |
-|---|---|---|
-| `{ }` | Braces | struct in datom; struct in ethos |
-| `[ ]` | Brackets | vector in datom; enum, bracket of kinds, capability list in ethos |
-| `« »` | Guillemets (U+00AB/U+00BB) | map: key value key value by position |
-| `< >` | Angles | kind constraints: `Vector<Text>`, `Processable<[Clonable Sendable] Serializable>` |
-| `“ ”` | Curly quotes (U+201C/U+201D) | opaque string: every glyph inside is content until the closing quote; no escapes |
-| `( )` | Parentheses | opaque, read by balance with backslash escapes (`\)` `\(` `\\`) |
-
-## Separators
-
-`.` Period, `!` Exclamation, `:` Colon. Inside a bare run a separator splits head from body when a non-whitespace, non-closing character follows: `Some.42` is Headed(Some, Period, Bare 42); `Reviewer.{` is Headed with an enclosed body. A separator followed by whitespace, a closer, or end of text is a MissingBody fault; a run beginning with a separator is a MissingHead fault.
-
-## Heads
-
-A head is a symbol. A headed structure is a head, a separator, and a body: the dot is the default separator, written right after the head, and it opens the body's delimiter. `a:b:c` chains right-associatively.
-
-## Bare words
-
-A bare word is a maximal run of characters containing no whitespace and no delimiter glyph.
+## Structure
 
-## Comments
+Structure is the word for every unit of the text; its type is the `Protos` enum: headed, enclosed, opaque, or bare. A headed structure is a head, a separator and a body; the separators are period, exclamation and colon; the head is a symbol; heads daisy-chain, separators differing. An enclosed structure stands between its delimiters; a bare structure has none.
 
-A single `;` opens a comment to end of line. Comments are never printed.
+## Every layer carries its own context
 
-## Canonical spacing
-
-`{ a b }`, `[ a b ]`, `« k v k v »` — one space inside at both ends when non-empty; `{}` `[]` `«»` when empty. Angles tight: `<a b>`. `Head.body` with nothing around the separator. Siblings one space apart. Opaque regions verbatim with their glyphs. One line.
-
-## Layers
-
-| layer | type | descent (may fault) | ascent (cannot fault) |
-|---|---|---|---|
-| Text | `protos::Text`, `protos::Potential<T>` | `Structural::delineate` on Text -> `Delineation` | -- |
-| Protoform | `protos::Protoform`, `protos::Delineation` | `Conceptual<C>::conceive` on Protoform -> C | `Printing::print` -> Text |
-| Concept | the dialect's data model | dialect-specific (datom: `Datomic::incorporate`) | `Protosizable::protosize` -> Protoform |
-| Corporal | the Rust value | -- | dialect-specific (datom: `Datomic::datomize`) |
-
-Descent is realization and may fault. Ascent is textualization and cannot fault. `Actualizable<T>::actualize` on `Potential<T>` chains the whole descent. `Textualizable::textualize` chains the whole ascent.
+The extent, where a structure sits in the text, is a fact of the protosic layer, and every `Protos` node carries one. The path, where a datom sits in its tree, is a fact of the datomic layer. The budget, how much reading one act allows, is a fact of the reader and lives on it. The composition carries no position.
 
 ## Kinds
 
-| kind | layer | what it does |
-|---|---|---|
-| Structural | Text | `delineate` -> `Delineation` |
-| Protosizable | Concept | `protosize` -> `Protoform` |
-| Conceptual\<C\> | Protoform | `conceive` -> C |
-| Actualizable\<T\> | Potential | `actualize` -> T (blanket: delineate, conceive, incorporate) |
-| Printing | Protoform | `print` -> Text |
-| Corporal\<C\> | Concept -> Corporal | `incorporate` (static) takes a concept C and yields Self; borne by every corporal type |
-| Embodied | -- | the bound: alias of Sized, blanket-implemented |
-
-## What protos does not know
+A kind is borne by the type converted and named for the layer it becomes. No type bears a kind two layers away; a chain is written in the open where it is used, never folded into a kind on its first type.
 
-Protos has no struct, no vector, no map, no integer, no string, no interpretation. What a structure means is said by the dialect, never by protos alone.
+| type | kind | becomes |
+|---|---|---|
+| text | `Protosizable` | protos |
+| `Protos` | `Textualizable` | text |
+| `Protos` | `Datomizable`, or `Ethosizable` further on | its concept |
+| `Datom` | `Protosizable` | protos |
+| `Datom` | `Composable` | any compositional type |
+| composition | `Datomizable` | datom |
+| composition | `Composing` | read from a datom; a struct form states its positions through `Compositional` |
+
+```rust
+pub enum Protos {
+    Headed { extent: Extent, head: Symbol, constraints: Option<Box<Protos>>, separator: Separator, body: Box<Protos> },
+    Enclosed { extent: Extent, enclosure: Enclosure, children: Vec<Protos> },
+    Opaque { extent: Extent, boundary: Boundary, content: String },
+    Bare { extent: Extent, text: String },
+}
+pub enum Enclosure { Braced, Bracketed, Angled }
+pub enum Boundary  { Guillemets, Parentheses }
+pub enum Separator { Period, Exclamation, Colon }
+pub struct Extent { pub start: usize, pub end: usize }
+pub struct Error  { pub extent: Extent, pub problem: Problem }
+pub struct ReaderBudget { pub remaining: usize }
+
+pub trait Protosizable    { type Output; fn protosize(&self) -> Self::Output; }  // String -> Result<Protos, Error>; Datom -> Protos
+pub trait Textualizable   { fn textualize(&self) -> String; }
+pub trait Canonicalizable { fn canonicalize(&mut self); }
+
+let text = person.datomize(Path::new()).protosize().textualize();
+```
+
+## String, escape, error
+
+The text type is `String`. A closing guillemet inside a string is escaped with a backslash, so the ascent never refuses. The word is error, not fault, through the chain.
+
+```
+«she said \»no\» and left»
+```
+
+## Canonical print
+
+A space inside every delimiter at both ends when non-empty, and never inside the guillemets, where every glyph is content and a space would be load-bearing. `Head.body` with nothing around the separator. A single `;` opens a comment to end of line; comments are not printed.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d. Replacement body approved; two corrections the flow's own.
View: candidate for the approved body.

````diff
--- a/skills/protos.md
+++ b/skills/protos.md
@@ -5,3 +5,24 @@ dependencies: []
 
-Protos is the universal textual structure every dialect shares. It owns the only character reader and the only character writer. A dialect receives already-delineated structure and supplies its own types.
+Protos is the style every dialect shares: the context-switching parse, the delimiters, the heads, the recursive structure. It owns the only character reader and the only character writer. It knows form and nothing else: what a structure means — struct, vector, string, integer, variant — is said by the conceptual layer that reads it, never by protos.
+
+## Four layers
+
+Textual, protosic, conceptual, compositional. Going down, the information gains density and strictness; going up, visibility. Each step converts into a wholly different type, and nothing of the previous step is used after it. Implement the descent as multiple passes; a single pass is not an option.
+
+```
+; textual: these characters, uninterpreted
+{ Ada 1990 }
+```
+```rust
+// protosic: an enclosure of two bare runs, each structure at its extent in the text
+Protos::Enclosed { extent: Extent { start: 0, end: 12 }, enclosure: Braced, children: vec![
+    Protos::Bare { extent: Extent { start: 2, end: 5 }, text: "Ada".to_owned() },
+    Protos::Bare { extent: Extent { start: 6, end: 10 }, text: "1990".to_owned() } ] }
+// conceptual, here datomic: a struct of two positions, each datom at its path
+Datom { path: vec![], form: Form::Struct(vec![
+    Datom { path: vec![0], form: Form::Bare("Ada".to_owned()) },
+    Datom { path: vec![1], form: Form::Bare("1990".to_owned()) } ]) }
+// compositional: the meaning fully absorbed; no position, because the tree is consumed
+Person { name: String::from("Ada"), born: 1990 }
+```
 
@@ -9,43 +30,11 @@ Protos is the universal textual structure every dialect shares. It owns the only
 
-Six delimiter pairs, four structural and two opaque:
+Five pairs. `{ }`, `[ ]`, `< >` structural; `« »` opaque, every glyph content; `( )` reserved for meaning, its type unspecified, read by balance as opaque until it is. The curly quotes are not delimiters. There is no key-value map in protos or in any dialect. A brace enclosure's arity is anatomical; a bracket enclosure's is not.
 
-| glyph | name | encloses |
-|---|---|---|
-| `{ }` | Braces | struct in datom; struct in ethos |
-| `[ ]` | Brackets | vector in datom; enum, bracket of kinds, capability list in ethos |
-| `« »` | Guillemets (U+00AB/U+00BB) | map: key value key value by position |
-| `< >` | Angles | kind constraints: `Vector<Text>`, `Processable<[Clonable Sendable] Serializable>` |
-| `“ ”` | Curly quotes (U+201C/U+201D) | opaque string: every glyph inside is content until the closing quote; no escapes |
-| `( )` | Parentheses | opaque, read by balance with backslash escapes (`\)` `\(` `\\`) |
-
-## Separators
-
-`.` Period, `!` Exclamation, `:` Colon. Inside a bare run a separator splits head from body when a non-whitespace, non-closing character follows: `Some.42` is Headed(Some, Period, Bare 42); `Reviewer.{` is Headed with an enclosed body. A separator followed by whitespace, a closer, or end of text is a MissingBody fault; a run beginning with a separator is a MissingHead fault.
-
-## Heads
-
-A head is a symbol. A headed structure is a head, a separator, and a body: the dot is the default separator, written right after the head, and it opens the body's delimiter. `a:b:c` chains right-associatively.
-
-## Bare words
-
-A bare word is a maximal run of characters containing no whitespace and no delimiter glyph.
+## Structure
 
-## Comments
+Structure is the word for every unit of the text; its type is the `Protos` enum: headed, enclosed, opaque, or bare. A headed structure is a head, a separator and a body; the separators are period, exclamation and colon; the head is a symbol; heads daisy-chain, separators differing. An enclosed structure stands between its delimiters; a bare structure has none.
 
-A single `;` opens a comment to end of line. Comments are never printed.
+## Every layer carries its own context
 
-## Canonical spacing
-
-`{ a b }`, `[ a b ]`, `« k v k v »` — one space inside at both ends when non-empty; `{}` `[]` `«»` when empty. Angles tight: `<a b>`. `Head.body` with nothing around the separator. Siblings one space apart. Opaque regions verbatim with their glyphs. One line.
-
-## Layers
-
-| layer | type | descent (may fault) | ascent (cannot fault) |
-|---|---|---|---|
-| Text | `protos::Text`, `protos::Potential<T>` | `Structural::delineate` on Text -> `Delineation` | -- |
-| Protoform | `protos::Protoform`, `protos::Delineation` | `Conceptual<C>::conceive` on Protoform -> C | `Printing::print` -> Text |
-| Concept | the dialect's data model | dialect-specific (datom: `Datomic::incorporate`) | `Protosizable::protosize` -> Protoform |
-| Corporal | the Rust value | -- | dialect-specific (datom: `Datomic::datomize`) |
-
-Descent is realization and may fault. Ascent is textualization and cannot fault. `Actualizable<T>::actualize` on `Potential<T>` chains the whole descent. `Textualizable::textualize` chains the whole ascent.
+The extent, where a structure sits in the text, is a fact of the protosic layer, and every `Protos` node carries one. The path, where a datom sits in its tree, is a fact of the datomic layer. The budget, how much reading one act allows, is a fact of the reader and lives on it. The composition carries no position.
 
@@ -53,14 +42,45 @@ Descent is realization and may fault. Ascent is textualization and cannot fault.
 
-| kind | layer | what it does |
-|---|---|---|
-| Structural | Text | `delineate` -> `Delineation` |
-| Protosizable | Concept | `protosize` -> `Protoform` |
-| Conceptual\<C\> | Protoform | `conceive` -> C |
-| Actualizable\<T\> | Potential | `actualize` -> T (blanket: delineate, conceive, incorporate) |
-| Printing | Protoform | `print` -> Text |
-| Corporal\<C\> | Concept -> Corporal | `incorporate` (static) takes a concept C and yields Self; borne by every corporal type |
-| Embodied | -- | the bound: alias of Sized, blanket-implemented |
-
-## What protos does not know
+A kind is borne by the type converted and named for the layer it becomes. No type bears a kind two layers away; a chain is written in the open where it is used, never folded into a kind on its first type.
 
-Protos has no struct, no vector, no map, no integer, no string, no interpretation. What a structure means is said by the dialect, never by protos alone.
+| type | kind | becomes |
+|---|---|---|
+| text | `Protosizable` | protos |
+| `Protos` | `Textualizable` | text |
+| `Protos` | `Datomizable`, or `Ethosizable` further on | its concept |
+| `Datom` | `Protosizable` | protos |
+| `Datom` | `Composable` | any compositional type |
+| composition | `Datomizable` | datom |
+| composition | `Compositional` | states its own positions, so a datom can compose it |
+
+```rust
+pub enum Protos {
+    Headed { extent: Extent, head: Symbol, constraints: Option<Box<Protos>>, separator: Separator, body: Box<Protos> },
+    Enclosed { extent: Extent, enclosure: Enclosure, children: Vec<Protos> },
+    Opaque { extent: Extent, boundary: Boundary, content: String },
+    Bare { extent: Extent, text: String },
+}
+pub enum Enclosure { Braced, Bracketed, Angled }
+pub enum Boundary  { Guillemets, Parentheses }
+pub enum Separator { Period, Exclamation, Colon }
+pub struct Extent { pub start: usize, pub end: usize }
+pub struct Error  { pub extent: Extent, pub problem: Problem }
+pub struct ReaderBudget { pub remaining: usize }
+
+pub trait Protosizable    { type Output; fn protosize(&self) -> Self::Output; }  // String -> Result<Protos, Error>; Datom -> Protos
+pub trait Textualizable   { fn textualize(&self) -> String; }
+pub trait Canonicalizable { fn canonicalize(&mut self); }
+
+let text = person.datomize(Path::new()).protosize().textualize();
+```
+
+## String, escape, error
+
+The text type is `String`. A closing guillemet inside a string is escaped with a backslash, so the ascent never refuses. The word is error, not fault, through the chain.
+
+```
+«she said \»no\» and left»
+```
+
+## Canonical print
+
+A space inside every delimiter at both ends when non-empty, and never inside the guillemets, where every glyph is content and a space would be load-bearing. `Head.body` with nothing around the separator. A single `;` opens a comment to end of line; comments are not printed.
````

### d6b5b07, 2026-09-25 15:38 -0600: Name Composing as the composition kind in the protos skill

Made by: Claude Opus 5.5 (1M context) <noreply@anthropic.com>.

Flow 38de5b, as for datom.
View: candidate, same reason as datom 02a3770.

````diff
--- a/skills/protos.md
+++ b/skills/protos.md
@@ -52,3 +52,3 @@ A kind is borne by the type converted and named for the layer it becomes. No typ
 | composition | `Datomizable` | datom |
-| composition | `Compositional` | states its own positions, so a datom can compose it |
+| composition | `Composing` | read from a datom; a struct form states its positions through `Compositional` |
````

