// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod builder;
mod connector;
mod controller;
mod message_broker;
mod state_watcher;
mod traits;

#[allow(unreachable_pub)]
pub use self::builder::ComponentBuilder;
#[allow(unreachable_pub)]
pub use self::connector::Connector;
#[allow(unreachable_pub)]
pub use self::controller::{ComponentController, Controller};
#[allow(unreachable_pub)]
pub use self::message_broker::MessageBroker;
#[allow(unreachable_pub)]
pub use self::state_watcher::StateWatcher;
#[allow(unreachable_pub)]
pub use self::traits::Component;
#[allow(unreachable_pub)]
pub use self::traits::SimpleComponent;

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;

/// A future returned by a component's command method.
pub type CommandFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Contains the initial model and widgets being docked into a component.
#[derive(Debug)]
pub struct ComponentParts<C: Component> {
    /// The model of the component.
    pub model: C,
    /// The widgets created for the view.
    pub widgets: C::Widgets,
}

/// Type which supports signaling when it has been destroyed.
pub trait OnDestroy {
    /// Runs the given function when destroyed.
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F);
}

impl<T: AsRef<gtk::glib::Object>> OnDestroy for T {
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F) {
        use gtk::prelude::ObjectExt;
        let func = std::cell::RefCell::new(Some(func));
        self.as_ref().add_weak_ref_notify_local(move || {
            if let Some(func) = func.take() {
                func();
            }
        });
    }
}

/// An empty root type.
/// Construct it using [`Default`].
///
/// Use this type if you have a component that does no
/// root (e.g. a wrapper around dialogs).
///
/// Note: When this type is dropped, the component will
/// be dropped as well. The root is usually stored in
/// a [`Controller`] so you don't have to keep it alive
/// yourself.
#[derive(Default)]
pub struct EmptyRoot {
    func: RefCell<Option<Box<dyn FnOnce()>>>,
}

impl std::fmt::Debug for EmptyRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmptyRoot")
            .field(
                "func",
                &self
                    .func
                    .try_borrow()
                    .map(|opt| opt.as_ref().map(|_| "Destroy function")),
            )
            .finish()
    }
}

impl OnDestroy for EmptyRoot {
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F) {
        *self.func.borrow_mut() = Some(Box::new(func));
    }
}

impl Drop for EmptyRoot {
    fn drop(&mut self) {
        if let Some(func) = self.func.borrow_mut().take() {
            // Call OnDestroy function
            func();
        }
    }
}
