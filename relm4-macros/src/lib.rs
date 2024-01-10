//! A collection of macros for gtk-rs, Relm4 and Rust in general.
//!
//! Docs of related crates:
//! [relm4](https://docs.rs/relm4)
//! | [relm4-macros](https://docs.rs/relm4_macros)
//! | [relm4-components](https://docs.rs/relm4_components)
//! | [gtk4-rs](https://gtk-rs.org/gtk4-rs/git/docs)
//! | [gtk-rs-core](https://gtk-rs.org/gtk-rs-core/git/docs)
//! | [libadwaita-rs](https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/git/docs/libadwaita)
//! | [libpanel-rs](https://world.pages.gitlab.gnome.org/Rust/libpanel-rs/git/docs/libpanel)
//!
//! [GitHub](https://github.com/Relm4/Relm4)
//! | [Website](https://relm4.org)
//! | [Book](https://relm4.org/book/stable/)
//! | [Blog](https://relm4.org/blog)

#![doc(html_logo_url = "https://relm4.org/icons/relm4_logo.svg")]
#![doc(html_favicon_url = "https://relm4.org/icons/relm4_org.svg")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications,
    clippy::cargo,
    clippy::must_use_candidate
)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl};

mod additional_fields;
mod args;
mod attrs;
mod component;
mod menu;
mod view;
mod visitors;
mod widgets;

#[macro_use]
mod util;
mod factory;
mod token_streams;
mod widget_template;

use attrs::{Attrs, SyncOnlyAttrs};
use menu::Menus;

fn gtk_import() -> syn::Path {
    if cfg!(feature = "relm4") {
        util::strings_to_path(&["relm4", "gtk"])
    } else {
        util::strings_to_path(&["gtk"])
    }
}

/// Macro that implements `relm4::Component` or `relm4::SimpleComponent`
/// and generates the corresponding widgets struct.
///
/// # Attributes
///
/// To create public struct use `#[component(pub)]` or `#[component(visibility = pub)]`.
///
/// # Example
///
/// ```
/// use relm4::prelude::*;
/// use gtk::prelude::*;
///
/// #[derive(Default)]
/// struct App {
///     counter: u8,
/// }
///
/// #[derive(Debug)]
/// enum Msg {
///     Increment,
///     Decrement,
/// }
///
/// #[relm4_macros::component(pub)]
/// impl SimpleComponent for App {
///     type Init = u8;
///     type Input = Msg;
///     type Output = ();
///
///     view! {
///         gtk::Window {
///             set_title: Some("Simple app"),
///             set_default_size: (300, 100),
///             gtk::Box {
///                 set_orientation: gtk::Orientation::Vertical,
///                 set_margin_all: 5,
///                 set_spacing: 5,
///
///                 gtk::Button {
///                     set_label: "Increment",
///                     connect_clicked => Msg::Increment,
///                 },
///                 gtk::Button {
///                     set_label: "Decrement",
///                     connect_clicked[sender] => move |_| {
///                         sender.input(Msg::Decrement);
///                     },
///                 },
///                 gtk::Label {
///                     set_margin_all: 5,
///                     #[watch]
///                     set_label: &format!("Counter: {}", model.counter),
///                 }
///             },
///         }
///     }
///
///     fn init(
///         counter: Self::Init,
///         root: Self::Root,
///         sender: ComponentSender<Self>,
///     ) -> ComponentParts<Self> {
///         let model = Self { counter };
///
///         let widgets = view_output!();
///
///         ComponentParts { model, widgets }
///     }
///
///     fn update(&mut self, msg: Msg, _sender: ComponentSender<Self>) {
///         match msg {
///             Msg::Increment => {
///                 self.counter = self.counter.wrapping_add(1);
///             }
///             Msg::Decrement => {
///                 self.counter = self.counter.wrapping_sub(1);
///             }
///         }
///     }
/// }
/// ```
///
/// # Notes on `pre_view`
///
/// Using `return` in `pre_view` will cause a compiler warning.
/// In general, you don't want to use `return` in `pre_view` as it will
/// cause all following update functionality to be skipped.
///
/// ```compile_fail
/// # use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
/// # use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
/// #
/// struct App {}
///
/// #[relm4_macros::component]
/// impl SimpleComponent for App {
///       /* Code omitted */
/// #     type Init = ();
/// #     type Input = ();
/// #     type Output = ();
/// #
/// #     view! {
/// #         gtk::Window {}
/// #     }
///
///       fn pre_view() {
///           return;
///       }
/// #
/// #     fn init(
/// #         counter: Self::Init,
/// #         root: &Self::Root,
/// #         sender: ComponentSender<Self>,
/// #     ) -> ComponentParts<Self> {
/// #         let model = Self {};
/// #
/// #         let widgets = view_output!();
/// #
/// #         ComponentParts { model, widgets }
/// #     }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let global_attributes: Attrs = parse_macro_input!(attributes);
    let backup_input = input.clone();
    let component_impl_res = syn::parse::<ItemImpl>(input);

    match component_impl_res {
        Ok(component_impl) => component::generate_tokens(global_attributes, component_impl).into(),
        Err(_) => util::item_impl_error(backup_input),
    }
}

