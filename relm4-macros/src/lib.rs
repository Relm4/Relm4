#![doc(
    html_logo_url = "https://raw.githubusercontent.com/AaronErhardt/relm4/main/assets/Relm_logo.svg"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/AaronErhardt/relm4/main/assets/Relm_logo.svg"
)]
#![allow(clippy::single_component_path_imports)]

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::parse_macro_input;

mod additional_fields;
mod args;
mod attrs;
mod component;
mod derive_components;
mod factory_prototype_macro;
mod item_impl;
mod macros;
mod menu;
mod micro_widget_macro;

#[macro_use]
mod util;

mod widget_macro;
mod widgets;

// Hack to make the macro visibile for other parts of this crate.
pub(crate) use parse_func;

use attrs::Attrs;
use item_impl::ItemImpl;
use menu::Menus;
use widgets::Widget;

#[proc_macro_attribute]
pub fn component(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let Attrs {
        visibility,
        relm4_path,
    } = parse_macro_input!(attributes as Attrs);
    let data = parse_macro_input!(input as ItemImpl);

    component::generate_tokens(visibility, relm4_path, data).into()
}

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
pub fn widget(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let Attrs {
        visibility,
        relm4_path,
    } = parse_macro_input!(attributes as Attrs);
    let data = parse_macro_input!(input as ItemImpl);

    widget_macro::generate_tokens(visibility, relm4_path, data).into()
}

/// Macro that implements [`relm4::MicrosWidgets`](https://aaronerhardt.github.io/docs/relm4/relm4/trait.MicroWidgets.html) and generates the corresponding struct.
///
/// It works very similar to [`macro@widget`].
#[proc_macro_attribute]
pub fn micro_widget(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let Attrs {
        visibility,
        relm4_path,
    } = parse_macro_input!(attributes as Attrs);
    let data = parse_macro_input!(input as ItemImpl);

    micro_widget_macro::generate_tokens(visibility, relm4_path, data).into()
}

/// Macro that implements [`relm4::factory::FactoryPrototype`](https://aaronerhardt.github.io/docs/relm4/relm4/factory/trait.FactoryPrototype.html)
/// and generates the corresponding widget struct.
///
/// It works very similar to [`macro@widget`].
#[proc_macro_attribute]
pub fn factory_prototype(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let Attrs {
        visibility,
        relm4_path,
    } = parse_macro_input!(attributes as Attrs);
    let data = parse_macro_input!(input as ItemImpl);

    factory_prototype_macro::generate_tokens(visibility, relm4_path, data).into()
}

#[proc_macro_derive(Components, attributes(components))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input);
    let output = derive_components::generate_stream(&derive_input);

    match output {
        Ok(output) => output.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

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
///         env: args!("HOME", "/home/relm4"),
///     }
/// }
///
/// // Output of "ls -la" at "/"
/// dbg!(process.output());
/// ```
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let widgets = parse_macro_input!(input as Widget);
    let default_relm4_path = util::default_relm4_path();

    let model_type = syn::Type::Tuple(syn::TypeTuple {
        paren_token: syn::token::Paren::default(),
        elems: syn::punctuated::Punctuated::new(),
    });

    let mut streams = widget_macro::token_streams::TokenStreams::default();
    widgets.generate_widget_tokens_recursively(
        &mut streams,
        &None,
        &model_type,
        &default_relm4_path,
    );
    let widget_macro::token_streams::TokenStreams {
        init_widgets,
        assign_properties,
        connect,
        ..
    } = streams;

    let output = quote! {
        #init_widgets
        #assign_properties
        #connect
    };
    output.into()
}
