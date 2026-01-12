//! Game server with WebSocket handling and tick loop
//!
//! Manages WebSocket connections and runs the authoritative game simulation.

use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::game_state::{GameState, MatchConfig};
use crate::map_system::Grid;
use crate::player::{Direction, Position};
use crate::protocol::{ClientMessage, ServerMessage};

/// Channel capacity for internal messaging
const CHANNEL_CAPACITY: usize = 256;

/// Command from a client connection to the game loop
#[derive(Debug)]
pub enum GameCommand {
    /// Player joined the game
    PlayerJoin {
        player_id: Uuid,
        spawn_position: Position,
        response_tx: mpsc::Sender<ServerMessage>,
    },
    /// Player sent a movement command
    PlayerMove {
        player_id: Uuid,
        direction: Direction,
        sequence: u64,
    },
    /// Player placed a bomb
    PlaceBomb {
        player_id: Uuid,
        sequence: u64,
    },
    /// Player disconnected
    PlayerDisconnect { player_id: Uuid },
}

/// Shared state for the game server
#[derive(Clone)]
pub struct GameServer {
    /// The game state
    state: Arc<RwLock<GameState>>,
    /// Broadcast channel for state updates
    broadcast_tx: broadcast::Sender<ServerMessage>,
    /// Command channel for receiving commands from connections
    command_tx: mpsc::Sender<GameCommand>,
    /// Per-player response channels
    player_channels: Arc<RwLock<HashMap<Uuid, mpsc::Sender<ServerMessage>>>>,
}

impl GameServer {
    /// Create a new game server with the given grid
    pub fn new(match_id: Uuid, grid: Grid, config: MatchConfig) -> (Self, mpsc::Receiver<GameCommand>) {
        let state = Arc::new(RwLock::new(GameState::new(match_id, grid, config)));
        let (broadcast_tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (command_tx, command_rx) = mpsc::channel(CHANNEL_CAPACITY);
        let player_channels = Arc::new(RwLock::new(HashMap::new()));

        (
            GameServer {
                state,
                broadcast_tx,
                command_tx,
                player_channels,
            },
            command_rx,
        )
    }

    /// Run the WebSocket server on the given address
    pub async fn run_websocket_server(
        &self,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(addr).await?;
        info!("WebSocket server listening on {}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            let command_tx = self.command_tx.clone();
            let broadcast_rx = self.broadcast_tx.subscribe();
            let player_channels = self.player_channels.clone();
            let tick_rate_ms = {
                let state = self.state.read().await;
                state.config.tick_rate_ms as u32
            };

            tokio::spawn(async move {
                if let Err(e) =
                    handle_connection(stream, peer_addr, command_tx, broadcast_rx, player_channels, tick_rate_ms)
                        .await
                {
                    error!("Connection error for {}: {}", peer_addr, e);
                }
            });
        }

        Ok(())
    }

    /// Run the game tick loop
    pub async fn run_tick_loop(&self, mut command_rx: mpsc::Receiver<GameCommand>) {
        let tick_rate_ms = {
            let state = self.state.read().await;
            state.config.tick_rate_ms
        };

        let mut tick_interval = interval(Duration::from_millis(tick_rate_ms));

        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    self.process_tick().await;
                }
                Some(command) = command_rx.recv() => {
                    self.process_command(command).await;
                }
            }

            // Check if match is still active
            let is_active = {
                let state = self.state.read().await;
                state.is_active
            };

