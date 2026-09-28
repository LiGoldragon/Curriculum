# lojix: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/lojix.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/lojix.md
+++ b/skills/lojix.md
@@ -4,47 +4,26 @@ dependencies: [nix-workflow]
 ---
 
-`lojix-daemon` owns durable state and two authority-tiered sockets. The ordinary contract is `signal-lojix`; the owner contract is `meta-signal-lojix`.
+`lojix-nexus` owns durable state and two authority-tiered sockets. The ordinary contract is `signal-lojix`; the owner contract is `meta-signal-lojix`.
 
-Use `lojix` on the ordinary socket for `Query`, `WatchDeployments`, `WatchCacheRetention`, `Unwatch`, and `CheckHostKeyMaterial`.
+Use `lojix` on the ordinary socket for `Query`, `WatchDeployments`, `WatchCacheRetention`, and `Unwatch`.
 
-Use `meta-lojix` on the owner socket for `Deploy`, `Pin`, `Unpin`, `Retire`, and `Test`. The owner contract is not optional.
+Use `lojix-meta` on the owner socket for `Deploy`, `Pin`, `Unpin`, `Retire`, and `Test`. The owner contract is not optional.
 
 Use `LOJIX_ORDINARY_SOCKET` and `LOJIX_OWNER_SOCKET`; neither socket has a default path.
 
-## Dotos syntax
+## Request syntax
 
-Each public client accepts exactly one inline Dotos object. It rejects files, signal files, flags, subcommands, zero arguments, and extra arguments.
+Each public client accepts exactly one inline datom value and rejects files, flags, subcommands, zero arguments, and extra arguments. A request root is one value.
 
-Inline decoding requires the client build's `dotos-text` feature. Missing Dotos support is a client-build defect, not permission to pass a file or flag.
-
-A request root is one object.
-
-A variant is `Head.Payload`, with the period glued to both sides. Unit variants are bare.
-
-Lojix products are positional and parenthesized:
-
-```text
-Variant.(field0 field1 field2)
-```
-
-Vectors use square brackets:
-
-```text
-[field0 field1]
-```
-
-`None` is bare. `Some` carries one glued payload:
+A struct is brace-enclosed and positional, a vector is bracket-enclosed, a variant is a head with the period glued to both sides, and a variant carrying nothing is bare. `None` is bare and `Some` carries one glued payload. A string with a space or a delimiter is written in guillemets.
 
 ```text
+Variant.{ field0 field1 field2 }
+[ field0 field1 ]
 Some.Value
+«alpha beta»
 ```
 
-A period is structural and right-associative. When a string is expected, a dotted bare value is reconstructed as one string. Use current Dotos curly text for a string that cannot be bare:
-
-```text
-“alpha beta”
-```
-
-Never name fields or copy the braces from an Ethos type declaration into a socket-client request. The generated Lojix product readers require parentheses.
+Never name a field in a request; the position carries the data.
 
 ## Ordinary requests
@@ -61,5 +40,5 @@ Exact witnessed form:
 
 ```sh
-lojix 'Query.ByNode.(alpha node-1 None)'
+lojix 'Query.ByNode.{ alpha node-1 None }'
 ```
 
@@ -88,5 +67,5 @@ The all-target schema-derived form is:
 
 ```sh
-lojix 'WatchDeployments.(None None None)'
+lojix 'WatchDeployments.{ None None None }'
 ```
 
@@ -99,5 +78,5 @@ The all-target schema-derived form is:
 
 ```sh
-lojix 'WatchCacheRetention.(None None)'
+lojix 'WatchCacheRetention.{ None None }'
 ```
 
@@ -105,5 +84,5 @@ A successful watch request returns:
 
 ```text
-Watching.(subscription-token commit-sequence)
+Watching.{ subscription-token commit-sequence }
 ```
 
@@ -114,10 +93,4 @@ The current `lojix` executable exchanges one request for one reply and exits. It
 `Unwatch` carries one subscription token.
 
-`CheckHostKeyMaterial` has, in order:
-
-1. cluster name
-2. node name
-3. proposal source
-
 ## Owner requests
 
