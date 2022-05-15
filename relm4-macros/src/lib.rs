#![doc(html_logo_url = "https://relm4.org/icons/relm4_logo.svg")]
#![doc(html_favicon_url = "https://relm4.org/icons/relm4_org.svg")]
#![allow(clippy::single_component_path_imports)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod additional_fields;
mod args;
mod attrs;
mod component;
//mod factory_prototype;
mod item_impl;
mod macros;
mod menu;
mod view;
mod widgets;

#[macro_use]
mod util;

use attrs::Attrs;
use item_impl::ItemImpl;
use menu::Menus;

/// Macro that implements [`relm4::Widgets`](https://aaronerhardt.github.io/docs/relm4/relm4/trait.Widgets.html) and generates the corresponding struct.
///
/// # Attributes
///
/// To create public struct use `#[widget(pub)]` or `#[widget(visibility = pub)]`.
///
/// If you use reexports to provide relm4, then you can use `#[widget(relm4= ::myreexports::my_relm)]` to override relm4 used during generating struct.
///
/// # Example
///
/// ```
/// use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
/// use relm4::{gtk, send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};
///
/// #[derive(Default)]
/// struct AppModel {
///     counter: u8,
/// }
///
/// enum AppMsg {
///     Increment,
///     Decrement,
/// }
///
/// impl Model for AppModel {
///     type Msg = AppMsg;
///     type Widgets = AppWidgets;
///     type Components = ();
/// }
///
/// impl AppUpdate for AppModel {
///     fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
///         match msg {
///             AppMsg::Increment => {
///                 self.counter = self.counter.wrapping_add(1);
///             }
///             AppMsg::Decrement => {
///                 self.counter = self.counter.wrapping_sub(1);
///             }
///         }
///         true
///     }
/// }
///
/// #[relm4_macros::widget]
/// impl Widgets<AppModel, ()> for AppWidgets {
///     view! {
///         gtk::ApplicationWindow {
///             set_title: Some("Simple app"),
///             set_default_width: 300,
///             set_default_height: 100,
///             set_child = Some(&gtk::Box) {
///                 set_orientation: gtk::Orientation::Vertical,
///                 set_margin_all: 5,
///                 set_spacing: 5,
///
///                 append = &gtk::Button {
///                     set_label: "Increment",
///                     connect_clicked(sender) => move |_| {
///                         send!(sender, AppMsg::Increment);
///                     },
///                 },
///                 append = &gtk::Button {
///                     set_label: "Decrement",
///                     connect_clicked(sender) => move |_| {
///                         send!(sender, AppMsg::Decrement);
///                     },
///                 },
///                 append = &gtk::Label {
///                     set_margin_all: 5,
///                     set_label: watch! { &format!("Counter: {}", model.counter) },
///                 }
///             },
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let Attrs {
        visibility,
        relm4_path,
    } = parse_macro_input!(attributes as Attrs);
    let data = parse_macro_input!(input as ItemImpl);

    component::generate_tokens(visibility, relm4_path, data).into()
}

// Macro that implements [`relm4::factory::FactoryPrototype`](https://aaronerhardt.github.io/docs/relm4/relm4/factory/trait.FactoryPrototype.html)
// and generates the corresponding widget struct.
//
// It works very similar to [`macro@widget`].
// #[proc_macro_attribute]
//pub fn factory_prototype(attributes: TokenStream, input: TokenStream) -> TokenStream {
// let Attrs {
//     visibility,
//     relm4_path,
// } = parse_macro_input!(attributes as Attrs);
// let data = parse_macro_input!(input as ItemImpl);

// factory_prototype_macro::generate_tokens(visibility, relm4_path, data).into()
//    quote! {}.into()
// }

/// A macro to create menus.
///
/// # Example
///
/// ```
/// // Define some actions
/// relm4::new_action_group!(WindowActionGroup, "win");
/// relm4::new_stateless_action!(TestAction, WindowActionGroup, "test");
/// relm4::new_stateful_action!(TestU8Action, WindowActionGroup, "test2", u8, u8);
///
/// // Create a `MenuModel` called `menu_model`
/// relm4_macros::menu! {
///     main_menu: {
///         "Test" => TestAction,
///         "Test2" => TestAction,
///         "Test toggle" => TestU8Action(1_u8),
///         section! {
///             "Section test" => TestAction,
///             "Test toggle" => TestU8Action(1_u8),
///         },
///         section! {
///             "Test" => TestAction,
///             "Test2" => TestAction,
///             "Test Value" => TestU8Action(1_u8),
///         }
///     }
/// };
/// ```
#[proc_macro]
pub fn menu(input: TokenStream) -> TokenStream {
    let menus = parse_macro_input!(input as Menus);
    let default_relm4_path = util::default_relm4_path();

    menus.menus_stream(&default_relm4_path).into()
}

/// The [`view!`] macro allows you to construct your UI easily and cleanly.
///
/// It does the same as inside the [`macro@widget`] attribute macro,
/// but with less features (no factories, components, etc).
///
/// You can even use the `relm4-macros` crate independently from Relm4 to build your GTK4 UI.
///
/// ```no_run
/// use relm4::gtk;
/// use gtk::prelude::{BoxExt, ButtonExt};
///
/// // Creating a box with a button inside.
/// relm4_macros::view! {
///     vbox = gtk::Box {
///         append = &gtk::Button {
///             set_label: "Click me!",
///             connect_clicked => |_| {
///                 println!("Hello world!");
///             }
///         },
///     }
/// }
///
/// // You can simply use the vbox created in the macro.
/// let spacing = vbox.spacing();
/// ```
///
/// Also, the macro doesn't rely on any special gtk4-rs features
/// so you can even use the macro for other purposes.
///
/// In this example, we use it to construct a [`Command`](std::process::Command).
///
/// ```
/// use std::process::Command;
///
/// let path = "/";
///
/// relm4_macros::view! {
///     mut process = Command::new("ls") {
///         args: ["-la"],
///         current_dir = mut &String {
///             push_str: path,
///         },
///         env: ("HOME", "/home/relm4"),
///     }
/// }
///
/// // Output of "ls -la" at "/"
/// dbg!(process.output());
/// ```
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view::generate_tokens(input)
}
