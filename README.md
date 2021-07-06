# Relm4

An experimental port of [relm](https://github.com/antoyo/relm) to use GTK4. Actually it's rather a rewrite/redesign of relm's core functionality than a port by now but it's already functional. 

It's not finished yet so macros are still missing and the API might change in the future.

## Example

Please have a look at the examples folder. Examples can be run with `cargo run --example NAME`. The following examples are available:

+ components: A simple app that counts up or down and also has two components that hide and show each other. 
This demonstrates how to use components that can send messages to each other but are fully independent apart from that.

+ tracker: A simple app that can show different widgets and also count up.
For each update of the UI only the actual changes to the model are considered to minimize UI updates.
For example counting up by toggling the button will not affect the other widgets and will not trigger a regeneration of the selectable widget.


**Feedback on the design and contributions are highly appreciated!**
