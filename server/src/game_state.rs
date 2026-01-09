//! Game state management for a single match
//!
//! Manages players, the grid, bombs, and match lifecycle.

use std::collections::HashMap;
use uuid::Uuid;

use crate::map_system::{Grid, Tile};
use crate::player::{BombPlacementError, Direction, MoveError, Player, Position};
use crate::protocol::{BombState, PlayerState};

/// Configuration for a match
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Match duration in milliseconds
    pub duration_ms: u64,
    /// Tick rate in milliseconds (50ms = 20Hz)
    pub tick_rate_ms: u64,
}

impl Default for MatchConfig {
    fn default() -> Self {
        MatchConfig {
            duration_ms: 5 * 60 * 1000, // 5 minutes
            tick_rate_ms: 50,           // 20Hz
        }
    }
}

/// Detonation event data
pub type DetonationEvent = (Uuid, Position, Vec<Position>, Vec<Position>);

/// Damage event data (player_id, damage_amount, new_health, killer_id)
pub type DamageEvent = (Uuid, i32, i32, Option<Uuid>);

/// Death event data (player_id, killer_id, position)
pub type DeathEvent = (Uuid, Option<Uuid>, Position);

/// Game state for a single match
#[derive(Debug)]
pub struct GameState {
    /// The match ID
    pub match_id: Uuid,
    /// Current tick number
    pub tick: u64,
    /// The game grid
    pub grid: Grid,
    /// All players in the match
    pub players: HashMap<Uuid, Player>,
    /// Active bombs
    pub bombs: HashMap<Uuid, Bomb>,
    /// Match configuration
    pub config: MatchConfig,
    /// Remaining time in milliseconds
    pub time_remaining_ms: u64,
    /// Whether the match is active
    pub is_active: bool,
    /// Detonation events from this tick (bomb_id, position, blast_tiles, destroyed_tiles)
    pub pending_detonations: Vec<DetonationEvent>,
    /// Damage events from this tick (player_id, damage_amount, new_health, killer_id)
    pub pending_damage_events: Vec<DamageEvent>,
    /// Death events from this tick (player_id, killer_id, position)
    pub pending_death_events: Vec<DeathEvent>,
}

/// A bomb entity
#[derive(Debug, Clone)]
pub struct Bomb {
    pub id: Uuid,
    pub position: Position,
    pub owner_id: Uuid,
    /// Ticks remaining until detonation
    pub ticks_remaining: u32,
    /// Blast range in tiles
    pub range: u32,
}

impl Bomb {
    pub fn new(owner_id: Uuid, position: Position, fuse_ticks: u32, range: u32) -> Self {
        Bomb {
            id: Uuid::new_v4(),
            position,
            owner_id,
            ticks_remaining: fuse_ticks,
            range,
        }
    }

    /// Returns time remaining in milliseconds (assuming 50ms tick rate)
    pub fn timer_ms(&self, tick_rate_ms: u64) -> u32 {
        (self.ticks_remaining as u64 * tick_rate_ms) as u32
    }
}

impl GameState {
    /// Create a new game state with the given grid
    pub fn new(match_id: Uuid, grid: Grid, config: MatchConfig) -> Self {
        let time_remaining_ms = config.duration_ms;
        GameState {
            match_id,
            tick: 0,
            grid,
            players: HashMap::new(),
            bombs: HashMap::new(),
            config,
            time_remaining_ms,
            is_active: true,
            pending_detonations: Vec::new(),
            pending_damage_events: Vec::new(),
            pending_death_events: Vec::new(),
        }
    }

    /// Add a player to the match at the given spawn position
    pub fn add_player(&mut self, player_id: Uuid, spawn_position: Position) -> Result<(), String> {
        if self.players.contains_key(&player_id) {
            return Err("Player already in match".to_string());
        }

        if !self.grid.is_walkable(spawn_position.x, spawn_position.y) {
            return Err("Invalid spawn position".to_string());
        }

        let player = Player::new(player_id, spawn_position);
        self.players.insert(player_id, player);
        Ok(())
    }

