//! Thermite Game Client
//!
//! Bevy-based game client with WASD movement, client-side prediction,
//! and WebSocket communication with the game server.

use bevy::prelude::*;
use std::collections::VecDeque;
use tokio::sync::mpsc;

use thermite_server::player::{Direction, Position};
use thermite_server::protocol::{ClientMessage, PlayerState, ServerMessage};

// =============================================================================
// Constants
// =============================================================================

/// Tile size in pixels for rendering
const TILE_SIZE: f32 = 32.0;

/// Grid dimensions (should match server)
const GRID_WIDTH: usize = 20;
const GRID_HEIGHT: usize = 20;

/// Server address
const SERVER_ADDR: &str = "ws://127.0.0.1:9001";

/// Maximum pending inputs for client-side prediction
const MAX_PENDING_INPUTS: usize = 64;

// =============================================================================
// Components
// =============================================================================

/// Marker component for the local player entity
#[derive(Component)]
struct LocalPlayer;

/// Marker component for remote player entities
#[derive(Component)]
struct RemotePlayer {
    player_id: uuid::Uuid,
}

/// Grid position component (authoritative from server)
#[derive(Component, Clone, Copy, Debug)]
struct GridPosition {
    x: usize,
    y: usize,
}

impl From<Position> for GridPosition {
    fn from(pos: Position) -> Self {
        GridPosition { x: pos.x, y: pos.y }
    }
}

impl From<GridPosition> for Position {
    fn from(pos: GridPosition) -> Self {
        Position::new(pos.x, pos.y)
    }
}

/// Predicted position for client-side prediction
#[derive(Component, Clone, Copy, Debug)]
struct PredictedPosition {
    x: usize,
    y: usize,
}

/// Pending input for reconciliation
#[derive(Clone, Debug)]
struct PendingInput {
    sequence: u64,
    direction: Direction,
    predicted_position: Position,
}

// =============================================================================
// Resources
// =============================================================================

/// Network connection state
#[derive(Resource)]
struct NetworkState {
    /// Channel to send messages to the network task
    send_tx: Option<mpsc::UnboundedSender<ClientMessage>>,
    /// Channel to receive messages from the network task
    recv_rx: Option<mpsc::UnboundedReceiver<ServerMessage>>,
    /// Our assigned player ID from the server
    player_id: Option<uuid::Uuid>,
    /// Whether we're connected
    connected: bool,
}

impl Default for NetworkState {
    fn default() -> Self {
        NetworkState {
            send_tx: None,
            recv_rx: None,
            player_id: None,
            connected: false,
        }
    }
}

/// Input sequence counter
#[derive(Resource, Default)]
struct InputSequence(u64);

/// Pending inputs for client-side prediction reconciliation
#[derive(Resource, Default)]
struct PendingInputs(VecDeque<PendingInput>);

/// Game tick from server
#[derive(Resource, Default)]
struct ServerTick(u64);

/// Time remaining in match
#[derive(Resource)]
struct TimeRemaining(u64);

impl Default for TimeRemaining {
    fn default() -> Self {
        TimeRemaining(5 * 60 * 1000) // 5 minutes default
    }
}

/// Map of remote players by UUID
#[derive(Resource, Default)]
struct RemotePlayers(std::collections::HashMap<uuid::Uuid, Entity>);

// =============================================================================
// Main Entry Point
// =============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Thermite".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Resources
        .init_resource::<NetworkState>()
        .init_resource::<InputSequence>()
        .init_resource::<PendingInputs>()
        .init_resource::<ServerTick>()
        .init_resource::<TimeRemaining>()
        .init_resource::<RemotePlayers>()
        // Startup systems
        .add_systems(Startup, (setup_camera, setup_grid, spawn_local_player))
        // Update systems
        .add_systems(
            Update,
            (
                handle_input,
                receive_network_messages,
                update_player_transforms,
                update_timer_display,
            ),
        )
        .run();
}

// =============================================================================
// Startup Systems
// =============================================================================

fn setup_camera(mut commands: Commands) {
    // Center camera on grid
    let center_x = (GRID_WIDTH as f32 * TILE_SIZE) / 2.0;
    let center_y = (GRID_HEIGHT as f32 * TILE_SIZE) / 2.0;

    commands.spawn((
        Camera2d,
        Transform::from_xyz(center_x, center_y, 1000.0),
    ));
}

fn setup_grid(mut commands: Commands) {
    // Render grid tiles
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let world_x = x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let world_y = y as f32 * TILE_SIZE + TILE_SIZE / 2.0;

            // Checkerboard pattern for visibility
            let color = if (x + y) % 2 == 0 {
                Color::srgb(0.2, 0.2, 0.2)
            } else {
                Color::srgb(0.25, 0.25, 0.25)
            };

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 0.0),
            ));
        }
    }
}

