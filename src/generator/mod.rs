
mod collections;
mod widgets;

pub use collections::VecGen;
pub use widgets::GridPosition;

pub struct GeneratorBlueprint<Data, Key, Widget, Positioning, Msg> {
    pub generate: fn(data: &Data, key: &Key, sender: glib::Sender<Msg>) -> (Widget, Positioning),
    pub update: fn(data: &Data, key: &Key, &Widget),
    pub remove: fn(&Widget) -> &Widget,
}

pub trait Generator<W, T, Widget, Positioning, Msg>
where
    W: GeneratorWidget<Widget, Positioning>,
{
    fn generate(&self, widget: &W, sender: glib::Sender<Msg>);
}

pub trait GeneratorWidget<Widget, Positioning> {
    fn add(&self, widget: &Widget, position: &Positioning);
    fn remove(&self, widget: &Widget);
}

