use std::fmt::{self, Debug};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use futures::{pin_mut, Future, Stream};

use crate::{Component, ComponentController, Receiver, Sender};

use super::StateWatcher;

impl<C: Component> ComponentController<C> for ComponentStream<C> {
    fn sender(&self) -> &Sender<C::Input> {
        &self.sender
    }

    fn state(&self) -> &Rc<StateWatcher<C>> {
        &self.state
    }

    fn widget(&self) -> &C::Root {
        &self.widget
    }
}

/// Yields [`Component::Output`] values as a stream and contains the
/// input sender and the root widget.
///
/// Use this as alternative to [`Controller`](crate::Controller) when
/// you prefer a stream of futures or want to unlock the potential of
/// [`StreamExt`](futures::StreamExt).
pub struct ComponentStream<C: Component> {
    /// The models and widgets maintained by the component.
    pub(super) state: Rc<StateWatcher<C>>,

    /// The widget that this component manages.
    pub(super) widget: C::Root,

    /// Used for emitting events to the component.
    pub(super) sender: Sender<C::Input>,

    /// The outputs being received by the component.
    pub(super) receiver: Receiver<C::Output>,
}

impl<C: Component> Stream for ComponentStream<C> {
    type Item = C::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let future = self.receiver.recv();
        pin_mut!(future);
        future.poll(cx)
    }
}

impl<C> Debug for ComponentStream<C>
where
    C: Component + Debug,
    C::Widgets: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connector")
            .field("state", &self.state)
            .field("widget", &self.widget)
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .finish()
    }
}