fn spawn_local_player(mut commands: Commands) {
    // Spawn local player at default position (will be updated by server)
    let start_pos = GridPosition { x: 1, y: 1 };

    commands.spawn((
        LocalPlayer,
        start_pos,
        PredictedPosition {
            x: start_pos.x,
            y: start_pos.y,
        },
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2), // Green for local player
            custom_size: Some(Vec2::splat(TILE_SIZE - 4.0)),
            ..default()
        },
        Transform::from_xyz(
            grid_to_world_x(start_pos.x),
            grid_to_world_y(start_pos.y),
            1.0,
        ),
    ));
}

// =============================================================================
// Update Systems
// =============================================================================

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sequence: ResMut<InputSequence>,
    mut pending_inputs: ResMut<PendingInputs>,
    network: Res<NetworkState>,
    mut query: Query<(&GridPosition, &mut PredictedPosition), With<LocalPlayer>>,
) {
    // Get direction from WASD/Arrow keys
    let direction = if keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp)
    {
        Some(Direction::North)
    } else if keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown) {
        Some(Direction::South)
    } else if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
        Some(Direction::West)
    } else if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
        Some(Direction::East)
    } else {
        None
    };

    if let Some(dir) = direction {
        // Increment sequence
        sequence.0 += 1;
        let seq = sequence.0;

        // Client-side prediction: update predicted position
        if let Ok((_grid_pos, mut predicted_pos)) = query.single_mut() {
            let new_pos = apply_direction(predicted_pos.x, predicted_pos.y, dir);
            if let Some((nx, ny)) = new_pos {
                // Only predict if within bounds
                if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                    let old_x = predicted_pos.x;
                    let old_y = predicted_pos.y;
                    predicted_pos.x = nx;
                    predicted_pos.y = ny;

                    // Store pending input for reconciliation
                    pending_inputs.0.push_back(PendingInput {
                        sequence: seq,
                        direction: dir,
                        predicted_position: Position::new(nx, ny),
                    });

                    // Trim old inputs
                    while pending_inputs.0.len() > MAX_PENDING_INPUTS {
                        pending_inputs.0.pop_front();
                    }

                    info!(
                        "Input seq={}: {:?} -> ({}, {}) -> ({}, {})",
                        seq, dir, old_x, old_y, nx, ny
                    );
                }
            }
        }

        // Send to server
        if let Some(tx) = &network.send_tx {
            let msg = ClientMessage::Move {
                direction: dir,
                sequence: seq,
            };
            let _ = tx.send(msg);
        }
    }
}

fn receive_network_messages(
    mut commands: Commands,
    mut network: ResMut<NetworkState>,
    mut server_tick: ResMut<ServerTick>,
    mut time_remaining: ResMut<TimeRemaining>,
    mut pending_inputs: ResMut<PendingInputs>,
    mut remote_players: ResMut<RemotePlayers>,
    mut local_player_query: Query<
        (&mut GridPosition, &mut PredictedPosition),
        With<LocalPlayer>,
    >,
    mut remote_player_query: Query<&mut GridPosition, (With<RemotePlayer>, Without<LocalPlayer>)>,
) {
    // Collect all pending messages first to avoid borrow issues
    let mut messages = Vec::new();
    if let Some(rx) = &mut network.recv_rx {
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }
    }

    // Get current player_id for comparison
    let my_player_id = network.player_id;

    // Process collected messages
    for msg in messages {
        match msg {
            ServerMessage::Welcome {
                player_id,
                tick_rate_ms,
            } => {
                info!(
                    "Connected! Player ID: {}, tick rate: {}ms",
                    player_id, tick_rate_ms
                );
                network.player_id = Some(player_id);
                network.connected = true;
            }

            ServerMessage::StateUpdate {
                tick,
                players,
                bombs: _,
                time_remaining_ms,
            } => {
                server_tick.0 = tick;
                time_remaining.0 = time_remaining_ms;

                // Update player positions
                for player_state in players {
                    if Some(player_state.id) == my_player_id {
                        // Update local player's authoritative position
                        if let Ok((mut grid_pos, mut predicted_pos)) =
                            local_player_query.single_mut()
                        {
                            grid_pos.x = player_state.position.x;
                            grid_pos.y = player_state.position.y;

                            // If no pending inputs, sync predicted to authoritative
                            if pending_inputs.0.is_empty() {
                                predicted_pos.x = grid_pos.x;
                                predicted_pos.y = grid_pos.y;
                            }
                        }
                    } else {
                        // Update or create remote player
                        update_remote_player(
                            &mut commands,
                            &mut remote_players,
                            &mut remote_player_query,
                            player_state,
                        );
                    }
                }
            }

            ServerMessage::CommandAck {
                sequence,
                success,
                position,
                error,
            } => {
                // Remove acknowledged input
                pending_inputs.0.retain(|input| input.sequence > sequence);

                if !success {
                    // Server rejected move - rollback prediction
                    warn!("Move rejected: {:?}", error);

                    if let Ok((grid_pos, mut predicted_pos)) = local_player_query.single_mut() {
                        // Rollback to server position and replay pending inputs
                        let mut current_x = grid_pos.x;
                        let mut current_y = grid_pos.y;

                        for pending in pending_inputs.0.iter() {
                            if let Some((nx, ny)) =
                                apply_direction(current_x, current_y, pending.direction)
                            {
                                if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                                    current_x = nx;
                                    current_y = ny;
                                }
                            }
                        }

                        predicted_pos.x = current_x;
                        predicted_pos.y = current_y;
                    }
                } else if let Some(pos) = position {
                    // Successful move - verify prediction was correct
                    info!("Move ack seq={}: position {:?}", sequence, pos);
                }
            }

            ServerMessage::PlayerDied {
                player_id,
                killer_id,
                position,
            } => {
                info!(
                    "Player {} died at {:?}, killed by {:?}",
                    player_id, position, killer_id
                );
            }

            ServerMessage::MatchEnded { reason } => {
                info!("Match ended: {:?}", reason);
            }

            ServerMessage::Pong { timestamp } => {
                info!("Pong: {}", timestamp);
            }

            ServerMessage::Error { message } => {
                error!("Server error: {}", message);
            }
        }
    }
}

