---
description: A Lojix request must be constructed, submitted, observed, or interpreted.
dependencies: [nix-workflow]
---

`lojix-daemon` owns durable state and two authority-tiered sockets. The ordinary contract is `signal-lojix`; the owner contract is `meta-signal-lojix`.

Use `lojix` on the ordinary socket for `Query`, `WatchDeployments`, `WatchCacheRetention`, `Unwatch`, and `CheckHostKeyMaterial`.

Use `meta-lojix` on the owner socket for `Deploy`, `Pin`, `Unpin`, `Retire`, and `Test`. The owner contract is not optional.

Use `LOJIX_ORDINARY_SOCKET` and `LOJIX_OWNER_SOCKET`; neither socket has a default path.

## Dotos syntax

Each public client accepts exactly one inline Dotos object. It rejects files, signal files, flags, subcommands, zero arguments, and extra arguments.

Inline decoding requires the client build's `dotos-text` feature. Missing Dotos support is a client-build defect, not permission to pass a file or flag.

A request root is one object.

A variant is `Head.Payload`, with the period glued to both sides. Unit variants are bare.

Lojix products are positional and parenthesized:

```text
Variant.(field0 field1 field2)
```

Vectors use square brackets:

```text
[field0 field1]
```

`None` is bare. `Some` carries one glued payload:

```text
Some.Value
```

A period is structural and right-associative. When a string is expected, a dotted bare value is reconstructed as one string. Use current Dotos curly text for a string that cannot be bare:

```text
“alpha beta”
```

Never name fields or copy the braces from an Ethos type declaration into a socket-client request. The generated Lojix product readers require parentheses.

## Ordinary requests

`Query` carries `ByNode`, `ByGeneration`, `ByDeployment`, `ByEventLog`, or `ByTestRun`.

`ByNode` has, in order:

1. cluster name
2. node name
3. optional requested generation artifact

Exact witnessed form:

```sh
lojix 'Query.ByNode.(alpha node-1 None)'
```

`ByGeneration` carries one generation identifier.

`ByDeployment` carries one deployment identifier.

`ByEventLog` has, in order:

1. first event-log position
2. last event-log position

`ByTestRun` has, in order:

1. cluster name
2. node name
3. optional test-run identifier

`WatchDeployments` has, in order:

1. optional deployment identifier
2. optional cluster name
3. optional node name

The all-target schema-derived form is:

```sh
lojix 'WatchDeployments.(None None None)'
```

`WatchCacheRetention` has, in order:

1. optional cluster name
2. optional node name

The all-target schema-derived form is:

```sh
lojix 'WatchCacheRetention.(None None)'
```

A successful watch request returns:

```text
Watching.(subscription-token commit-sequence)
```

A rejection is `WatchRejected.MalformedWatch`, `WatchRejected.SubscriptionLimitReached`, or `WatchRejected.StreamUnavailable`.

The current `lojix` executable exchanges one request for one reply and exits. It cannot consume ongoing subscription events. Do not use it as a streaming terminal monitor; re-query with `Query.ByDeployment` or `Query.ByEventLog`.

`Unwatch` carries one subscription token.

`CheckHostKeyMaterial` has, in order:

1. cluster name
2. node name
3. proposal source

## Owner requests

`Deploy.Host` has, in order:

1. cluster name
2. node name
3. host composition
4. proposal source
5. flake reference
6. deployment transport
7. deployment input mode
8. deployment output selector
9. activation backend
10. host deploy action
11. source revision policy
12. optional Nix builder
13. extra substituters

`Deploy.UserEnvironment` has, in order:

1. cluster name
2. node name
3. user name
4. proposal source
5. flake reference
6. deployment transport
7. deployment input mode
8. deployment output selector
9. activation backend
10. user-environment action
11. source revision policy
12. optional Nix builder
13. extra substituters

A deployment transport is the positional product of:

1. Nix store URI
2. SSH destination

A deployment output selector is a one-field positional product.

An optional builder is `None` or `Some` carrying its string.

Extra substituters are a vector of two-string positional products.

Host compositions are `CompleteHost` and `BaseHost`.

Host deploy actions are `Evaluate`, `Realize`, `SetBootProfile`, `ActivateNow`, `TestActivation`, and `ScheduleBootOnce`.

User-environment actions are `Realize`, `SetProfile`, and `ActivateNow`.

Deployment input modes are `Horizon` and `Direct`.

Activation backends are `NixosSystemdBootV1` and `HomeManagerNixProfileV1`.

Source revision policies are `RequireImmutable` and `ResolveAndRecord`.

`Pin` has, in order:

1. cluster name
2. node name
3. generation identifier
4. pin label

Exact witnessed form:

```sh
meta-lojix 'Pin.(alpha node-1 42 keep)'
```

`Unpin` has, in order:

1. cluster name
2. node name
3. pin label

`Retire` has, in order:

1. cluster name
2. node name
3. generation identifier

`Test` carries `Run` or `Check`.

`Test.Run` has, in order:

1. cluster name
2. node selection
3. host selection
4. test execution profile

A node selection is bare `All` or `Nodes` carrying a node-name vector.

A host selection is bare `DefaultHost` or `OnHost` carrying a node name.

A test execution profile has, in order:

1. test mode
2. Nix system
3. deployment output selector
4. optional deployment transport

`Test.Check` carries a node-name vector.

## Replies and terminal state

Ordinary reply families are `Queried`, `DeploymentEventsQueried`, `TestRunsQueried`, `Watching`, `Unwatched`, `KeyMaterialChecked`, `QueryRejected`, `WatchRejected`, `UnwatchRejected`, and `KeyMaterialCheckRejected`.

