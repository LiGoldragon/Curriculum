---
description: Horizon, the cluster state it answers, or where a sandbox virtual machine runs is being designed or judged.
dependencies: [vision-nexus]
---

## Horizon is a proper Nexus

Horizon becomes a proper Nexus, so that the current state of the
cluster is queried from it. Earlier records treat Horizon as emitted
configuration data; how that sense and this one relate is not yet
ruled.

## A sandbox virtual machine is a feature of a node

A virtual machine runs on a node, and sandboxing is a feature a node
has. Which node has it is known by querying Horizon. Prometheus is
mostly the workhorse. Allocating resources is not easy and is not yet
designed.

## Sources

f38926 horizon
b81560 operational-horizonNexusAndNodeResources
