use std::task::Poll;

use flume::r#async::RecvStream;
use futures::{future::FusedFuture, pin_mut, Future, Stream};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::mpsc;

use crate::{
    shutdown::{self, ShutdownSender},
    Receiver, Sender, ShutdownReceiver,
};

/// Stores the shutdown senders of all components ever created during
/// the runtime of the application.
static SHUTDOWN_SENDERS: Lazy<Mutex<Vec<mpsc::Sender<()>>>> = Lazy::new(Mutex::default);

/// On application shutdown, components won't trigger their shutdown
/// method automatically, so we make sure they are shutdown by sending
/// a shutdown message to all components.
pub(crate) fn shutdown_all() {
    let mut guard = SHUTDOWN_SENDERS.lock().unwrap();
    for sender in guard.drain(..) {
        sender.blocking_send(()).ok();
    }
}

/// A type that destroys an [`AsyncComponent`](crate::async_component::AsyncComponent)
/// as soon as it is dropped.
#[derive(Debug)]
pub(super) struct ShutdownOnDrop {
    /// Sender used to indicate that the async component should shut down.
    shutdown_event_sender: Option<mpsc::Sender<()>>,
}

impl ShutdownOnDrop {
    /// Creates a new [`DestroyOnDrop`] type.
    ///
    /// When this type is dropped, a message will be sent through the channel.
    pub(crate) fn new(shutdown_event_sender: mpsc::Sender<()>) -> Self {
        Self {
            shutdown_event_sender: Some(shutdown_event_sender),
        }
    }

    pub(crate) fn deactivate(&mut self) {
        self.shutdown_event_sender = None;
    }
}

impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown_event_sender.take() {
            sender.blocking_send(()).ok();
        }
    }
}

pub(super) struct RuntimeSenders<Output, Command> {
    pub(super) output_sender: Sender<Output>,
    pub(super) output_receiver: Receiver<Output>,
    pub(super) cmd_sender: Sender<Command>,
    pub(super) cmd_receiver: Receiver<Command>,
    pub(super) shutdown_notifier: ShutdownSender,
    pub(super) shutdown_recipient: ShutdownReceiver,
    pub(super) shutdown_on_drop: ShutdownOnDrop,
    pub(super) shutdown_event: ShutdownEvent,
}

impl<Output, Command> RuntimeSenders<Output, Command> {
    pub(super) fn new() -> Self {
        // Used by this component to send events to be handled externally by the caller.
        let (output_sender, output_receiver) = crate::channel::<Output>();

        // Sends messages from commands executed from the background.
        let (cmd_sender, cmd_receiver) = crate::channel::<Command>();

        // Notifies the component's child commands that it is being shut down.
        let (shutdown_notifier, shutdown_recipient) = shutdown::channel();

        // Cannel to tell the component to shutdown.
        // Use a capacity of 2 to prevent blocking when the runtime is dropped while app shutdown is running as well.
        // This rare case will emit 2 messages (but certainly not more).
        let (shutdown_event_sender, shutdown_event_receiver) = mpsc::channel(2);

        SHUTDOWN_SENDERS
            .lock()
            .unwrap()
            .push(shutdown_event_sender.clone());

        let shutdown_on_drop = ShutdownOnDrop::new(shutdown_event_sender);
        let shutdown_event = ShutdownEvent::new(shutdown_event_receiver);

        Self {
            output_sender,
            output_receiver,
            cmd_sender,
            cmd_receiver,
            shutdown_notifier,
            shutdown_recipient,
            shutdown_on_drop,
            shutdown_event,
        }
    }
}

/// A type that wraps around a shutdown event receiver.
///
/// This type is a future that will only resolve if the
/// shutdown event was transmitted successfully.
/// If the runtime is detached, the receiver will resolve
/// with an error, but this catches the error and returns
/// [`Poll::Pending`] instead so the shutdown isn't triggered.
pub(super) struct ShutdownEvent {
    shutdown_receiver: mpsc::Receiver<()>,
    detached: bool,
}

impl ShutdownEvent {
    fn new(shutdown_receiver: mpsc::Receiver<()>) -> Self {
        Self {
            shutdown_receiver,
            detached: false,
        }
    }
}

impl Future for ShutdownEvent {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if self.detached {
            Poll::Pending
        } else {
            let receiver = &mut self.shutdown_receiver;
            pin_mut!(receiver);

            match receiver.poll_recv(cx) {
                Poll::Ready(result) => {
                    if result.is_some() {
                        Poll::Ready(())
                    } else {
                        self.detached = true;
                        Poll::Pending
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

impl FusedFuture for ShutdownEvent {
    fn is_terminated(&self) -> bool {
        self.detached
    }
}

/// A type that wraps around receivers.
///
/// This type is a stream that will only yield items
/// until the sender is dropped.
pub(super) struct GuardedReceiver<'a, T>
where
    T: 'static,
{
    receive_stream: RecvStream<'a, T>,
    sender_dropped: bool,
}

impl<'a, T> GuardedReceiver<'a, T>
where
    T: 'static,
{
    pub(super) fn new(receiver: Receiver<T>) -> Self {
        Self {
            receive_stream: receiver.into_stream(),
            sender_dropped: false,
        }
    }
}

impl<'a, T> Future for GuardedReceiver<'a, T>
where
    T: 'static,
{
    type Output = T;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if self.sender_dropped {
            Poll::Pending
        } else {
            let stream = &mut self.receive_stream;
            pin_mut!(stream);

            match stream.poll_next(cx) {
                Poll::Ready(Some(value)) => Poll::Ready(value),
                Poll::Ready(None) => {
                    self.sender_dropped = true;
                    Poll::Pending
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

impl<'a, T> FusedFuture for GuardedReceiver<'a, T> {
    fn is_terminated(&self) -> bool {
        self.sender_dropped
    }
}
