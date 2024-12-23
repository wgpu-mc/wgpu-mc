{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell rec {
  packages = with pkgs; [
    rustc
    cargo
    clippy
    rustfmt
    rust-analyzer

    jdk17

    nixfmt-rfc-style
  ];

  # rust deps
  buildInputs = with pkgs; [
    libxkbcommon
    openssl
    pkg-config
    vulkan-loader
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
