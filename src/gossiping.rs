use futures_channel::mpsc::UnboundedSender;
use std::time::Duration;
use tungstenite::Message;

/// Allows endlessly to send message with sleep defining by `period` argument
pub(crate) async fn gossiping(tx: UnboundedSender<Message>, period: u32) {
    loop {
        if tx
            .unbounded_send(Message::Text(period.to_string()))
            .is_err()
        {
            println!("Unable to send message");
            return;
        };

        tokio::time::sleep(Duration::from_secs(period.into())).await;
    }
}
