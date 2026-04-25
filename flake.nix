{
  description = "Build truly native applications with ease!";

  inputs = {
    # Stable for keeping thins clean
    # nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";

    # Fresh and new for testing
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    # The flake-utils library
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    # @ inputs
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        # Nix script formatter
        formatter = pkgs.nixfmt;

        # Development environment
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Nix
            nixd
            statix
            deadnix
            nixfmt

            # Rust
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            cargo-watch
            openssl
            # libressl

            # Gnome related
            gtk4
            gtk4
            meson
            ninja
            pango
            parted
            polkit
            gdk-pixbuf
            libadwaita
            pkg-config
            gnome-desktop
            appstream
            appstream-glib
            wrapGAppsHook4
            desktop-file-utils
            gobject-introspection
            rustPlatform.bindgenHook
          ];
        };
      }
    );
}
