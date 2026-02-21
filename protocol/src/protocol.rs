use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::{Direction, Position};

/// Messages sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Move { direction: Direction, sequence: u64 },
    PlaceBomb { sequence: u64 },
    Extract { sequence: u64 },
    Ping { timestamp: u64 },
}

/// Messages sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome {
        player_id: Uuid,
        tick_rate_ms: u32,
    },
    StateUpdate {
        tick: u64,
        players: Vec<PlayerState>,
        bombs: Vec<BombState>,
        time_remaining_ms: u64,
    },
    CommandAck {
        sequence: u64,
        success: bool,
        position: Option<Position>,
        error: Option<String>,
    },
    PlayerDied {
        player_id: Uuid,
        killer_id: Option<Uuid>,
        position: Position,
    },
    MatchEnded {
        reason: MatchEndReason,
    },
    BombDetonation {
        bomb_id: Uuid,
        position: Position,
        blast_tiles: Vec<Position>,
        destroyed_tiles: Vec<Position>,
    },
    PlayerDamaged {
        player_id: Uuid,
        damage_amount: i32,
        new_health: i32,
        killer_id: Option<Uuid>,
    },
    PlayerExtracted {
        player_id: Uuid,
        position: Position,
    },
    Pong {
        timestamp: u64,
    },
    Error {
        message: String,
    },
    LobbyCountdown {
        seconds_remaining: u32,
    },
    PlayerDisconnected {
        player_id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: Uuid,
    pub position: Position,
    pub health: i32,
    pub is_alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BombState {
    pub id: Uuid,
    pub position: Position,
    pub owner_id: Uuid,
    pub timer_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchEndReason {
    TimerExpired,
    AllPlayersExtracted,
    AllPlayersDead,
    ServerShutdown,
}

impl ClientMessage {
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    pub fn from_msgpack(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl ServerMessage {
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    pub fn from_msgpack(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
