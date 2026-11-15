---
description: "Nix builds are made, kept or collected."
dependencies: []
---

## Nothing is built twice

Nothing is built twice. A compiled thing is a derivation named by its inputs, and the same inputs give the same store path on every host; a second compilation of the same inputs is a fault in the build's design, and is removed at its cause.

What changes is what rebuilds: a crate's source changes, and the crate and its dependents rebuild; a test changes, and the test build alone rebuilds; a dependency changes, and it changes once, for every repository at once. Everything else is reused from the store.

One place holds the facts every build shares, the toolchain, the package set and the version and feature set of every third-party crate; a repository holds only what is its own.

```mermaid
flowchart TB
    i["inputs: source, toolchain, dependency set"] -- "name" --> d["one derivation"]
    d -- "has, on every host" --> p["one store path"]
    c["a change to one input"] -- "rebuilds" --> r["that derivation and its dependents, nothing else"]
```

## Nix builds

Nix builds live in the remote builder's store. A machine keeps links only to the profiles in use on it. A Nix build worth keeping is listed in one registry profile; every other Nix build is collected by how recently and how often it was used.

A flake input is the bare repository URL; the lock is the pin. A rev or a ref is written in the lock alone.

```nix
inputs.flow.url = "github:LiGoldragon/flow";   # the repository, bare; flake.lock holds the commit
```