@@ -128,13 +101,14 @@ The current `lojix` executable exchanges one request for one reply and exits. It
 3. host composition
 4. proposal source
-5. flake reference
-6. deployment transport
-7. deployment input mode
-8. deployment output selector
-9. activation backend
-10. host deploy action
-11. source revision policy
-12. optional Nix builder
-13. extra substituters
+5. secrets input
+6. flake reference
+7. deployment transport
+8. deployment input mode
+9. deployment output selector
+10. activation backend
+11. host deploy action
+12. source revision policy
+13. optional Nix builder
+14. extra substituters
 
 `Deploy.UserEnvironment` has, in order:
@@ -144,13 +118,14 @@ The current `lojix` executable exchanges one request for one reply and exits. It
 3. user name
 4. proposal source
-5. flake reference
-6. deployment transport
-7. deployment input mode
-8. deployment output selector
-9. activation backend
-10. user-environment action
-11. source revision policy
-12. optional Nix builder
-13. extra substituters
+5. secrets input
+6. flake reference
+7. deployment transport
+8. deployment input mode
+9. deployment output selector
+10. activation backend
+11. user-environment action
+12. source revision policy
+13. optional Nix builder
+14. extra substituters
 
 A deployment transport is the positional product of:
@@ -187,5 +162,5 @@ Exact witnessed form:
 
 ```sh
-meta-lojix 'Pin.(alpha node-1 42 keep)'
+lojix-meta 'Pin.{ alpha node-1 42 keep }'
 ```
 
@@ -226,7 +201,7 @@ A test execution profile has, in order:
 ## Replies and terminal state
 
-Ordinary reply families are `Queried`, `DeploymentEventsQueried`, `TestRunsQueried`, `Watching`, `Unwatched`, `KeyMaterialChecked`, `QueryRejected`, `WatchRejected`, `UnwatchRejected`, and `KeyMaterialCheckRejected`.
+Ordinary reply families are `Queried`, `DeploymentEventsQueried`, `TestRunsQueried`, `Watching`, `Unwatched`, `QueryRejected`, `WatchRejected`, and `UnwatchRejected`.
 
-Owner reply families are `DeployAccepted`, `DeployRejected`, `DeployTerminal`, `Pinned`, `PinRejected`, `Unpinned`, `UnpinRejected`, `Retired`, `RetireRejected`, `Tested`, and `TestRejected`.
+Owner reply families are `DeployAccepted`, `DeployRejected`, `DeployRefused`, `DeployTerminal`, `Pinned`, `PinRejected`, `Unpinned`, `UnpinRejected`, `Retired`, `RetireRejected`, `Tested`, and `TestRejected`.
 
 `DeployAccepted` has, in order:
@@ -238,17 +213,19 @@ Exact witnessed reply:
 
 ```text
-DeployAccepted.(13 (263 263))
+DeployAccepted.{ 13 { 263 263 } }
 ```
 
 `DeployAccepted` is admission only. It does not prove evaluation, build, copy, activation, or completion.
 
+`DeployRefused` carries a `DeployRefusalReason` (`ContinuationBudgetExhausted`, `NoCorrelatedDeployment`, or `DurableWriteFailed`) and a state marker read best-effort. Unlike `DeployRejected`, it names no deployment, because for these three reasons none exists to name.
+
 `DeployTerminal` carries the terminal deployment record.
 
-A deployment terminal is bare `Succeeded`, `Rejected` carrying a terminal reason, or `Failed` carrying failure stage and terminal reason.
+A deployment terminal is bare `Succeeded`, `Rejected` carrying a terminal reason, or `Failed` carrying failure stage and terminal reason — for example `Failed.{ CopyClosure ClosureCopyFailed }`, when the copy to the target store itself fails. `BuilderUnreachable` is not produced by a copy failure; it names an unreachable build target, not a copy target.
 
 Exact witnessed failed-activation form:
 
 ```text
-Some.Failed.(Activate ActivationFailed)
+Some.Failed.{ Activate ActivationFailed }
 ```
 
