---
description: A command is heavy on CPU, memory or disk: a Nix evaluation, a cargo build or test, a search over a large tree, a flake update.
dependencies: []
---

Heavy work runs on the builder, the host the laptop's Nix already builds on: the host of the `ssh-ng://` line in `/etc/nix/machines`. The laptop keeps editing, git, the flows' clients and reading results.

A Nix build already goes to the builder by itself. A Nix evaluation is pure for a pushed revision: the same revision gives the same derivation on any host, so it runs on the builder by its revision.

```sh
ssh <builder> "nix eval --raw 'github:<owner>/<repo>?rev=<full-rev>#<attribute>'"   # nix build the same way
```

Work in progress is pushed first, to the cluster remote or a branch. Work that cannot be pushed is copied once and run there:

```sh
rsync -a --delete --exclude target --exclude .git ./ <builder>:/tmp/offload/<repo>/
ssh <builder> 'cd /tmp/offload/<repo> && cargo test'
```

Output returns on standard output; a file is copied back only when it is wanted. A host's whole system is evaluated by Lojix, not by hand.
