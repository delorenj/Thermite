//! Thermite Game Client
//!
//! Bevy-based game client with WASD movement, client-side prediction,
//! and WebSocket communication with the game server.

mod puzzle;

use bevy::prelude::*;
use std::collections::VecDeque;

use puzzle::PuzzleLevelResource;
use thermite_protocol::player::{Direction, Position};
use thermite_protocol::protocol::{ClientMessage, PlayerState, ServerMessage};

#[cfg(target_arch = "wasm32")]
type ClientMessageSender = crossbeam_channel::Sender<ClientMessage>;
#[cfg(target_arch = "wasm32")]
type ServerMessageReceiver = crossbeam_channel::Receiver<ServerMessage>;

#[cfg(not(target_arch = "wasm32"))]
type ClientMessageSender = tokio::sync::mpsc::UnboundedSender<ClientMessage>;
#[cfg(not(target_arch = "wasm32"))]
type ServerMessageReceiver = tokio::sync::mpsc::UnboundedReceiver<ServerMessage>;

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

/// Marker for the lobby countdown UI text
#[derive(Component)]
struct LobbyCountdownText;

/// Marker for puzzle goal tile
#[derive(Component)]
struct PuzzleGoal;

/// Marker for puzzle status UI text
#[derive(Component)]
struct PuzzleStatusText;

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
    send_tx: Option<ClientMessageSender>,
    /// Channel to receive messages from the network task
    recv_rx: Option<ServerMessageReceiver>,
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

/// Lobby countdown time remaining
#[derive(Resource, Default)]
struct LobbyTime {
    seconds_remaining: u32,
    in_lobby: bool,
}

/// Map of remote players by UUID
#[derive(Resource, Default)]
struct RemotePlayers(std::collections::HashMap<uuid::Uuid, Entity>);

/// Current bomb states from server
#[derive(Resource, Default)]
struct BombStates(Vec<thermite_protocol::protocol::BombState>);

/// Previous bomb states for detecting explosions
#[derive(Resource, Default)]
struct PreviousBombStates(Vec<thermite_protocol::protocol::BombState>);

/// Death state tracking
#[derive(Resource, Default)]
struct DeathState {
    is_dead: bool,
    killer_id: Option<uuid::Uuid>,
    death_position: Option<Position>,
    death_time: Option<f32>,
}

/// Death overlay UI marker
#[derive(Component)]
struct DeathOverlay;

/// Death overlay text marker
#[derive(Component)]
struct DeathOverlayText;

/// Extraction state tracking
#[derive(Resource, Default)]
struct ExtractionState {
    is_extracting: bool,
    ticks_extracting: u32,
    extraction_position: Option<Position>,
}

/// Extraction progress bar UI marker
#[derive(Component)]
struct ExtractionProgressBar;

