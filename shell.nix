{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rustfmt

    # Fixes: "setlocale: LC_COLLATE: cannot change locale (en_GB.UTF-8)"
    pkgs.glibcLocales
  ];

  shellHook = ''
    # Bash parameter expansion — must be escaped for Nix strings
    export SOURCE_DATE_EPOCH=''${SOURCE_DATE_EPOCH:-1}

    # Point libc at the locale archive provided by glibcLocales
    export LOCALE_ARCHIVE="${pkgs.glibcLocales}/lib/locale/locale-archive"
  '';
}