/// Macro that implements `relm4::factory::FactoryComponent` and generates the corresponding widgets struct.
///
/// # Attributes
///
/// To create public struct use `#[factory(pub)]` or `#[factory(visibility = pub)]`.
///
/// # Example
///
/// ```
/// use relm4::prelude::*;
/// use relm4::factory::*;
/// use gtk::prelude::*;
///
/// # #[derive(Debug)]
/// # enum AppMsg {
/// #     AddCounter,
/// #     RemoveCounter,
/// #     SendFront(DynamicIndex)
/// # }
///
/// #[derive(Debug)]
/// struct Counter {
///     value: u8,
/// }
///
/// #[derive(Debug)]
/// enum CounterMsg {
///     Increment,
///     Decrement,
/// }
///
/// #[derive(Debug)]
/// enum CounterOutput {
///     SendFront(DynamicIndex),
/// }
///
/// #[relm4_macros::factory(pub)]
/// impl FactoryComponent for Counter {
///     type CommandOutput = ();
///     type Init = u8;
///     type Input = CounterMsg;
///     type Output = CounterOutput;
///     type ParentWidget = gtk::Box;
///     
///
///     view! {
///         root = gtk::Box {
///             set_orientation: gtk::Orientation::Horizontal,
///             set_spacing: 10,
///
///             #[name(label)]
///             gtk::Label {
///                 #[watch]
///                 set_label: &self.value.to_string(),
///                 set_width_chars: 3,
///             },
///
///             #[name(add_button)]
///             gtk::Button {
///                 set_label: "+",
///                 connect_clicked => CounterMsg::Increment,
///             },
///
///             #[name(remove_button)]
///             gtk::Button {
///                 set_label: "-",
///                 connect_clicked => CounterMsg::Decrement,
///             },
///
///             #[name(to_front_button)]
///             gtk::Button {
///                 set_label: "To start",
///                 connect_clicked[sender, index] => move |_| {
///                     sender.output(CounterOutput::SendFront(index.clone()));
///                 }
///             }
///         }
///     }
///
///     fn init_model(
///         value: Self::Init,
///         _index: &DynamicIndex,
///         _sender: FactorySender<Self>,
///     ) -> Self {
///         Self { value }
///     }
///
///     fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
///         match msg {
///             CounterMsg::Increment => {
///                 self.value = self.value.wrapping_add(1);
///             }
///             CounterMsg::Decrement => {
///                 self.value = self.value.wrapping_sub(1);
///             }
///         }
///     }
/// }
/// ```
///
/// Note: the enclosing App view (which has AppMsg as its input) is responsible for adding a
/// forward handler to the `FactoryVecDeque`, which will translate CounterOutput events into AppMsg
/// events. For example `CounterOutput::SendFront(index) => AppMsg::SendFront(index)`
///
#[proc_macro_attribute]
pub fn factory(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attributes);
    let backup_input = input.clone();
    let factory_impl_res = syn::parse::<ItemImpl>(input);

    match factory_impl_res {
        Ok(factory_impl) => factory::generate_tokens(attrs, factory_impl).into(),
        Err(_) => util::item_impl_error(backup_input),
    }
}

