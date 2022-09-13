use std::marker::PhantomData;

use relm4::gtk;
use relm4::gtk::prelude::{ComboBoxExt, ComboBoxExtManual};
use relm4::{component, ComponentParts, SimpleComponent};

#[derive(Debug)]
pub struct VecComboBox<Fields> {
    fields: PhantomData<Fields>,
}

#[component(pub)]
impl<Fields, Str> SimpleComponent for VecComboBox<Fields>
where
    Fields: Iterator<Item = Str> + 'static,
    Str: AsRef<str>,
{
    type Input = ();
    type Output = usize;
    type Init = Fields;
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
        let model = Self {
            fields: PhantomData,
        };

        let widgets = view_output!();

        for (idx, string) in init.enumerate() {
            root.insert_text(idx as i32, string.as_ref());
        }

        ComponentParts { model, widgets }
    }
}