@@ -273,5 +250,5 @@ A `UserEnvironment` deployment uses an explicit user-scoped Nix store URI and SS
 When a deployment directly names a target pair, use it without asking for a second transport confirmation. Otherwise derive the canonical internal hostname as `<node>.<cluster>.<internal suffix>` from Horizon cluster data and use it as the host in the required `CompleteHost` or `UserEnvironment` Nix store URI and SSH destination. If the supplied or derived pair is invalid, report it rather than substituting another route.
 
-A deployment proposal must be an existing absolute regular non-symlink `proposal.datom` file.
+A deployment proposal must be an existing absolute regular non-symlink `horizon-definition.datom` file.
 
 Use `RequireImmutable` when production deployment must identify one exact source revision. Push producer revisions before pushing the consumer revision that pins them.
@@ -283,7 +260,7 @@ A terminal activation failure can follow a partial target change. Inspect the ta
 ## Startup configuration
 
-The daemon does not accept operator request Dotos. `lojix-write-configuration` is the Dotos-to-startup boundary and writes the archive consumed by `lojix-daemon`.
+The Nexus does not accept operator requests. `lojix-write-configuration` is the datom-to-startup boundary and writes the archive consumed by `lojix-nexus`.
 
-Its single request is the curly positional product `ConfigurationWriteRequest` with:
+Its single request is the `ConfigurationWriteRequest` struct with:
 
 1. ordinary socket path
@@ -293,5 +270,5 @@ Its single request is the curly positional product `ConfigurationWriteRequest` w
 5. state directory
 6. store path
-7. daemon host
+7. Nexus host
 8. test-default choice
 9. output path
@@ -300,5 +277,5 @@ Exact tested form:
 
 ```text
-ConfigurationWriteRequest.{/run/fixture-lojix/ordinary.sock 432 /run/fixture-lojix/owner.sock 384 /var/lib/fixture-lojix /var/lib/fixture-lojix/configured-lojix-store.db fixture-daemon NoTestDefaults /tmp/startup.rkyv}
+ConfigurationWriteRequest.{ /run/fixture-lojix/ordinary.sock 432 /run/fixture-lojix/owner.sock 384 /var/lib/fixture-lojix /var/lib/fixture-lojix/configured-lojix-store.db fixture-nexus NoTestDefaults /tmp/startup.rkyv }
 ```
 
@@ -318,7 +295,17 @@ Success prints:
 
 ```text
-(ConfigurationWritten [path])
+ConfigurationWritten.[ path ]
 ```
 
+## Readiness
+
+`lojix-nexus` announces readiness on standard output once both sockets are bound and started:
+
+```text
+(LojixNexusReady /run/lojix/ordinary.sock /run/lojix/meta.sock)
+```
+
+A supervisor waits on this line, or on standard output closing — which is what a Nexus that dies before readiness does — never on a clock. The announcement never reaches a socket; the wire stays pure signal.
+
 ## Store inspection and reset
 
@@ -326,5 +313,5 @@ Inspect a store read-only with exactly:
 
 ```sh
-lojix-inspect-store '(InspectStore /tmp/lojix.sema)'
+lojix-inspect-store 'InspectStore.{ /tmp/lojix.sema }'
 ```
 
@@ -334,27 +321,27 @@ Reset accepts only:
 
 ```sh
-lojix-reset-store '(ResetStore)'
+lojix-reset-store 'ResetStore'
 ```
 
 It takes no path. Store selection comes from the service-owned `LOJIX_CONFIGURATION` archive.
 
-Stop the daemon before reset.
+Stop the Nexus before reset.
 
-The daemon accepts schema v4 and refuses earlier schemas. Reset removes and recreates recognized v2/v3 stores as v4. An existing v4 store is left intact.
+The Nexus accepts schema v5 and refuses earlier schemas. Reset removes and recreates recognized v2/v3/v4 stores as v5. An existing v5 store is left intact.
 
 Successful reset replies are:
 
 ```text
-(LojixStoreReset path=path schema=4 removed_sidecars=count)
-(LojixStoreAlreadyCurrent path=path schema=4)
+LojixStoreReset.{ path 5 count }
+LojixStoreAlreadyCurrent.{ path 5 }
 ```
 
