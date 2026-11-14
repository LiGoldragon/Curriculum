{
  description = "Curriculum: the module registry and its Nexus.";

  inputs.base.url = "github:LiGoldragon/build-base";

  outputs = { self, base }:
    let
      perContext = context:
        let
          system = context.pkgs.system;
          workspace = base.lib.workspace.${system} {
            cargoNix = ./Cargo.nix;
            memberChecks = { check, ... }: {
              # The datom CLI and its declaration tests are the checked
              # surface; the Nexus binary is checked as a plain build below.
              curriculum = check "curriculum" [ "default" "datom" ];
            };
            extraChecks = { check, ... }: {
              curriculum-plain = check "curriculum" [ "default" ];
            };
          };
          plainNexus = workspace.binaries "curriculum" [ "default" ];
          datomCli = workspace.binaries "curriculum" [ "default" "datom" ];
          cliOnly = context.pkgs.runCommand "curriculum-cli" { } ''
            mkdir -p $out/bin
            cp ${datomCli}/bin/curriculum $out/bin/curriculum
          '';
          binaryChecks = context.pkgs.runCommand "curriculum-binary-feature-separation" { } ''
            export NIX_STATE_DIR=$TMPDIR/nix-state
            mkdir -p "$NIX_STATE_DIR"
            test -x ${plainNexus}/bin/curriculum-nexus
            test ! -e ${plainNexus}/bin/curriculum
            test -x ${cliOnly}/bin/curriculum
            test ! -e ${cliOnly}/bin/curriculum-nexus
            grep -aq datom_codec ${cliOnly}/bin/curriculum
            mkdir $out
          '';
          allChecks = builtins.removeAttrs workspace.checks [ "default" ] // {
            binary-feature-separation = binaryChecks;
          };
        in
        let
          normalClosure = root:
            builtins.genericClosure {
              startSet = [ { key = root.name; value = { name = root.name; dependencies = root.dependencies or []; }; } ];
              operator = entry:
                builtins.map (dependency: {
                  key = dependency.name;
                  value = { name = dependency.name; dependencies = dependency.dependencies or []; };
                }) entry.value.dependencies;
            };
          plainNormalDependencies = normalClosure plainNexus;
          datomNormalDependencies = normalClosure datomCli;
          normalNames = dependencies: builtins.map (entry: entry.value.name) dependencies;
        in
        assert !(builtins.any (name: context.pkgs.lib.hasInfix "datom-codec" name || context.pkgs.lib.hasInfix "protos" name) (normalNames plainNormalDependencies));
        assert builtins.any (name: context.pkgs.lib.hasInfix "datom-codec" name) (normalNames datomNormalDependencies);
        {
          packages = workspace.packages // {
            # The default package is the plain Nexus; the datom CLI is
            # explicit so its dependency closure is never inherited by Nexus.
            default = plainNexus;
            curriculum-nexus = plainNexus;
            curriculum = cliOnly;
          };
          checks = allChecks // {
            default = context.pkgs.linkFarm "checks" allChecks;
          };
          devShells.default = workspace.devShell;
        };
    in
    {
      packages = base.lib.forSystems (context: (perContext context).packages);
      checks = base.lib.forSystems (context: (perContext context).checks);
      devShells = base.lib.forSystems (context: (perContext context).devShells);
    };
}
