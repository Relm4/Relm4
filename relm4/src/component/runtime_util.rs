use std::task::Poll;

use flume::r#async::RecvStream;
use futures::{future::FusedFuture, pin_mut, Future, Stream};
use tokio::sync::oneshot;

use crate::{
    shutdown::{self, ShutdownSender},
    Receiver, Sender, ShutdownReceiver,
};

use super::shutdown_on_drop::ShutdownOnDrop;

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
        let (shutdown_event_sender, shutdown_event_receiver) = oneshot::channel::<()>();

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
    shutdown_receiver: oneshot::Receiver<()>,
    detached: bool,
}

impl ShutdownEvent {
    fn new(shutdown_receiver: oneshot::Receiver<()>) -> Self {
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

            match receiver.poll(cx) {
                Poll::Ready(result) => {
                    if result.is_ok() {
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
