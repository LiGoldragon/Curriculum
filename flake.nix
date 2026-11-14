{
  description = "Curriculum: the module registry and its Nexus.";

  # The build base: nixpkgs, the Rust toolchain and the per-crate machinery,
  # shared with every repository on it. After editing Cargo.toml or Cargo.lock:
  #   nix run github:LiGoldragon/build-base#crate2nix -- generate
  inputs.base.url = "github:LiGoldragon/build-base";

  outputs =
    { self, base }:
    base.lib.workspaceOutputs {
      cargoNix = ./Cargo.nix;
      # Curriculum reads its sources' declarations, its roles and its
      # requests as datom, so its check builds and lints it with that feature.
      memberChecks = { check, ... }: {
        curriculum = check "curriculum" [
          "default"
          "datom"
        ];
      };
    };
}
