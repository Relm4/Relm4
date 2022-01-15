//! Defines traits and data types used to efficiently generating widgets from collections.

use gtk::glib::Sender;
use gtk::prelude::WidgetExt;

pub mod collections;
pub mod positions;
mod widgets;

pub use collections::*;

/// Define behavior to create, update and remove widgets according to
/// data stored in a factory.
pub trait FactoryPrototype: Sized {
    /// Factory container that stores the data.
    type Factory: Factory<Self, Self::View>;

    /// Type that stores all widgets needed to update them
    /// in the [`view`](FactoryPrototype::view) function.
    type Widgets: std::fmt::Debug;

    /// Outermost type of the newly created widgets.
    /// Similar to the `Root` type in [`crate::Widgets`].
    type Root: WidgetExt;

    /// Widget that the generated widgets are added to.
    type View: FactoryView<Self::Root>;

    /// Message type used to send messages back to the component or app
    /// where this factory is used
    type Msg;

    /// Create new widgets when self is inserted into the factory.
    fn init_view(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
        sender: Sender<Self::Msg>,
    ) -> Self::Widgets;

    /// Set the widget position upon creation, useful for [`gtk::Grid`] or similar.
    fn position(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
    ) -> <Self::View as FactoryView<Self::Root>>::Position;

    /// Function called when self is modified.
    fn view(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
        widgets: &Self::Widgets,
    );

    /// Get the outermost widget from the widgets.
    fn root_widget(widgets: &Self::Widgets) -> &Self::Root;
}

/// A container that is a able to efficiently update, generate and remove widgets
/// that represent the data stored in the container.
pub trait Factory<Data, View>
where
    Data: FactoryPrototype<View = View>,
    View: FactoryView<Data::Root>,
{
    /// Key that provides additional information for the [`FactoryPrototype`] functions.
    type Key: ?Sized;

    /// Efficiently update the view according to data changes.
    fn generate(&self, view: &View, sender: Sender<Data::Msg>);
}

/*/// A trait to simplify the implementation of [`FactoryView`] for most
/// GTK4 widgets.
trait SimpleFactoryView<Widget: std::fmt::Debug> {
    /// Position type used by this widget.
    ///
    /// For example [`GridPosition`] for [`gtk::Grid`] or `()` for [`gtk::Box`]
    type Position;

    /// Adds a new widget to self at the end.
    fn add(&self, widget: &Widget, position: &Self::Position);

    /// Removes a widget from self at the end.
    fn remove(&self, widget: &Widget);
}*/

/// A trait implemented for GTK4 widgets that allows a factory to create and remove widgets.
pub trait FactoryView<Widget> {
    /// Widget type that's stored inside a factory data type.
    type Root: std::fmt::Debug;

    /// Position type used by this widget.
    ///
    /// For example [`GridPosition`](positions::GridPosition) for [`gtk::Grid`] or `()` for [`gtk::Box`]
    type Position;

    /// Adds a new widget to self at the end.
    fn add(&self, widget: &Widget, position: &Self::Position) -> Self::Root;

    /// Removes a widget from self at the end.
    fn remove(&self, widget: &Self::Root);
}

/*impl<Widget, M> FactoryView<Widget> for M
where
    M: SimpleFactoryView<Widget>,
    Widget: Clone + std::fmt::Debug,
{
    type Position = M::Position;
    type Root = Widget;

    fn add(&self, widget: &Widget, position: &Self::Position) -> Self::Root {
        self.add(widget, position);
        widget.clone()
    }

    fn remove(&self, widget: &Widget) {
        self.remove(widget);
    }
}*/

/// Extends [`FactoryView`] for containers that work similar to lists.
/// This means that the container can insert widgets before and after other
/// widgets.
pub trait FactoryListView<Widget>
where
    Self: FactoryView<Widget>,
{
    /// Insert a widget after another widget.
    fn insert_after(&self, widget: &Widget, other: &Self::Root) -> Self::Root;

    /// Add an widget to the front.
    fn push_front(&self, widget: &Widget) -> Self::Root;
}
