# Relm4-examples

A collection of small example apps using relm4.

## Run examples

If you're not already in the examples directory run:

```bash
cd relm4-examples
```

Execute examples by running:

```bash
cargo run --example NAME
```

## List of examples:

+ simple: A simple counter app.

+ simple_manual: A simple counter app that doesn't use relm4-macros.

+ components: A simple app that counts up or down and also has two components that hide and show each other. 
This demonstrates how to use components that can send messages to each other but are fully independent apart from that.

+ tracker: A simple app that shows two images that can are randomly selected from a list of icons.
The app uses a tracker for efficient UI updates.

+ factory and grid_factory: Simple apps that use a factory to create and update widgets. Factories brings the concept of trackers to collections.
A `FactoryVec` can be modified during the update method just like a normal vector and during the view function the factory will update only the affected widgets.
To know how to update the widgets the `FactoryPrototype` trait is used to define the functions needed to generate, update and remove widgets.

+ future: A small app that demonstrates how futures can be executed in relm4 by using the surf crate to download HTML from websites.
Sadly this doesn't work for tokio (async-std and similar are fine though) but you can still spawn a thread that runs your asynchronous code inside a tokio runtime.

+ list: A simple app using `gtk::ListView` to efficiently render large list. It's very close to the example in the [gtk4-rs book](https://gtk-rs.org/gtk4-rs/git/book/lists.html).

+ tokio: An app using the tokio-rt feature and an `AsyncRelmWorker` to efficiently fetch favicons and HTML text from websites.
Note that by default delays for the HTTP-requests are enabled that makes UI updates better visible.
To disable the delays comment the two lines starting with `tokio::time::delay`.

+ macro: A simple app with a counter that demonstrates how to use the `relm4-macros::widget` macro.

+ save_dialog: A simple app that uses the save dialog provided by relm4-components.

+ alert: A simple app that uses the alert provided by relm4-components.

+ popover: A simple app that uses a popover.

+ drawing: A simple app that draws moving circles on a canvas.
