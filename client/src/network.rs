//! WebSocket networking for the game client
//!
//! Handles connection to the game server and message routing.

use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

use thermite_server::protocol::{ClientMessage, ServerMessage};

/// Connect to the game server and return channels for communication
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

    // Channels for communication with the main thread
    let (send_tx, mut send_rx) = mpsc::unbounded_channel::<ClientMessage>();
    let (recv_tx, recv_rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Spawn task to handle outgoing messages
    let send_tx_clone = send_tx.clone();
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

    // Spawn task to handle incoming messages
    tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(Message::Binary(data)) => {
                    match ServerMessage::from_msgpack(&data) {
                        Ok(msg) => {
                            if let Err(e) = recv_tx.send(msg) {
                                error!("Failed to forward message: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {}", e);
                        }
                    }
                }
                Ok(Message::Text(text)) => {
                    // Also support JSON for debugging
                    match ServerMessage::from_json(&text) {
                        Ok(msg) => {
                            if let Err(e) = recv_tx.send(msg) {
                                error!("Failed to forward message: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse JSON message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Connection closed by server");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    // Pong is handled automatically by tungstenite
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

/// Attempt to connect with retries
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
                warn!(
                    "Connection failed (attempt {}/{}): {}. Retrying in {}ms...",
                    attempts, max_retries, e, retry_delay_ms
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms)).await;
            }
        }
    }
}
