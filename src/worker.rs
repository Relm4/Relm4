// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use crate::{Receiver, Sender};
use std::future::Future;
use std::pin::Pin;

/// A future returned by a component's command method.
pub type WorkerFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Receives inputs and outputs in the background.
pub trait Worker: Sized + Send {
    /// The initial parameters that will be used to build the worker state.
    type InputParams: 'static + Send;
    /// The type of inputs that this worker shall receive.
    type Input: 'static + Send;
    /// The typue of outputs that this worker shall send.
    type Output: 'static + Send;

    /// Defines the initial state of the worker.
    fn init_inner(
        params: Self::InputParams,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Self;

    /// Spawns the worker task in the background.
    fn init(params: Self::InputParams) -> WorkerHandle<Self::Input, Self::Output> {
        let (input_tx, mut input_rx) = crate::channel::<Self::Input>();
        let (mut output_tx, output_rx) = crate::channel::<Self::Output>();

        let worker = {
            let mut input_tx = input_tx.clone();
            crate::spawn(async move {
                let mut worker = Self::init_inner(params, &mut input_tx, &mut output_tx);

                while let Some(input) = input_rx.recv().await {
                    worker.update(input, &mut input_tx, &mut output_tx).await;
                }
            })
        };

        WorkerHandle {
            sender: input_tx,
            receiver: output_rx,
            worker,
        }
    }

    /// Defines how inputs will bep processed
    #[allow(unused)]
    fn update(
        &mut self,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> WorkerFuture {
        Box::pin(async move {})
    }
}

#[derive(Debug)]
#[must_use = "Dropping without aborting or handling the receiver causes the worker to live forever."]
/// Handle to a worker task in the background
pub struct WorkerHandle<Input, Output> {
    /// Sends inputs to the worker.
    pub sender: Sender<Input>,

    /// Where the worker will send its outputs to.
    pub receiver: Receiver<Output>,

    worker: crate::JoinHandle<()>,
}

impl<Input: 'static, Output: 'static> WorkerHandle<Input, Output> {
    /// Drops the handle and shuts down the service.
    pub fn abort(self) {
        self.worker.abort();
    }

    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<Input>, Output) + 'static>(
        self,
        mut func: F,
    ) -> WorkerController<Input> {
        let WorkerHandle {
            worker,
            sender,
            mut receiver,
        } = self;

        let mut sender_ = sender.clone();
        crate::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                func(&mut sender_, event);
            }
        });

        WorkerController { worker, sender }
    }

    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(Output) -> X) + 'static>(
        self,
        sender_: Sender<X>,
        transform: F,
    ) -> WorkerController<Input> {
        let WorkerHandle {
            sender,
            receiver,
            worker,
        } = self;

        crate::spawn_local(receiver.forward(sender_, transform));
        WorkerController { sender, worker }
    }
}

/// Sends inputs to a worker. On drop, shuts down the worker.
#[derive(Debug)]
pub struct WorkerController<Input> {
    /// Sends inputs to the worker.
    pub sender: Sender<Input>,

    worker: crate::JoinHandle<()>,
}

impl<Input> Drop for WorkerController<Input> {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
