// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

#[derive(Debug)]
pub(crate) struct CompBurner {
    /// The `SourceId` of the event loop this is attached to.
    pub(crate) runtime_id: std::cell::RefCell<Option<gtk::glib::SourceId>>,

    /// The `SourceId` is sent here on drop to stop the event loop.
    pub(crate) burn_notifier: async_oneshot::Sender<gtk::glib::SourceId>,
}

impl Drop for CompBurner {
    fn drop(&mut self) {
        if let Some(id) = self.runtime_id.borrow_mut().take() {
            let _ = self.burn_notifier.send(id);
        }
    }
}
