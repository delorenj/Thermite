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

/// Health bar component
#[derive(Component)]
struct HealthBar {
    /// Owner entity
    owner: Entity,
}

/// Health value component
#[derive(Component, Clone, Copy)]
struct Health {
    current: i32,
    max: i32,
}

impl Health {
    fn new(max: i32) -> Self {
        Health { current: max, max }
    }

    fn percentage(&self) -> f32 {
        (self.current as f32 / self.max as f32).clamp(0.0, 1.0)
    }
}

/// Marker for the raid timer UI text
#[derive(Component)]
struct RaidTimerText;

/// Marker for minimap elements
#[derive(Component)]
struct MinimapElement;

/// Marker for minimap player indicators
#[derive(Component)]
struct MinimapPlayer {
    tracked_player: Entity,
}

/// Marker for bomb sprites
#[derive(Component)]
struct BombMarker {
    bomb_id: uuid::Uuid,
}

/// Bomb timer text component
#[derive(Component)]
struct BombTimerText {
    bomb_entity: Entity,
}

/// Explosion animation component
#[derive(Component)]
struct Explosion {
    /// Time remaining before despawn (in seconds)
    lifetime: f32,
    /// Initial alpha for fade out
    initial_alpha: f32,
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

/// Current bomb states from server
#[derive(Resource, Default)]
struct BombStates(Vec<thermite_server::protocol::BombState>);

/// Previous bomb states for detecting explosions
#[derive(Resource, Default)]
struct PreviousBombStates(Vec<thermite_server::protocol::BombState>);

// =============================================================================
// Main Entry Point
// =============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Thermite".to_string(),
                resolution: (800.0, 600.0).into(),
                resizable: true,
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
        .init_resource::<BombStates>()
        .init_resource::<PreviousBombStates>()
        // Startup systems
        .add_systems(
            Startup,
            (setup_camera, setup_grid, setup_ui, spawn_local_player),
        )
        // Update systems
        .add_systems(
            Update,
            (
                handle_input,
                receive_network_messages,
                update_player_transforms,
                update_health_bars,
                update_raid_timer_ui,
                update_minimap,
                update_bomb_sprites,
                spawn_explosions,
                update_explosions,
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
    // Render grid tiles with enhanced visual polish
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let world_x = x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let world_y = y as f32 * TILE_SIZE + TILE_SIZE / 2.0;

            // Enhanced checkerboard pattern with better contrast
            let is_dark = (x + y) % 2 == 0;
            let color = if is_dark {
                Color::srgb(0.15, 0.16, 0.18) // Darker tiles
            } else {
                Color::srgb(0.20, 0.22, 0.24) // Lighter tiles
            };

            // Spawn main tile
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)), // Gap for grid lines
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 0.0),
            ));

            // Add subtle highlight on lighter tiles for depth
            if !is_dark {
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.3, 0.35, 0.4, 0.15), // Subtle blue highlight
                        custom_size: Some(Vec2::new(TILE_SIZE - 4.0, 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(world_x, world_y + TILE_SIZE / 2.0 - 3.0, 0.01),
                ));
            }
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
        Health::new(100),
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

