use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

use flume::r#async::RecvStream;
use futures::{pin_mut, Stream};

use crate::Component;

/// Yields [`Component::Output`] values as a stream and contains the
/// input sender and the root widget.
///
/// Use this as alternative to [`Controller`](crate::Controller) when
/// you prefer a stream of futures or want to unlock the potential of
/// [`StreamExt`](futures::StreamExt).
/// Also, this type implements [`Send`] so using it in commands is
/// possible.
pub struct ComponentStream<C: Component> {
    /// The outputs being received by the component.
    pub(super) stream: RecvStream<'static, C::Output>,
}

impl<C: Component> Stream for ComponentStream<C> {
    type Item = C::Output;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = &mut self.stream;
        pin_mut!(stream);
        stream.poll_next(cx)
    }
}

impl<C: Component> Debug for ComponentStream<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentStream")
            .field("stream", &"<RecvStream>")
            .finish()
    }
}
