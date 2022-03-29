// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::{Component, ComponentController, Controller, StateWatcher};
use crate::{Receiver, Sender};
use std::rc::Rc;

/// Contains the post-launch input sender and output receivers with the root widget.
///
/// The receiver can be separated from the `Fairing` by choosing a method for handling it.
#[allow(missing_debug_implementations)]
pub struct Connector<C: Component> {
    /// The models and widgets maintained by the component.
    pub(super) state: Rc<StateWatcher<C>>,

    /// The widget that this component manages.
    pub(super) widget: C::Root,

    /// Used for emitting events to the component.
    pub(super) sender: Sender<C::Input>,

    /// The outputs being received by the component.
    pub(super) receiver: Receiver<C::Output>,

    /// Allows the caller to stop the event loop remotely.
    pub(super) killswitch: flume::Sender<()>,
}

impl<C: Component> Connector<C> {
    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(C::Output) -> X) + 'static>(
        self,
        sender_: &Sender<X>,
        transform: F,
    ) -> Controller<C> {
        let Connector {
            killswitch,
            receiver,
            sender,
            state,
            widget,
        } = self;

        crate::spawn_local(receiver.forward(sender_.clone(), transform));

        Controller {
            killswitch,
            sender,
            state,
            widget,
        }
    }

    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<C::Input>, C::Output) + 'static>(
        self,
        mut func: F,
    ) -> Controller<C> {
        let Connector {
            killswitch,
            sender,
            state,
            widget,
            mut receiver,
        } = self;

        let mut sender_ = sender.clone();
        crate::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                func(&mut sender_, event);
            }
        });

        Controller {
            killswitch,
            sender,
            state,
            widget,
        }
    }

    /// Ignore outputs from the component and take the handle.
    pub fn detach(self) -> Controller<C> {
        let Self {
            killswitch,
            sender,
            state,
            widget,
            ..
        } = self;

        Controller {
            killswitch,
            sender,
            state,
            widget,
        }
    }
}

impl<C: Component> ComponentController<C> for Connector<C> {
    fn killswitch(&self) -> &flume::Sender<()> {
        &self.killswitch
    }

    fn sender(&self) -> &Sender<C::Input> {
        &self.sender
    }

    fn state(&self) -> &Rc<StateWatcher<C>> {
        &self.state
    }

    fn widget(&self) -> &C::Root {
        &self.widget
    }
}
