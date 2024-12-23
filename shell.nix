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

  buildInputs = with pkgs; [
    # rust deps
    libxkbcommon
    openssl
    pkg-config
    renderdoc
    vulkan-loader
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr

    # minecraft deps
    alsa-lib
    libjack2
    libpulseaudio
    openal
    pipewire
    udev
  ];

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
