---
description: Constructing, reading or interpreting datom text, or giving a Rust type its datom kinds.
dependencies: [protos]
---

Datom is the pure-data dialect on the protos substrate: data, strictly typed, super dense, no field names. Its whole work is carrying data between text and typed form. Schema-driven and positional: the reader walks the expected type, writing is the exact reverse projection. All naming lives in the type; the text carries only the data. The library is datom-codec.

## A datom is a form at a path

```rust
pub struct Datom { pub path: Path, pub form: Form }
pub enum Form { Struct(Vec<Datom>), Vector(Vec<Datom>), Variant(Symbol, Box<Datom>), Bare(String), String(String), Meaning(Opaque) }
```

## Syntax

A brace structure is a struct, a bracket structure is a vector, and a head in front of a structure is a variant carrying it. In datom a head is always a variant, so it is capitalized. A symbol alone, in a position expecting an enum, is a variant carrying nothing; a variant's name is written as the head every time, one carrying nothing included. Guillemets are the string delimiter and parentheses are reserved for Meaning. A datom is not preceded by a Datom root. What a structure means — struct, vector, string, integer, variant — is said by the position it sits in, never by the structure alone.

A string has two forms: bare, a run with no space and no delimiter glyph, which may be a whole sentence written without spaces in any casing; and guillemets, where every glyph is content until the closing guillemet, which is escaped with a backslash where it is content. Because the position already knows it holds a string, a bare run may carry characters that are syntax elsewhere, the colon among them. An integer is bare ASCII decimal, no leading plus and no leading zero except `0` itself. A decimal is finite and point-mandatory. Today a parenthesized text lands as a plain String, with the Meaning type marked in code.

There is no map. What a map would hold is a struct when its keys are fixed, and a vector of structs when they are not.

```
; datom, in a position expecting Person: a struct of name String, born Integer, address Address, roles Vector<Role>.
{ Ada 1990 { «12 Rue de la Paix» Paris 75002 } [ Author Reviewer.{ 2024 17 } ] }

; Reply: an enum of Accepted.{ id Integer  at String }, Refused.{ reason String  code Integer }, Pending
Accepted.{ 42 2026-09-03T17:46:20 }          ; the timestamp has no space and no delimiter, so it is bare
Refused.{ «no such file: { } is content» 2 } ; delimited: the string has spaces and braces; inside the guillemets they are content
Pending                                      ; a variant carrying nothing

[ 0 42 -42 ]                                 ; a vector of Integer
Observed.Locks.[]                            ; the Observed variant, its Locks variant, the empty vector
[ Some.42 None ]   Ok.{ Ada 1990 }   Err.«no such lock»   ; Vector, Option and Result read as ordinary variants
```

## The datom composes; the type states its positions

The descent into a composition is the datom's act, written once. What only the type can supply, its positions in order, is stated by the type through the derive, so arity, budget and locus live in one place and no type repeats them.

```rust
pub trait Composable { fn compose<T: Compositional>(&self, budget: &mut Budget) -> Result<T, Error>; }
pub trait Compositional: Sized { fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error>; }
pub trait Datomizable { type Output; fn datomize(&self, at: Path) -> Self::Output; }
```

## Any Rust type

Any Rust type bears the two kinds through datom-codec's derive, with no attributes, because datom is structural all the way down: field order is position order, a field's type is the position's type, a bare variant carries nothing, a single-field variant carries its type's own form, a multi-field variant carries an inline struct. Hand-written impls are reserved to the intrinsics.

```rust
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub struct Locus { pub path: Path, pub extent: Extent }

impl Compositional for Locus {                              // generated
    fn compose(datom: &Datom, budget: &mut Budget) -> Result<Self, Error> {
        budget.spend(&datom.path)?;                         // the arity the type knows, asked of the datom
        let mut positions = datom.positions(2)?;
        Ok(Self { path: positions.position(budget)?, extent: positions.position(budget)? })
    }
}
impl Datomizable for Locus {                                // generated: each child placed as the tree is built
    type Output = Datom;
    fn datomize(&self, at: Path) -> Datom {
        Datom { path: at.clone(), form: Form::Struct(vec![self.path.datomize(at.child(0)), self.extent.datomize(at.child(1))]) }
    }
}
```

## From text and back

```rust
let query: Query = Potential::<Query>::from(text).actualize(&mut budget)?;
let out = response.datomize(Path::new()).protosize().textualize();
```

## Errors

An error names the layer that raised it and the path of the datom where it arose; the extent is the protos node at that path. An error is itself datomizable.

```
[ 1 x ]                                  ; read as Vector<Integer>
Error.{ Composition [ 1 ] Value.{ Integer x } }   ; at path 1, the bare string x is not an integer
```

## The interface shape

A program's configuration surface is the datom's shape itself: a data enum at the root whose variants are the main operations, a variant's data carrying what follows. Output is an enum, always. A datom-speaking CLI takes exactly one inline datom value and no flags; datom passes inline at a CLI boundary, never as a datom file.

```sh
orchestrate 'Lock.{ MyLock 6329f1 [ /abs/path ] «why I hold it» }'
# -> Locked.{ 442 MyLock 6329f1 [ /abs/path ] «why I hold it» }
```

A written datom gives every position; omittable fields are not yet.
