{
  description = "Rust flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      nativeBuildInputs = with pkgs; [rustc cargo rustfmt cargo-watch rustup trunk lld pkg-config wasm-bindgen-cli cutechess stockfish];
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = nativeBuildInputs;
      };
    });
}
