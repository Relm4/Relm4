use std::sync::Mutex;

use crate::{Component, Receiver, Sender};
use once_cell::sync::Lazy;
use std::fmt::Debug;

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
/// static MY_COMPONENT: MessageBroker<MyComponent> = MessageBroker::new();
///
/// // Initialize the component and the message broker with `launch_with_broker`.
/// let controller = MyComponent::builder().launch_with_broker((), &MY_COMPONENT).detach();
/// ```
pub struct MessageBroker<C: Component> {
    inner: Lazy<MessageBrokerInner<C>>,
}

impl<C: Component> MessageBroker<C> {
    /// Creates a new [`MessageBroker`].
    ///
    /// The returned message broker will not forward messages until it's initialized.
    #[must_use]
    pub const fn new() -> Self {
        let inner: Lazy<MessageBrokerInner<C>> = Lazy::new(|| MessageBrokerInner::<C>::new());
        Self { inner }
    }

    /// Get the input sender of the component.
    pub fn sender(&self) -> &Sender<C::Input> {
        &self.inner.sender
    }

    /// Send an input message to the component.
    pub fn send(&self, input: C::Input) {
        self.inner.sender.send(input).unwrap();
    }

    pub(super) fn get_channel(&self) -> (Sender<C::Input>, Option<Receiver<C::Input>>) {
        let inner = &self.inner;
        (
            inner.sender.clone(),
            inner.input_receiver.lock().unwrap().take(),
        )
    }
}

struct MessageBrokerInner<C: Component> {
    sender: Sender<C::Input>,
    input_receiver: Mutex<Option<Receiver<C::Input>>>,
}

impl<C: Component> MessageBrokerInner<C> {
    fn new() -> Self {
        // Used for all events to be processed by this component's internal service.
        let (sender, input_receiver) = crate::channel::<C::Input>();
        Self {
            sender,
            input_receiver: Mutex::new(Some(input_receiver)),
        }
    }
}

impl<C: Component> Debug for MessageBrokerInner<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBrokerInner")
            .field("sender", &self.sender)
            .finish()
    }
}
