//! WebSocket protocol messages for client-server communication
//!
//! Uses MessagePack for efficient binary serialization.
//! All messages include sequence numbers for client-side prediction reconciliation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::{Direction, Position};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Player movement command
    Move {
        direction: Direction,
        sequence: u64,
    },
    /// Place a bomb at current position
    PlaceBomb { sequence: u64 },
    /// Request extraction at current position
    Extract { sequence: u64 },
    /// Ping for latency measurement
    Ping { timestamp: u64 },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Welcome message on connection
    Welcome {
        player_id: Uuid,
        tick_rate_ms: u32,
    },
    /// Full game state update (broadcast at 20Hz)
    StateUpdate {
        tick: u64,
        players: Vec<PlayerState>,
        bombs: Vec<BombState>,
        time_remaining_ms: u64,
    },
    /// Acknowledgment of client command (for prediction reconciliation)
    CommandAck {
        sequence: u64,
        success: bool,
        position: Option<Position>,
        error: Option<String>,
    },
    /// Player died notification
    PlayerDied {
        player_id: Uuid,
        killer_id: Option<Uuid>,
        position: Position,
    },
    /// Match ended
    MatchEnded { reason: MatchEndReason },
    /// Pong response
    Pong { timestamp: u64 },
    /// Error message
    Error { message: String },
}

/// Player state for state updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: Uuid,
    pub position: Position,
    pub health: i32,
    pub is_alive: bool,
}

/// Bomb state for state updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BombState {
    pub id: Uuid,
    pub position: Position,
    pub owner_id: Uuid,
    pub timer_ms: u32,
}

/// Reason for match ending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchEndReason {
    TimerExpired,
    AllPlayersExtracted,
    AllPlayersDead,
    ServerShutdown,
}

impl ClientMessage {
    /// Serialize to MessagePack bytes
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    /// Deserialize from MessagePack bytes
    pub fn from_msgpack(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }

    /// Serialize to JSON (for debugging/testing)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl ServerMessage {
    /// Serialize to MessagePack bytes
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    /// Deserialize from MessagePack bytes
    pub fn from_msgpack(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }

    /// Serialize to JSON (for debugging/testing)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_move_json() {
        let msg = ClientMessage::Move {
            direction: Direction::North,
            sequence: 42,
        };

        let json = msg.to_json().expect("Failed to serialize");
        let deserialized = ClientMessage::from_json(&json).expect("Failed to deserialize");

        match deserialized {
            ClientMessage::Move {
                direction,
                sequence,
            } => {
                assert_eq!(direction, Direction::North);
                assert_eq!(sequence, 42);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_client_message_move_msgpack() {
        let msg = ClientMessage::Move {
            direction: Direction::South,
            sequence: 100,
        };

        let bytes = msg.to_msgpack().expect("Failed to serialize");
        let deserialized = ClientMessage::from_msgpack(&bytes).expect("Failed to deserialize");

        match deserialized {
            ClientMessage::Move {
                direction,
                sequence,
            } => {
                assert_eq!(direction, Direction::South);
                assert_eq!(sequence, 100);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_welcome() {
        let player_id = Uuid::new_v4();
        let msg = ServerMessage::Welcome {
            player_id,
            tick_rate_ms: 50,
        };

        let bytes = msg.to_msgpack().expect("Failed to serialize");
        let deserialized = ServerMessage::from_msgpack(&bytes).expect("Failed to deserialize");

        match deserialized {
            ServerMessage::Welcome {
                player_id: id,
                tick_rate_ms,
            } => {
                assert_eq!(id, player_id);
                assert_eq!(tick_rate_ms, 50);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_state_update() {
        let player_id = Uuid::new_v4();
        let msg = ServerMessage::StateUpdate {
            tick: 1234,
            players: vec![PlayerState {
                id: player_id,
                position: Position::new(5, 5),
                health: 100,
                is_alive: true,
            }],
            bombs: vec![],
            time_remaining_ms: 300000,
        };

        let bytes = msg.to_msgpack().expect("Failed to serialize");
        let deserialized = ServerMessage::from_msgpack(&bytes).expect("Failed to deserialize");

        match deserialized {
            ServerMessage::StateUpdate {
                tick,
                players,
                bombs,
                time_remaining_ms,
            } => {
                assert_eq!(tick, 1234);
                assert_eq!(players.len(), 1);
                assert_eq!(players[0].id, player_id);
                assert_eq!(players[0].position, Position::new(5, 5));
                assert!(bombs.is_empty());
                assert_eq!(time_remaining_ms, 300000);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_command_ack_success() {
        let msg = ServerMessage::CommandAck {
            sequence: 42,
            success: true,
            position: Some(Position::new(5, 4)),
            error: None,
        };

        let json = msg.to_json().expect("Failed to serialize");
        let deserialized = ServerMessage::from_json(&json).expect("Failed to deserialize");

        match deserialized {
            ServerMessage::CommandAck {
                sequence,
                success,
                position,
                error,
            } => {
                assert_eq!(sequence, 42);
                assert!(success);
                assert_eq!(position, Some(Position::new(5, 4)));
                assert!(error.is_none());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_command_ack_failure() {
        let msg = ServerMessage::CommandAck {
            sequence: 43,
            success: false,
            position: None,
            error: Some("Tile blocked".to_string()),
        };

        let bytes = msg.to_msgpack().expect("Failed to serialize");
        let deserialized = ServerMessage::from_msgpack(&bytes).expect("Failed to deserialize");

        match deserialized {
            ServerMessage::CommandAck {
                sequence,
                success,
                position,
                error,
            } => {
                assert_eq!(sequence, 43);
                assert!(!success);
                assert!(position.is_none());
                assert_eq!(error, Some("Tile blocked".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_msgpack_is_smaller_than_json() {
        let player_id = Uuid::new_v4();
        let msg = ServerMessage::StateUpdate {
            tick: 1234,
            players: vec![
                PlayerState {
                    id: player_id,
                    position: Position::new(5, 5),
                    health: 100,
                    is_alive: true,
                },
                PlayerState {
                    id: Uuid::new_v4(),
                    position: Position::new(10, 10),
                    health: 75,
                    is_alive: true,
                },
            ],
            bombs: vec![BombState {
                id: Uuid::new_v4(),
                position: Position::new(7, 7),
                owner_id: player_id,
                timer_ms: 2500,
            }],
            time_remaining_ms: 300000,
        };

        let msgpack_bytes = msg.to_msgpack().expect("Failed to serialize msgpack");
        let json_string = msg.to_json().expect("Failed to serialize json");

        // MessagePack should be more compact
        assert!(
            msgpack_bytes.len() < json_string.len(),
            "MessagePack ({} bytes) should be smaller than JSON ({} bytes)",
            msgpack_bytes.len(),
            json_string.len()
        );
    }
}
