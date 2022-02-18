mod attached;
mod receiver;
mod sender;

pub use self::attached::AttachedShutdown;
pub use self::receiver::ShutdownReceiver;
pub use self::sender::ShutdownSender;

use tokio::sync::broadcast;

/// Creates a broadcasting shutdown channel.
///
/// The sending side is responsible for initiating a shutdown.
/// The receiving side is responsible for responding to shutdowns.
pub fn channel() -> (ShutdownSender, ShutdownReceiver) {
    let (sender, receiver) = broadcast::channel(1);
    (
        ShutdownSender {
            sender: sender.clone(),
        },
        ShutdownReceiver { sender, receiver },
    )
}
