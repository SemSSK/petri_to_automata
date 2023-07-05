{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # pest-ide-tools.url = "github:SemSSK/pest-ide-tools";
  };

  outputs = { self, naersk, nixpkgs, utils, rust-overlay}:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs { inherit system overlays; };
        naersk-lib = pkgs.callPackage naersk { };
        libPath = pkgs.lib.makeLibraryPath (with pkgs; [
          libGL
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ]);
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [
            # pest-ide-tools.defaultPackage.${system}
            pkg-config
            rust-bin.stable.latest.default
            udev            
            protobuf
            bacon
            graphviz
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = libPath;
        };
      });
}
