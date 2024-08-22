{ pkgs ? import ./pkgs.nix {}, ci ? false }:

with pkgs;
mkShell {
  nativeBuildInputs = [
    gitAndTools.gh
    # Rust
    rustc
    rustc-wasm32
    cargo
    gcc
    llvmPackages.bintools
    rustfmt
    clippy
    cmake
    wasm-pack
    # Deps
    fontconfig
    freetype
    pkg-config
    alsa-lib
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libGL
    vulkan-loader
    udev
  ];
  # Don't set rpath for native addons
  NIX_DONT_SET_RPATH = true;
  NIX_NO_SELF_RPATH = true;
  RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  PKG_CONFIG_PATH = "${pkgs.pkg-config}/lib/pkgconfig";
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    fontconfig
    freetype
    alsa-lib
    libGL
    vulkan-loader
    udev
  ];
  CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
  shellHook = ''

  '';
}
