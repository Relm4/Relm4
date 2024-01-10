use relm4::prelude::*;

struct ComponentInitBadIdentifiers;

#[relm4_macros::component]
impl SimpleComponent for ComponentInitBadIdentifiers {
    type Init = ();
    type Input = ();
    type Output = ();
    type Root = (i32, i32);

    view! {
        gtk::Window {}
    }

    fn init(
        _: Self::Init,
        (a, b): Self::Root,
        _: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        todo!();
    }

    fn init_root() -> Self::Root {
        todo!();
    }
}

struct ComponentInitNoArgs;

#[relm4_macros::component]
impl SimpleComponent for ComponentInitNoArgs {
    type Init = ();
    type Input = ();
    type Output = ();
    type Root = ();

    view! {
        gtk::Window {}
    }

    fn init() -> ComponentParts<Self> {
        todo!();
    }

    fn init_root() -> Self::Root {
        todo!();
    }
}

fn main() {}
