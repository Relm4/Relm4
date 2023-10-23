//! A wrapper around [`gtk::ComboBoxText`] that makes it easier to use
//! from regular Rust code.

use std::fmt::Debug;

use relm4::gtk::prelude::{ComboBoxExt, ComboBoxExtManual};
use relm4::{gtk, ComponentSender};
use relm4::{Component, ComponentParts};

#[derive(Debug, Clone, PartialEq, Eq)]
/// A simple wrapper around [`gtk::ComboBox`].
///
/// This can be used with enums, [`String`]s or any custom type you want.
/// The only requirement is that the inner type implements [`ToString`] and [`Debug`].
///
/// To get notified when the selection changed, you can use
/// [`Connector::forward()`](relm4::component::Connector::forward())
/// after launching the component.
pub struct SimpleComboBox<E: ToString> {
    /// The variants that can be selected.
    pub variants: Vec<E>,
    /// The index of the active element or [`None`] is nothing is selected.
    pub active_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The message type of [`SimpleComboBox`].
pub enum SimpleComboBoxMsg<E: ToString> {
    /// Overwrite the current values.
    UpdateData(SimpleComboBox<E>),
    /// Set the index of the active element.
    SetActiveIdx(usize),
    #[doc(hidden)]
    UpdateIndex(usize),
}

impl<E> Component for SimpleComboBox<E>
where
    E: ToString + 'static + Debug,
{
    type CommandOutput = ();
    type Input = SimpleComboBoxMsg<E>;
    type Output = usize;
    type Init = Self;
    type Root = gtk::ComboBoxText;
    type Widgets = gtk::ComboBoxText;

    fn init_root() -> Self::Root {
        gtk::ComboBoxText::default()
    }

    fn init(
        model: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = root.clone();

        model.render(&widgets);

        widgets.connect_changed(move |combo_box| {
            if let Some(active_idx) = combo_box.active() {
                sender.input(Self::Input::UpdateIndex(active_idx as usize));
            }
        });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        input: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match input {
            SimpleComboBoxMsg::UpdateIndex(idx) => {
                // Ignore send errors because the component might
                // be detached.
                sender.output(idx).ok();
                self.active_index = Some(idx);
            }
            SimpleComboBoxMsg::SetActiveIdx(idx) => {
                if idx < self.variants.len() {
                    self.active_index = Some(idx);
                    widgets.set_active(u32::try_from(idx).ok());
                }
            }
            SimpleComboBoxMsg::UpdateData(data) => {
                *self = data;
                self.render(widgets);
            }
        }
    }
}

impl<E> SimpleComboBox<E>
where
    E: ToString,
{
    fn render(&self, combo_box: &gtk::ComboBoxText) {
        combo_box.remove_all();

        for (idx, e) in self.variants.iter().enumerate() {
            combo_box.insert_text(idx as i32, &e.to_string());
        }

        combo_box.set_active(self.active_index.and_then(|val| u32::try_from(val).ok()));
    }

    /// Return the value of the currently selected element or [`None`] if nothing is selected.
    #[must_use]
    pub fn get_active_elem(&self) -> Option<&E> {
        self.active_index.map(|idx| &self.variants[idx])
    }
}
