#[derive(Debug, Default)]
/// An abstraction over [`adw::ToastOverlay`] that
/// makes it easy to store it in the model of components.
///
/// The only allowed action is to add toasts, effectively
/// keeping the separation between UI and application state.
pub struct Toaster {
    overlay: adw::ToastOverlay,
}

impl Toaster {
    /// Create a new [`Toaster`] with a given overlay.
    #[must_use]
    pub fn new(overlay: &adw::ToastOverlay) -> Self {
        Self {
            overlay: overlay.clone(),
        }
    }

    /// The [`adw::ToastOverlay`] used internally.
    #[must_use]
    pub fn overlay_widget(&self) -> &adw::ToastOverlay {
        &self.overlay
    }

    /// Create a simple [`adw::Toast`] that only contains
    /// a text message.
    pub fn toast(&self, title: &str) {
        let toast = adw::Toast::new(title);
        self.overlay.add_toast(toast);
    }

    /// Add a [`adw::Toast`] to the overlay.
    pub fn add_toast(&self, toast: adw::Toast) {
        self.overlay.add_toast(toast);
    }
}
