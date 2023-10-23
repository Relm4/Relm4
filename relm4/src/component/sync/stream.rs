use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

use flume::r#async::RecvStream;
use futures::{pin_mut, Stream, StreamExt};

use crate::{Component, ShutdownOnDrop};

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
    pub(super) shutdown_on_drop: ShutdownOnDrop,
}

impl<C: Component> Stream for ComponentStream<C> {
    type Item = C::Output;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = &mut self.stream;
        pin_mut!(stream);
        stream.poll_next(cx)
    }
}

impl<C: Component> ComponentStream<C> {
    /// Receive one message and drop the component afterwards.
    /// This can be used for dialogs.
    pub async fn recv_one(mut self) -> Option<C::Output> {
        self.stream.next().await
    }
}

impl<C: Component> ComponentStream<C> {
    /// Dropping this type will usually stop the runtime of the component.
    /// With this method you can give the runtime a static lifetime.
    /// In other words, dropping the stream will not stop
    /// the runtime anymore, instead it will run until the app is closed.
    pub fn detach_runtime(&mut self) {
        self.shutdown_on_drop.deactivate();
    }
}

impl<C: Component> Debug for ComponentStream<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentStream")
            .field("stream", &"<RecvStream>")
            .finish()
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::{Component, ComponentParts, SimpleComponent};

    fn assert_send<T: Send>(_stream: T) {}

    struct Test(Rc<()>);

    impl SimpleComponent for Test {
        type Input = ();
        type Output = ();
        type Init = ();
        type Root = Rc<()>;
        type Widgets = Rc<()>;

        fn init_root() -> Self::Root {
            Rc::default()
        }

        fn init(
            _init: Self::Init,
            _root: Self::Root,
            _sender: crate::ComponentSender<Self>,
        ) -> ComponentParts<Self> {
            ComponentParts {
                model: Test(Rc::default()),
                widgets: Rc::default(),
            }
        }
    }

    #[gtk::test]
    fn stream_is_send() {
        let stream = Test::builder().launch(()).into_stream();
        assert_send(stream);
    }
}
