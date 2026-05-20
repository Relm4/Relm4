/// Dummy function for marking the `#[transition]` macro attribute as deprecated.
/// See <https://stackoverflow.com/a/77267752>.
#[deprecated = "The `#[transition]` macro attribute is deprecated. Call `set_transition_type` on the returned widget instead."]
pub const fn transition() {}
