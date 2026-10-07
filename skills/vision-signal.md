---
description: Signal — the binary messaging layer, its queries and responses, meta signal or protocol — is being designed or judged against what the living wants.
dependencies: [vision-ethos, vision-nexus]
---

## Name

The serialized form is called signal.

## What signal is

Signal is the messaging layer: fully binary, portable rkyv fixed
across endianness, fully typed with both sides knowing the full
schema, nothing on the wire labeling itself. It is one step from the
composition, beside the text chain.

## Query and response

A Signal declares queries and responses; input and output are too
low-level for it.

```
Signal
[]                                     ; imports
[ Lock.LockRequest  Release.LockId ]   ; queries
[ Locked.Lock  Released.Lock ]         ; responses
[ LockId.Integer                       ; types
  LockName.String
  LockRequest.{ LockName }
  Lock.{ LockId LockName } ]
```

## Text and signal

The textual form is datom; a CLI actualizes it and sends signal; a
Nexus never textualizes.

## Meta signal

The meta signal is never optional: the daemon is configured only over
its meta surface.

## Protocol

Signal is portable rkyv plus whatever protocol is standardized on top
of it. The protocol is to be decided.

## Sources

564f55 signal
564f55 protos
55d18f4f signalIsOurMessagingLayer
6863ef19 signalIsOurMessagingLayer
ba906ae2 signalIsOurMessagingLayer
98fbfa47 metaSignalNotOptional
fe34eb signal
