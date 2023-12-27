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
        devShells.default = with pkgs; mkShell rec {
          nativeBuildInputs = [pkg-config];

          buildInputs = [
            # TODO: use rust-toolchain.toml to pinpoint nightly version to avoid breakage
            (rust-bin.nightly.latest.default.override {
              extensions = ["rust-analyzer" "rust-src"];
            })

            # rust deps
            openssl vulkan-loader
            xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # x11
            libxkbcommon wayland # wayland

            jdk17
          ];

          packages = [nil];

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs; # im lazy
        };

        formatter = pkgs.alejandra;
      };
    };
}
