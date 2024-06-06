{ pkgs ? import ./pkgs.nix {}, ci ? false }:

with pkgs;
mkShell {
  nativeBuildInputs = [
    gitAndTools.gh
    # Rust
    rustc
    cargo
    gcc
    rustfmt
    clippy
    # Deps
    pkg-config
    alsa-lib
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libGL
  ];
  # Don't set rpath for native addons
  NIX_DONT_SET_RPATH = true;
  NIX_NO_SELF_RPATH = true;
  RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  PKG_CONFIG_PATH = "${pkgs.pkg-config}/lib/pkgconfig";
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    alsa-lib
    libGL
  ];
  shellHook = ''

  '';
}
