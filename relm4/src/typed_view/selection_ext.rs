use gtk::{gio, prelude::IsA};

/// Allows to create new or change existing [`gtk::SelectionModel`](gtk::SelectionModel)
/// for Relm's type safe abstractions
/// over [`gtk::ColumnView`](gtk::ColumnView)
/// (i.e. [`TypedColumnView`](crate::typed_view::column::TypedColumnView)),
/// [`gtk::ListView`](gtk::ListView)
/// (i.e. [`TypedListView`](crate::typed_view::list::TypedListView))
/// and [`gtk::GridView`](gtk::GridView)
/// (i.e. [`TypedGridView`](crate::typed_view::grid::TypedGridView)).
pub trait RelmSelectionExt: IsA<gtk::SelectionModel> {
    /// Creates new [`gtk::SelectionModel`](gtk::SelectionModel) to view it is implemented on
    ///
    /// Typically called during `init`
    /// phase of [`TypedColumnView`](crate::typed_view::column::TypedColumnView),
    /// [`TypedListView`](crate::typed_view::list::TypedListView)
    /// or [`TypedGridView`](crate::typed_view::grid::TypedGridView)
    /// to provide a chosen selection model for view instantiation.
    fn new_model(model: gio::ListModel) -> Self;

    /// Set provided [`gio::ListModel`](gio::ListModel) on the chosen (i.e. [`Self`]) selection model
    ///
    /// Usually it is useful to do during the addition of new filters,
    /// in that case the existing model has to be replaced with provided [`gtk::FilterListModel`](gtk::FilterListModel).
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
