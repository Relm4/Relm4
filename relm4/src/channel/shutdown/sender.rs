// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::broadcast;

/// Sends shutdown signals to receivers.
#[derive(Debug)]
pub struct ShutdownSender {
    pub(super) sender: broadcast::Sender<()>,
}

impl ShutdownSender {
    /// Broadcasts a shutdown signal to listening receivers.
    pub(crate) fn shutdown(self) {
        drop(self);
    }
}

impl Drop for ShutdownSender {
    fn drop(&mut self) {
        let _ = self.sender.send(());
    }
}
