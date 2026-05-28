use relm4::ComponentController;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, Controller, SimpleComponent, gtk};
pub struct App(Controller<Inner>);

#[relm4_macros::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            #[local_ref]
            inner -> gtk::Stack {},
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self(Inner::builder().launch(()).detach());
        let inner = model.0.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

pub struct Inner(bool);

#[relm4_macros::component(pub)]
impl SimpleComponent for Inner {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        stack = if model.0 {
            gtk::Box {
                append: &label,
            }
        } else {
            gtk::Label {
                set_label: "False",
            }
        } -> {
            set_transition_type: gtk::StackTransitionType::Crossfade,
        },

        label = gtk::Label {
            set_label: "True",
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self(true);
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {
        self.0 = !self.0;
    }
}
