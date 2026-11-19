---
description: A command needs root on a host.
dependencies: []
---

A command that needs root runs through SSH as root: `ssh root@<host> '<command>'`, the local host being `localhost`. The user's SSH key is authorized for root on every host of the cluster, and the command runs with no password and no prompt. sudo asks for a password, and an agent has none.

```sh
ssh root@localhost 'systemctl restart example.service'   # an example unit, on this host
ssh root@localhost 'kill 12345'                          # an example root process
```
