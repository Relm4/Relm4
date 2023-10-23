//! A wrapper around [`adw::ComboRow`] that makes it easier to use
//! from regular Rust code.

use std::fmt::Debug;

use relm4::{adw, Component, ComponentParts, ComponentSender};

use adw::gtk::StringList;
use adw::prelude::ComboRowExt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A simple wrapper around [`adw::ComboRow`].
///
/// This can be used with enums, [`String`]s or any custom type you want.
/// The only requirement is that the inner type implements [`ToString`] and [`Debug`].
///
/// To get notified when the selection changed, you can use
/// [`Connector::forward()`](relm4::component::Connector::forward())
/// after launching the component.
pub struct SimpleComboRow<E: ToString> {
    /// The variants that can be selected.
    pub variants: Vec<E>,
    /// The index of the active element or [`None`] is nothing is selected.
    pub active_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The message type of [`SimpleComboRow`].
pub enum SimpleComboRowMsg<E: ToString> {
    /// Overwrite the current values.
    UpdateData(SimpleComboRow<E>),
    /// Set the index of the active element.
    SetActiveIdx(usize),
    #[doc(hidden)]
    UpdateIndex(usize),
}

impl<E> Component for SimpleComboRow<E>
where
    E: ToString + 'static + Debug,
{
    type CommandOutput = ();
    type Input = SimpleComboRowMsg<E>;
    type Output = usize;
    type Init = Self;
    type Root = adw::ComboRow;
    type Widgets = adw::ComboRow;

    fn init_root() -> Self::Root {
        adw::ComboRow::default()
    }

    fn init(
        model: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = root.clone();

        model.render(&widgets);

        widgets.connect_selected_notify(move |combo_box| {
            sender.input(Self::Input::UpdateIndex(combo_box.selected() as _));
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
            SimpleComboRowMsg::UpdateIndex(idx) => {
                // Ignore send errors because the component might
                // be detached.
                sender.output(idx).ok();
                self.active_index = Some(idx);
            }
            SimpleComboRowMsg::SetActiveIdx(idx) => {
                if idx < self.variants.len() {
                    self.active_index = Some(idx);
                    widgets.set_selected(idx as u32);
                }
            }
            SimpleComboRowMsg::UpdateData(data) => {
                *self = data;
                self.render(widgets);
            }
        }
    }
}

impl<E> SimpleComboRow<E>
where
    E: ToString,
{
    fn render(&self, combo_box: &adw::ComboRow) {
        let model: StringList = self.variants.iter().map(ToString::to_string).collect();
        combo_box.set_model(Some(&model));

        if let Some(idx) = self.active_index {
            combo_box.set_selected(idx as u32);
        }
    }

    /// Return the value of the currently selected element or [`None`] if nothing is selected.
    #[must_use]
    pub fn get_active_elem(&self) -> Option<&E> {
        self.active_index.map(|idx| &self.variants[idx])
    }
}