Owner reply families are `DeployAccepted`, `DeployRejected`, `DeployTerminal`, `Pinned`, `PinRejected`, `Unpinned`, `UnpinRejected`, `Retired`, `RetireRejected`, `Tested`, and `TestRejected`.

`DeployAccepted` has, in order:

1. deployment identifier
2. state marker

Exact witnessed reply:

```text
DeployAccepted.(13 (263 263))
```

`DeployAccepted` is admission only. It does not prove evaluation, build, copy, activation, or completion.

`DeployTerminal` carries the terminal deployment record.

A deployment terminal is bare `Succeeded`, `Rejected` carrying a terminal reason, or `Failed` carrying failure stage and terminal reason.

Exact witnessed failed-activation form:

```text
Some.Failed.(Activate ActivationFailed)
```

`Pinned` and `Unpinned` carry generation identifier, pin label, source slot, destination slot, and state marker.

`Retired` carries generation identifier, generation slot, and state marker.

`Tested` carries test-run identifier and state marker.

After `DeployAccepted`, re-query by deployment identifier or event-log position until a terminal deployment record exists.

## Deployment contract

Every deployment transport is explicit. Lojix uses the supplied Nix store URI and SSH destination verbatim; it never derives a route from cluster, node, or user names.

The logical node selects what is built; the activation destination selects which machine is changed. Before a state-changing deployment, verify that they identify the same node. If they do not, stop.

A `CompleteHost` deployment uses an explicit root-privileged Nix store URI and SSH destination.

A `UserEnvironment` deployment uses an explicit user-scoped Nix store URI and SSH destination.

When a deployment directly names a target pair, use it without asking for a second transport confirmation. Otherwise derive the canonical internal hostname as `<node>.<cluster>.<internal suffix>` from Horizon cluster data and use it as the host in the required `CompleteHost` or `UserEnvironment` Nix store URI and SSH destination. If the supplied or derived pair is invalid, report it rather than substituting another route.

A deployment proposal must be an existing absolute regular non-symlink `proposal.datom` file.

Use `RequireImmutable` when production deployment must identify one exact source revision. Push producer revisions before pushing the consumer revision that pins them.

A `Current` generation is Lojix's committed state. It does not establish the target's live Nix profile or runtime links.

A terminal activation failure can follow a partial target change. Inspect the target profile, runtime links, and activation journal, then report Lojix state and live state separately.

## Startup configuration

The daemon does not accept operator request Dotos. `lojix-write-configuration` is the Dotos-to-startup boundary and writes the archive consumed by `lojix-daemon`.

Its single request is the curly positional product `ConfigurationWriteRequest` with:

1. ordinary socket path
2. ordinary socket mode
3. owner socket path
4. owner socket mode
5. state directory
6. store path
7. daemon host
8. test-default choice
9. output path

Exact tested form:

```text
ConfigurationWriteRequest.{/run/fixture-lojix/ordinary.sock 432 /run/fixture-lojix/owner.sock 384 /var/lib/fixture-lojix /var/lib/fixture-lojix/configured-lojix-store.db fixture-daemon NoTestDefaults /tmp/startup.rkyv}
```

Production uses bare `NoTestDefaults`.

The nested development form is `TestDefaults` with, in order:

1. cluster name
2. VM host
3. mode
4. flake
5. Nix system
6. output selector
7. proposal source

Success prints:

```text
(ConfigurationWritten [path])
```

## Store inspection and reset

Inspect a store read-only with exactly:

```sh
lojix-inspect-store '(InspectStore /tmp/lojix.sema)'
```

Inspection does not create or register missing tables.

Reset accepts only:

```sh
lojix-reset-store '(ResetStore)'
```

It takes no path. Store selection comes from the service-owned `LOJIX_CONFIGURATION` archive.

Stop the daemon before reset.

The daemon accepts schema v4 and refuses earlier schemas. Reset removes and recreates recognized v2/v3 stores as v4. An existing v4 store is left intact.

Successful reset replies are:

```text
(LojixStoreReset path=path schema=4 removed_sidecars=count)
(LojixStoreAlreadyCurrent path=path schema=4)
```

Reset is destructive for a recognized v2/v3 store.

## Bootstrap

`lojix-bootstrap` is a separate daemon-free ingress. It accepts exactly one inline `BootstrapRun` object and does not read daemon sockets, configuration, or store state.

`BootstrapRun` is a curly positional product with:

1. request identifier
2. bootstrap mode

`BuildOnly` carries:

1. direct immutable build request
2. optional builder
3. journal parent
4. GC root
5. terminal-evidence path

The direct immutable build request carries:

1. immutable flake
2. Nix system
3. output selector

`BootOnce` additionally carries a test plan and either `RemoteNixosSystemdBootV1` or `LocalBootstrapV1`.

`RemoteNixosSystemdBootV1` carries:

1. Nix store URI
2. SSH destination
3. SSH policy
4. system profile
5. boot entries

Its SSH policy carries caller-owned identity, caller-owned known-hosts, and bare `RequireKnownHost`. Ambient SSH configuration is not accepted.

`LocalBootstrapV1` carries:

1. system profile
2. boot entries

Bootstrap immutable flakes use:

```text
github:owner/repository/40-lowercase-hex-revision
```

This differs from daemon deployment flake syntax.

Terminal output is bare `(BootstrapTerminal.Succeeded)` or `(BootstrapTerminal.Failed)`. Parse or validation failure prints a redacted `(BootstrapRejected [...])`.

## Placement

Keep Lojix configuration in the operating-system source. Do not add setup-specific deployment scripts to the user environment.

The supported deployment and observation interface is `lojix` and `meta-lojix`; setup-specific wrapper scripts are not an alternative interface.
