// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use crate::{Receiver, Sender};
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};

#[async_trait::async_trait]
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
        let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Self::Input>();
        let (mut output_tx, output_rx) = mpsc::unbounded_channel::<Self::Output>();

        // This is used to drop the worker when the handle to the worker has been dropped.
        let drop_notifier = Arc::new(Notify::new());

        {
            // Future which awaits a drop notice.
            let drop_notifier = drop_notifier.clone();
            let drop_notice = async move {
                drop_notifier.notified().await;
            };

            // Future which handles inputs.
            let mut input_tx = input_tx.clone();
            let worker = async move {
                let mut worker = Self::init_inner(params, &mut input_tx, &mut output_tx);

                while let Some(input) = input_rx.recv().await {
                    worker.update(input, &mut input_tx, &mut output_tx).await;
                }
            };

            tokio::spawn(async move {
                futures::pin_mut!(drop_notice);
                futures::pin_mut!(worker);
                futures::future::select(drop_notice, worker).await;
            });
        }

        WorkerHandle {
            sender: input_tx,
            receiver: output_rx,
            drop_notifier,
        }
    }

    /// Defines how inputs will bep processed
    #[allow(unused)]
    async fn update(
        &mut self,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) {
    }
}

#[derive(Debug)]
/// Handle to a worker task in the background
pub struct WorkerHandle<Input, Output> {
    /// Sends inputs to the worker.
    pub sender: Sender<Input>,
    /// Where the worker will send its outputs to.
    pub receiver: Receiver<Output>,

    drop_notifier: Arc<Notify>,
}

impl<Input, Output> Drop for WorkerHandle<Input, Output> {
    fn drop(&mut self) {
        self.drop_notifier.notify_one();
    }
}
