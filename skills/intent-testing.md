---
description: A proof of concept is about to be tested and where it runs must be chosen.
dependencies: [testing, vision-horizon]
---

## A proof of concept is tested in a sandbox first

A proof of concept is tested in a sandbox first. The sandbox is a
virtual machine running on a node that has that feature; which node is
found by querying Horizon. A proof of concept runs outside a sandbox
only when it cannot run in one, such as a browser login with the
living's credentials.

## Sources

f38926 operational-openCodeRemoteAccess
f38926 horizon
