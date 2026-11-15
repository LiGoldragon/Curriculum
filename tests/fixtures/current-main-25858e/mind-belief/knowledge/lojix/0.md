---
description: A Lojix request must be constructed, submitted, observed, or interpreted.
dependencies: []
---

`lojix-nexus` owns Lojix's state and two sockets. `lojix` speaks the ordinary contract, `signal-lojix`, on `LOJIX_ORDINARY_SOCKET`, to query and watch; `lojix-meta` speaks the owner contract, `meta-signal-lojix`, on `LOJIX_OWNER_SOCKET`, to configure, deploy, pin, unpin, retire and test. Neither socket has a default path.

`lojix-nexus` and both clients are the lojix revision CriomOS's `flake.lock` pins; its source is read there, not at lojix main. CriomOS sets both variables for every user (`modules/nixos/lojix.nix`): `/run/lojix/ordinary.sock` and `/run/lojix/meta.sock`.

Each client takes exactly one argument, one datom, and returns one reply. Positions carry the data and a field is never named: a struct is `{ }`, a vector `[ ]`, a variant a head glued to its payload by a period, `None` bare and `Some.value` glued, and a string holding a space in guillemets. A request may span lines.

## Observing

```sh
lojix 'Query.ByNode.{ cluster node None }'
lojix 'Query.ByDeployment.{ 13 }'
lojix 'Query.ByEventLog.{ 0 0 }'
lojix 'Query.ByGeneration.{ 42 }'
lojix 'Query.ByTestRun.{ cluster node None }'
```

`ByNode` may name the artifact in its third position: `Some.CompleteHost`, `Some.BaseHost` or `Some.UserEnvironment`. `ByEventLog` takes the first and the last event-log position. `Queried` carries the generations and the deployment records, `DeploymentEventsQueried` the phase and cache-retention events, and `TestRunsQueried` the test runs, each ending in the database marker; `QueryRejected` names its reason.

`lojix` exits after one reply, so a deployment is observed by re-querying.

## Deploying

```sh
lojix-meta 'Deploy.Host.{ fixture-cluster target
  BaseHost
  /var/lib/lojix/horizon-definition.datom
  NoSecrets
  github:fixture-owner/fixture-flake?ref=main
  { ssh-ng://root@target root@target }
  Direct
  { packages.x86_64-linux.fixture }
  NixosSystemdBootV1
  TestActivation
  ResolveAndRecord
  Some.@/etc/nix/machines
  [] }'
```

A `Deploy.Host` request's positions are, in order: cluster, node, `CompleteHost` or `BaseHost`, the proposal file, `NoSecrets` or a secrets directory, the flake reference, the transport `{ nix-store-uri ssh-destination }`, `Horizon` or `Direct`, the output selector `{ flake-attribute }`, the activation backend `NixosSystemdBootV1` or `HomeManagerNixProfileV1`, the action, `RequireImmutable` or `ResolveAndRecord`, an optional builder, and a vector of extra substituters. A host's action is `Evaluate`, `Realize`, `SetBootProfile`, `ActivateNow`, `TestActivation` or `ScheduleBootOnce`. `Deploy.UserEnvironment` carries the user name in the third position, and its action is `Realize`, `SetProfile` or `ActivateNow`.

`DeployAccepted.{ 13 { 263 263 } }` gives the deployment identifier and the database marker. It is admission only. `DeployRefused` and `DeployRejected` refuse the request.

A deployment is done when its record, read with `Query.ByDeployment.{ 13 }`, ends in its terminal: `Some.Succeeded`, `Some.Rejected.reason`, or `Some.Failed.{ stage reason evidence }`, where the optional evidence carries the failed command, its exit code and its output.

`lojix-meta 'Pin.{ alpha node-1 42 keep }'` pins generation 42 under the label `keep`; `Unpin.{ cluster node label }` and `Retire.{ cluster node generation }` take the same form.

## Deployment contract

Every deployment transport is explicit. Lojix uses the supplied Nix store URI and SSH destination verbatim; it never derives a route from cluster, node, or user names.

The logical node selects what is built; the activation destination selects which machine is changed. Before a state-changing deployment, verify that they identify the same node. If they do not, stop.

A `CompleteHost` deployment uses an explicit root-privileged Nix store URI and SSH destination; a `UserEnvironment` deployment uses an explicit user-scoped one.

When a deployment directly names a target pair, use it without asking for a second transport confirmation. Otherwise derive the canonical internal hostname as `<node>.<cluster>.<internal suffix>` from Horizon cluster data and use it as the host in the Nix store URI and SSH destination. If the supplied or derived pair is invalid, report it rather than substituting another route.

A deployment proposal must be an existing absolute regular non-symlink `horizon-definition.datom` file.

A production deployment is `RequireImmutable`, at a pushed CriomOS revision; producers are pushed before the consumer that pins them.

A `Current` generation is Lojix's committed state. It does not establish the target's live Nix profile or runtime links.

A terminal activation failure can follow a partial target change. Inspect the target profile, runtime links, and activation journal, then report Lojix state and live state separately.
