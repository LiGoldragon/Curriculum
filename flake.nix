{
  description = "Curriculum: the module registry and its Nexus.";

  inputs.base.url = "github:LiGoldragon/build-base";

  outputs =
    { self, base }:
    let
      perContext =
        context:
        let
          system = context.pkgs.system;
          lib = context.pkgs.lib;
          plainFeatures = [ "default" ];
          datomFeatures = [
            "default"
            "datom"
          ];
          workspace = base.lib.workspace.${system} {
            cargoNix = ./Cargo.nix;
            memberChecks = { check, ... }: {
              curriculum = check "curriculum" datomFeatures;
            };
            extraChecks = { check, ... }: {
              curriculum-plain = check "curriculum" plainFeatures;
            };
          };
          plainNexus = workspace.binaries "curriculum" plainFeatures;
          datomCli = (workspace.binaries "curriculum" datomFeatures).overrideBinaries (arguments: {
            crateBin = builtins.filter (target: target.name == "curriculum") arguments.crateBin;
          });
          plainTestBuild = workspace.checks.curriculum-plain.tests;
          datomTestBuild = workspace.checks.curriculum.tests;
          # Normal dependencies describe linked code. Build dependencies,
          # including ethos-zero, generate it and are a separate graph.
          linkedCrates =
            root:
            builtins.genericClosure {
              startSet = [
                {
                  key = root.drvPath;
                  crate = root;
                }
              ];
              operator =
                node:
                map (crate: {
                  key = crate.drvPath;
                  inherit crate;
                }) node.crate.dependencies;
            };
          hasDatom = root: builtins.any (node: node.crate.crateName == "datom-codec") (linkedCrates root);
          nexusClosure = context.pkgs.closureInfo { rootPaths = [ plainNexus ]; };
          cliOnly = context.pkgs.runCommand "curriculum-cli" { } ''
            mkdir -p $out/bin
            cp ${datomCli}/bin/curriculum $out/bin/curriculum
          '';
          binaryChecks = context.pkgs.runCommand "curriculum-binary-feature-separation" { } ''
            test -x ${plainNexus}/bin/curriculum-nexus
            test ! -e ${plainNexus}/bin/curriculum
            test -x ${datomCli}/bin/curriculum
            test ! -e ${datomCli}/bin/curriculum-nexus
            test -x ${cliOnly}/bin/curriculum
            test ! -e ${cliOnly}/bin/curriculum-nexus
            grep -aq datom_codec ${cliOnly}/bin/curriculum
            if grep -F datom-codec ${nexusClosure}/store-paths; then
              echo "The plain Nexus runtime closure contains datom-codec" >&2
              exit 1
            fi
            mkdir $out
            cp ${nexusClosure}/store-paths $out/nexus-store-paths
          '';
          allChecks = builtins.removeAttrs workspace.checks [ "default" ] // {
            binary-feature-separation = binaryChecks;
          };
        in
        assert lib.assertMsg (
          !(builtins.elem "datom" plainNexus.crateFeatures) && !(hasDatom plainNexus)
        ) "The plain Nexus links datom";
        assert lib.assertMsg (
          builtins.elem "datom" datomCli.crateFeatures && hasDatom datomCli
        ) "The CLI must link datom";
        assert lib.assertMsg (
          plainTestBuild.crateFeatures == plainNexus.crateFeatures && !(hasDatom plainTestBuild)
        ) "The plain check must exercise the plain Nexus feature graph";
        assert lib.assertMsg (
          datomTestBuild.crateFeatures == datomCli.crateFeatures && hasDatom datomTestBuild
        ) "The CLI check must exercise the datom feature graph";
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