/// A macro to create menus.
///
/// Use
///
/// + `"Label text" => ActionType,` to create new entries.
/// + `"Label text" => ActionType(value),` to create new entries with action value.
/// + `custom => "widget_id",` add a placeholder for custom widgets you can add later with [`set_attribute_name`](https://gtk-rs.org/gtk-rs-core/stable/0.15/docs/gio/struct.MenuItem.html#method.set_attribute_value).
/// + `section! { ... }` to create new sections.
///
/// # Example
///
/// ```
/// # fn gettext(string: &str) -> String {
/// #     string.to_owned()
/// # }
/// #
/// // Define some actions
/// relm4::new_action_group!(WindowActionGroup, "win");
/// relm4::new_stateless_action!(TestAction, WindowActionGroup, "test");
/// relm4::new_stateful_action!(TestU8Action, WindowActionGroup, "test2", u8, u8);
///
/// // Create a `MenuModel` called `menu_model`
/// relm4_macros::menu! {
///     main_menu: {
///         custom: "my_widget",
///         // Translate with gettext-rs, for example.
///         &gettext("Test") => TestAction,
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
///
/// # Macro expansion
///
/// The code generation for the example above looks like this (plus comments):
///
/// ```
/// # fn gettext(string: &str) -> String {
/// #     string.to_owned()
/// # }
/// #
/// struct WindowActionGroup;
/// impl relm4::actions::ActionGroupName for WindowActionGroup {
///     const NAME: &'static str = "win";
/// }
///
/// struct TestAction;
/// impl relm4::actions::ActionName for TestAction {
///     type Group = WindowActionGroup;
///     type State = ();
///     type Target = ();
///
///     const NAME: &'static str = "test";
/// }
///
/// struct TestU8Action;
/// impl relm4::actions::ActionName for TestU8Action {
///     type Group = WindowActionGroup;
///     type State = u8;
///     type Target = u8;
///
///     const NAME: &'static str = "test2";
/// }
///
/// // Main menu
/// let main_menu = relm4::gtk::gio::Menu::new();
///
/// // Placeholder for custom widget
/// let new_entry = relm4::gtk::gio::MenuItem::new(None, None);
/// let variant = relm4::gtk::glib::variant::ToVariant::to_variant("my_widget");
/// new_entry.set_attribute_value("custom", Some(&variant));
/// main_menu.append_item(&new_entry);
///
/// let new_entry = relm4::actions::RelmAction::<TestAction>::to_menu_item(&gettext("Test"));
/// main_menu.append_item(&new_entry);
/// let new_entry = relm4::actions::RelmAction::<TestAction>::to_menu_item("Test2");
/// main_menu.append_item(&new_entry);
/// let new_entry = relm4::actions::RelmAction::<TestU8Action>::to_menu_item_with_target_value(
///     "Test toggle",
///     &1_u8,
/// );
/// main_menu.append_item(&new_entry);
///
/// // Section 0
/// let _section_0 = relm4::gtk::gio::Menu::new();
/// main_menu.append_section(None, &_section_0);
/// let new_entry = relm4::actions::RelmAction::<TestAction>::to_menu_item("Section test");
/// _section_0.append_item(&new_entry);
/// let new_entry = relm4::actions::RelmAction::<TestU8Action>::to_menu_item_with_target_value(
///     "Test toggle",
///     &1_u8,
/// );
/// _section_0.append_item(&new_entry);
///
/// // Section 1
/// let _section_1 = relm4::gtk::gio::Menu::new();
/// main_menu.append_section(None, &_section_1);
/// let new_entry = relm4::actions::RelmAction::<TestAction>::to_menu_item("Test");
/// _section_1.append_item(&new_entry);
/// let new_entry = relm4::actions::RelmAction::<TestAction>::to_menu_item("Test2");
/// _section_1.append_item(&new_entry);
/// let new_entry = relm4::actions::RelmAction::<TestU8Action>::to_menu_item_with_target_value(
///     "Test Value",
///     &1_u8,
/// );
/// _section_1.append_item(&new_entry);
/// ```
#[proc_macro]
pub fn menu(input: TokenStream) -> TokenStream {
    let menus = parse_macro_input!(input as Menus);
    menus.menus_stream().into()
}