fn update_player_transforms(
    mut query: Query<
        (&PredictedPosition, &mut Transform),
        (With<LocalPlayer>, Changed<PredictedPosition>),
    >,
    mut remote_query: Query<
        (&GridPosition, &mut Transform),
        (With<RemotePlayer>, Without<LocalPlayer>, Changed<GridPosition>),
    >,
) {
    // Update local player transform based on predicted position
    for (predicted, mut transform) in query.iter_mut() {
        transform.translation.x = grid_to_world_x(predicted.x);
        transform.translation.y = grid_to_world_y(predicted.y);
    }

    // Update remote player transforms
    for (grid_pos, mut transform) in remote_query.iter_mut() {
        transform.translation.x = grid_to_world_x(grid_pos.x);
        transform.translation.y = grid_to_world_y(grid_pos.y);
    }
}

fn update_timer_display(time_remaining: Res<TimeRemaining>) {
    // Timer display would update a UI element
    // For now, just log periodically (every 10 seconds)
    if time_remaining.is_changed() {
        let seconds = time_remaining.0 / 1000;
        let minutes = seconds / 60;
        let secs = seconds % 60;
        if secs == 0 {
            info!("Time remaining: {}:{:02}", minutes, secs);
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn grid_to_world_x(x: usize) -> f32 {
    x as f32 * TILE_SIZE + TILE_SIZE / 2.0
}

fn grid_to_world_y(y: usize) -> f32 {
    y as f32 * TILE_SIZE + TILE_SIZE / 2.0
}

fn apply_direction(x: usize, y: usize, direction: Direction) -> Option<(usize, usize)> {
    match direction {
        Direction::North => {
            if y > 0 {
                Some((x, y - 1))
            } else {
                None
            }
        }
        Direction::South => Some((x, y + 1)),
        Direction::East => Some((x + 1, y)),
        Direction::West => {
            if x > 0 {
                Some((x - 1, y))
            } else {
                None
            }
        }
    }
}

fn update_remote_player(
    commands: &mut Commands,
    remote_players: &mut RemotePlayers,
    remote_query: &mut Query<&mut GridPosition, (With<RemotePlayer>, Without<LocalPlayer>)>,
    player_state: PlayerState,
) {
    if let Some(&entity) = remote_players.0.get(&player_state.id) {
        // Update existing remote player
        if let Ok(mut grid_pos) = remote_query.get_mut(entity) {
            grid_pos.x = player_state.position.x;
            grid_pos.y = player_state.position.y;
        }
    } else {
        // Spawn new remote player
        let entity = commands
            .spawn((
                RemotePlayer {
                    player_id: player_state.id,
                },
                GridPosition {
                    x: player_state.position.x,
                    y: player_state.position.y,
                },
                Sprite {
                    color: Color::srgb(0.8, 0.2, 0.2), // Red for remote players
                    custom_size: Some(Vec2::splat(TILE_SIZE - 4.0)),
                    ..default()
                },
                Transform::from_xyz(
                    grid_to_world_x(player_state.position.x),
                    grid_to_world_y(player_state.position.y),
                    1.0,
                ),
            ))
            .id();

        remote_players.0.insert(player_state.id, entity);
        info!("Spawned remote player {}", player_state.id);
    }
}
