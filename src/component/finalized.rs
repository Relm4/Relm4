// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use crate::component::Handle;
use crate::{Receiver, Sender};

#[derive(Debug)]
/// A newly-registered component which supports destructuring the handle
/// by forwarding or ignoring outputs from the component.
pub struct Finalized<W: Clone + AsRef<gtk::Widget>, I, O> {
    /// The widget that this component manages.
    pub widget: W,

    /// Used for emitting events to the component.
    pub sender: Sender<I>,

    /// The outputs being received by the component.
    pub receiver: Receiver<O>,
}

impl<W: Clone + AsRef<gtk::Widget>, I: 'static, O: 'static> Finalized<W, I, O> {
    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(O) -> X) + 'static>(
        self,
        sender_: Sender<X>,
        transform: F,
    ) -> Handle<W, I> {
        let Finalized {
            widget,
            sender,
            receiver,
        } = self;
        crate::spawn_local(crate::forward(receiver, sender_, transform));
        Handle { widget, sender }
    }

    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<I>, O) + 'static>(
        self,
        mut func: F,
    ) -> Handle<W, I> {
        let Finalized {
            widget,
            sender,
            mut receiver,
        } = self;

        let mut sender_ = sender.clone();
        crate::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                func(&mut sender_, event);
            }
        });

        Handle { widget, sender }
    }

    /// Ignore outputs from the component and take the handle.
    pub fn ignore_outputs(self) -> Handle<W, I> {
        let Self { widget, sender, .. } = self;
        Handle { widget, sender }
    }
}
