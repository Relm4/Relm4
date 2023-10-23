use relm4::prelude::*;

struct App;

#[relm4_macros::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();
    
    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        todo!();
    }
}

fn main() {

}
