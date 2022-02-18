// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use tokio::sync::mpsc;

pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (Sender(tx), Receiver(rx))
}

/// A Relm4 sender sends messages to a component or worker.
#[derive(Debug)]
pub struct Sender<T>(pub(crate) mpsc::UnboundedSender<T>);

impl<T> From<mpsc::UnboundedSender<T>> for Sender<T> {
    fn from(tokio: mpsc::UnboundedSender<T>) -> Self {
        Self(tokio)
    }
}

impl<T> Sender<T> {
    /// Sends messages to a component or worker.
    pub fn send(&self, message: T) {
        if self.0.send(message).is_err() {
            panic!("receiver was dropped");
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// A Relm4 receiver receives messages from a component or worker.
#[derive(Debug)]
pub struct Receiver<T>(pub(crate) mpsc::UnboundedReceiver<T>);

impl<T> Receiver<T> {
    /// Receives a message from a component or worker.
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }

    /// Forwards an event from one channel to another.
    pub async fn forward<Transformer, Output>(
        mut self,
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
