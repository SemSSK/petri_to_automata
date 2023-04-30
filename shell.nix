{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    name="dev-environment";
    buildInputs = with pkgs; [
        nusmv
        rustup
        cargo
        pkgconfig
        openssl
        graphviz
    ];

}
