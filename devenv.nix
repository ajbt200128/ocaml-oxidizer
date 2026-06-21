{ pkgs, ... }:
let
  ocamlPackages = pkgs.ocaml-ng.ocamlPackages_5_4;
  ocaml = ocamlPackages.ocaml;
in
{
  languages.ocaml.enable = true;
  languages.ocaml.packages = ocamlPackages;
  languages.rust.enable = true;

  packages = [
    ocamlPackages.dune_3
    ocamlPackages.findlib
    pkgs.pkg-config
  ];

  # build.rs link/include root. The 5.4 `ocaml` toplevel rejects -where; use `ocamlc -where`.
  env.OCAML_WHERE_PATH = "${ocaml}/lib/ocaml";
  env.OCAML_VERSION = ocaml.version;

  enterShell = ''
    echo "ocaml $(ocamlc -version)  dune $(dune --version)  rustc $(rustc --version)"
    echo "OCAML_WHERE_PATH=$OCAML_WHERE_PATH"
  '';
}