-Reset is destructive for a recognized v2/v3 store.
+Reset is destructive for a recognized v2/v3/v4 store.
 
 ## Bootstrap
 
-`lojix-bootstrap` is a separate daemon-free ingress. It accepts exactly one inline `BootstrapRun` object and does not read daemon sockets, configuration, or store state.
+`lojix-bootstrap` is a separate Nexus-free ingress. It accepts exactly one inline `BootstrapRun` value and does not read Nexus sockets, configuration, or store state.
 
-`BootstrapRun` is a curly positional product with:
+`BootstrapRun` is a struct with:
 
 1. request identifier
@@ -363,5 +350,5 @@ Reset is destructive for a recognized v2/v3 store.
 `BuildOnly` carries:
 
-1. direct immutable build request
+1. a `BootstrapInput`
 2. optional builder
 3. journal parent
@@ -369,5 +356,9 @@ Reset is destructive for a recognized v2/v3 store.
 5. terminal-evidence path
 
-The direct immutable build request carries:
+The journal parent, the GC root's parent, and the terminal-evidence path's parent must each already exist, be owned by the caller, and be mode `0700`. A parent left at the default umask (`0755`) is refused with a bare `BootstrapRejected.[ InvalidRequest ]` that names no permission problem — `chmod 700` each parent before submitting the request.
+
+A `BootstrapInput` is `Direct` or `Horizon`.
+
+`Direct` carries:
 
 1. immutable flake
@@ -375,4 +366,15 @@ The direct immutable build request carries:
 3. output selector
 
+`Horizon` carries:
+
+1. proposal source
+2. cluster name
+3. node name
+4. host composition
+5. secrets input
+6. immutable flake
+7. Nix system
+8. output selector
+
 `BootOnce` additionally carries a test plan and either `RemoteNixosSystemdBootV1` or `LocalBootstrapV1`.
 
