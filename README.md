# Relm4

An experimental port of [relm](https://github.com/antoyo/relm) to use GTK4. Actually it's rather a rewrite/redesign of relm's core functionality than a port by now but it's already functional. 

It's not finished yet so macros are still missing and the API might change in the future.

## Example

Please have a look at the [main.rs](src/main.rs) file. It contains an example with an app that counts up or down and also has two components that hide and show each other. This demonstrates how to use components that can send messages to each other but are fully independent apart from that.

**Feedback on the design and contributions are highly appreciated!**