fn setup_ui(mut commands: Commands) {
    // Spawn raid timer UI at top center
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-100.0), // Center offset
                    ..default()
                },
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            RaidTimerText,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("5:00"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));
        });

    // Spawn minimap background at bottom right
    let minimap_size = 150.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(minimap_size),
                height: Val::Px(minimap_size),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            MinimapElement,
        ))
        .with_children(|parent| {
            // Add minimap border
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
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
    // Check for bomb placement (Spacebar)
    if keyboard.just_pressed(KeyCode::Space) {
        // Increment sequence
        sequence.0 += 1;
        let seq = sequence.0;

        info!("Bomb placement requested: seq={}", seq);

        // Send to server
        if let Some(tx) = &network.send_tx {
            let msg = ClientMessage::PlaceBomb { sequence: seq };
            let _ = tx.send(msg);
        }
    }

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
    mut bomb_states: ResMut<BombStates>,
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
                bombs,
                time_remaining_ms,
            } => {
                server_tick.0 = tick;
                time_remaining.0 = time_remaining_ms;
                bomb_states.0 = bombs;

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

fn update_health_bars(
    mut commands: Commands,
    players: Query<(Entity, &Health, &Transform), Or<(With<LocalPlayer>, With<RemotePlayer>)>>,
    mut health_bars: Query<(&HealthBar, &mut Transform, &mut Sprite), Without<LocalPlayer>>,
    existing_bars: Query<&HealthBar>,
) {
    // Create health bars for players that don't have one
    for (player_entity, _health, player_transform) in players.iter() {
        let has_bar = existing_bars
            .iter()
            .any(|bar| bar.owner == player_entity);

        if !has_bar {
            // Spawn health bar above player
            commands.spawn((
                HealthBar {
                    owner: player_entity,
                },
                Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(TILE_SIZE - 4.0, 4.0)),
                    ..default()
                },
                Transform::from_xyz(
                    player_transform.translation.x,
                    player_transform.translation.y + TILE_SIZE / 2.0 + 4.0,
                    2.0,
                ),
            ));
        }
    }

    // Update existing health bars
    for (health_bar, mut bar_transform, mut bar_sprite) in health_bars.iter_mut() {
        if let Ok((_, health, player_transform)) = players.get(health_bar.owner) {
            // Position above owner
            bar_transform.translation.x = player_transform.translation.x;
            bar_transform.translation.y = player_transform.translation.y + TILE_SIZE / 2.0 + 4.0;

            // Update color based on health percentage
            let health_pct = health.percentage();
            bar_sprite.color = if health_pct > 0.6 {
                Color::srgb(0.0, 1.0, 0.0) // Green
            } else if health_pct > 0.3 {
                Color::srgb(1.0, 1.0, 0.0) // Yellow
            } else {
                Color::srgb(1.0, 0.0, 0.0) // Red
            };

            // Scale width based on health
            bar_sprite.custom_size = Some(Vec2::new((TILE_SIZE - 4.0) * health_pct, 4.0));
        }
    }
}

fn update_raid_timer_ui(
    time_remaining: Res<TimeRemaining>,
    timer_query: Query<Entity, With<RaidTimerText>>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    if !time_remaining.is_changed() {
        return;
    }

    for timer_entity in timer_query.iter() {
        if let Ok(children) = children_query.get(timer_entity) {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    let total_seconds = time_remaining.0 / 1000;
                    let minutes = total_seconds / 60;
                    let seconds = total_seconds % 60;

                    **text = format!("{}:{:02}", minutes, seconds);
                }
            }
        }
    }
}

