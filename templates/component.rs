use relm4::{Component, ComponentParts, Sender};

fn main() {
    let component = CustomModel::init(()).finalize();
}

pub struct CustomModel;

pub struct CustomWidgets;

pub enum CustomInputs {}

pub enum CustomOutputs {}

#[async_trait::async_trait]
impl Component for CustomModel {
    type Command = ();
    type InitParams = ();
    type Input = CustomInputs;
    type Output = CustomOutputs;
    type Root = gtk::Box;
    type Widgets = CustomWidgets;

    fn init_root() -> Self::Root {
        Default::default()
    }

    fn init_inner(
        params: Self::InitParams,
        root: &Self::Root,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> ComponentParts<Self, Self::Widgets> {
        ComponentParts {
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

    async fn command(message: Self::Command) -> Option<Self::Input> {
        None
    }
}