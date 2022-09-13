use relm4::gtk::prelude::{ComboBoxExt, ComboBoxExtManual};
use relm4::{component, ComponentParts, SimpleComponent};
use relm4::{gtk, ComponentSender};

#[derive(Debug)]
pub struct VecComboBox {
    data: VecComboBoxData,
    root: gtk::ComboBoxText,
}

#[derive(Debug)]
pub struct VecComboBoxData {
    pub strings: Vec<String>,
    pub active_index: usize,
}

#[derive(Debug)]
pub enum VecComboBoxInput {
    UpdateData(VecComboBoxData),
    #[doc(hidden)]
    UpdateIndex(usize),
}

#[component(pub)]
impl SimpleComponent for VecComboBox {
    type Input = VecComboBoxInput;
    type Output = usize;
    type Init = VecComboBoxData;
    type Widgets = EnumComboBoxWidget;

    view! {
        gtk::ComboBoxText {
            connect_changed => move |combo_box| {
                if let Some(active_idx) = combo_box.active() {
                    sender.input(Self::Input::UpdateIndex(active_idx as usize));
                }
            }
        }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            VecComboBoxInput::UpdateIndex(idx) => {
                sender.output(idx);
                self.data.active_index = idx;
            }
            VecComboBoxInput::UpdateData(data) => {
                self.data = data;
                self.render();
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        let model = Self {
            data: init,
            root: root.clone(),
        };

        model.render();

        ComponentParts { model, widgets }
    }
}

impl VecComboBox {
    fn render(&self) {
        for (idx, string) in self.data.strings.iter().enumerate() {
            self.root.insert_text(idx as i32, string);
        }

        self.root
            .set_active(u32::try_from(self.data.active_index).ok());
    }
}
