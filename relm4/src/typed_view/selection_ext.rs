use gtk::{gio, prelude::IsA};

pub trait RelmSelectionExt: IsA<gtk::SelectionModel> {
    fn new_model(model: gio::ListModel) -> Self;
    fn set_list_model(&mut self, model: &gio::ListModel);
}

macro_rules! impl_selection (
    ($ty:ty) => {
        impl RelmSelectionExt for $ty {
            fn new_model(model: gio::ListModel) -> Self {
                Self::new(Some(model))
            }
            fn set_list_model(&mut self, model: &gio::ListModel) {
                self.set_model(Some(model));
            }
        }
    }
);

impl_selection!(gtk::SingleSelection);
impl_selection!(gtk::MultiSelection);
impl_selection!(gtk::NoSelection);
