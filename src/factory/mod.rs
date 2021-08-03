use gtk::glib::Sender;
use gtk::prelude::WidgetExt;

mod collections;
mod widgets;

pub use collections::FactoryVec;
pub use widgets::GridPosition;

pub struct GeneratorBlueprint<Data, Key, Widget, Positioning, Msg> {
    pub generate: fn(data: &Data, key: &Key, sender: Sender<Msg>) -> (Widget, Positioning),
    pub update: fn(data: &Data, key: &Key, &Widget),
    pub remove: fn(&Widget) -> &Widget,
}

pub trait FactoryPrototype: Sized {
    type Factory: Factory<Self, Self::View>;
    type Widget: WidgetExt;
    type View: FactoryView<Self::Widget>;
    type Msg;

    fn generate(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
        sender: Sender<Self::Msg>,
    ) -> Self::Widget;
    fn position(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
    ) -> <Self::View as FactoryView<Self::Widget>>::Position;
    fn update(
        &self,
        key: &<Self::Factory as Factory<Self, Self::View>>::Key,
        widget: &Self::Widget,
    );
    fn remove(widget: &Self::Widget) -> &Self::Widget;
}

pub trait Factory<Data, View>
where
    Data: FactoryPrototype<View = View>,
    View: FactoryView<Data::Widget>,
{
    type Key;

    fn generate(&self, view: &View, sender: Sender<Data::Msg>);
}

pub trait FactoryView<Widget: WidgetExt> {
    type Position;

    fn add(&self, widget: &Widget, position: &Self::Position);
    fn remove(&self, widget: &Widget);
}
