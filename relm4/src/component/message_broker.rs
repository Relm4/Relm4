use std::sync::Mutex;

use crate::{Receiver, Sender};
use std::fmt::Debug;
use std::sync::LazyLock;

#[derive(Debug)]
/// A type that can be used in static variables to pass messages to components.
///
/// The primary use-case for this type is to communicate between components on different levels.
///
/// Imagine you have three components: A, B and C.
/// A and B are children of the main application, but C is a child of B.
/// If C wants to pass a message to A, it relies on B to forward that message to A.
/// This is not great because B has nothing to do this message but has to implement additional
/// logic only to pass the message through.
/// [`MessageBroker`] allows you to use statics to remove this limitation.
///
/// # Note
///
/// [`MessageBroker`] will not forward any messages until you initialize them with
/// [`ComponentBuilder::launch_with_broker()`](crate::ComponentBuilder::launch_with_broker()).
///
/// **Only initialize the message broker once!**
///
/// ```
/// use relm4::{MessageBroker, Component};
/// # type MyComponent = ();
///
/// static MY_COMPONENT: MessageBroker<()> = MessageBroker::new();
///
/// // Initialize the component and the message broker with `launch_with_broker`.
/// let controller = MyComponent::builder().launch_with_broker((), &MY_COMPONENT).detach();
/// ```
pub struct MessageBroker<M: Debug> {
    inner: LazyLock<MessageBrokerInner<M>>,
}

impl<M: Debug> Default for MessageBroker<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Debug> MessageBroker<M> {
    /// Creates a new [`MessageBroker`].
    ///
    /// The returned message broker will not forward messages until it's initialized.
    #[must_use]
    pub const fn new() -> Self {
        let inner: LazyLock<MessageBrokerInner<M>> = LazyLock::new(MessageBrokerInner::<M>::new);
        Self { inner }
    }

    /// Get the input sender of the component.
    pub fn sender(&self) -> &Sender<M> {
        &self.inner.sender
    }

    /// Send an input message to the component.
    pub fn send(&self, input: M) {
        self.inner.sender.send(input).unwrap();
    }

    pub(super) fn get_channel(&self) -> (Sender<M>, Option<Receiver<M>>) {
        let inner = &self.inner;
        (
            inner.sender.clone(),
            inner.input_receiver.lock().unwrap().take(),
        )
    }
}

struct MessageBrokerInner<M> {
    sender: Sender<M>,
    input_receiver: Mutex<Option<Receiver<M>>>,
}

impl<M> MessageBrokerInner<M> {
    fn new() -> Self {
        // Used for all events to be processed by this component's internal service.
        let (sender, input_receiver) = crate::channel::<M>();
        Self {
            sender,
            input_receiver: Mutex::new(Some(input_receiver)),
        }
    }
}

impl<M> Debug for MessageBrokerInner<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBrokerInner")
            .field("sender", &self.sender)
            .finish()
    }
}
