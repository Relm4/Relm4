use relm4::{Component, Fuselage, Sender};

fn main() {
    let component = CustomModel::init().launch(());
}

pub struct CustomModel;

pub struct CustomWidgets;

pub enum CustomInputs {}

pub enum CustomOutputs {}

impl Component for CustomModel {
    type Command = ();
    type Payload = ();
    type Input = CustomInputs;
    type Output = CustomOutputs;
    type Root = gtk::Box;
    type Widgets = CustomWidgets;

    fn init_root() -> Self::Root {
        Default::default()
    }

    fn dock(
        params: Self::Payload,
        root: &Self::Root,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Fuselage<Self, Self::Widgets> {
        Fuselage {
            model: CustomModel {},
            widgets: CustomWidgets {},
        }
    }

    fn update(
        &mut self,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Option<Self::Command> {
        None
    }

    fn update_view(
        &mut self,
        widgets: &mut Self::Widgets,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>
    ) {

    }

    fn command(message: Self::Command, input: Sender<Self::Input>) -> ComponentFuture {
        Box::pin(async move {})
    }
}