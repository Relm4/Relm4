use tokio::sync::oneshot;

/// A type that destroys an [`AsyncComponent`](crate::async_component::AsyncComponent)
/// as soon as it is dropped.
#[derive(Debug)]
pub(super) struct ShutdownOnDrop {
    /// Sender used to indicate that the async component should shut down.
    shutdown_event_sender: Option<oneshot::Sender<()>>,
}

impl ShutdownOnDrop {
    /// Creates a new [`DestroyOnDrop`] type.
    ///
    /// When this type is dropped, a message will be sent through the channel.
    pub(crate) fn new(shutdown_event_sender: oneshot::Sender<()>) -> Self {
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
            sender.send(()).ok();
        }
    }
}
