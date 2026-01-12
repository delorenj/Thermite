use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection,
    ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum MatchEvent {
    MatchStarted {
        match_id: Uuid,
        player_count: usize,
        map_name: String,
        timestamp: i64,
    },
    MatchEnded {
        match_id: Uuid,
        duration_ms: u64,
        survivors: Vec<Uuid>,
        timestamp: i64,
    },
}

pub struct RabbitMQPublisher {
    channel: Arc<Mutex<Channel>>,
    exchange: String,
}

impl RabbitMQPublisher {
    pub async fn connect(url: &str, exchange: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;

        // Declare exchange (topic exchange for routing)
        channel
            .exchange_declare(
                exchange,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            channel: Arc::new(Mutex::new(channel)),
            exchange: exchange.to_string(),
        })
    }

    pub async fn publish_event(&self, event: &MatchEvent) -> Result<(), Box<dyn std::error::Error>> {
        let routing_key = match event {
            MatchEvent::MatchStarted { .. } => "match.started",
            MatchEvent::MatchEnded { .. } => "match.ended",
        };

        let payload = serde_json::to_vec(event)?;
        let channel = self.channel.lock().await;

        channel
            .basic_publish(
                &self.exchange,
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2), // persistent
            )
            .await?;

        Ok(())
    }
}
