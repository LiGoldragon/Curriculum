{
  description = "Curriculum skill registry and Nexus";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/2d1e72b652ee13fd1297641ce735e06416d22827";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      curriculumFor = system:
        let pkgs = import nixpkgs { inherit system; };
        in pkgs.rustPlatform.buildRustPackage {
          pname = "curriculum";
          version = "0.2.0";
          src = self;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "datom-codec-0.33.0" = "sha256-T25440qwkh6WlwK7pXZO4n6iEdsJUZYYtQnXPGVRWNw=";
              "ethos-zero-19.2.0" = "sha256-U1k7KoWtaNW6WW+xmoEzOlqCDVtixW6QwuoDDF/dPFk=";
              "nexus-0.5.0" = "sha256-ztAbsHMvobXgbkvvi4SG4EwtAqIoMBLvO2LTyI2pGDA=";
              "protos-0.33.0" = "sha256-ehjUoKl6OadQds1S/+MwgZCt/HV/q5tm7+Cgddg7PLM=";
            };
          };
          cargoBuildFlags = [ "--features" "datom" "--bins" ];
          cargoInstallFlags = [ "--path" "." "--features" "datom" "--bins" ];
          cargoTestFlags = [ "--all-targets" "--features" "datom" ];
          doCheck = true;
        };
    in
    {
      packages = forAllSystems (system: { default = curriculumFor system; curriculum = curriculumFor system; });
      checks = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          default = (curriculumFor system).overrideAttrs (_: {
            pname = "curriculum-default";
            cargoBuildFlags = [ "--bins" ];
            cargoTestFlags = [ "--all-targets" ];
          });
        in { test = curriculumFor system; test-default = default; });
    };
}