    /// Remove a player from the match
    pub fn remove_player(&mut self, player_id: &Uuid) -> Option<Player> {
        self.players.remove(player_id)
    }

    /// Get a player by ID
    pub fn get_player(&self, player_id: &Uuid) -> Option<&Player> {
        self.players.get(player_id)
    }

    /// Get a mutable player by ID
    pub fn get_player_mut(&mut self, player_id: &Uuid) -> Option<&mut Player> {
        self.players.get_mut(player_id)
    }

    /// Process a movement command for a player
    pub fn process_move(
        &mut self,
        player_id: &Uuid,
        direction: Direction,
        sequence: u64,
    ) -> Result<Position, MoveError> {
        let player = self
            .players
            .get_mut(player_id)
            .ok_or(MoveError::PlayerDead)?;

        let result = player.try_move(direction, &self.grid)?;
        player.last_processed_sequence = sequence;
        Ok(result)
    }

    /// Process a bomb placement command for a player
    pub fn place_bomb(
        &mut self,
        player_id: &Uuid,
        sequence: u64,
    ) -> Result<Uuid, BombPlacementError> {
        // Cooldown: 1 second = 20 ticks at 50ms per tick
        let cooldown_ticks = 20;

        // Get player and validate they can place a bomb
        let player = self
            .players
            .get_mut(player_id)
            .ok_or(BombPlacementError::PlayerDead)?;

        // Check if player can place bomb (alive, has bombs, cooldown elapsed)
        if !player.can_place_bomb(self.tick, cooldown_ticks) {
            let ticks_since_last = self.tick.saturating_sub(player.last_bomb_placement_tick);
            if ticks_since_last < cooldown_ticks {
                return Err(BombPlacementError::CooldownNotElapsed);
            }
            if player.bombs_remaining == 0 {
                return Err(BombPlacementError::NoBombsRemaining);
            }
            if !player.is_alive {
                return Err(BombPlacementError::PlayerDead);
            }
        }

        let position = player.position;

        // Check if tile already has a bomb
        if self.bombs.values().any(|b| b.position == position) {
            return Err(BombPlacementError::TileOccupied);
        }

        // Place the bomb
        player.place_bomb(self.tick)?;
        player.last_processed_sequence = sequence;

        // Create bomb with 3 second timer (60 ticks at 50ms per tick)
        let fuse_ticks = 60;
        let range = 2; // Basic bomb range
        let bomb = Bomb::new(*player_id, position, fuse_ticks, range);
        let bomb_id = bomb.id;

        self.bombs.insert(bomb_id, bomb);

        Ok(bomb_id)
    }

    /// Advance the game state by one tick
    pub fn tick(&mut self) {
        if !self.is_active {
            return;
        }

        self.tick += 1;

        // Clear previous tick's events
        self.pending_detonations.clear();
        self.pending_damage_events.clear();
        self.pending_death_events.clear();

        // Decrement timer
        let tick_ms = self.config.tick_rate_ms;
        if self.time_remaining_ms > tick_ms {
            self.time_remaining_ms -= tick_ms;
        } else {
            self.time_remaining_ms = 0;
            self.is_active = false;
        }

        // Process bombs and capture detonation events
        let detonations = self.update_bombs();
        self.pending_detonations = detonations;
    }

    /// Calculate blast pattern from bomb position
    /// Returns (blast_tiles, destroyed_tiles) tuple
    fn calculate_blast(&self, position: Position, range: u32) -> (Vec<Position>, Vec<Position>) {
        let mut blast_tiles = vec![position]; // Bomb position itself
        let mut destroyed_tiles = Vec::new();

        // Four cardinal directions
        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];

