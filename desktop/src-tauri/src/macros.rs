/// Send a message to the actor, awaiting its response with a oneshot channel.
#[macro_export]
macro_rules! send_tx {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = $handle.sender.send(Message::$message { tx $(,$item)* });
    rx.await?
  }};
}

/// Send a message to the actor, awaiting to be notified by it.
#[macro_export]
macro_rules! send_notify {
  ($handle:expr, $message:ident { $($item:tt),* }) => {{
    use tokio::sync::Notify;
    use std::sync::Arc;

    let notify = Arc::new(Notify::new());
    let message = Message::$message { nt: Arc::clone(&notify) $(,$item)* };
    let _ = $handle.sender.send(message);
    notify.notified().await;
  }};
}
