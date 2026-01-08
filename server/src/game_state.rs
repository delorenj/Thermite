//! Game state management for a single match
//!
//! Manages players, the grid, bombs, and match lifecycle.

use std::collections::HashMap;
use uuid::Uuid;

use crate::map_system::Grid;
use crate::player::{Direction, MoveError, Player, Position};
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

    /// Advance the game state by one tick
    pub fn tick(&mut self) {
        if !self.is_active {
            return;
        }

        self.tick += 1;

        // Decrement timer
        let tick_ms = self.config.tick_rate_ms;
        if self.time_remaining_ms > tick_ms {
            self.time_remaining_ms -= tick_ms;
        } else {
            self.time_remaining_ms = 0;
            self.is_active = false;
        }

        // Process bombs
        self.update_bombs();
    }

    /// Update bomb timers and handle detonations
    fn update_bombs(&mut self) {
        let mut detonated_bombs = Vec::new();

        for (bomb_id, bomb) in self.bombs.iter_mut() {
            if bomb.ticks_remaining > 0 {
                bomb.ticks_remaining -= 1;
            }
            if bomb.ticks_remaining == 0 {
                detonated_bombs.push(*bomb_id);
            }
        }

        // Handle detonations (simplified for now - full blast mechanics in STORY-004)
        for bomb_id in detonated_bombs {
            self.bombs.remove(&bomb_id);
        }
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
}
