{ pkgs, ...}: 
  pkgs.stdenv.mkDerivation {
    name = "relm4";

    # Compile time dependencies
    nativeBuildInputs = with pkgs; [
      # Hail the Nix
      nixd
      statix
      deadnix
      alejandra

      # Rust
      rustc
      cargo
      rustfmt
      clippy
      rust-analyzer
      cargo-watch

      # Other compile time dependencies
      openssl
      # libressl

      # Gnome related
      gtk4
      meson
      ninja
      pango
      parted
      polkit
      gettext
      vte-gtk4
      pkg-config
      gdk-pixbuf
      libadwaita
      pkg-config
      libgweather
      gnome-desktop
      appstream
      appstream-glib
      wrapGAppsHook4
      desktop-file-utils
      gobject-introspection
      rustPlatform.bindgenHook
    ];

    # Set Environment Variables
    RUST_BACKTRACE = "full";
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  }
