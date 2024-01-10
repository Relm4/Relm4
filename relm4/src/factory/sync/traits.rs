//! Traits for for managing and updating factories.

use crate::factory::{FactorySender, FactoryView, Position};
use crate::Sender;

use std::fmt::Debug;

/// A component that's stored inside a factory.
/// Similar to [`Component`](crate::Component) but adjusted to fit the life cycle
/// of factories.
pub trait FactoryComponent:
    Position<<Self::ParentWidget as FactoryView>::Position, Self::Index> + Sized + 'static
{
    /// Container widget to which all widgets of the factory will be added.
    type ParentWidget: FactoryView + 'static;

    /// Messages which are received from commands executing in the background.
    type CommandOutput: Debug + Send + 'static;

    /// The message type that the factory component accepts as inputs.
    type Input: Debug + 'static;

    /// The message type that the factory component provides as outputs.
    type Output: Debug + 'static;

    /// The parameter used to initialize the factory component.
    type Init;

    /// The top-level widget of the factory component.
    type Root: AsRef<<Self::ParentWidget as FactoryView>::Children> + Debug + Clone;

    /// The type that's used for storing widgets created for this factory component.
    type Widgets: 'static;

    /// The type that's used by a factory collection as index.
    ///
    /// For example, for [`FactoryVecDeque`](crate::factory::FactoryVecDeque), this type
    /// is [`DynamicIndex`](crate::factory::DynamicIndex).
    /// For [`FactoryHashMap`](crate::factory::FactoryHashMap), this type is equal to the key
    /// you use for inserting values.
    type Index;

    /// Initializes the model.
    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self;

    /// Initializes the root widget
    fn init_root(&self) -> Self::Root;

    /// Initializes the widgets.
    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: Self::Root,
        returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {}

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(&mut self, message: Self::CommandOutput, sender: FactorySender<Self>) {}

    /// Handles updates from a command.
    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: FactorySender<Self>,
    ) {
        self.update_cmd(message, sender.clone());
        self.update_view(widgets, sender);
    }

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: FactorySender<Self>) {}

    /// Updates the model and view. Optionally returns a command to run.
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        self.update(message, sender.clone());
        self.update_view(widgets, sender);
    }

    /// Last method called before a component is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}

    /// An identifier for the component used for debug logging.
    ///
    /// The default implementation of this method uses the address of the component, but
    /// implementations are free to provide more meaningful identifiers.
    fn id(&self) -> String {
        format!("{:p}", &self)
    }
}

/// Extension for [`FactoryComponent`] that makes elements cloneable.
pub trait CloneableFactoryComponent: FactoryComponent {
    /// Retrieve the initialization data from an initialized factory component.
    /// This is necessary for cloning the factory.
    fn get_init(&self) -> Self::Init;
}