/// The [`view!`] macro allows you to construct your UI easily and cleanly.
///
/// It does the same as inside the [`macro@component`] attribute macro,
/// but with less features.
///
/// You can even use the `relm4-macros` crate independently from Relm4 to build your GTK4 UI.
///
/// ```no_run
/// use gtk::prelude::{BoxExt, ButtonExt};
/// use relm4::gtk;
///
/// // Creating a box with a button inside.
/// relm4_macros::view! {
///     vbox = gtk::Box {
///         gtk::Button {
///             set_label: "Click me!",
///             connect_clicked => |_| {
///                 println!("Hello world!");
///             }
///         },
///         prepend: my_label = &gtk::Label::builder()
///             .label("The view macro works!")
///             .build(),
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
/// # Macro expansion
///
/// Let's have a look the this example:
///
/// ```no_run
/// # use gtk::prelude::{BoxExt, ButtonExt};
/// # use relm4::gtk;
/// // Creating a box with a button inside.
/// relm4_macros::view! {
///     vbox = gtk::Box {
///         gtk::Button {
///             set_label: "Click me!",
///             connect_clicked => |_| {
///                 println!("Hello world!");
///             }
///         },
///         prepend: my_label = &gtk::Label::builder()
///             .label("The view macro works!")
///             .build(),
///     }
/// }
/// ```
///
/// The code generation for this example looks like this (plus comments):
///
/// ```no_run
/// # use gtk::prelude::{BoxExt, ButtonExt};
/// # use relm4::gtk;
///
/// // We've just used `gtk::Box` so we assume it has a `default()` method
/// let vbox = gtk::Box::default();
/// // `vbox` was named, yet the button doesn't have an explicit name and gets a generated one instead.
/// let _gtk_button_5 = gtk::Button::default();
/// // For the label, we used a manual constructor method, so no `default()` method is required.
/// let my_label = gtk::Label::builder().label("The view macro works!").build();
///
/// // Connect the signal
/// {
///     _gtk_button_5.connect_clicked(|_| {
///         println!("Hello world!");
///     });
/// }
///
/// // The button was added without any further instructions, so we assume `container_add()` will work.
/// relm4::RelmContainerExt::container_add(&vbox, &_gtk_button_5);
/// _gtk_button_5.set_label("Click me!");
/// // For the label, we used the `prepend` method, so we don't need `container_add()` here.
/// vbox.prepend(&my_label);
/// ```
///
/// The widgets are first initialized, then signals are connected and then
/// properties and widgets are assigned to each other.
///
/// The nested structure of the UI is translated into regular Rust code.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view::generate_tokens(input)
}

/// A macro to generate widget templates.
///
/// This macro generates a new type that implements `relm4::WidgetTemplate`.
///
/// # Example
///
/// ```
/// use relm4::prelude::*;
/// use gtk::prelude::*;
///
/// #[relm4::widget_template]
/// impl WidgetTemplate for MyBox {
///     view! {
///         gtk::Box {
///             set_margin_all: 10,
///            // Make the boxes visible
///             inline_css: "border: 2px solid blue",
///         }
///     }
/// }
/// ```
///
/// The template allows you the generate deeply nested
/// structures. All named items will be directly accessible
/// as a child of the template, even if they are nested.
/// In this example the "child_label" is a template child.
///
/// ```
/// # use relm4::prelude::*;
/// # use gtk::prelude::*;
/// #
/// # #[relm4::widget_template]
/// # impl WidgetTemplate for MyBox {
/// #     view! {
/// #         gtk::Box {
/// #             set_margin_all: 10,
/// #            // Make the boxes visible
/// #             inline_css: "border: 2px solid blue",
/// #         }
/// #     }
/// # }
/// #
/// #[relm4::widget_template]
/// impl WidgetTemplate for MySpinner {
///     view! {
///         gtk::Spinner {
///             set_spinning: true,
///         }
///     }
/// }
///
/// #[relm4::widget_template]
/// impl WidgetTemplate for CustomBox {
///     view! {
///         gtk::Box {
///             set_orientation: gtk::Orientation::Vertical,
///             set_margin_all: 5,
///             set_spacing: 5,
///
///             #[template]
///             MyBox {
///                 #[template]
///                 MySpinner,
///
///                 #[template]
///                 MyBox {
///                     #[template]
///                     MySpinner,
///
///                     #[template]
///                     MyBox {
///                         #[template]
///                         MySpinner,
///
///                         // Deeply nested!
///                         #[name = "child_label"]
///                         gtk::Label {
///                             set_label: "This is a test",
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn widget_template(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let SyncOnlyAttrs { visibility } = parse_macro_input!(attributes);

    let item_impl = parse_macro_input!(input as ItemImpl);
    widget_template::generate_tokens(visibility, item_impl).into()
}

#[cfg(test)]
#[rustversion::all(stable, since(1.72))]
mod test {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/compile-fail/**/*.rs");
    }
}