            if !is_active {
                info!("Match ended, stopping tick loop");
                break;
            }
        }
    }

    /// Process a single game tick
    async fn process_tick(&self) {
        let mut state = self.state.write().await;
        state.tick();

        // Broadcast detonation events first (before state update)
        for (bomb_id, position, blast_tiles, destroyed_tiles) in &state.pending_detonations {
            let detonation = ServerMessage::BombDetonation {
                bomb_id: *bomb_id,
                position: *position,
                blast_tiles: blast_tiles.clone(),
                destroyed_tiles: destroyed_tiles.clone(),
            };
            let _ = self.broadcast_tx.send(detonation);
        }

        // Broadcast damage events
        for (player_id, damage_amount, new_health, killer_id) in &state.pending_damage_events {
            let damage_event = ServerMessage::PlayerDamaged {
                player_id: *player_id,
                damage_amount: *damage_amount,
                new_health: *new_health,
                killer_id: *killer_id,
            };
            let _ = self.broadcast_tx.send(damage_event);
        }

        // Broadcast death events
        for (player_id, killer_id, position) in &state.pending_death_events {
            let death_event = ServerMessage::PlayerDied {
                player_id: *player_id,
                killer_id: *killer_id,
                position: *position,
            };
            let _ = self.broadcast_tx.send(death_event);
        }

        // Broadcast state update to all players
        let update = ServerMessage::StateUpdate {
            tick: state.tick,
            players: state.get_player_states(),
            bombs: state.get_bomb_states(),
            time_remaining_ms: state.time_remaining_ms,
        };

        // Ignore send errors (no receivers)
        let _ = self.broadcast_tx.send(update);
    }

    /// Process a command from a client
    async fn process_command(&self, command: GameCommand) {
        match command {
            GameCommand::PlayerJoin {
                player_id,
                spawn_position,
                response_tx,
            } => {
                let mut state = self.state.write().await;
                match state.add_player(player_id, spawn_position) {
                    Ok(()) => {
                        info!("Player {} joined at {:?}", player_id, spawn_position);
                        // Store the player's response channel
                        let mut channels = self.player_channels.write().await;
                        channels.insert(player_id, response_tx);
                    }
                    Err(e) => {
                        error!("Failed to add player {}: {}", player_id, e);
                    }
                }
            }
            GameCommand::PlayerMove {
                player_id,
                direction,
                sequence,
            } => {
                let mut state = self.state.write().await;
                let result = state.process_move(&player_id, direction, sequence);

                // Send acknowledgment to the specific player
                let channels = self.player_channels.read().await;
                if let Some(tx) = channels.get(&player_id) {
                    let ack = match result {
                        Ok(pos) => ServerMessage::CommandAck {
                            sequence,
                            success: true,
                            position: Some(pos),
                            error: None,
                        },
                        Err(e) => ServerMessage::CommandAck {
                            sequence,
                            success: false,
                            position: None,
                            error: Some(e.to_string()),
                        },
                    };
                    let _ = tx.send(ack).await;
                }
            }
            GameCommand::PlaceBomb {
                player_id,
                sequence,
            } => {
                let mut state = self.state.write().await;
                let result = state.place_bomb(&player_id, sequence);

                // Send acknowledgment to the specific player
                let channels = self.player_channels.read().await;
                if let Some(tx) = channels.get(&player_id) {
                    let ack = match result {
                        Ok(_bomb_id) => ServerMessage::CommandAck {
                            sequence,
                            success: true,
                            position: None,
                            error: None,
                        },
                        Err(e) => ServerMessage::CommandAck {
                            sequence,
                            success: false,
                            position: None,
                            error: Some(e.to_string()),
                        },
                    };
                    let _ = tx.send(ack).await;
                }
            }
            GameCommand::PlayerDisconnect { player_id } => {
                let mut state = self.state.write().await;
                state.remove_player(&player_id);

                let mut channels = self.player_channels.write().await;
                channels.remove(&player_id);

                info!("Player {} disconnected", player_id);
            }
        }
    }

    /// Get current tick count
    pub async fn get_tick(&self) -> u64 {
        self.state.read().await.tick
    }

    /// Check if match is active
    pub async fn is_active(&self) -> bool {
        self.state.read().await.is_active
    }
}

/// Handle a single WebSocket connection
async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    command_tx: mpsc::Sender<GameCommand>,
    mut broadcast_rx: broadcast::Receiver<ServerMessage>,
    _player_channels: Arc<RwLock<HashMap<Uuid, mpsc::Sender<ServerMessage>>>>,
    tick_rate_ms: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Assign player ID
    let player_id = Uuid::new_v4();
    info!("New connection from {} assigned player_id {}", peer_addr, player_id);

    // Create response channel for this player
    let (response_tx, mut response_rx) = mpsc::channel::<ServerMessage>(CHANNEL_CAPACITY);

    // Send welcome message
    let welcome = ServerMessage::Welcome {
        player_id,
        tick_rate_ms,
    };
    let welcome_bytes = welcome.to_msgpack()?;
    ws_sender.send(Message::Binary(welcome_bytes.into())).await?;

    // Join the game at a default spawn position (in real game, this would be assigned)
    command_tx
        .send(GameCommand::PlayerJoin {
            player_id,
            spawn_position: Position::new(1, 1),
            response_tx,
        })
        .await?;

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        match ClientMessage::from_msgpack(&data) {
                            Ok(client_msg) => {
                                handle_client_message(client_msg, player_id, &command_tx).await;
                            }
                            Err(e) => {
                                warn!("Failed to parse message from {}: {}", peer_addr, e);
                            }
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Also support JSON for debugging
                        match ClientMessage::from_json(&text) {
                            Ok(client_msg) => {
                                handle_client_message(client_msg, player_id, &command_tx).await;
                            }
                            Err(e) => {
                                warn!("Failed to parse JSON message from {}: {}", peer_addr, e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Connection closed for {}", peer_addr);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        ws_sender.send(Message::Pong(data)).await?;
                    }
                    Some(Ok(_)) => {} // Ignore other message types
                    Some(Err(e)) => {
                        error!("WebSocket error for {}: {}", peer_addr, e);
                        break;
                    }
                }
            }

            // Handle broadcast state updates
            Ok(server_msg) = broadcast_rx.recv() => {
                let bytes = server_msg.to_msgpack()?;
                if let Err(e) = ws_sender.send(Message::Binary(bytes.into())).await {
                    error!("Failed to send broadcast to {}: {}", peer_addr, e);
                    break;
                }
            }

            // Handle direct responses (command acks)
            Some(server_msg) = response_rx.recv() => {
                let bytes = server_msg.to_msgpack()?;
                if let Err(e) = ws_sender.send(Message::Binary(bytes.into())).await {
                    error!("Failed to send response to {}: {}", peer_addr, e);
                    break;
                }
            }
        }
    }

    // Clean up on disconnect
    let _ = command_tx
        .send(GameCommand::PlayerDisconnect { player_id })
        .await;

    Ok(())
}

