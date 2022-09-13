use relm4::gtk;
use relm4::gtk::prelude::{ComboBoxExt, ComboBoxExtManual};
use relm4::{component, ComponentParts, SimpleComponent};

#[derive(Debug)]
pub struct VecComboBox;

#[component(pub)]
impl SimpleComponent for VecComboBox {
    type Input = ();
    type Output = usize;
    type Init = Vec<String>;
    type Widgets = EnumComboBoxWidget;

    view! {
        gtk::ComboBoxText {
            connect_changed => move |combo_box| {
                if let Some(active_idx) = combo_box.active() {
                    sender.output(active_idx as usize);
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        for (idx, string) in init.iter().enumerate() {
            root.insert_text(idx as i32, string);
        }

        ComponentParts { model, widgets }
    }
}
