// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use std::fmt;

use flume::r#async::RecvStream;

/// Create an unbounded channel to send messages
/// between different parts of you application.
#[must_use]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::unbounded();
    (Sender(tx), Receiver(rx))
}

/// A Relm4 sender sends messages to a component or worker.
pub struct Sender<T>(pub(crate) flume::Sender<T>);

impl<T> From<flume::Sender<T>> for Sender<T> {
    fn from(sender: flume::Sender<T>) -> Self {
        Self(sender)
    }
}

impl<T> Sender<T> {
    /// Sends a message to a component or worker.
    pub fn send(&self, message: T) {
        assert!(self.0.send(message).is_ok(), "Receiver was dropped");
    }

    /// Tries to send a message to a component or worker.
    ///
    /// If all receivers where dropped, [`None`] is returned.
    pub fn try_send(&self, message: T) -> Option<()> {
        self.0.send(message).ok()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Sender").field(&self.0).finish()
    }
}

/// A Relm4 receiver receives messages from a component or worker.
pub struct Receiver<T>(pub(crate) flume::Receiver<T>);

impl<T> Receiver<T> {
    /// Receives a message from a component or worker.
    pub async fn recv(&self) -> Option<T> {
        self.0.recv_async().await.ok()
    }

    /// Receives a message synchronously from a component or worker.
    #[must_use]
    pub fn recv_sync(&self) -> Option<T> {
        self.0.recv().ok()
    }

    pub(crate) fn into_stream(self) -> RecvStream<'static, T> {
        self.0.into_stream()
    }

    /// Forwards an event from one channel to another.
    pub async fn forward<Transformer, Output>(
        self,
        sender: impl Into<Sender<Output>>,
        transformer: Transformer,
    ) where
        Transformer: (Fn(T) -> Output) + 'static,
        Output: 'static,
    {
        let sender = sender.into();
        while let Some(event) = self.recv().await {
            if sender.0.send(transformer(event)).is_err() {
                break;
            }
        }
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Receiver").field(&self.0).finish()
    }
}
