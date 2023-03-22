{
  description = "KV DB";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        fenix-stable = fenix.packages.${system}.stable;
        toolchain = fenix-stable.toolchain;
        pkgs = nixpkgs.legacyPackages.${system};
        nightly-rustfmt = fenix.packages.${system}.default.rustfmt;
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            toolchain
          ];

          packages = (with pkgs; [
            cargo-watch
            cargo-deny
            cargo-edit
            cargo-expand

            nightly-rustfmt
          ]);
        };

      }
  );
}
