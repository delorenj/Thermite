//! WebSocket networking for the game client.
//! Native implementation today; wasm implementation is scaffolded for Day-1 split.

use thermite_protocol::protocol::{ClientMessage, ServerMessage};

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio::sync::mpsc;
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tracing::{error, info, warn};

    pub async fn connect(
        server_url: &str,
    ) -> Result<
        (
            mpsc::UnboundedSender<ClientMessage>,
            mpsc::UnboundedReceiver<ServerMessage>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        info!("Connecting to {}", server_url);

        let (ws_stream, _) = connect_async(server_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let (send_tx, mut send_rx) = mpsc::unbounded_channel::<ClientMessage>();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel::<ServerMessage>();

        tokio::spawn(async move {
            while let Some(msg) = send_rx.recv().await {
                match msg.to_msgpack() {
                    Ok(bytes) => {
                        if let Err(e) = ws_sender.send(Message::Binary(bytes.into())).await {
                            error!("Failed to send message: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Some(result) = ws_receiver.next().await {
                match result {
                    Ok(Message::Binary(data)) => match ServerMessage::from_msgpack(&data) {
                        Ok(msg) => {
                            if let Err(e) = recv_tx.send(msg) {
                                error!("Failed to forward message: {}", e);
                                break;
                            }
                        }
                        Err(e) => warn!("Failed to parse message: {}", e),
                    },
                    Ok(Message::Text(text)) => match ServerMessage::from_json(&text) {
                        Ok(msg) => {
                            if let Err(e) = recv_tx.send(msg) {
                                error!("Failed to forward message: {}", e);
                                break;
                            }
                        }
                        Err(e) => warn!("Failed to parse JSON message: {}", e),
                    },
                    Ok(Message::Close(_)) => {
                        info!("Connection closed by server");
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        });

        info!("Connected to server");
        Ok((send_tx, recv_rx))
    }

    pub async fn connect_with_retry(
        server_url: &str,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Result<
        (
            mpsc::UnboundedSender<ClientMessage>,
            mpsc::UnboundedReceiver<ServerMessage>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let mut attempts = 0;

        loop {
            match connect(server_url).await {
                Ok(channels) => return Ok(channels),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_retries {
                        return Err(e);
                    }
                    tracing::warn!(
                        "Connection failed (attempt {}/{}): {}. Retrying in {}ms...",
                        attempts,
                        max_retries,
                        e,
                        retry_delay_ms
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms)).await;
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;

    pub async fn connect(
        _server_url: &str,
    ) -> Result<
        (
            crossbeam_channel::Sender<ClientMessage>,
            crossbeam_channel::Receiver<ServerMessage>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Err("wasm networking path not implemented yet; use puzzle/local mode".into())
    }

    pub async fn connect_with_retry(
        _server_url: &str,
        _max_retries: u32,
        _retry_delay_ms: u64,
    ) -> Result<
        (
            crossbeam_channel::Sender<ClientMessage>,
            crossbeam_channel::Receiver<ServerMessage>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Err("wasm networking path not implemented yet; use puzzle/local mode".into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{connect, connect_with_retry};

#[cfg(target_arch = "wasm32")]
pub use wasm::{connect, connect_with_retry};
