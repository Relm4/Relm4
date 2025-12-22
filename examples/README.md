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

Some examples require additional features to be enabled, such as support for
[Adwaita](https://gnome.pages.gitlab.gnome.org/adwaita/) or newer GNOME versions.
The requirements for each example are provided in 
[the Relm4 `Cargo.toml`](/relm4/Cargo.toml).

## List of examples:

### `actions.rs`

Use [actions](https://docs.rs/relm4/latest/relm4/actions/index.html) to trigger [messages] or other events with
keyboard shortcuts and [menus](https://docs.rs/gio/latest/gio/struct.Menu.html).

### `components.rs`

A [`MessageDialog`](https://docs.rs/gtk4/latest/gtk4/struct.MessageDialog.html) made of multiple [components].

### `data_binding.rs`

Automatically update the UI using [data binding](https://docs.rs/relm4/latest/relm4/binding/index.html).

### `drag_and_drop.rs`

Implement drag and drop functionality.

### `drawing.rs`

Draw coloured circles on a [`DrawingArea`](https://docs.rs/gtk4/latest/gtk4/struct.DrawingArea.html) with 
[Cairo](https://cairographics.org/) (a 2D graphics library with GTK integration) and a
[`DrawHandler`](https://docs.rs/relm4/latest/relm4/abstractions/drawing/struct.DrawHandler.html).

### `drop_sub_components.rs`

Drop subcomponents instead of hiding them when they aren't being used.
Useful for reducing memory usage.

### `embedded_logo.rs`

Include and display an image embedded in the executable.

### `entry.rs`

Use the [`Entry`](https://docs.rs/gtk4/latest/gtk4/struct.Entry.html) widget and
[`EntryBuffer`](https://docs.rs/gtk4/latest/gtk4/struct.EntryBuffer.html).

### `factory.rs`

Create dynamic lists of [components] using [factories].

### `factory_async.rs`

Use [factories] with asynchronous [components].

### `factory_hash_map.rs`

Use [factories] with a [`FactoryHashMap`](https://docs.rs/relm4/latest/relm4/factory/struct.FactoryHashMap.html).

### `grid_factory.rs`

Create a grid of [components] using [factories].

### `icons`

Use [`relm4-icons`](https://docs.rs/relm4-icons) to add icons to your app.

Run with `cargo run --manifest-path icons/Cargo.toml`

### `leaflet_sidebar.rs`

Responsive sidebar layout example inspired by the example code in the
[libadwaita documentation](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/adaptive-layouts.html#leaflet).

### `log.rs`

Logging with the [`tracing`](https://docs.rs/tracing) crate.

### `macro_reference.rs`

A comprehensive reference for the [`relm4::component`](https://docs.rs/relm4/latest/relm4/attr.component.html) and
[`view!`](https://docs.rs/relm4/latest/relm4/macro.view.html) macros.

Further details on these macros are provided in
[the relevant section of the Relm4 book](https://relm4.org/book/stable/component_macro/reference.html).

### `menu.rs`

Create [menus](https://docs.rs/gio/latest/gio/struct.Menu.html) (and submenus!) using the
[`relm4::menu`](https://docs.rs/relm4/latest/relm4/macro.menu.html) macro.

### `message_broker.rs`

Communicate between nested components using a
[`MessageBroker`](https://docs.rs/relm4/latest/relm4/component/struct.MessageBroker.html).

### `message_from_grid_view.rs`

Send [messages] from items within a [`GridView`](https://docs.rs/gtk4/latest/gtk4/struct.GridView.html).

### `message_stream.rs`

Use [`Sender`](https://docs.rs/relm4/latest/relm4/struct.Sender.html) and
[`Receiver`](https://docs.rs/relm4/latest/relm4/struct.Receiver.html) for asynchronous communication.

### `multi_window.rs`

Open and manage multiple [`Window`](https://docs.rs/gtk4/latest/gtk4/struct.Window.html)s.

### `navigation_splitview_with_stack.rs`

An [`adw::NavigationSplitView`](https://docs.rs/libadwaita/latest/libadwaita/struct.NavigationSplitView.html) with a
[`Stack`](https://docs.rs/gtk4/latest/gtk4/struct.Stack.html).

### `non_blocking_async.rs`

Perform non-blocking asynchronous operations with
[commands](https://relm4.org/book/stable/threads_and_async/commands.html).

### `non_blocking_sync.rs`

Perform non-blocking synchronous operations with
[synchronous commands](https://relm4.org/book/stable/threads_and_async/commands.html#synchronous-tasks).

### `popover.rs`

Use [`Popover`](https://docs.rs/gtk4/latest/gtk4/struct.Popover.html)s to show additional information or controls.

### `progress.rs`

Use a [`ProgressBar`](https://docs.rs/gtk4/latest/gtk4/struct.ProgressBar.html) to monitor the progress of a background
task.

### `settings_list.rs`

A [component][components] that allows the caller to define what options are in its list.

### `simple.rs`

A basic "Hello World"-style example showing the core Relm4 concepts; namely, [components], 
[messages], and the `view!` macro.

### `simple_async.rs`

A basic example showing how to use
[`AsyncComponent`](https://docs.rs/relm4/latest/relm4/component/trait.AsyncComponent.html).

### `simple_manual.rs`

Manual implementation of the
[`Component`](https://docs.rs/relm4/latest/relm4/component/trait.Component.html) trait without the `view!` macro.

### `split_layout.rs`

A split layout using an
[`adw::OverlaySplitView`](https://docs.rs/libadwaita/latest/libadwaita/struct.OverlaySplitView.html).

### `state_management.rs`

State management with data flow based on [messages] and [factories].

### `tab_factory.rs`

Manage tabs in an [`adw::TabView`](https://docs.rs/libadwaita/latest/libadwaita/struct.TabView.html) with [factories].

### `tab_game.rs`

A simple game demonstrating tab management and
[`SharedState`](https://docs.rs/relm4/latest/relm4/shared_state/struct.SharedState.html).

### `to_do.rs`

A simple to-do list application using [factories].

### `toast.rs`

Show toasts (small popup notifications) using 
[`adw::ToastOverlay`](https://docs.rs/libadwaita/latest/libadwaita/struct.ToastOverlay.html).

### `tracker.rs`

Efficient UI updates using the [`tracker`](https://docs.rs/tracker) crate.
[Discussed further in the Relm4 book](https://relm4.org/book/stable/efficient_ui/tracker.html).

### `transient_dialog.rs`

Create and show transient [`Dialog`](https://docs.rs/gtk4/latest/gtk4/struct.Dialog.html)s.

### `typed_column_view.rs`

Idiomatic and type-safe column views using
[`TypedColumnView`](https://docs.rs/relm4/latest/relm4/typed_view/column/struct.TypedColumnView.html).

### `typed_grid_view.rs`

Idiomatic and type-safe grid views using
[`TypedGridView`](https://docs.rs/relm4/latest/relm4/typed_view/grid/struct.TypedGridView.html).

### `typed_list_view.rs`

Idiomatic and type-safe list views using
[`TypedListView`](https://docs.rs/relm4/latest/relm4/typed_view/list/struct.TypedListView.html).

### `typed_list_view_async.rs`

Use [`TypedListView`](https://docs.rs/relm4/latest/relm4/typed_view/list/struct.TypedListView.html) with asynchronous
data loading.

### `widget_template.rs`

Create reusable UI elements with the [`widget_template`](https://docs.rs/relm4/latest/relm4/attr.widget_template.html)
macro.
[Discussed further in the Relm4 Book](https://relm4.org/book/stable/widget_templates/index.html).

### `worker.rs`

Perform background tasks without blocking the UI using
[`Worker`](https://docs.rs/relm4/latest/relm4/component/worker/trait.Worker.html)s.

<!-- links -->
[components]: https://relm4.org/book/stable/components.html

[factories]: https://relm4.org/book/stable/efficient_ui/factory.html

[messages]: https://relm4.org/book/stable/basic_concepts/messages.html
