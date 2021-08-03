<h1>
  <img src="assets/Relm_logo_with_text.svg" height="65" alt="Relm4">
</h1>

![CI](https://github.com/AaronErhardt/relm4/actions/workflows/CI/badge.svg)

An experimental port of [relm](https://github.com/antoyo/relm) to use GTK4. Actually it's rather a rewrite/redesign of relm's core functionality than a port by now but it's already functional. 

It's not finished yet so macros are still missing and the API might change in the future.

## Example

Please have a look at the examples folder. Examples can be run with:

```bash
cd relm4-examples
cargo run --example NAME
```

The following examples are available:

+ components: A simple app that counts up or down and also has two components that hide and show each other. 
This demonstrates how to use components that can send messages to each other but are fully independent apart from that.

+ tracker: A simple app that can show different widgets and also count up.
For each update of the UI only the actual changes to the model are considered to minimize UI updates.
For example counting up by toggling the button will not affect the other widgets and will not trigger a regeneration of the selectable widget.

+ generator and grid_generator: Simple apps that use a generator to create and update widgets. A generator brings the concept of trackers to collections.
A `VecGen` can be modified during the update method just like a normal vector and during the view function the generator will update only the affected widgets.
To know how to update the widgets, a `GeneratorBlueprint` is used that defines the functions needed to generate, update and remove widgets.

+ future: A small app that demonstrates how futures can be executed in relm4 by using the surf crate to download HTML from websites.
Sadly this doesn't work for tokio (async-std and similar are fine though) but you can still spawn a thread that runs your asynchronous code inside a tokio runtime.

+ list: A simple app using `gtk::ListView` to efficiently render large list. It's very close to the example in the [gtk4-rs book](https://gtk-rs.org/gtk4-rs/git/book/lists.html).

+ tokio: An app using the tokio-rt feature and an `AsyncRelmWorker` to efficiently fetch favicons and HTML text from websites.
Note that by default delays for the HTTP-requests are enabled that makes UI updates better visible.
To disable the delays comment the two lines starting with `tokio::time::delay`.

+ macro: A simple app with a counter that demonstrates how to use the `relm4-macros::widget` macro.

**Feedback on the design and contributions are highly appreciated!**
