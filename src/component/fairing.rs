// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::Controller;
use crate::{Receiver, Sender};

#[derive(Debug)]
/// Contains the post-launch input sender and output receivers with the root widget.
///
/// The receiver can be separated from the `Fairing` by choosing a method for handling it.
pub struct Fairing<W: Clone + AsRef<gtk::Widget>, I, O> {
    /// The widget that this component manages.
    pub widget: W,

    /// Used for emitting events to the component.
    pub sender: Sender<I>,

    /// The outputs being received by the component.
    pub receiver: Receiver<O>,
}

impl<W: Clone + AsRef<gtk::Widget>, I: 'static, O: 'static> Fairing<W, I, O> {
    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(O) -> X) + 'static>(
        self,
        sender_: Sender<X>,
        transform: F,
    ) -> Controller<W, I> {
        let Fairing {
            widget,
            sender,
            receiver,
        } = self;
        crate::spawn_local(crate::forward(receiver, sender_, transform));
        Controller { widget, sender }
    }

    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<I>, O) + 'static>(
        self,
        mut func: F,
    ) -> Controller<W, I> {
        let Fairing {
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

        Controller { widget, sender }
    }

    /// Ignore outputs from the component and take the handle.
    pub fn detach(self) -> Controller<W, I> {
        let Self { widget, sender, .. } = self;
        Controller { widget, sender }
    }
}
