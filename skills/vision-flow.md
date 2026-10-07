---
description: Flow — the Nexus that launches, names, tracks and ends flows — or a voice, a side flow, a hook or the Capsule is being designed or judged against what the living wants.
dependencies: [vision-nexus, vision-ethos]
---

## What it does

The Flow Nexus sets up and starts a model flow: its working
directory, system prompt, training files and instruction prompt. It
takes the place of the abandoned training daemon.

Flow is the Nexus that manages flows. A flow is one run of a voice; the component is named for what it manages. A flow the living asks for is launched, properly and surely. Flow holds the lock on flows.

The Capsule is the component that makes where a flow runs. Its first embodiment is the semi-sandbox: only the credential files are copied, everything else is recreated, with its own sockets and store, running light models. Later it encrypts the sensitive parts of the filesystem under a volatile key.

## Starting flows

A Nexus component decides the system prompt and everything about a
launch, replacing the harness's subagents with specialized harnesses
launched with specialized system prompts.

A voice is an aspect carrying a rank, `Psyche.Primary`: Psyche, Mind, Field by Primary, Secondary, Tertiary, nine voices. The model behind a voice is configuration, declared once in Flow and changed only over its meta wire; not every model is exposed to every voice. Voices are addressed by name; a flow id is for the ledger and the archive, and is written in words. A side flow is a job, not a voice: focused, not long-lived, gone when its mission is done; a message sent to it after that returns to its sender with notice that the flow has ended.

Speech runs horizontally between aspects at equal rank and vertically within an aspect one rung at a time. Field speaks to Mind, for what must be changed in code and documentation and tested before Field deploys it; Mind speaks to Psyche for judgment on design and choice, and rarely. Sol speaks to Opus, not to Fable; Fable is spoken to least. Design is Astra's, not Sol's.

## Repository and skills

The flow repository holds the machinery of the Flow Nexus and is a
runtime repository. Every skill lives outside it, the basic skills
included, so that a change to a skill causes no Nix rebuild. The
basic skills give our own take on how an agent behaves in a harness,
replacing the prompt the harnesses build in.

## A session is named after its direct ancestor

A session cannot be named for what it will become, because nothing is known
about it when it is created. Its ancestor is known exactly, so the ancestor
is the name.

## A replaced session is reaped by the refresh itself

Reaping belongs to the refresh event, not to a later sweep. A refreshed flow
takes the replaced end out of receiving messages, so a dead end is never
left registered and addressable.

Every harness event reaches Flow through the harness's hooks calling the Flow CLI, so Flow knows each flow's state without polling; a marked block landing in a transcript becomes an action, a book among them, with no tool call by the flow; a flow nearing its context limit is told, writes its handover, and is refreshed as it goes idle. Polling is forbidden; a poller that must exist is registered and reported.

## Subflows replace the harness subagent facility

The harness subagent facility is replaced. It puts two flows into one
synchronous user interface and locks them both into a single main
flow. A subflow is instead an independent flow with its own system
prompt, which can reply to the successor of whoever it was meant to
answer; that makes the system asynchronous. Subflows run under their
own system prompts because they need different prompts.

## Subflows are created from the questions and requests a flow ends with

Creating a subflow is a routing job: whether there is already a flow
that should simply get this message or this question. A special field
flow running on ultra-low power checks every question and every
request a flow ends with, and, according to the ending flow's
authority, spawns subflows given those questions and requests to
answer or fulfill.

## The requester holds only a request ID

The requester holds nothing of the subflow itself. It is assigned a
request ID, by which it asks later for status, asks for more detail
about what that subflow is doing, and sends the subflow messages
while it is alive. When the subflow is done, the requester receives a
message if it is still the flow in charge.

## Sources

358f143a flowDaemon
e06e4c07 flowDaemon
acbb6006 nexus
1a6ca4 nexus
1ac573 operational-nameSessionAfterAncestor
1ac573 operational-reapReplacedSessions
f38926 subflows
b81560 operational-asyncSubflowsAndMeaningLanguage
b81560 operational-fieldUltraLowRoutesSubflowRequests
b81560 operational-subflowRequestIdAndAsync
