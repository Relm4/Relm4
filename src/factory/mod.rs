//! Defines traits and data types used to efficiently generating widgets from collections.

use gtk::glib::Sender;
use gtk::prelude::WidgetExt;

mod collections;
mod widgets;

pub use collections::FactoryVec;
pub use widgets::GridPosition;

/// Define behavior to create, update and remove widgets according to
/// data stored in a factory.
pub trait FactoryPrototype: Sized {
    /// Factory container that stores the data.
    type Factory: Factory<Self, Self::View>;

    /// Outermost type of the newly created widgets.
    /// Similar to the `Root` type in [`crate::Widgets`].
    type Widget: WidgetExt;

    /// Widget that the generated widgets are added to.
    type View: FactoryView<Self::Widget>;

    /// Message type used to send messages back to the component or app
    /// where this factory is used
    type Msg;

    /// Create new widgets when self is inserted into the factory.
    fn generate(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
        sender: Sender<Self::Msg>,
    ) -> Self::Widget;

    /// Set the widget position upon creation, useful for [`gtk::Grid`] or similar.
    fn position(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
    ) -> <Self::View as FactoryView<Self::Widget>>::Position;

    /// Function called when self is modified.
    fn update(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
        widget: &Self::Widget,
    );

    /// Function called when self is removed from the factory.
    fn remove(widget: &Self::Widget) -> &Self::Widget;
}

/// A container that is a able to efficiently update, generate and remove widgets
/// that represent the data stored in the container.
pub trait Factory<Data, View>
where
    Data: FactoryPrototype<View = View>,
    View: FactoryView<Data::Widget>,
{
    /// Key that provides additional information for the [`FactoryPrototype`] functions.
    type Key;

    /// Efficiently update the view according to data changes.
    fn generate(&self, view: &View, sender: Sender<Data::Msg>);
}

/// A trait implemented for GTK4 widgets that allows a factory to create and remove widgets.
pub trait FactoryView<Widget: WidgetExt> {
    /// Position type used by this widget.
    ///
    /// For example [`GridPosition`] for [`gtk::Grid`] or `()` for [`gtk::Box`]
    type Position;

    /// Adds a new widget to self at the end.
    fn add(&self, widget: &Widget, position: &Self::Position);

    /// Removes a widget from self at the end.
    fn remove(&self, widget: &Widget);
}
