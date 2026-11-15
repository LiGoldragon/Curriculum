---
description: Orchestrate itself — its deployment, its meta binary, or its locking — is being designed or judged.
dependencies: []
---

## Deployment

Orchestrate is deployed unconditionally, in the home, for every
user. A deployment of orchestrate includes meta-orchestrate.

## A shared path is locked before it is edited

A flow reserves its whole write set with a Lock before editing a shared path and releases it when editing ends; a flow's own directory is written freely, because only that flow writes there.