/// Puzzle completion state
#[derive(Resource, Default)]
struct PuzzleProgress {
    solved: bool,
}

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
        .init_resource::<LobbyTime>()
        .init_resource::<RemotePlayers>()
        .init_resource::<BombStates>()
        .init_resource::<PreviousBombStates>()
        .init_resource::<DeathState>()
        .init_resource::<ExtractionState>()
        .init_resource::<PuzzleProgress>()
        // Startup systems
        .add_systems(
            Startup,
            (
                load_puzzle_level,
                setup_camera,
                setup_grid,
                setup_ui,
                spawn_local_player,
                spawn_puzzle_goal,
            )
                .chain(),
        )
        // Update systems
        .add_systems(
            Update,
            (
                handle_input,
                receive_network_messages,
                update_player_transforms,
                check_puzzle_completion,
                update_health_bars,
                update_raid_timer_ui,
                update_lobby_countdown_ui,
                update_minimap,
                spawn_death_overlay,
                update_death_overlay,
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

fn load_puzzle_level(mut commands: Commands) {
    let puzzle_level = puzzle::load_default_level().unwrap_or_else(|error| {
        error!("Failed to load embedded puzzle level: {}", error);
        puzzle::PuzzleLevelResource::fallback()
    });

    info!("Loaded puzzle level: {}", puzzle_level.level.name);
    commands.insert_resource(puzzle_level);
}

fn setup_camera(mut commands: Commands, puzzle_level: Res<PuzzleLevelResource>) {
    // Center camera on puzzle grid
    let center_x = (puzzle_level.level.width as f32 * TILE_SIZE) / 2.0;
    let center_y = (puzzle_level.level.height as f32 * TILE_SIZE) / 2.0;

    commands.spawn((
        Camera2d,
        Transform::from_xyz(center_x, center_y, 1000.0),
    ));
}

fn setup_grid(mut commands: Commands, puzzle_level: Res<PuzzleLevelResource>) {
    // Render puzzle grid and walls
    for y in 0..puzzle_level.level.height {
        for x in 0..puzzle_level.level.width {
            let world_x = x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let world_y = y as f32 * TILE_SIZE + TILE_SIZE / 2.0;

            let is_wall = puzzle_level.is_wall(x, y);
            let is_dark = (x + y) % 2 == 0;
            let color = if is_wall {
                Color::srgb(0.32, 0.18, 0.18) // Puzzle walls
            } else if is_dark {
                Color::srgb(0.15, 0.16, 0.18) // Darker floor tiles
            } else {
                Color::srgb(0.20, 0.22, 0.24) // Lighter floor tiles
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

            // Add subtle highlight on lighter floor tiles for depth
            if !is_dark && !is_wall {
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

fn spawn_local_player(mut commands: Commands, puzzle_level: Res<PuzzleLevelResource>) {
    // Spawn local player from puzzle level spawn point
    let spawn = puzzle_level.level.player_spawn;
    let start_pos = GridPosition {
        x: spawn.x,
        y: spawn.y,
    };

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

fn spawn_puzzle_goal(mut commands: Commands, puzzle_level: Res<PuzzleLevelResource>) {
    let goal = puzzle_level.level.goal;

    commands.spawn((
        PuzzleGoal,
        Sprite {
            color: Color::srgb(0.95, 0.82, 0.2),
            custom_size: Some(Vec2::splat(TILE_SIZE - 8.0)),
            ..default()
        },
        Transform::from_xyz(grid_to_world_x(goal.x), grid_to_world_y(goal.y), 0.8),
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

    // Puzzle objective status
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn((
                PuzzleStatusText,
                Text::new("Puzzle: reach the yellow tile"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ));
        });

    // Spawn lobby countdown UI at center (hidden by default)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-150.0), // Center offset
                    top: Val::Px(-50.0),   // Center offset
                    ..default()
                },
                padding: UiRect::all(Val::Px(20.0)),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            LobbyCountdownText,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Match starting in 5..."),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
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
    puzzle_level: Res<PuzzleLevelResource>,
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
                // Only predict if within bounds and not a puzzle wall
                if nx < puzzle_level.level.width
                    && ny < puzzle_level.level.height
                    && !puzzle_level.is_wall(nx, ny)
                {
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
    mut lobby_time: ResMut<LobbyTime>,
    mut bomb_states: ResMut<BombStates>,
    mut pending_inputs: ResMut<PendingInputs>,
    mut remote_players: ResMut<RemotePlayers>,
    mut death_state: ResMut<DeathState>,
    time: Res<Time>,
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

            ServerMessage::BombDetonation {
                bomb_id,
                position,
                blast_tiles,
                destroyed_tiles,
            } => {
                info!(
                    "Bomb {} detonated at {:?}, blast {} tiles, destroyed {} tiles",
                    bomb_id,
                    position,
                    blast_tiles.len(),
                    destroyed_tiles.len()
                );
                // TODO STORY-008: Trigger explosion animation/audio here
                // For now, just log the event - visual effects already exist as placeholder
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

            ServerMessage::PlayerDamaged {
                player_id,
                damage_amount,
                new_health,
                killer_id,
            } => {
                info!(
                    "Player {} took {} damage (health: {}), source: {:?}",
                    player_id, damage_amount, new_health, killer_id
                );
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

                // Check if it's our local player who died
                if let Some(local_player_id) = network.player_id {
                    if player_id == local_player_id {
                        death_state.is_dead = true;
                        death_state.killer_id = killer_id;
                        death_state.death_position = Some(position);
                        death_state.death_time = Some(time.elapsed_secs());
                        info!("Local player died! Triggering death overlay");
                    }
                }
            }

            ServerMessage::PlayerExtracted {
                player_id,
                position,
            } => {
                info!(
                    "Player {} extracted at {:?}",
                    player_id, position
                );

                // Check if it's our local player who extracted
                if let Some(local_player_id) = network.player_id {
                    if player_id == local_player_id {
                        info!("Local player successfully extracted!");
                        // TODO: Show extraction success screen, return to lobby
                    }
                }
            }

            ServerMessage::MatchEnded { reason } => {
                info!("Match ended: {:?}", reason);
            }

            ServerMessage::Pong { timestamp } => {
                info!("Pong: {}", timestamp);
            }

            ServerMessage::LobbyCountdown { seconds_remaining } => {
                lobby_time.seconds_remaining = seconds_remaining;
                lobby_time.in_lobby = true;
            }

            ServerMessage::PlayerDisconnected { player_id } => {
                info!("Player {} disconnected from match", player_id);
                // Remove from remote players if present
                if let Some(entity) = remote_players.0.remove(&player_id) {
                    commands.entity(entity).despawn_recursive();
                }
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

fn check_puzzle_completion(
    puzzle_level: Res<PuzzleLevelResource>,
    mut puzzle_progress: ResMut<PuzzleProgress>,
    local_player_query: Query<&PredictedPosition, With<LocalPlayer>>,
    mut puzzle_status_query: Query<&mut Text, With<PuzzleStatusText>>,
) {
    if puzzle_progress.solved {
        return;
    }

    let Ok(predicted_position) = local_player_query.single() else {
        return;
    };

    let goal = puzzle_level.level.goal;
    if predicted_position.x == goal.x && predicted_position.y == goal.y {
        puzzle_progress.solved = true;
        info!("Puzzle solved: reached goal ({}, {})", goal.x, goal.y);

        if let Ok(mut status_text) = puzzle_status_query.single_mut() {
            **status_text = "Puzzle solved! Press R to retry soon".to_string();
        }
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
    mut text_query: Query<(&mut Text, &mut TextColor)>,
    mut timer_bg_query: Query<&mut BackgroundColor, With<RaidTimerText>>,
    children_query: Query<&Children>,
    time: Res<Time>,
) {
    if !time_remaining.is_changed() && time_remaining.0 > 60000 {
        return;
    }

    let total_seconds = time_remaining.0 / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    // Determine color based on remaining time
    let (text_color, bg_pulse) = if total_seconds <= 60 {
        // Under 1 minute: red text with pulsing background
        let pulse = (time.elapsed_secs() * 3.0).sin() * 0.5 + 0.5; // Pulse between 0 and 1
        let bg_alpha = 0.7 + (pulse * 0.3); // Background pulses between 0.7 and 1.0 alpha
        (Color::srgb(1.0, 0.3, 0.3), bg_alpha)
    } else {
        // Normal: white text
        (Color::srgb(1.0, 1.0, 1.0), 0.7)
    };

    for timer_entity in timer_query.iter() {
        // Update background color for pulsing effect
        if let Ok(mut bg_color) = timer_bg_query.get_mut(timer_entity) {
            *bg_color = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, bg_pulse));
        }

        // Update text content and color
        if let Ok(children) = children_query.get(timer_entity) {
            for child in children.iter() {
                if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                    **text = format!("{}:{:02}", minutes, seconds);
                    *color = TextColor(text_color);
                }
            }
        }
    }
}

fn update_lobby_countdown_ui(
    lobby_time: Res<LobbyTime>,
    countdown_query: Query<Entity, With<LobbyCountdownText>>,
    mut text_query: Query<&mut Text>,
    mut node_query: Query<&mut Node, With<LobbyCountdownText>>,
    children_query: Query<&Children>,
) {
    for countdown_entity in countdown_query.iter() {
        // Show/hide lobby countdown based on lobby state
        if let Ok(mut node) = node_query.get_mut(countdown_entity) {
            if lobby_time.in_lobby && lobby_time.seconds_remaining > 0 {
                node.display = Display::Flex;
            } else {
                node.display = Display::None;
            }
        }

        // Update text content
        if let Ok(children) = children_query.get(countdown_entity) {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = format!("Match starting in {}...", lobby_time.seconds_remaining);
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

/// Spawn death overlay UI when player dies
fn spawn_death_overlay(
    mut commands: Commands,
    death_state: Res<DeathState>,
    overlay_query: Query<Entity, With<DeathOverlay>>,
) {
    // Only spawn if dead and overlay doesn't exist
    if !death_state.is_dead || !overlay_query.is_empty() {
        return;
    }

    // Spawn full-screen semi-transparent overlay
    commands
        .spawn((
            DeathOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // "YOU ARE DEAD" text
            parent.spawn((
                DeathOverlayText,
                Text::new("YOU ARE DEAD"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.1, 0.1)),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Killer info (if available)
            if let Some(killer_id) = death_state.killer_id {
                parent.spawn((
                    Text::new(format!("Killed by: {}", killer_id)),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            } else {
                parent.spawn((
                    Text::new("Suicide"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }

            // Timer countdown text
            parent.spawn((
                DeathOverlayText,
                Text::new("Returning to stash in 5s..."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::all(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Return to Stash button placeholder
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("Return to Stash"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        });

    info!("Death overlay spawned");
}

/// Update death overlay countdown timer
fn update_death_overlay(
    death_state: Res<DeathState>,
    time: Res<Time>,
    mut text_query: Query<&mut Text, With<DeathOverlayText>>,
) {
    if !death_state.is_dead {
        return;
    }

    let Some(death_time) = death_state.death_time else {
        return;
    };

    const FREEZE_DURATION: f32 = 5.0; // 5 seconds
    let elapsed = time.elapsed_secs() - death_time;
    let remaining = (FREEZE_DURATION - elapsed).max(0.0);

    // Update countdown timer text (second text element)
    for (i, mut text) in text_query.iter_mut().enumerate() {
        if i == 1 {
            // Second text is the countdown
            **text = format!("Returning to stash in {:.0}s...", remaining);
        }
    }

    // TODO: After timer expires, disconnect and return to stash UI
    // This will be implemented when lobby/stash system exists (Sprint 5)
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
