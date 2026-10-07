---
description: The meaning language — its logographic signs, roots, verbs, links, storage or names — is being designed or judged against what the living wants.
dependencies: [vision-ethos, datom]
---

## What the meaning language is

The meaning language is a computer language that is purely
logographic, the way Hanzi characters are logographic: a sign stands
for a meaning rather than for a sound. Its signs are typed structs
and enums forming one ontology, defined in Ethos and backed by Rust,
and held as a datom graph — a body of datom values, data written in
the strictly typed positional notation datom, joined to one another
by links rather than laid out as a tree of separate files. The
ontology is meant to reach as far as meaning does, so the best
ontology available in the world is taken and put into a data shape of
enums and structs that carry qualities. The language may take a
poetic Latin or Greek name of its own.

## Roots

The roots of the ontology are the seven Vaiśeṣika categories, and
its verbs follow the Aṣṭādhyāyī of Pāṇini, which an earlier ruling
makes the base for how the system thinks, communicates, and
classifies things in the world (flow 5851f4). How the two divide
beyond that is not yet ruled.

## Verbs

The verb set is Sanskrit. What the verbs define are the situations a
statement can be in: the relations of time, person, number, gender,
and intention.

## Annotation

A layer added to a meaning is a layer of annotation on it: a comment
on the first layer of meaning, which can itself be commented on or
linked to something else. It can expand recursively without limit,
but in practice it settles at three or four layers. A statement has
subparts, and each subpart can carry an annotation of its own, on
that specific piece of the data.

## Identity and links

A statement's subparts are reached by content-addressed link: a link
whose address is a checksum taken over the piece's own content and
over its links, so that the address names the content instead of a
position. Such a link is verifiable on its own, and an index into the
containing data is created on demand so the piece is easy to find. If
a meaning changes, its identity changes, because a small alteration
can make it mean something quite different. An annotation on the old
meaning therefore still points at the old meaning, and whether the
annotation is still valid is judged anew.

## Retention

A piece that is linked to is kept by virtue of the link, and released
when the last link to it goes, the way Nix keeps a store path while
something still references it. What must be kept is at least the part
that was checksummed in; the rest may go when nothing links to it.
This is how history kept writings such as those of Heraclitus:
through the annotations and references other authors made to them.

## Storage

Once a statement is complete it is stored content-addressed at its
root, whatever kind of thing it is — a response, a statement. A
series of responses is a vector.

## English names

Every Sanskrit-rooted type carries an English name as well. A
translation need not be a single word: a PascalCase sentence
expression may name one of the guṇas, or any other division of a
statement.

## Until the verb set lands

Until the full verb set lands, statements stay short prose: a limited
number of words holding one idea or one statement. A reader who meets
a block whose spec it does not have treats that block as an opaque
string, and the basic structure is made stable first while the inside
keeps changing.

## Sources

f38926 meaningLanguage
b81560 operational-asyncSubflowsAndMeaningLanguage
b81560 operational-meaningLanguageLogographic
b81560 operational-meaningLanguageAnnotationLayers
b81560 operational-meaningContentAddressedAnnotation
b81560 operational-meaningGarbageCollectionAndOntology
b81560 operational-vaisheshikaRuledAndSyntaxQuestion
b81560 operational-meaningDualSanskritEnglishNames
5851f4 ashtadhyayiKnowledgeBase
