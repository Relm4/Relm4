use gtk::gio;

pub(super) trait RelmSelectionExt {
    fn set_list_model(&mut self, model: &gio::ListModel);
}

macro_rules! impl_selection (
    ($ty:ty) => {
        impl RelmSelectionExt for $ty {
            fn set_list_model(&mut self, model: &gio::ListModel) {
                self.set_model(Some(model));
            }
        }
    }
);

impl_selection!(gtk::SingleSelection);
impl_selection!(gtk::MultiSelection);
impl_selection!(gtk::NoSelection);
