<h1>
  <a href="https://relm4.org">
    <img src="assets/Relm_logo_with_text.png" width="190" alt="Relm4">
  </a>
</h1>

[![CI](https://github.com/Relm4/Relm4/actions/workflows/rust.yml/badge.svg)](https://github.com/Relm4/Relm4/actions/workflows/rust.yml)
[![Matrix](https://img.shields.io/matrix/relm4:matrix.org?label=matrix%20chat)](https://matrix.to/#/#relm4:matrix.org)
[![Relm4 on crates.io](https://img.shields.io/crates/v/relm4.svg)](https://crates.io/crates/relm4)
[![Relm4 docs](https://img.shields.io/badge/rust-documentation-blue)](https://relm4.org/docs/stable/relm4/)
[![Relm4 book](https://img.shields.io/badge/rust-book-fc0060)](https://relm4.org/book/stable/)
![Minimum Rust version 1.61](https://img.shields.io/badge/rustc-1.61+-06a096.svg)
[![dependency status](https://deps.rs/repo/github/Relm4/Relm4/status.svg)](https://deps.rs/repo/github/Relm4/Relm4)

An idiomatic GUI library inspired by [Elm](https://elm-lang.org/) and based on [gtk4-rs](https://crates.io/crates/gtk4). 
Relm4 is a new version of [relm](https://github.com/antoyo/relm) that's built from scratch and is compatible with [GTK4](https://www.gtk.org/) and [libadwaita](https://gitlab.gnome.org/GNOME/libadwaita).

## Why Relm4

We believe that GUI development should be easy, productive and delightful.  
The [gtk4-rs](https://crates.io/crates/gtk4) crate already provides everything you need to write modern, beautiful and cross-platform applications.
Built on top of this foundation, Relm4 makes developing more idiomatic, simpler and faster and enables you to become productive in just a few hours.

## Our goals

+ ⏱️ **Productivity**
+ ✨ **Simplicity**
+ 📎 **Outstanding documentation**
+ 🔧 **Maintainability**

## Documentation

+ 📖 **[The Relm4 book](https://relm4.org/book/stable/)**
+ 📜 **[Rust documentation](https://relm4.org/docs/stable/relm4/)**

## Dependencies

Relm4 depends on GTK4: [How to install GTK4](https://www.gtk.org/docs/installations/).

## Ecosystem

Relm4 has two crates that extend the core functionality:

+ [relm4-macros](https://crates.io/crates/relm4-macros) provides a `widget` macro that simplifies UI creation
+ [relm4-components](https://crates.io/crates/relm4-components) is a collections of reusable components you can easily integrate into your application

To use all features, just add this to your `Cargo.toml`:

```toml
relm4 = { version = "0.4", features = ["macros"] }
relm4-components = "0.4"
```

### Features

The `relm4` crate has four feature flags:

| &nbsp;Flag       | &nbsp;Purpose                                                                                                                                                  |
| :--------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| &nbsp;macros     | &nbsp;Enable macros by re-exporting [`relm4-macros`](https://crates.io/crates/relm4-macros)                                                                    |
| &nbsp;tokio-rt   | &nbsp;Adds the [`AsyncRelmWorker`](https://aaronerhardt.github.io/docs/relm4/relm4/struct.AsyncRelmWorker.html) type that uses an asynchronous update function |
| &nbsp;libadwaita | &nbsp;Improved support for [libadwaita](https://gitlab.gnome.org/World/Rust/libadwaita-rs)                                                                     |
| &nbsp;all        | &nbsp;Enable all features                                                                                                                                      |

## Examples

Several example applications are available at [relm4-examples/](relm4-examples/).

#### [📸 Screenshots from the example apps](assets/screenshots)

### A simple counter app

![Simple app screenshot light](assets/screenshots/simple-light.png)
![Simple app screenshot dark](assets/screenshots/simple-dark.png)

```rust
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

#[derive(Default)]
struct AppModel {
    counter: u8,
}

enum AppMsg {
    Increment,
    Decrement,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Increment);
                    },
                },
                append = &gtk::Button {
                    set_label: "Decrement",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Decrement);
                    },
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    set_label: watch! { &format!("Counter: {}", model.counter) },
                }
            },
        }
    }
}

fn main() {
    let model = AppModel::default();
    let app = RelmApp::new(model);
    app.run();
}

```

## Projects using Relm4

- [fm](https://github.com/euclio/fm) — A small, general-purpose file manager.
- [Done](https://github.com/edfloreshz/done) - A simple and versatile to do app.
- [Reovim](https://github.com/songww/reovim) - GUI frontend for neovim.
- [NixOS Configuration Editor](https://github.com/vlinkz/nixos-conf-editor) - A graphical configuration editor for [NixOS](https://nixos.org).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

**Feedback and contributions are highly appreciated!**
