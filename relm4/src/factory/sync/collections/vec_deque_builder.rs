use crate::factory::FactoryVecDeque;
use crate::prelude::{FactoryComponent, DynamicIndex};
use crate::Sender;

pub(super) struct FactoryVecDequeBuilder<C>
where
    C: FactoryComponent<Index = DynamicIndex>
{
    pub(super) root_widget: C::Root,
}

impl <C> Default for FactoryVecDequeBuilder<C>
where
    C: FactoryComponent<Index = DynamicIndex>
{
    fn default() -> Self {
        FactoryVecDequeBuilder {
            root_widget: (),
        }
    }
}

impl <C> FactoryVecDequeBuilder<C>
where
    C: FactoryComponent<Index = DynamicIndex>
{
    pub fn init(/* component_iter: impl IntoIterator<Item = C::Init> */) -> FactoryVecDequeConnector<C> {
        FactoryVecDequeConnector {
            parent_widget: (),
            parent_sender: (),
        }
    }
}


pub(super) struct FactoryVecDequeConnector<C>
where
    C: FactoryComponent<Index = DynamicIndex>
{
    pub(super) parent_widget: C::ParentWidget,
    pub(super) parent_sender: Sender<C::ParentInput>,
}

impl <C> FactoryVecDequeConnector<C>
where
    C: FactoryComponent<Index = DynamicIndex>
{
    /// Ignore output events from child components and just create the [`FactoryVecDeque`].
    pub(super) fn detach(&self) -> FactoryVecDeque<C> {
        // FIXME implement
        FactoryVecDeque::new(self.parent_widget.clone(), &self.parent_sender)
    }

    /// Forwards output events from the child components to the designated sender.
    pub(super) fn forward_from_children<X: 'static, F: (Fn(C::Output) -> X) + 'static>(
        &mut self,
        sender_: &Sender<X>,
        transform: F,
    ) -> FactoryVecDeque<C> {
        // FIXME implement
        FactoryVecDeque::new(self.parent_widget.clone(), sender_)
    }
}