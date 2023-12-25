use gtk::{prelude::IsA, glib};
use std::fmt::Debug;

/// A trait implemented for GTK4 widgets that allows a factory to create and remove widgets.
pub trait FactoryView: IsA<glib::Object> {
    /// The widget returned when inserting a widget.
    ///
    /// This doesn't matter on containers like [`gtk::Box`].
    /// However, widgets such as [`gtk::Stack`] return a [`gtk::StackPage`]
    /// which might be used to set additional parameters.
    ///
    /// Therefore, this "returned widget" is explicitly handled here.
    type ReturnedWidget: Debug + std::hash::Hash;

    /// Widget type that is attached to the container
    /// and also the root of the components.
    type Children: Debug + AsRef<Self::Children>;

    /// Position type used by this widget.
    ///
    /// For example [`GridPosition`] for [`gtk::Grid`] or `()` for [`gtk::Box`]
    ///
    /// [`GridPosition`]: crate::factory::positions::GridPosition
    type Position;

    /// Removes a widget.
    fn factory_remove(&self, widget: &Self::ReturnedWidget);
    /// Adds a new widget to self at the end.
    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
    ) -> Self::ReturnedWidget;

    /// Add an widget to the front.
    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
    ) -> Self::ReturnedWidget;

    /// Insert a widget after another widget.
    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
        other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget;

    /// Converts a returned widget to the children type.
    ///
    fn returned_widget_to_child(root_child: &Self::ReturnedWidget) -> Self::Children;

    /// Move an item after another item.
    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget);

    /// Move an item to the start.
    fn factory_move_start(&self, widget: &Self::ReturnedWidget);

    /// Update the position inside positioned containers like [`gtk::Grid`].
    fn factory_update_position(&self, _widget: &Self::ReturnedWidget, _position: &Self::Position) {}
}

/// Returns the position of an element inside a
/// container like [`gtk::Grid`] where the position isn't
/// clearly defined by the index.
pub trait Position<Pos, Index> {
    /// Returns the position.
    ///
    /// This function can be called very often
    /// if widgets are moved a lot, so it should
    /// be cheap to call.
    fn position(&self, index: &Index) -> Pos;
}

impl<C, I> Position<(), I> for C {
    fn position(&self, _index: &I) {}
}

/// Returns the position of an element inside a
/// container like [`gtk::Grid`] where the position isn't
/// clearly defined by the index.
///
/// Unlike [`Position`], this trait doesn't get access to self,
/// because the model might not be initialized when the widgets
/// are updated in the factory.
pub trait AsyncPosition<Pos> {
    /// Returns the position.
    ///
    /// This function can be called very often
    /// if widgets are moved a lot, so it should
    /// be cheap to call.
    fn position(index: usize) -> Pos;
}

impl<C> AsyncPosition<()> for C {
    fn position(_index: usize) {}
}
