---
description: Reading or writing any protos dialect, or touching the protos crate.
dependencies: []
---

Protos is the universal textual structure every dialect shares. It owns the only character reader and the only character writer. A dialect receives already-delineated structure and supplies its own types.

## Delimiters

Six delimiter pairs, four structural and two opaque:

| glyph | name | encloses |
|---|---|---|
| `{ }` | Braces | struct in datom; struct in ethos |
| `[ ]` | Brackets | vector in datom; enum, bracket of kinds, capability list in ethos |
| `« »` | Guillemets (U+00AB/U+00BB) | map: key value key value by position |
| `< >` | Angles | kind constraints: `Vector<Text>`, `Processable<[Clonable Sendable] Serializable>` |
| `“ ”` | Curly quotes (U+201C/U+201D) | opaque string: every glyph inside is content until the closing quote; no escapes |
| `( )` | Parentheses | opaque, read by balance with backslash escapes (`\)` `\(` `\\`) |

## Separators

`.` Period, `!` Exclamation, `:` Colon. Inside a bare run a separator splits head from body when a non-whitespace, non-closing character follows: `Some.42` is Headed(Some, Period, Bare 42); `Reviewer.{` is Headed with an enclosed body. A separator followed by whitespace, a closer, or end of text is a MissingBody fault; a run beginning with a separator is a MissingHead fault.

## Heads

A head is a symbol. A headed structure is a head, a separator, and a body: the dot is the default separator, written right after the head, and it opens the body's delimiter. `a:b:c` chains right-associatively.

## Bare words

A bare word is a maximal run of characters containing no whitespace and no delimiter glyph.

## Comments

A single `;` opens a comment to end of line. Comments are never printed.

## Canonical spacing

`{ a b }`, `[ a b ]`, `« k v k v »` — one space inside at both ends when non-empty; `{}` `[]` `«»` when empty. Angles tight: `<a b>`. `Head.body` with nothing around the separator. Siblings one space apart. Opaque regions verbatim with their glyphs. One line.

## Layers

| layer | type | descent (may fault) | ascent (cannot fault) |
|---|---|---|---|
| Text | `protos::Text`, `protos::Potential<T>` | `Structural::delineate` on Text -> `Delineation` | -- |
| Protoform | `protos::Protoform`, `protos::Delineation` | `Conceptual<C>::conceive` on Protoform -> C | `Printing::print` -> Text |
| Concept | the dialect's data model | dialect-specific (datom: `Datomic::incorporate`) | `Protosizable::protosize` -> Protoform |
| Corporal | the Rust value | -- | dialect-specific (datom: `Datomic::datomize`) |

Descent is realization and may fault. Ascent is textualization and cannot fault. `Actualizable<T>::actualize` on `Potential<T>` chains the whole descent. `Textualizable::textualize` chains the whole ascent.

## Kinds

| kind | layer | what it does |
|---|---|---|
| Structural | Text | `delineate` -> `Delineation` |
| Protosizable | Concept | `protosize` -> `Protoform` |
| Conceptual\<C\> | Protoform | `conceive` -> C |
| Actualizable\<T\> | Potential | `actualize` -> T (blanket: delineate, conceive, incorporate) |
| Printing | Protoform | `print` -> Text |
| Corporal\<C\> | Concept -> Corporal | `incorporate` (static) takes a concept C and yields Self; borne by every corporal type |
| Embodied | -- | the bound: alias of Sized, blanket-implemented |

## What protos does not know

Protos has no struct, no vector, no map, no integer, no string, no interpretation. What a structure means is said by the dialect, never by protos alone.
