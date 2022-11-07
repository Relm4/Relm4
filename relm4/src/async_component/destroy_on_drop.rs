use tokio::sync::oneshot;

/// A type that destroys an [`AsyncComponent`](crate::async_component::AsyncComponent)
/// as soon as it is dropped.
#[derive(Debug)]
pub(crate) struct DestroyOnDrop {
    /// Sender used to indicate that the async component should shut down.
    destroy_sender: Option<oneshot::Sender<()>>,
}

impl DestroyOnDrop {
    /// Creates a new [`DestroyOnDrop`] type.
    ///
    /// When this type is dropped, a message will be sent through the channel.
    pub(crate) fn new(destroy_sender: oneshot::Sender<()>) -> Self {
        Self {
            destroy_sender: Some(destroy_sender),
        }
    }

    pub(super) fn deactivate(&mut self) {
        self.destroy_sender = None;
    }
}

impl Drop for DestroyOnDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.destroy_sender.take() {
            sender.send(()).ok();
        }
    }
}
