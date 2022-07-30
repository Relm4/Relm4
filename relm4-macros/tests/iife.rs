use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

struct AppModel;

#[relm4_macros::component]
impl SimpleComponent for AppModel {
    type InitParams = ();
    type Input = ();
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        gtk::Window {
            gtk::Label {
                #[watch]
                set_label: &format!("Counter: {}", counter),
            }
        }
    }

    fn pre_view() {
        // Only works if pre_view isn't wrapped inside an IIFE
        // because the local variable counter is used in the
        // update_view method.
        let counter = 1;
    }

    fn init(
        _counter: Self::InitParams,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let counter = 1;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn assert_impls_debug<T: std::fmt::Debug>() {}

#[test]
fn assert_widgets_impl_debug() {
    assert_impls_debug::<AppWidgets>();
}
