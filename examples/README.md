# Examples

A collection of example apps built with Relm4.

## Setup

```bash
git clone https://github.com/Relm4/Relm4.git
cd Relm4
```

The example `menu_actions_and_settings` requires:

```bash
mkdir -p ~/.local/share/glib-2.0/schemas/
ln -s $(pwd)/examples/relm4.example.gschema.xml ~/.local/share/glib-2.0/schemas/
glib-compile-schemas ~/.local/share/glib-2.0/schemas/
```

## Run examples

```bash
cargo run --example NAME
```

## List of examples:

**TODO...**
