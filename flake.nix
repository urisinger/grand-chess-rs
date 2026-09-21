{
  description = "Rust dev env";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      forEachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          rustToolchain = pkgs.rust-bin.nightly."2025-07-30".default.override {
            extensions = [
              "clippy"
              "rust-analyzer"
              "rust-src"
              "rustfmt"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              mold

              pkg-config
              openssl
            ];

          };
        }
      );
    };
}
