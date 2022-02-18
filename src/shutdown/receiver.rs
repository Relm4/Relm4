// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::AttachedShutdown;
use tokio::sync::broadcast::{Receiver, Sender};

/// Listens to shutdown signals and constructs shutdown futures.
#[derive(Debug)]
pub struct ShutdownReceiver {
    pub(super) sender: Sender<()>,
    pub(super) receiver: Receiver<()>,
}

impl ShutdownReceiver {
    /// Create a future which will be cancelled on shutdown.
    pub fn register<F>(self, future: F) -> AttachedShutdown<F> {
        AttachedShutdown {
            receiver: self,
            future,
        }
    }

    /// Waits until a shutdown signal is received.
    pub async fn wait(mut self) {
        let _ = self.receiver.recv().await;
    }
}

impl Clone for ShutdownReceiver {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.sender.subscribe(),
        }
    }
}
