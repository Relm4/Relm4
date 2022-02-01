// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod bridge;
mod controller;
mod elm_like;
mod fairing;
mod state_watcher;
mod stateful;

#[allow(unreachable_pub)]
pub use self::bridge::Bridge;
#[allow(unreachable_pub)]
pub use self::controller::{ComponentController, Controller};
#[allow(unreachable_pub)]
pub use self::elm_like::Component;
#[allow(unreachable_pub)]
pub use self::elm_like::SimpleComponent;
#[allow(unreachable_pub)]
pub use self::fairing::Fairing;
#[allow(unreachable_pub)]
pub use self::state_watcher::StateWatcher;
#[allow(unreachable_pub)]
pub use self::stateful::StatefulComponent;

use std::future::Future;
use std::pin::Pin;

/// A future returned by a component's command method.
pub type CommandFuture<T> = Pin<Box<dyn Future<Output = Option<T>> + Send>>;

/// Contains the initial model and widgets being docked into a component.
#[derive(Debug)]
pub struct Fuselage<Model, Widgets> {
    /// The model of the component.
    pub model: Model,
    /// The widgets created for the view.
    pub widgets: Widgets,
}

/// Type which supports signaling when it has been destroyed.
pub trait OnDestroy {
    /// Runs the given function when destroyed.
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F);
}

impl<T: AsRef<gtk::Widget>> OnDestroy for T {
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F) {
        use gtk::prelude::WidgetExt;
        let func = std::cell::RefCell::new(Some(func));
        self.as_ref().connect_destroy(move |_| {
            if let Some(func) = func.take() {
                func();
            }
        });
    }
}