        for direction in directions.iter() {
            let mut current_pos = position;

            // Propagate blast in this direction up to range
            for _ in 0..range {
                // Try to step in this direction
                if let Some(next_pos) = current_pos.step(*direction) {
                    // Check if in bounds
                    if !self.grid.in_bounds(next_pos.x, next_pos.y) {
                        break;
                    }

                    if let Some(tile) = self.grid.get_tile_at(next_pos.x, next_pos.y) {
                        match tile {
                            Tile::Wall => {
                                // Blast stops at walls
                                break;
                            }
                            Tile::Destructible => {
                                // Blast destroys destructible and stops
                                blast_tiles.push(next_pos);
                                destroyed_tiles.push(next_pos);
                                break;
                            }
                            Tile::Floor | Tile::Loot | Tile::Extraction => {
                                // Blast continues through open tiles
                                blast_tiles.push(next_pos);
                                current_pos = next_pos;
                            }
                        }
                    } else {
                        // Invalid tile position
                        break;
                    }
                } else {
                    // Out of bounds
                    break;
                }
            }
        }

        (blast_tiles, destroyed_tiles)
    }

    /// Update bomb timers and handle detonations
    fn update_bombs(&mut self) -> Vec<(Uuid, Position, Vec<Position>, Vec<Position>)> {
        let mut detonation_events = Vec::new();
        let mut bombs_to_remove = Vec::new();
        let mut chain_reaction_bombs = Vec::new();

        // Decrement timers and collect detonations
        for (bomb_id, bomb) in self.bombs.iter_mut() {
            if bomb.ticks_remaining > 0 {
                bomb.ticks_remaining -= 1;
            }
            if bomb.ticks_remaining == 0 {
                bombs_to_remove.push(*bomb_id);
            }
        }

        // Process detonations
        for bomb_id in bombs_to_remove {
            if let Some(bomb) = self.bombs.remove(&bomb_id) {
                // Calculate blast pattern
                let (blast_tiles, destroyed_tiles) = self.calculate_blast(bomb.position, bomb.range);

                // Remove destructible tiles from grid
                for pos in &destroyed_tiles {
                    self.grid.set_tile_at(pos.x, pos.y, Tile::Floor);
                }

                // Apply damage to players in blast area
                for player in self.players.values_mut() {
                    if player.is_alive && blast_tiles.contains(&player.position) {
                        const BOMB_DAMAGE: i32 = 100; // Basic bomb is one-hit kill
                        let position_at_death = player.position;
                        player.take_damage(BOMB_DAMAGE);

                        // Track damage event
                        self.pending_damage_events.push((
                            player.id,
                            BOMB_DAMAGE,
                            player.health,
                            Some(bomb.owner_id),
                        ));

                        // Check if player died from this damage
                        if !player.is_alive {
                            self.pending_death_events.push((
                                player.id,
                                Some(bomb.owner_id),
                                position_at_death,
                            ));
                        }
                    }
                }

                // Check for chain reactions (other bombs in blast area)
                for other_bomb_id in self.bombs.keys() {
                    let other_bomb = &self.bombs[other_bomb_id];
                    if blast_tiles.contains(&other_bomb.position) {
                        chain_reaction_bombs.push(*other_bomb_id);
                    }
                }

                // Store detonation event for broadcasting
                detonation_events.push((bomb_id, bomb.position, blast_tiles, destroyed_tiles));
            }
        }

        // Trigger chain reactions by setting timer to 0
        for bomb_id in chain_reaction_bombs {
            if let Some(bomb) = self.bombs.get_mut(&bomb_id) {
                bomb.ticks_remaining = 0;
            }
        }

        detonation_events
    }

    /// Get all player states for state update broadcast
    pub fn get_player_states(&self) -> Vec<PlayerState> {
        self.players
            .values()
            .map(|p| PlayerState {
                id: p.id,
                position: p.position,
                health: p.health,
                is_alive: p.is_alive,
            })
            .collect()
    }

    /// Get all bomb states for state update broadcast
    pub fn get_bomb_states(&self) -> Vec<BombState> {
        self.bombs
            .values()
            .map(|b| BombState {
                id: b.id,
                position: b.position,
                owner_id: b.owner_id,
                timer_ms: b.timer_ms(self.config.tick_rate_ms),
            })
            .collect()
    }

    /// Get count of alive players
    pub fn alive_player_count(&self) -> usize {
        self.players.values().filter(|p| p.is_alive).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> GameState {
        let grid = Grid::new(20, 20);
        GameState::new(Uuid::new_v4(), grid, MatchConfig::default())
    }

    #[test]
    fn test_game_state_creation() {
        let state = create_test_state();

        assert_eq!(state.tick, 0);
        assert!(state.players.is_empty());
        assert!(state.bombs.is_empty());
        assert!(state.is_active);
        assert_eq!(state.time_remaining_ms, 5 * 60 * 1000);
    }

    #[test]
    fn test_add_player() {
        let mut state = create_test_state();
        let player_id = Uuid::new_v4();

        let result = state.add_player(player_id, Position::new(5, 5));
        assert!(result.is_ok());
        assert_eq!(state.players.len(), 1);

        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.position, Position::new(5, 5));
    }

    #[test]
    fn test_add_player_duplicate() {
        let mut state = create_test_state();
        let player_id = Uuid::new_v4();

        state.add_player(player_id, Position::new(5, 5)).unwrap();
        let result = state.add_player(player_id, Position::new(6, 6));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already in match"));
    }

    #[test]
    fn test_add_player_invalid_spawn() {
        let mut state = create_test_state();
        state.grid.set_tile_at(5, 5, crate::map_system::Tile::Wall);

        let result = state.add_player(Uuid::new_v4(), Position::new(5, 5));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid spawn"));
    }

    #[test]
    fn test_process_move_success() {
        let mut state = create_test_state();
        let player_id = Uuid::new_v4();
        state.add_player(player_id, Position::new(5, 5)).unwrap();

        let result = state.process_move(&player_id, Direction::North, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Position::new(5, 4));

        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.position, Position::new(5, 4));
        assert_eq!(player.last_processed_sequence, 1);
    }

    #[test]
    fn test_process_move_blocked() {
        let mut state = create_test_state();
        let player_id = Uuid::new_v4();
        state.add_player(player_id, Position::new(5, 5)).unwrap();
        state.grid.set_tile_at(5, 4, crate::map_system::Tile::Wall);

        let result = state.process_move(&player_id, Direction::North, 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MoveError::TileBlocked);

        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.position, Position::new(5, 5)); // Unchanged
    }

    #[test]
    fn test_process_move_invalid_player() {
        let mut state = create_test_state();
        let fake_id = Uuid::new_v4();

        let result = state.process_move(&fake_id, Direction::North, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_tick_decrements_timer() {
        let mut state = create_test_state();
        let initial_time = state.time_remaining_ms;

        state.tick();

        assert_eq!(state.tick, 1);
        assert_eq!(
            state.time_remaining_ms,
            initial_time - state.config.tick_rate_ms
        );
    }

    #[test]
    fn test_tick_timer_expiry() {
        let mut state = create_test_state();
        state.time_remaining_ms = 50; // One tick remaining

        state.tick();

        assert_eq!(state.time_remaining_ms, 0);
        assert!(!state.is_active);
    }

    #[test]
    fn test_inactive_state_doesnt_tick() {
        let mut state = create_test_state();
        state.is_active = false;
        let initial_tick = state.tick;

        state.tick();

        assert_eq!(state.tick, initial_tick);
    }

    #[test]
    fn test_get_player_states() {
        let mut state = create_test_state();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        state.add_player(id1, Position::new(1, 1)).unwrap();
        state.add_player(id2, Position::new(10, 10)).unwrap();

        let player_states = state.get_player_states();
        assert_eq!(player_states.len(), 2);
    }

    #[test]
    fn test_alive_player_count() {
        let mut state = create_test_state();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        state.add_player(id1, Position::new(1, 1)).unwrap();
        state.add_player(id2, Position::new(10, 10)).unwrap();

        assert_eq!(state.alive_player_count(), 2);

        // Kill one player
        state.get_player_mut(&id1).unwrap().is_alive = false;

        assert_eq!(state.alive_player_count(), 1);
    }

    #[test]
    fn test_bomb_timer() {
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 60, 2);

        assert_eq!(bomb.timer_ms(50), 3000); // 60 ticks * 50ms = 3000ms
    }

    #[test]
    fn test_bomb_detonation() {
        let mut state = create_test_state();
        let owner_id = Uuid::new_v4();
        state.add_player(owner_id, Position::new(5, 5)).unwrap();

        // Add a bomb with 2 ticks remaining
        let bomb = Bomb::new(owner_id, Position::new(5, 5), 2, 2);
        let bomb_id = bomb.id;
        state.bombs.insert(bomb_id, bomb);

        // Tick once - bomb should still exist
        state.tick();
        assert!(state.bombs.contains_key(&bomb_id));

        // Tick twice - bomb should detonate and be removed
        state.tick();
        assert!(!state.bombs.contains_key(&bomb_id));
    }

    // Blast propagation tests
    #[test]
    fn test_blast_pattern_cross_shape() {
        let state = create_test_state();
        let (blast_tiles, destroyed_tiles) = state.calculate_blast(Position::new(5, 5), 2);

        // Should include: center + 2 in each direction (if no walls)
        // Center (5,5) + North (5,4), (5,3) + South (5,6), (5,7) + East (6,5), (7,5) + West (4,5), (3,5)
        assert_eq!(blast_tiles.len(), 9); // 1 center + 2*4 directions
        assert!(blast_tiles.contains(&Position::new(5, 5))); // Center
        assert!(blast_tiles.contains(&Position::new(5, 4))); // North 1
        assert!(blast_tiles.contains(&Position::new(5, 3))); // North 2
        assert!(blast_tiles.contains(&Position::new(5, 6))); // South 1
        assert!(blast_tiles.contains(&Position::new(5, 7))); // South 2
        assert!(destroyed_tiles.is_empty()); // No destructibles on open floor
    }

    #[test]
    fn test_blast_stops_at_wall() {
        let mut state = create_test_state();
        // Place wall north of bomb
        state.grid.set_tile_at(5, 4, crate::map_system::Tile::Wall);

        let (blast_tiles, _) = state.calculate_blast(Position::new(5, 5), 2);

        // North should be blocked, but other directions should work
        assert!(!blast_tiles.contains(&Position::new(5, 4))); // Wall blocks
        assert!(blast_tiles.contains(&Position::new(5, 6))); // South works
        assert!(blast_tiles.contains(&Position::new(6, 5))); // East works
        assert!(blast_tiles.contains(&Position::new(4, 5))); // West works
    }

    #[test]
    fn test_blast_destroys_destructible() {
        let mut state = create_test_state();
        // Place destructible east of bomb
        state.grid.set_tile_at(6, 5, crate::map_system::Tile::Destructible);

        let (blast_tiles, destroyed_tiles) = state.calculate_blast(Position::new(5, 5), 2);

        // Destructible should be in blast
        assert!(blast_tiles.contains(&Position::new(6, 5)));
        assert!(destroyed_tiles.contains(&Position::new(6, 5)));

        // Blast should stop at destructible (not reach 7,5)
        assert!(!blast_tiles.contains(&Position::new(7, 5)));
    }

    #[test]
    fn test_blast_removes_destructible_from_grid() {
        let mut state = create_test_state();
        // Place destructible east of bomb
        state.grid.set_tile_at(6, 5, crate::map_system::Tile::Destructible);

        // Add bomb at center
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Verify destructible exists
        assert_eq!(
            state.grid.get_tile_at(6, 5),
            Some(crate::map_system::Tile::Destructible)
        );

        // Tick to detonate
        state.tick();

        // Destructible should be replaced with floor
        assert_eq!(
            state.grid.get_tile_at(6, 5),
            Some(crate::map_system::Tile::Floor)
        );
    }

    #[test]
    fn test_blast_respects_range() {
        let state = create_test_state();

        // Range 1 blast
        let (blast_tiles_1, _) = state.calculate_blast(Position::new(5, 5), 1);
        assert_eq!(blast_tiles_1.len(), 5); // Center + 1 in each direction

        // Range 3 blast
        let (blast_tiles_3, _) = state.calculate_blast(Position::new(5, 5), 3);
        assert_eq!(blast_tiles_3.len(), 13); // Center + 3 in each direction
    }

    #[test]
    fn test_blast_handles_edge_of_grid() {
        let state = create_test_state();

        // Bomb at top-left corner
        let (blast_tiles, _) = state.calculate_blast(Position::new(0, 0), 2);

        // Should only blast south and east (not north/west out of bounds)
        assert!(blast_tiles.contains(&Position::new(0, 0))); // Center
        assert!(blast_tiles.contains(&Position::new(1, 0))); // East
        assert!(blast_tiles.contains(&Position::new(2, 0))); // East
        assert!(blast_tiles.contains(&Position::new(0, 1))); // South
        assert!(blast_tiles.contains(&Position::new(0, 2))); // South
        // North and West out of bounds, shouldn't be included
    }

    // Chain reaction tests
    #[test]
    fn test_chain_reaction_triggers_nearby_bomb() {
        let mut state = create_test_state();

        // Place two bombs: one detonating immediately, one with time remaining
        let bomb1 = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        let bomb2 = Bomb::new(Uuid::new_v4(), Position::new(6, 5), 10, 2); // 10 ticks left
        let bomb2_id = bomb2.id;

        state.bombs.insert(bomb1.id, bomb1);
        state.bombs.insert(bomb2_id, bomb2);

        // Verify bomb2 has 10 ticks
        assert_eq!(state.bombs.get(&bomb2_id).unwrap().ticks_remaining, 10);

        // Tick - bomb1 detonates, should trigger bomb2
        state.tick();

        // Bomb2 should now have 0 ticks (triggered by chain reaction)
        assert_eq!(state.bombs.get(&bomb2_id).unwrap().ticks_remaining, 0);

        // Another tick - bomb2 detonates
        state.tick();
        assert!(!state.bombs.contains_key(&bomb2_id));
    }

    #[test]
    fn test_chain_reaction_cascade() {
        let mut state = create_test_state();

        // Test that chain reactions can cascade: bomb1 -> bomb2 -> bomb3
        // Note: When triggered bombs detonate in the same tick, they all process together
        let bomb1 = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2); // Detonates first
        let bomb2 = Bomb::new(Uuid::new_v4(), Position::new(6, 5), 10, 2); // In range of bomb1
        let bomb2_id = bomb2.id;

        state.bombs.insert(bomb1.id, bomb1);
        state.bombs.insert(bomb2_id, bomb2);

        // Verify bomb2 has time remaining
        assert!(state.bombs.get(&bomb2_id).unwrap().ticks_remaining > 0);

        // Tick 1: bomb1 detonates, triggers bomb2
        state.tick();

        // bomb1 should be removed, bomb2 should have timer set to 0
        assert_eq!(state.bombs.len(), 1);
        assert_eq!(state.bombs.get(&bomb2_id).unwrap().ticks_remaining, 0);

        // Tick 2: bomb2 detonates
        state.tick();
        assert_eq!(state.bombs.len(), 0);
    }

    #[test]
    fn test_detonation_events_captured() {
        let mut state = create_test_state();

        // Add bomb
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        let bomb_id = bomb.id;
        state.bombs.insert(bomb_id, bomb);

        // Tick to detonate
        state.tick();

        // Check detonation event was captured
        assert_eq!(state.pending_detonations.len(), 1);
        let (event_bomb_id, event_pos, blast_tiles, destroyed_tiles) =
            &state.pending_detonations[0];
        assert_eq!(*event_bomb_id, bomb_id);
        assert_eq!(*event_pos, Position::new(5, 5));
        assert!(!blast_tiles.is_empty());
        assert!(destroyed_tiles.is_empty()); // No destructibles in test
    }

    #[test]
    fn test_detonation_events_cleared_each_tick() {
        let mut state = create_test_state();

        // Add bomb
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();
        assert_eq!(state.pending_detonations.len(), 1);

        // Next tick - events should be cleared
        state.tick();
        assert_eq!(state.pending_detonations.len(), 0);
    }

    #[test]
    fn test_player_takes_damage_from_bomb() {
        let mut state = create_test_state();

        // Add player at position (5, 5)
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 5))
            .expect("Failed to add player");

        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.health, 100);
        assert!(player.is_alive);

        // Place bomb at same position
        let bomb_owner_id = Uuid::new_v4();
        let bomb = Bomb::new(bomb_owner_id, Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate bomb
        state.tick();

        // Player should be dead
        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.health, 0);
        assert!(!player.is_alive);

        // Should have damage event
        assert_eq!(state.pending_damage_events.len(), 1);
        let (event_player_id, damage, new_health, killer_id) = &state.pending_damage_events[0];
        assert_eq!(*event_player_id, player_id);
        assert_eq!(*damage, 100);
        assert_eq!(*new_health, 0);
        assert_eq!(*killer_id, Some(bomb_owner_id));
    }

    #[test]
    fn test_player_takes_damage_from_blast_radius() {
        let mut state = create_test_state();

        // Add player 2 tiles north of bomb
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 3))
            .expect("Failed to add player");

        // Place bomb with range 2
        let bomb_owner_id = Uuid::new_v4();
        let bomb = Bomb::new(bomb_owner_id, Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // Player should be dead
        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.health, 0);
        assert!(!player.is_alive);

        // Should have damage event
        assert_eq!(state.pending_damage_events.len(), 1);
    }

    #[test]
    fn test_player_not_damaged_outside_blast() {
        let mut state = create_test_state();

        // Add player 3 tiles away (outside range 2)
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 2))
            .expect("Failed to add player");

        // Place bomb with range 2
        let bomb_owner_id = Uuid::new_v4();
        let bomb = Bomb::new(bomb_owner_id, Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // Player should be unharmed
        let player = state.get_player(&player_id).unwrap();
        assert_eq!(player.health, 100);
        assert!(player.is_alive);

        // No damage events
        assert_eq!(state.pending_damage_events.len(), 0);
    }

    #[test]
    fn test_multiple_players_damaged_by_bomb() {
        let mut state = create_test_state();

        // Add 3 players in blast area
        let player1_id = Uuid::new_v4();
        let player2_id = Uuid::new_v4();
        let player3_id = Uuid::new_v4();

        state
            .add_player(player1_id, Position::new(5, 5))
            .expect("Failed to add player1");
        state
            .add_player(player2_id, Position::new(5, 4))
            .expect("Failed to add player2");
        state
            .add_player(player3_id, Position::new(6, 5))
            .expect("Failed to add player3");

        // Place bomb
        let bomb_owner_id = Uuid::new_v4();
        let bomb = Bomb::new(bomb_owner_id, Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // All 3 players should be dead
        assert!(!state.get_player(&player1_id).unwrap().is_alive);
        assert!(!state.get_player(&player2_id).unwrap().is_alive);
        assert!(!state.get_player(&player3_id).unwrap().is_alive);

        // Should have 3 damage events
        assert_eq!(state.pending_damage_events.len(), 3);
    }

    #[test]
    fn test_dead_player_cannot_take_damage() {
        let mut state = create_test_state();

        // Add dead player
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 5))
            .expect("Failed to add player");

        let player = state.players.get_mut(&player_id).unwrap();
        player.is_alive = false;
        player.health = 0;

        // Place bomb at same position
        let bomb_owner_id = Uuid::new_v4();
        let bomb = Bomb::new(bomb_owner_id, Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // No damage events (dead player can't take damage)
        assert_eq!(state.pending_damage_events.len(), 0);
    }

    #[test]
    fn test_damage_events_cleared_each_tick() {
        let mut state = create_test_state();

        // Add player
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 5))
            .expect("Failed to add player");

        // Place bomb
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();
        assert_eq!(state.pending_damage_events.len(), 1);

        // Next tick - events should be cleared
        state.tick();
        assert_eq!(state.pending_damage_events.len(), 0);
    }

    #[test]
    fn test_bomb_damage_is_100() {
        let mut state = create_test_state();

        // Add player
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 5))
            .expect("Failed to add player");

        // Place bomb
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // Verify damage amount is 100
        assert_eq!(state.pending_damage_events.len(), 1);
        let (_player_id, damage, _new_health, _killer) = &state.pending_damage_events[0];
        assert_eq!(*damage, 100);
    }

    #[test]
    fn test_death_event_emitted_on_fatal_damage() {
        let mut state = create_test_state();

        // Add player
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 5))
            .expect("Failed to add player");

        // Place bomb
        let bomb_owner_id = Uuid::new_v4();
        let bomb = Bomb::new(bomb_owner_id, Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // Should have death event
        assert_eq!(state.pending_death_events.len(), 1);
        let (event_player_id, event_killer_id, event_position) = &state.pending_death_events[0];
        assert_eq!(*event_player_id, player_id);
        assert_eq!(*event_killer_id, Some(bomb_owner_id));
        assert_eq!(*event_position, Position::new(5, 5));
    }

    #[test]
    fn test_death_events_for_multiple_players() {
        let mut state = create_test_state();

        // Add 3 players
        let player1_id = Uuid::new_v4();
        let player2_id = Uuid::new_v4();
        let player3_id = Uuid::new_v4();

        state
            .add_player(player1_id, Position::new(5, 5))
            .expect("Failed to add player1");
        state
            .add_player(player2_id, Position::new(5, 4))
            .expect("Failed to add player2");
        state
            .add_player(player3_id, Position::new(6, 5))
            .expect("Failed to add player3");

        // Place bomb
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // Should have 3 death events
        assert_eq!(state.pending_death_events.len(), 3);

        // Verify all player IDs are in death events
        let death_player_ids: Vec<Uuid> = state
            .pending_death_events
            .iter()
            .map(|(id, _, _)| *id)
            .collect();

        assert!(death_player_ids.contains(&player1_id));
        assert!(death_player_ids.contains(&player2_id));
        assert!(death_player_ids.contains(&player3_id));
    }

    #[test]
    fn test_no_death_event_for_surviving_player() {
        let mut state = create_test_state();

        // Add player outside blast range
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(10, 10))
            .expect("Failed to add player");

        // Place bomb far away
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // No death events
        assert_eq!(state.pending_death_events.len(), 0);

        // Player still alive
        let player = state.get_player(&player_id).unwrap();
        assert!(player.is_alive);
    }

    #[test]
    fn test_death_events_cleared_each_tick() {
        let mut state = create_test_state();

        // Add player
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 5))
            .expect("Failed to add player");

        // Place bomb
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();
        assert_eq!(state.pending_death_events.len(), 1);

        // Next tick - events should be cleared
        state.tick();
        assert_eq!(state.pending_death_events.len(), 0);
    }

    #[test]
    fn test_death_event_records_correct_position() {
        let mut state = create_test_state();

        // Add player at specific position
        let player_id = Uuid::new_v4();
        let death_position = Position::new(7, 3);
        state
            .add_player(player_id, death_position)
            .expect("Failed to add player");

        // Place bomb at same position
        let bomb = Bomb::new(Uuid::new_v4(), death_position, 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // Death event should record position
        assert_eq!(state.pending_death_events.len(), 1);
        let (_, _, event_position) = &state.pending_death_events[0];
        assert_eq!(*event_position, death_position);
    }

    #[test]
    fn test_no_death_event_for_already_dead_player() {
        let mut state = create_test_state();

        // Add dead player
        let player_id = Uuid::new_v4();
        state
            .add_player(player_id, Position::new(5, 5))
            .expect("Failed to add player");

        let player = state.players.get_mut(&player_id).unwrap();
        player.is_alive = false;
        player.health = 0;

        // Place bomb
        let bomb = Bomb::new(Uuid::new_v4(), Position::new(5, 5), 1, 2);
        state.bombs.insert(bomb.id, bomb);

        // Tick to detonate
        state.tick();

        // No death event for already dead player
        assert_eq!(state.pending_death_events.len(), 0);
    }
}
