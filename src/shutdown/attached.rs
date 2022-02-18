use super::ShutdownReceiver;
use futures::future::Either;
use std::future::Future;

/// A future attached to a shutdown receiver.
#[derive(Debug)]
pub struct AttachedShutdown<F> {
    pub(super) receiver: ShutdownReceiver,
    pub(super) future: F,
}

impl<F, Out> AttachedShutdown<F>
where
    F: Future<Output = Out>,
{
    /// Creates a future which will resolve to this on shutdown.
    pub async fn on_shutdown<S>(self, shutdown: S) -> Out
    where
        S: Future<Output = Out>,
    {
        match self.wait().await {
            Either::Left(_) => shutdown.await,
            Either::Right(out) => out,
        }
    }

    /// Waits until a shutdown signal is received.
    ///
    /// - `Either::Left(())` on cancelation.
    /// - `Either::Right(Out)` on registered future completion.
    pub async fn wait(self) -> Either<(), Out> {
        let Self { receiver, future } = self;

        let cancel = receiver.wait();
        futures::pin_mut!(cancel);
        futures::pin_mut!(future);

        match futures::future::select(cancel, future).await {
            Either::Left(_) => Either::Left(()),
            Either::Right((out, _)) => Either::Right(out),
        }
    }
}
