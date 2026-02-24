{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust
            rustc
            cargo
            clippy
            rustfmt

            # whisper-rs-sys (builds whisper.cpp from source)
            cmake
            pkg-config
            libclang
            gcc

            # cpal (audio capture)
            alsa-lib
            pipewire

            # arboard (clipboard on Linux)
            libx11
            libxcursor
            libxrandr
            libxi
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          ALSA_PLUGIN_DIR = "${pkgs.pipewire}/lib/alsa-lib";

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.stdenv.cc.cc.lib
            pkgs.alsa-lib
            pkgs.pipewire
          ];
        };
      });
}
