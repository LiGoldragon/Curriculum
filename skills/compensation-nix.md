---
description: A `<repo>-test` repository is being created, or an integration scenario, sandbox, or other Nix code in one is being written, run, or landed.
dependencies: [nix-workflow, testing, repository-lifecycle, secrets]
---

A repository's integration scenarios live in its own `<repo>-test` repository, never in the tested repository's flake. The test repository takes each tested repository as a flake input that follows its `nixpkgs`:

    inputs.flow.url = "github:LiGoldragon/flow";
    inputs.flow.inputs.nixpkgs.follows = "nixpkgs";

Test unpushed code with `--override-input <input> path:<checkout>`. After the code lands, run `nix flake update <input>` and commit the lock.

Lay the repository out for numtide blueprint, as CriomOS does. `flake.nix` holds only the inputs and `outputs = inputs: inputs.blueprint { inherit inputs; };`.

    lib/default.nix             shared builders, reached elsewhere as flake.lib.<name>
    lib/components/<name>.nix   one component: how to build, configure, and start it
    checks/<scenario>.nix       a pure scenario
    packages/<scenario>.nix     a semi-sandbox scenario's runner
    fixtures/<scenario>/        data one scenario reads
    formatter.nix               { pkgs, ... }: pkgs.nixfmt
    checks/lint.nix             the style gate below

A scenario is named by its components joined by hyphens, in the order it drives them: `message-flow`, `message-flow-spirit`. Add a suffix only to separate two scenarios with the same components. A scenario takes its components from `flake.lib` and adds only its own drive and assertions.

A pure scenario runs in the build sandbox with no network and no credentials: `pkgs.testers.runNixOSTest` when it needs services or several machines, `pkgs.runCommand` otherwise.

A scenario that needs the living's logins, the network, or a live model is a semi-sandbox run by `nix run .#<scenario>`. Its `packages/<scenario>.nix` is a `pkgs.writeShellApplication` that makes a fresh state root with `mktemp -d`, copies in at run time only the login files its components need, starts every component against that root, drives the cheapest model (Haiku for Claude, Luna for Codex) unless given another, asserts, and removes the root in an exit trap. Credentials reach only that runtime root; a semi-sandbox is never a check, an `__impure` derivation, or a `__noChroot` build.

One file holds one scenario, component, or builder.
Name every package as `pkgs.<name>`; never `with pkgs;`.
Format with `nix fmt` before each commit. The style gate:

    { pkgs, flake, ... }:
    pkgs.runCommand "lint"
      {
        nativeBuildInputs = [
          pkgs.nixfmt
          pkgs.deadnix
          pkgs.statix
        ];
      }
      ''
        cd ${flake}
        find . -name '*.nix' -exec nixfmt --check {} +
        deadnix --fail .
        statix check .
        touch $out
      ''

Evaluate before building: `nix flake check --no-build --option allow-import-from-derivation false`, then `nix flake check`.

A new test repository lands with the style gate and one pure scenario passing. After each push, run `nix flake check github:LiGoldragon/<repo>-test` against the remote flake, and any semi-sandbox you changed as `nix run github:LiGoldragon/<repo>-test#<scenario>`.
