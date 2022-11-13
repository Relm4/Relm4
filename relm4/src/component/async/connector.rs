// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::{AsyncComponent, AsyncComponentController, AsyncController};
use crate::component::ShutdownOnDrop;
use crate::{Receiver, Sender};
use std::fmt::{self, Debug};

/// Contains the post-launch input sender and output receivers with the root widget.
///
/// The receiver can be separated from the `Fairing` by choosing a method for handling it.
pub struct AsyncConnector<C: AsyncComponent> {
    /// The widget that this component manages.
    pub(super) widget: C::Root,

    /// Used for emitting events to the component.
    pub(super) sender: Sender<C::Input>,

    /// The outputs being received by the component.
    pub(super) receiver: Receiver<C::Output>,

    /// Type used to destroy the async component when it's dropped.
    pub(super) shutdown_on_drop: ShutdownOnDrop,
}

impl<C: AsyncComponent> AsyncConnector<C> {
    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(C::Output) -> X) + 'static>(
        self,
        sender_: &Sender<X>,
        transform: F,
    ) -> AsyncController<C> {
        let Self {
            widget,
            sender,
            receiver,
            shutdown_on_drop,
        } = self;

        crate::spawn_local(receiver.forward(sender_.clone(), transform));

        AsyncController {
            widget,
            sender,
            shutdown_on_drop,
        }
    }

    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<C::Input>, C::Output) + 'static>(
        self,
        mut func: F,
    ) -> AsyncController<C> {
        let Self {
            widget,
            sender,
            receiver,
            shutdown_on_drop,
        } = self;

        let mut sender_ = sender.clone();
        crate::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                func(&mut sender_, event);
            }
        });

        AsyncController {
            widget,
            sender,
            shutdown_on_drop,
        }
    }

    /// Ignore outputs from the component and take the handle.
    pub fn detach(self) -> AsyncController<C> {
        let Self {
            widget,
            sender,
            shutdown_on_drop,
            ..
        } = self;

        AsyncController {
            widget,
            sender,
            shutdown_on_drop,
        }
    }
}

impl<C: AsyncComponent> AsyncComponentController<C> for AsyncConnector<C> {
    fn sender(&self) -> &Sender<C::Input> {
        &self.sender
    }

    fn widget(&self) -> &C::Root {
        &self.widget
    }

    fn detach_runtime(&mut self) {
        self.shutdown_on_drop.deactivate();
    }
}

impl<C> Debug for AsyncConnector<C>
where
    C: AsyncComponent + Debug,
    C::Widgets: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connector")
            .field("widget", &self.widget)
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .field("shutdown_on_drop", &self.shutdown_on_drop)
            .finish()
    }
}
