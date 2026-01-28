{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rustfmt
  ];

  shellHook = ''
    export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-1}
  '';
}
