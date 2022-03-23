use futures_channel::mpsc::UnboundedSender;
use std::time::Duration;
use tungstenite::Message;

use crate::nothing_to_do;

pub(crate) async fn gossiping(tx: UnboundedSender<Message>, period: u32) {
    loop {
        match tx.unbounded_send(Message::Text(period.to_string())) {
            Ok(_) => nothing_to_do(),
            Err(_) => {
                println!("Unable to send message");
                return;
            }
        }

        tokio::time::sleep(Duration::from_secs(period.into())).await;
    }
}
