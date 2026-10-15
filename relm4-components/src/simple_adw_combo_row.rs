//! A wrapper around [`adw::ComboRow`] that makes it easier to use
//! from regular Rust code.

use std::fmt::Debug;

use relm4::{Component, ComponentParts, ComponentSender, adw};

use adw::gtk::{StringList, glib::signal::SignalHandlerId, prelude::ObjectExt};
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

#[derive(Debug)]
#[doc = "Widgets of the simple combo row component."]
pub struct SimpleComboRowWidgets {
    combo_row: adw::ComboRow,
    combo_selected_signal: SignalHandlerId,
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
    type Widgets = SimpleComboRowWidgets;

    fn init_root() -> Self::Root {
        adw::ComboRow::default()
    }

    fn init(
        model: Self::Init,
        combo_row: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let combo_selected_signal = combo_row.connect_selected_notify(move |combo_box| {
            sender.input(Self::Input::UpdateIndex(combo_box.selected() as _));
        });

        let mut widgets = SimpleComboRowWidgets {
            combo_row,
            combo_selected_signal,
        };

        model.render(&mut widgets);

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
                    widgets.combo_row.set_selected(idx as u32);
                }
            }
            SimpleComboRowMsg::UpdateData(data) => {
                *self = data;
                widgets
                    .combo_row
                    .block_signal(&widgets.combo_selected_signal);
                self.render(widgets);
                widgets
                    .combo_row
                    .unblock_signal(&widgets.combo_selected_signal);
            }
        }
    }
}

impl<E> SimpleComboRow<E>
where
    E: ToString,
{
    fn render(&self, widgets: &mut SimpleComboRowWidgets) {
        let model: StringList = self.variants.iter().map(ToString::to_string).collect();
        widgets.combo_row.set_model(Some(&model));

        if let Some(idx) = self.active_index {
            widgets.combo_row.set_selected(idx as u32);
        }
    }

    /// Return the value of the currently selected element or [`None`] if nothing is selected.
    #[must_use]
    pub fn get_active_elem(&self) -> Option<&E> {
        self.active_index.map(|idx| &self.variants[idx])
    }
}
