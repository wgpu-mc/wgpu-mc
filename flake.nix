{
  inputs = {
    nixpkgs.url = "nixpkgs";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    flake-parts,
    rust-overlay,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        lib,
        pkgs,
        system,
        ...
      }: {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        # TODO: maybe add proper nix package derivations, to have patchelf-ed versions of the mod?
        devShells.default = pkgs.mkShell {
          # needed to compile and run stuff correctly
          LD_LIBRARY_PATH = with pkgs;
            lib.makeLibraryPath [
              openssl
              glfw
              xorg.libX11
            ];

          buildInputs = with pkgs; [
            # TODO: use rust-toolchain.toml to pinpoint nightly version to avoid breakage
            (rust-bin.nightly.latest.default.override {
              extensions = ["rust-analyzer" "rust-src"];
            })

            openssl
            pkg-config

            jdk17
          ];
        };
      };
    };
}