fn update_minimap(
    mut commands: Commands,
    minimap_query: Query<Entity, With<MinimapElement>>,
    players: Query<(Entity, &GridPosition), Or<(With<LocalPlayer>, With<RemotePlayer>)>>,
    local_player: Query<Entity, With<LocalPlayer>>,
    mut minimap_players: Query<(&MinimapPlayer, &mut Node, &mut BackgroundColor)>,
    existing_minimap_players: Query<&MinimapPlayer>,
) {
    let minimap_size = 150.0;
    let scale_x = minimap_size / (GRID_WIDTH as f32);
    let scale_y = minimap_size / (GRID_HEIGHT as f32);

    // Get the minimap parent entity
    let Ok(minimap_entity) = minimap_query.single() else {
        return;
    };

    let local_player_entity = local_player.single().ok();

    // Create minimap indicators for new players
    for (player_entity, _) in players.iter() {
        let has_indicator = existing_minimap_players
            .iter()
            .any(|indicator| indicator.tracked_player == player_entity);

        if !has_indicator {
            let is_local = Some(player_entity) == local_player_entity;
            let color = if is_local {
                Color::srgb(0.2, 0.8, 0.2) // Green for local
            } else {
                Color::srgb(0.8, 0.2, 0.2) // Red for remote
            };

            commands.entity(minimap_entity).with_children(|parent| {
                parent.spawn((
                    MinimapPlayer {
                        tracked_player: player_entity,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(4.0),
                        height: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
            });
        }
    }

    // Update minimap indicator positions
    for (minimap_player, mut node, _) in minimap_players.iter_mut() {
        if let Ok((_, grid_pos)) = players.get(minimap_player.tracked_player) {
            node.left = Val::Px(grid_pos.x as f32 * scale_x);
            node.top = Val::Px(grid_pos.y as f32 * scale_y);
        }
    }
}

/// Update bomb sprites based on server bomb states
fn update_bomb_sprites(
    mut commands: Commands,
    bomb_states: Res<BombStates>,
    existing_bombs: Query<(Entity, &BombMarker)>,
    mut bomb_sprites: Query<(&BombMarker, &mut Transform, &Children)>,
    mut bomb_timer_texts: Query<(&BombTimerText, &mut Text)>,
) {
    // Track which bombs should exist
    let current_bomb_ids: std::collections::HashSet<uuid::Uuid> =
        bomb_states.0.iter().map(|b| b.id).collect();

    // Remove bombs that no longer exist
    for (entity, bomb_marker) in existing_bombs.iter() {
        if !current_bomb_ids.contains(&bomb_marker.bomb_id) {
            commands.entity(entity).despawn();
        }
    }

    // Create or update bombs
    for bomb_state in bomb_states.0.iter() {
        let exists = existing_bombs
            .iter()
            .any(|(_, marker)| marker.bomb_id == bomb_state.id);

        if !exists {
            // Spawn new bomb sprite with timer text
            let bomb_entity = commands
                .spawn((
                    BombMarker {
                        bomb_id: bomb_state.id,
                    },
                    Sprite {
                        color: Color::srgb(0.8, 0.1, 0.1), // Red bomb placeholder
                        custom_size: Some(Vec2::new(TILE_SIZE * 0.8, TILE_SIZE * 0.8)),
                        ..default()
                    },
                    Transform::from_xyz(
                        bomb_state.position.x as f32 * TILE_SIZE,
                        bomb_state.position.y as f32 * TILE_SIZE,
                        1.0,
                    ),
                ))
                .id();

            // Spawn timer text as child
            commands.entity(bomb_entity).with_children(|parent| {
                parent.spawn((
                    BombTimerText {
                        bomb_entity,
                    },
                    Text2d::new(format!("{:.1}", bomb_state.timer_ms as f32 / 1000.0)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });
        } else {
            // Update existing bomb position (bombs don't move, but just in case)
            for (bomb_marker, mut transform, children) in bomb_sprites.iter_mut() {
                if bomb_marker.bomb_id == bomb_state.id {
                    transform.translation.x = bomb_state.position.x as f32 * TILE_SIZE;
                    transform.translation.y = bomb_state.position.y as f32 * TILE_SIZE;

                    // Update timer text
                    for child in children.iter() {
                        if let Ok((_, mut text)) = bomb_timer_texts.get_mut(child) {
                            **text = format!("{:.1}", bomb_state.timer_ms as f32 / 1000.0);
                        }
                    }
                }
            }
        }
    }
}

/// Spawn explosion animations when bombs disappear
fn spawn_explosions(
    mut commands: Commands,
    bomb_states: Res<BombStates>,
    mut previous_bomb_states: ResMut<PreviousBombStates>,
) {
    // Find bombs that existed before but don't exist now (likely exploded)
    let current_bomb_ids: std::collections::HashSet<uuid::Uuid> =
        bomb_states.0.iter().map(|b| b.id).collect();

    for prev_bomb in previous_bomb_states.0.iter() {
        if !current_bomb_ids.contains(&prev_bomb.id) {
            // Bomb disappeared - spawn explosion animation
            let explosion_size = TILE_SIZE * 3.0; // Placeholder explosion size

            commands.spawn((
                Explosion {
                    lifetime: 0.5, // 0.5 second animation
                    initial_alpha: 1.0,
                },
                Sprite {
                    color: Color::srgba(1.0, 0.5, 0.0, 1.0), // Orange explosion
                    custom_size: Some(Vec2::new(explosion_size, explosion_size)),
                    ..default()
                },
                Transform::from_xyz(
                    prev_bomb.position.x as f32 * TILE_SIZE,
                    prev_bomb.position.y as f32 * TILE_SIZE,
                    0.5,
                ),
            ));
        }
    }

    // Update previous states for next frame
    previous_bomb_states.0 = bomb_states.0.clone();
}

/// Update and fade out explosion animations
fn update_explosions(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(Entity, &mut Explosion, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut explosion, mut sprite, mut transform) in explosions.iter_mut() {
        explosion.lifetime -= time.delta_secs();

        if explosion.lifetime <= 0.0 {
            // Explosion finished, despawn
            commands.entity(entity).despawn();
        } else {
            // Fade out and expand
            let progress = 1.0 - (explosion.lifetime / 0.5);
            let alpha = explosion.initial_alpha * (1.0 - progress);

            // Update alpha
            sprite.color = Color::srgba(1.0, 0.5, 0.0, alpha);

            // Expand size
            let base_size = TILE_SIZE * 3.0;
            let expanded_size = base_size * (1.0 + progress * 0.5);
            sprite.custom_size = Some(Vec2::new(expanded_size, expanded_size));

            // Slight scale effect
            transform.scale = Vec3::splat(1.0 + progress * 0.3);
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
