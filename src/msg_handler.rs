use gtk::glib::Sender;

use std::marker::PhantomData;

use crate::{MessageHandler, Model};

/// [`RelmMsgHandler`]s are usually used to run expansive tasks on different threads and report back when they are finished
/// so that their parent components can keep handling UI events in the meantime.
/// For example you could use a [`RelmMsgHandler`] for sending a HTTP request or for copying files.
///
/// Multiple [`RelmMsgHandler`]s that have the same parent are usually bundled along with [`RelmComponent`](crate::RelmComponent)s
/// and [`RelmWorker`](crate::RelmWorker)s in a struct that implements [`Components`](crate::Components).
#[derive(Clone, Debug)]
pub struct RelmMsgHandler<Data, ParentModel>
where
    Data: MessageHandler<ParentModel> + 'static,
    ParentModel: Model,
{
    data: Data,
    parent_model: PhantomData<ParentModel>,
}

impl<Data, ParentModel> RelmMsgHandler<Data, ParentModel>
where
    Data: MessageHandler<ParentModel> + 'static,
    ParentModel: Model,
{
    /// Create a new [`RelmMsgHandler`].
    pub fn new(parent_model: &ParentModel, parent_sender: Sender<ParentModel::Msg>) -> Self {
        let data = Data::init(parent_model, parent_sender);

        RelmMsgHandler {
            data,
            parent_model: PhantomData,
        }
    }

    /// Send a message to this message handler.
    /// This can be used by the parent to send messages to this message handler.
    pub fn send(&self, msg: Data::Msg) {
        self.data.send(msg)
    }

    /// Get a sender to send messages to this message handler.
    pub fn sender(&self) -> Data::Sender {
        self.data.sender()
    }
}