@@ -398,7 +400,22 @@ github:owner/repository/40-lowercase-hex-revision
 ```
 
-This differs from daemon deployment flake syntax.
+This differs from Nexus deployment flake syntax.
+
+Terminal output is bare `BootstrapTerminal.Succeeded` or `BootstrapTerminal.Failed`. Parse or validation failure prints a redacted `BootstrapRejected.[ … ]`.
+
+## Rust library surface
+
+Since `lojix` 6.0.0, every public method on the store and the schema engine lives on a trait, never on an inherent `impl Store` or `impl SchemaRuntime` block. Import the trait that names the question being asked, not the type:
+
+- `LojixRecord` — a record type's table, family, and schema hash
+- `DurableStore` — the store's identity, write counter, and `records::<R>()`
+- `NexusPersistable` — the Nexus's durable configuration
+- `TransitionJournal` — exactly-once delivery of a durable transition
+- `DeploymentLedger` / `GenerationLedger` / `TestRunLedger` — durable deployment, generation, and test-run state
+- `RuntimeCore` — constructing the engine and driving one action to a reply
+- `DeployDriving` / `TestDriving` — driving one deployment or test run to its terminal
+- `NexusReadiness` — the readiness announcement (below)
 
-Terminal output is bare `(BootstrapTerminal.Succeeded)` or `(BootstrapTerminal.Failed)`. Parse or validation failure prints a redacted `(BootstrapRejected [...])`.
+No method above is reachable through an inherent method any more.
 
 ## Placement
@@ -406,3 +423,3 @@ Terminal output is bare `(BootstrapTerminal.Succeeded)` or `(BootstrapTerminal.F
 Keep Lojix configuration in the operating-system source. Do not add setup-specific deployment scripts to the user environment.
 
-The supported deployment and observation interface is `lojix` and `meta-lojix`; setup-specific wrapper scripts are not an alternative interface.
+The supported deployment and observation interface is `lojix` and `lojix-meta`; setup-specific wrapper scripts are not an alternative interface.
````

## Each edit

### 2ef2b53, 2026-09-12 04:13 -0600: Apply the approved skill proposals from flow f6db8d

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d. Datom request syntax, `lojix-nexus`/`lojix-meta` names, SecretsInput, schema v5: approved (named proposal, the living's "meta cli names is `<component>-meta`"). The removal of the KeyMaterialChecked reply families went beyond the proposal.
View: candidate, less the unshown removal.

````diff
--- a/skills/lojix.md
+++ b/skills/lojix.md
@@ -5,7 +5,7 @@ dependencies: [nix-workflow]
 
-`lojix-daemon` owns durable state and two authority-tiered sockets. The ordinary contract is `signal-lojix`; the owner contract is `meta-signal-lojix`.
+`lojix-nexus` owns durable state and two authority-tiered sockets. The ordinary contract is `signal-lojix`; the owner contract is `meta-signal-lojix`.
 
-Use `lojix` on the ordinary socket for `Query`, `WatchDeployments`, `WatchCacheRetention`, `Unwatch`, and `CheckHostKeyMaterial`.
+Use `lojix` on the ordinary socket for `Query`, `WatchDeployments`, `WatchCacheRetention`, and `Unwatch`.
 
-Use `meta-lojix` on the owner socket for `Deploy`, `Pin`, `Unpin`, `Retire`, and `Test`. The owner contract is not optional.
+Use `lojix-meta` on the owner socket for `Deploy`, `Pin`, `Unpin`, `Retire`, and `Test`. The owner contract is not optional.
 
@@ -13,37 +13,16 @@ Use `LOJIX_ORDINARY_SOCKET` and `LOJIX_OWNER_SOCKET`; neither socket has a defau
 
-## Dotos syntax
+## Request syntax
 
-Each public client accepts exactly one inline Dotos object. It rejects files, signal files, flags, subcommands, zero arguments, and extra arguments.
+Each public client accepts exactly one inline datom value and rejects files, flags, subcommands, zero arguments, and extra arguments. A request root is one value.
 
-Inline decoding requires the client build's `dotos-text` feature. Missing Dotos support is a client-build defect, not permission to pass a file or flag.
-
-A request root is one object.
-
-A variant is `Head.Payload`, with the period glued to both sides. Unit variants are bare.
-
-Lojix products are positional and parenthesized:
-
-```text
-Variant.(field0 field1 field2)
-```
-
-Vectors use square brackets:
-
-```text
-[field0 field1]
-```
-
-`None` is bare. `Some` carries one glued payload:
+A struct is brace-enclosed and positional, a vector is bracket-enclosed, a variant is a head with the period glued to both sides, and a variant carrying nothing is bare. `None` is bare and `Some` carries one glued payload. A string with a space or a delimiter is written in guillemets.
 
 ```text
+Variant.{ field0 field1 field2 }
+[ field0 field1 ]
 Some.Value
+«alpha beta»
 ```
 
-A period is structural and right-associative. When a string is expected, a dotted bare value is reconstructed as one string. Use current Dotos curly text for a string that cannot be bare:
-
-```text
-“alpha beta”
-```
-
-Never name fields or copy the braces from an Ethos type declaration into a socket-client request. The generated Lojix product readers require parentheses.
+Never name a field in a request; the position carries the data.
 
@@ -62,3 +41,3 @@ Exact witnessed form:
 ```sh
-lojix 'Query.ByNode.(alpha node-1 None)'
+lojix 'Query.ByNode.{ alpha node-1 None }'
 ```
@@ -89,3 +68,3 @@ The all-target schema-derived form is:
 ```sh
-lojix 'WatchDeployments.(None None None)'
+lojix 'WatchDeployments.{ None None None }'
 ```
@@ -100,3 +79,3 @@ The all-target schema-derived form is:
 ```sh
-lojix 'WatchCacheRetention.(None None)'
+lojix 'WatchCacheRetention.{ None None }'
 ```
@@ -106,3 +85,3 @@ A successful watch request returns:
 ```text
-Watching.(subscription-token commit-sequence)
+Watching.{ subscription-token commit-sequence }
 ```
@@ -115,8 +94,2 @@ The current `lojix` executable exchanges one request for one reply and exits. It
 
-`CheckHostKeyMaterial` has, in order:
-
-1. cluster name
-2. node name
-3. proposal source
-
 ## Owner requests
@@ -129,11 +102,12 @@ The current `lojix` executable exchanges one request for one reply and exits. It
 4. proposal source
-5. flake reference
-6. deployment transport
-7. deployment input mode
-8. deployment output selector
-9. activation backend
-10. host deploy action
-11. source revision policy
-12. optional Nix builder
-13. extra substituters
+5. secrets input
+6. flake reference
+7. deployment transport
+8. deployment input mode
+9. deployment output selector
+10. activation backend
+11. host deploy action
+12. source revision policy
+13. optional Nix builder
+14. extra substituters
 
@@ -145,11 +119,12 @@ The current `lojix` executable exchanges one request for one reply and exits. It
 4. proposal source
-5. flake reference
-6. deployment transport
-7. deployment input mode
-8. deployment output selector
-9. activation backend
-10. user-environment action
-11. source revision policy
-12. optional Nix builder
-13. extra substituters
+5. secrets input
+6. flake reference
+7. deployment transport
+8. deployment input mode
+9. deployment output selector
+10. activation backend
+11. user-environment action
+12. source revision policy
+13. optional Nix builder
+14. extra substituters
 
@@ -188,3 +163,3 @@ Exact witnessed form:
 ```sh
-meta-lojix 'Pin.(alpha node-1 42 keep)'
+lojix-meta 'Pin.{ alpha node-1 42 keep }'
 ```
@@ -227,3 +202,3 @@ A test execution profile has, in order:
 
-Ordinary reply families are `Queried`, `DeploymentEventsQueried`, `TestRunsQueried`, `Watching`, `Unwatched`, `KeyMaterialChecked`, `QueryRejected`, `WatchRejected`, `UnwatchRejected`, and `KeyMaterialCheckRejected`.
+Ordinary reply families are `Queried`, `DeploymentEventsQueried`, `TestRunsQueried`, `Watching`, `Unwatched`, `QueryRejected`, `WatchRejected`, and `UnwatchRejected`.
 
@@ -239,3 +214,3 @@ Exact witnessed reply:
 ```text
-DeployAccepted.(13 (263 263))
+DeployAccepted.{ 13 { 263 263 } }
 ```
@@ -251,3 +226,3 @@ Exact witnessed failed-activation form:
 ```text
-Some.Failed.(Activate ActivationFailed)
+Some.Failed.{ Activate ActivationFailed }
 ```
@@ -274,3 +249,3 @@ When a deployment directly names a target pair, use it without asking for a seco
 
-A deployment proposal must be an existing absolute regular non-symlink `proposal.datom` file.
+A deployment proposal must be an existing absolute regular non-symlink `horizon-definition.datom` file.
 
@@ -284,5 +259,5 @@ A terminal activation failure can follow a partial target change. Inspect the ta
 
-The daemon does not accept operator request Dotos. `lojix-write-configuration` is the Dotos-to-startup boundary and writes the archive consumed by `lojix-daemon`.
+The Nexus does not accept operator requests. `lojix-write-configuration` is the datom-to-startup boundary and writes the archive consumed by `lojix-nexus`.
 
-Its single request is the curly positional product `ConfigurationWriteRequest` with:
+Its single request is the `ConfigurationWriteRequest` struct with:
 
@@ -294,3 +269,3 @@ Its single request is the curly positional product `ConfigurationWriteRequest` w
 6. store path
-7. daemon host
+7. Nexus host
 8. test-default choice
@@ -301,3 +276,3 @@ Exact tested form:
 ```text
-ConfigurationWriteRequest.{/run/fixture-lojix/ordinary.sock 432 /run/fixture-lojix/owner.sock 384 /var/lib/fixture-lojix /var/lib/fixture-lojix/configured-lojix-store.db fixture-daemon NoTestDefaults /tmp/startup.rkyv}
+ConfigurationWriteRequest.{ /run/fixture-lojix/ordinary.sock 432 /run/fixture-lojix/owner.sock 384 /var/lib/fixture-lojix /var/lib/fixture-lojix/configured-lojix-store.db fixture-nexus NoTestDefaults /tmp/startup.rkyv }
 ```
@@ -319,3 +294,3 @@ Success prints:
 ```text
-(ConfigurationWritten [path])
+ConfigurationWritten.[ path ]
 ```
@@ -327,3 +302,3 @@ Inspect a store read-only with exactly:
 ```sh
-lojix-inspect-store '(InspectStore /tmp/lojix.sema)'
+lojix-inspect-store 'InspectStore.{ /tmp/lojix.sema }'
 ```
@@ -335,3 +310,3 @@ Reset accepts only:
 ```sh
-lojix-reset-store '(ResetStore)'
+lojix-reset-store 'ResetStore'
 ```
@@ -340,5 +315,5 @@ It takes no path. Store selection comes from the service-owned `LOJIX_CONFIGURAT
 
-Stop the daemon before reset.
+Stop the Nexus before reset.
 
-The daemon accepts schema v4 and refuses earlier schemas. Reset removes and recreates recognized v2/v3 stores as v4. An existing v4 store is left intact.
+The Nexus accepts schema v5 and refuses earlier schemas. Reset removes and recreates recognized v2/v3/v4 stores as v5. An existing v5 store is left intact.
 
@@ -347,7 +322,7 @@ Successful reset replies are:
 ```text
-(LojixStoreReset path=path schema=4 removed_sidecars=count)
-(LojixStoreAlreadyCurrent path=path schema=4)
+LojixStoreReset.{ path 5 count }
+LojixStoreAlreadyCurrent.{ path 5 }
 ```
 
-Reset is destructive for a recognized v2/v3 store.
+Reset is destructive for a recognized v2/v3/v4 store.
 
@@ -355,5 +330,5 @@ Reset is destructive for a recognized v2/v3 store.
 
-`lojix-bootstrap` is a separate daemon-free ingress. It accepts exactly one inline `BootstrapRun` object and does not read daemon sockets, configuration, or store state.
+`lojix-bootstrap` is a separate Nexus-free ingress. It accepts exactly one inline `BootstrapRun` value and does not read Nexus sockets, configuration, or store state.
 
-`BootstrapRun` is a curly positional product with:
+`BootstrapRun` is a struct with:
 
@@ -399,5 +374,5 @@ github:owner/repository/40-lowercase-hex-revision
 
-This differs from daemon deployment flake syntax.
+This differs from Nexus deployment flake syntax.
 
-Terminal output is bare `(BootstrapTerminal.Succeeded)` or `(BootstrapTerminal.Failed)`. Parse or validation failure prints a redacted `(BootstrapRejected [...])`.
+Terminal output is bare `BootstrapTerminal.Succeeded` or `BootstrapTerminal.Failed`. Parse or validation failure prints a redacted `BootstrapRejected.[ … ]`.
 
@@ -407,2 +382,2 @@ Keep Lojix configuration in the operating-system source. Do not add setup-specif
 
-The supported deployment and observation interface is `lojix` and `meta-lojix`; setup-specific wrapper scripts are not an alternative interface.
+The supported deployment and observation interface is `lojix` and `lojix-meta`; setup-specific wrapper scripts are not an alternative interface.
````

### 8484ecd, 2026-09-12 06:46 -0600: Apply lojix skill addendum 2 (B2, B3, B4/B5, B6)

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow f6db8d. Written after the living's last approval that morning; grounded only in the flow's own witness reports.
View: drop. The Rust library surface is for lojix's developers, not its callers; the bootstrap-permission and wire-vocabulary details belong to the tool's own documentation.

````diff
--- a/skills/lojix.md
+++ b/skills/lojix.md
@@ -204,3 +204,3 @@ Ordinary reply families are `Queried`, `DeploymentEventsQueried`, `TestRunsQueri
 
-Owner reply families are `DeployAccepted`, `DeployRejected`, `DeployTerminal`, `Pinned`, `PinRejected`, `Unpinned`, `UnpinRejected`, `Retired`, `RetireRejected`, `Tested`, and `TestRejected`.
+Owner reply families are `DeployAccepted`, `DeployRejected`, `DeployRefused`, `DeployTerminal`, `Pinned`, `PinRejected`, `Unpinned`, `UnpinRejected`, `Retired`, `RetireRejected`, `Tested`, and `TestRejected`.
 
@@ -219,5 +219,7 @@ DeployAccepted.{ 13 { 263 263 } }
 
+`DeployRefused` carries a `DeployRefusalReason` (`ContinuationBudgetExhausted`, `NoCorrelatedDeployment`, or `DurableWriteFailed`) and a state marker read best-effort. Unlike `DeployRejected`, it names no deployment, because for these three reasons none exists to name.
+
 `DeployTerminal` carries the terminal deployment record.
 
-A deployment terminal is bare `Succeeded`, `Rejected` carrying a terminal reason, or `Failed` carrying failure stage and terminal reason.
+A deployment terminal is bare `Succeeded`, `Rejected` carrying a terminal reason, or `Failed` carrying failure stage and terminal reason — for example `Failed.{ CopyClosure ClosureCopyFailed }`, when the copy to the target store itself fails. `BuilderUnreachable` is not produced by a copy failure; it names an unreachable build target, not a copy target.
 
@@ -297,2 +299,12 @@ ConfigurationWritten.[ path ]
 
+## Readiness
+
+`lojix-nexus` announces readiness on standard output once both sockets are bound and started:
+
+```text
+(LojixNexusReady /run/lojix/ordinary.sock /run/lojix/meta.sock)
+```
+
+A supervisor waits on this line, or on standard output closing — which is what a Nexus that dies before readiness does — never on a clock. The announcement never reaches a socket; the wire stays pure signal.
+
 ## Store inspection and reset
@@ -339,3 +351,3 @@ Reset is destructive for a recognized v2/v3/v4 store.
 
-1. direct immutable build request
+1. a `BootstrapInput`
 2. optional builder
@@ -345,3 +357,7 @@ Reset is destructive for a recognized v2/v3/v4 store.
 
-The direct immutable build request carries:
+The journal parent, the GC root's parent, and the terminal-evidence path's parent must each already exist, be owned by the caller, and be mode `0700`. A parent left at the default umask (`0755`) is refused with a bare `BootstrapRejected.[ InvalidRequest ]` that names no permission problem — `chmod 700` each parent before submitting the request.
+
+A `BootstrapInput` is `Direct` or `Horizon`.
+
+`Direct` carries:
 
@@ -351,2 +367,13 @@ The direct immutable build request carries:
 
+`Horizon` carries:
+
+1. proposal source
+2. cluster name
+3. node name
+4. host composition
+5. secrets input
+6. immutable flake
+7. Nix system
+8. output selector
+
 `BootOnce` additionally carries a test plan and either `RemoteNixosSystemdBootV1` or `LocalBootstrapV1`.
@@ -378,2 +405,17 @@ Terminal output is bare `BootstrapTerminal.Succeeded` or `BootstrapTerminal.Fail
 
+## Rust library surface
+
+Since `lojix` 6.0.0, every public method on the store and the schema engine lives on a trait, never on an inherent `impl Store` or `impl SchemaRuntime` block. Import the trait that names the question being asked, not the type:
+
+- `LojixRecord` — a record type's table, family, and schema hash
+- `DurableStore` — the store's identity, write counter, and `records::<R>()`
+- `NexusPersistable` — the Nexus's durable configuration
+- `TransitionJournal` — exactly-once delivery of a durable transition
+- `DeploymentLedger` / `GenerationLedger` / `TestRunLedger` — durable deployment, generation, and test-run state
+- `RuntimeCore` — constructing the engine and driving one action to a reply
+- `DeployDriving` / `TestDriving` — driving one deployment or test run to its terminal
+- `NexusReadiness` — the readiness announcement (below)
+
+No method above is reachable through an inherent method any more.
+
 ## Placement
````

