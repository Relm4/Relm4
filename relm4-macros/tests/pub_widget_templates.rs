use gtk::prelude::GtkWindowExt;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub mod templates {
    use relm4::{gtk, WidgetTemplate};

    #[relm4::widget_template(pub)]
    impl WidgetTemplate for TestTemplate {
        view! {
            gtk::Box {
                #[name(test_child)]
                gtk::Label {}
            }
        }
    }
}

#[derive(Default)]
struct AppModel;

#[relm4_macros::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = ();
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_size: (300, 100),

            #[template]
            templates::TestTemplate {
                #[template_child]
                test_child -> gtk::Label {
                    set_label: "It works!",
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: (), _sender: ComponentSender<Self>) {}
}
