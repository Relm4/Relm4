/// Creates a widget that is globally accessible.
///
/// # Examples
///
/// ```no_run
/// # use relm4::{gtk, global_widget};
/// global_widget!(my_box, gtk::Box);
///
/// // Get the global widget
/// let my_box = my_box();
/// ```
///
/// If your widget doesn't have a [`Default`] implementation
/// or you want to specify a constructor, you can use the
/// third argument of the macro.
///
/// ```no_run
/// # use relm4::{gtk, global_widget};
/// global_widget!(my_label, gtk::Label, gtk::Label::new(Some("text")));
///
/// // Get the global widget
/// let my_label = my_label();
/// ```
///
/// # Panics
///
/// This macro uses [`thread_local`] internally.
/// Using the generated function from another thread
/// will cause a panic, at least if you use a [`gtk::Widget`]
/// because GTK isn't thread safe.
///
/// ```should_panic
/// # use relm4::{gtk, global_widget};
/// global_widget!(my_box, gtk::Box);
///
/// # let join_handle =
/// std::thread::spawn(|| {
///     // Get the global widget from another thread...
///     let my_box= my_box();
/// });
/// # join_handle.join().unwrap();
/// ```
#[macro_export]
macro_rules! global_widget {
    ($name:ident, $ty:ty, $init:expr) => {
        // Paste is necessary to generate unique module names
        // to support multiple invocations in the same module.
        $crate::__relm4_private_paste! {
            mod [<__widget_private_ $name>] {
                use super::*;
                use $ty as __Type;
                thread_local!(static GLOBAL_WIDGET: __Type = $init);

                pub fn $name() -> $ty {
                    GLOBAL_WIDGET.with(|w| w.clone())
                }
            }

            pub use [<__widget_private_ $name>]::$name;
        }
    };

    ($name:ident, $ty:ty) => {
        global_widget!($name, $ty, __Type::default())
    };
}