/// Handle a parsed client message
async fn handle_client_message(
    msg: ClientMessage,
    player_id: Uuid,
    command_tx: &mpsc::Sender<GameCommand>,
) {
    match msg {
        ClientMessage::Move {
            direction,
            sequence,
        } => {
            let _ = command_tx
                .send(GameCommand::PlayerMove {
                    player_id,
                    direction,
                    sequence,
                })
                .await;
        }
        ClientMessage::PlaceBomb { sequence } => {
            let _ = command_tx
                .send(GameCommand::PlaceBomb {
                    player_id,
                    sequence,
                })
                .await;
        }
        ClientMessage::Extract { sequence: _ } => {
            // Extraction will be implemented in STORY-015
            warn!("Extraction not yet implemented");
        }
        ClientMessage::Ping { timestamp } => {
            // Pong handled at protocol level
            info!("Received ping with timestamp {}", timestamp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_game_server_creation() {
        let grid = Grid::new(20, 20);
        let (server, _rx) = GameServer::new(Uuid::new_v4(), grid, MatchConfig::default());

        assert!(server.is_active().await);
        assert_eq!(server.get_tick().await, 0);
    }

    #[tokio::test]
    async fn test_process_tick() {
        let grid = Grid::new(20, 20);
        let (server, _rx) = GameServer::new(Uuid::new_v4(), grid, MatchConfig::default());

        server.process_tick().await;

        assert_eq!(server.get_tick().await, 1);
    }

    #[tokio::test]
    async fn test_player_join_command() {
        let grid = Grid::new(20, 20);
        let (server, mut rx) = GameServer::new(Uuid::new_v4(), grid, MatchConfig::default());

        let player_id = Uuid::new_v4();
        let (response_tx, _response_rx) = mpsc::channel(16);

        // Send join command directly
        let join_cmd = GameCommand::PlayerJoin {
            player_id,
            spawn_position: Position::new(5, 5),
            response_tx,
        };

        // Simulate the command being processed
        server.process_command(join_cmd).await;

        // Verify player was added
        let state = server.state.read().await;
        assert!(state.get_player(&player_id).is_some());
    }

    #[tokio::test]
    async fn test_player_move_command() {
        let grid = Grid::new(20, 20);
        let (server, _rx) = GameServer::new(Uuid::new_v4(), grid, MatchConfig::default());

        let player_id = Uuid::new_v4();
        let (response_tx, mut response_rx) = mpsc::channel(16);

        // Add player first
        server
            .process_command(GameCommand::PlayerJoin {
                player_id,
                spawn_position: Position::new(5, 5),
                response_tx,
            })
            .await;

        // Send move command
        server
            .process_command(GameCommand::PlayerMove {
                player_id,
                direction: Direction::North,
                sequence: 1,
            })
            .await;

        // Check response
        let response = response_rx.recv().await.unwrap();
        match response {
            ServerMessage::CommandAck {
                sequence,
                success,
                position,
                error,
            } => {
                assert_eq!(sequence, 1);
                assert!(success);
                assert_eq!(position, Some(Position::new(5, 4)));
                assert!(error.is_none());
            }
            _ => panic!("Expected CommandAck"),
        }

        // Verify player position
        let state = server.state.read().await;
        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.position, Position::new(5, 4));
    }

    #[tokio::test]
    async fn test_player_disconnect_command() {
        let grid = Grid::new(20, 20);
        let (server, _rx) = GameServer::new(Uuid::new_v4(), grid, MatchConfig::default());

        let player_id = Uuid::new_v4();
        let (response_tx, _) = mpsc::channel(16);

        // Add player
        server
            .process_command(GameCommand::PlayerJoin {
                player_id,
                spawn_position: Position::new(5, 5),
                response_tx,
            })
            .await;

        // Verify player exists
        {
            let state = server.state.read().await;
            assert!(state.get_player(&player_id).is_some());
        }

        // Disconnect
        server
            .process_command(GameCommand::PlayerDisconnect { player_id })
            .await;

        // Verify player removed
        let state = server.state.read().await;
        assert!(state.get_player(&player_id).is_none());
    }
}
