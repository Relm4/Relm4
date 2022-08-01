#![doc(html_logo_url = "https://relm4.org/icons/relm4_logo.svg")]
#![doc(html_favicon_url = "https://relm4.org/icons/relm4_org.svg")]

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod additional_fields;
mod args;
mod attrs;
mod component;
mod macros;
mod menu;
mod view;
mod visitors;
mod widgets;

#[macro_use]
mod util;
mod factory;

use attrs::Attrs;
use menu::Menus;

fn gtk_import() -> syn::Path {
    if cfg!(feature = "relm4") {
        util::strings_to_path(&["relm4", "gtk"])
    } else {
        util::strings_to_path(&["gtk"])
    }
}

/// Macro that implements [`relm4::Component`](https://relm4.org/docs/next/relm4/trait.Component.html) and generates the corresponding struct.
///
/// # Attributes
///
/// To create public struct use `#[component(pub)]` or `#[component(visibility = pub)]`.
///
/// # Example
///
/// ```
/// use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
/// use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, WidgetPlus};
///
/// #[derive(Default)]
/// struct AppModel {
///     counter: u8,
/// }
///
/// #[derive(Debug)]
/// enum AppMsg {
///     Increment,
///     Decrement,
/// }
///
/// #[relm4_macros::component]
/// impl SimpleComponent for AppModel {
///     type InitParams = u8;
///     type Input = AppMsg;
///     type Output = ();
///     type Widgets = AppWidgets;
///
///     view! {
///         gtk::Window {
///             set_title: Some("Simple app"),
///             set_default_width: 300,
///             set_default_height: 100,
///             gtk::Box {
///                 set_orientation: gtk::Orientation::Vertical,
///                 set_margin_all: 5,
///                 set_spacing: 5,
///
///                 gtk::Button {
///                     set_label: "Increment",
///                     connect_clicked[sender] => move |_| {
///                         sender.input(AppMsg::Increment);
///                     },
///                 },
///                 gtk::Button {
///                     set_label: "Decrement",
///                     connect_clicked[sender] => move |_| {
///                         sender.input(AppMsg::Decrement);
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
///         counter: Self::InitParams,
///         root: &Self::Root,
///         sender: ComponentSender<Self>,
///     ) -> ComponentParts<Self> {
///         let model = Self { counter };
///
///         let widgets = view_output!();
///
///         ComponentParts { model, widgets }
///     }
///
///     fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
///         match msg {
///             AppMsg::Increment => {
///                 self.counter = self.counter.wrapping_add(1);
///             }
///             AppMsg::Decrement => {
///                 self.counter = self.counter.wrapping_sub(1);
///             }
///         }
///     }
/// }
/// ```
///
/// # Notes on pre_view
///
/// Using `return` in `pre_view` will cause a compiler warning.
/// In general, you don't want to use `return` in `pre_view` as it will
/// cause all following update functionality to be skipped.
///
/// ```compile_fail
/// #![deny(unreachable_code)]
///
/// # use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
/// # use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, WidgetPlus};
/// #
/// struct AppModel {}
///
/// #[relm4_macros::component]
/// impl SimpleComponent for AppModel {
///       /* Code omitted */
/// #     type InitParams = ();
/// #     type Input = ();
/// #     type Output = ();
/// #     type Widgets = AppWidgets;
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
/// #         counter: Self::InitParams,
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
    let Attrs { visibility } = parse_macro_input!(attributes as Attrs);
    let component_impl = parse_macro_input!(input as syn::ItemImpl);

    component::generate_tokens(visibility, component_impl).into()
}

#[proc_macro_attribute]
pub fn factory(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let Attrs { visibility } = parse_macro_input!(attributes as Attrs);
    let factory_impl = parse_macro_input!(input as syn::ItemImpl);

    factory::generate_tokens(visibility, factory_impl).into()
}

/// A macro to create menus.
///
/// Use
///
/// + `"Label text" => ActionType,` to create new entries.
/// + `"Label text" => ActionType(value),` to create new entries with action value.
/// + `custom => "widget_id",` add a placeholder for custom widgets you can add later with [set_attribute_name](https://gtk-rs.org/gtk-rs-core/stable/0.15/docs/gio/struct.MenuItem.html#method.set_attribute_value).
/// + `section! { ... }` to create new sections.
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
///         custom: "my_widget",
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
///
/// # Macro expansion
///
/// The code generation for the example above looks like this (plus comments):
///
/// ```
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
/// let new_entry = relm4::actions::RelmAction::<TestAction>::to_menu_item("Test");
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

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/compile-fail/**/*.rs");
}
