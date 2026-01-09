//! Player state and movement system
//!
//! This module handles player entities, positions, and movement validation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::map_system::Grid;

/// Cardinal directions for grid-based movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Apply direction to coordinates, returning new position
    /// Returns None if movement would result in negative coordinates
    pub fn apply(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        match self {
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
}

/// Player position on the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Position { x, y }
    }

    /// Apply a direction to get a new position
    /// Returns None if movement would result in negative coordinates
    pub fn step(&self, direction: Direction) -> Option<Position> {
        direction.apply(self.x, self.y).map(|(x, y)| Position { x, y })
    }
}

/// Player entity with all game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub position: Position,
    pub health: i32,
    pub is_alive: bool,
    /// Sequence number for last processed command (for client prediction reconciliation)
    pub last_processed_sequence: u64,
    /// Number of bombs remaining in inventory
    pub bombs_remaining: u32,
    /// Tick number when player last placed a bomb (for cooldown)
    pub last_bomb_placement_tick: u64,
}

impl Player {
    pub fn new(id: Uuid, spawn_position: Position) -> Self {
        Player {
            id,
            position: spawn_position,
            health: 100,
            is_alive: true,
            last_processed_sequence: 0,
            bombs_remaining: 1, // Start with 1 basic bomb
            last_bomb_placement_tick: 0,
        }
    }

    /// Check if player can move in the given direction on the grid
    pub fn can_move(&self, direction: Direction, grid: &Grid) -> bool {
        if !self.is_alive {
            return false;
        }

        match self.position.step(direction) {
            Some(new_pos) => grid.is_walkable(new_pos.x, new_pos.y),
            None => false,
        }
    }

    /// Attempt to move the player in the given direction
    /// Returns Ok(new_position) if successful, Err with reason if not
    pub fn try_move(&mut self, direction: Direction, grid: &Grid) -> Result<Position, MoveError> {
        if !self.is_alive {
            return Err(MoveError::PlayerDead);
        }

        let new_pos = self
            .position
            .step(direction)
            .ok_or(MoveError::OutOfBounds)?;

        if !grid.in_bounds(new_pos.x, new_pos.y) {
            return Err(MoveError::OutOfBounds);
        }

        if !grid.is_walkable(new_pos.x, new_pos.y) {
            return Err(MoveError::TileBlocked);
        }

        self.position = new_pos;
        Ok(new_pos)
    }

    /// Take damage and potentially die
    pub fn take_damage(&mut self, amount: i32) {
        if !self.is_alive {
            return;
        }

        self.health -= amount;
        if self.health <= 0 {
            self.health = 0;
            self.is_alive = false;
        }
    }

    /// Check if player can place a bomb at current tick
    /// Validates: alive, has bombs, and cooldown elapsed
    pub fn can_place_bomb(&self, current_tick: u64, cooldown_ticks: u64) -> bool {
        if !self.is_alive {
            return false;
        }

        if self.bombs_remaining == 0 {
            return false;
        }

        // Allow initial placement (never placed a bomb before)
        if self.last_bomb_placement_tick == 0 {
            return true;
        }

        // Check cooldown (1 second = 20 ticks at 50ms per tick)
        let ticks_since_last_placement = current_tick.saturating_sub(self.last_bomb_placement_tick);
        ticks_since_last_placement >= cooldown_ticks
    }

    /// Place a bomb, consuming one from inventory
    pub fn place_bomb(&mut self, current_tick: u64) -> Result<(), BombPlacementError> {
        if !self.is_alive {
            return Err(BombPlacementError::PlayerDead);
        }

        if self.bombs_remaining == 0 {
            return Err(BombPlacementError::NoBombsRemaining);
        }

        self.bombs_remaining -= 1;
        self.last_bomb_placement_tick = current_tick;
        Ok(())
    }
}

/// Errors that can occur during movement
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MoveError {
    #[error("Player is dead")]
    PlayerDead,
    #[error("Movement would go out of bounds")]
    OutOfBounds,
    #[error("Target tile is blocked")]
    TileBlocked,
}

/// Errors that can occur during bomb placement
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BombPlacementError {
    #[error("Player is dead")]
    PlayerDead,
    #[error("No bombs remaining in inventory")]
    NoBombsRemaining,
    #[error("Cooldown not elapsed")]
    CooldownNotElapsed,
    #[error("Tile already has a bomb")]
    TileOccupied,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map_system::Tile;

    #[test]
    fn test_direction_apply_north() {
        assert_eq!(Direction::North.apply(5, 5), Some((5, 4)));
        assert_eq!(Direction::North.apply(5, 0), None);
    }

    #[test]
    fn test_direction_apply_south() {
        assert_eq!(Direction::South.apply(5, 5), Some((5, 6)));
        assert_eq!(Direction::South.apply(5, 100), Some((5, 101)));
    }

    #[test]
    fn test_direction_apply_east() {
        assert_eq!(Direction::East.apply(5, 5), Some((6, 5)));
        assert_eq!(Direction::East.apply(100, 5), Some((101, 5)));
    }

    #[test]
    fn test_direction_apply_west() {
        assert_eq!(Direction::West.apply(5, 5), Some((4, 5)));
        assert_eq!(Direction::West.apply(0, 5), None);
    }

    #[test]
    fn test_position_step() {
        let pos = Position::new(5, 5);

        assert_eq!(pos.step(Direction::North), Some(Position::new(5, 4)));
        assert_eq!(pos.step(Direction::South), Some(Position::new(5, 6)));
        assert_eq!(pos.step(Direction::East), Some(Position::new(6, 5)));
        assert_eq!(pos.step(Direction::West), Some(Position::new(4, 5)));
    }

    #[test]
    fn test_position_step_at_origin() {
        let pos = Position::new(0, 0);

        assert_eq!(pos.step(Direction::North), None);
        assert_eq!(pos.step(Direction::West), None);
        assert_eq!(pos.step(Direction::South), Some(Position::new(0, 1)));
        assert_eq!(pos.step(Direction::East), Some(Position::new(1, 0)));
    }

    #[test]
    fn test_player_creation() {
        let id = Uuid::new_v4();
        let player = Player::new(id, Position::new(5, 5));

        assert_eq!(player.id, id);
        assert_eq!(player.position, Position::new(5, 5));
        assert_eq!(player.health, 100);
        assert!(player.is_alive);
    }

    #[test]
    fn test_player_can_move_on_floor() {
        let mut grid = Grid::new(10, 10);
        let player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        // All directions should be valid on open floor
        assert!(player.can_move(Direction::North, &grid));
        assert!(player.can_move(Direction::South, &grid));
        assert!(player.can_move(Direction::East, &grid));
        assert!(player.can_move(Direction::West, &grid));
    }

    #[test]
    fn test_player_cannot_move_into_wall() {
        let mut grid = Grid::new(10, 10);
        grid.set_tile_at(5, 4, Tile::Wall);

        let player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        assert!(!player.can_move(Direction::North, &grid));
        assert!(player.can_move(Direction::South, &grid));
    }

    #[test]
    fn test_player_cannot_move_into_destructible() {
        let mut grid = Grid::new(10, 10);
        grid.set_tile_at(6, 5, Tile::Destructible);

        let player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        assert!(!player.can_move(Direction::East, &grid));
    }

    #[test]
    fn test_player_can_move_onto_loot() {
        let mut grid = Grid::new(10, 10);
        grid.set_tile_at(5, 4, Tile::Loot);

        let player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        assert!(player.can_move(Direction::North, &grid));
    }

    #[test]
    fn test_player_can_move_onto_extraction() {
        let mut grid = Grid::new(10, 10);
        grid.set_tile_at(5, 4, Tile::Extraction);

        let player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        assert!(player.can_move(Direction::North, &grid));
    }

    #[test]
    fn test_player_try_move_success() {
        let grid = Grid::new(10, 10);
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        let result = player.try_move(Direction::North, &grid);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Position::new(5, 4));
        assert_eq!(player.position, Position::new(5, 4));
    }

    #[test]
    fn test_player_try_move_blocked() {
        let mut grid = Grid::new(10, 10);
        grid.set_tile_at(5, 4, Tile::Wall);

        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        let result = player.try_move(Direction::North, &grid);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MoveError::TileBlocked);
        assert_eq!(player.position, Position::new(5, 5)); // Position unchanged
    }

    #[test]
    fn test_player_try_move_out_of_bounds() {
        let grid = Grid::new(10, 10);
        let mut player = Player::new(Uuid::new_v4(), Position::new(0, 0));

        let result = player.try_move(Direction::North, &grid);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MoveError::OutOfBounds);
    }

    #[test]
    fn test_player_try_move_out_of_bounds_east() {
        let grid = Grid::new(10, 10);
        let mut player = Player::new(Uuid::new_v4(), Position::new(9, 5));

        let result = player.try_move(Direction::East, &grid);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MoveError::OutOfBounds);
    }

    #[test]
    fn test_dead_player_cannot_move() {
        let grid = Grid::new(10, 10);
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        player.is_alive = false;

        assert!(!player.can_move(Direction::North, &grid));

        let result = player.try_move(Direction::North, &grid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MoveError::PlayerDead);
    }

    #[test]
    fn test_player_take_damage() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        player.take_damage(30);
        assert_eq!(player.health, 70);
        assert!(player.is_alive);

        player.take_damage(70);
        assert_eq!(player.health, 0);
        assert!(!player.is_alive);
    }

    #[test]
    fn test_player_take_damage_overkill() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));

        player.take_damage(150);
        assert_eq!(player.health, 0);
        assert!(!player.is_alive);
    }

    #[test]
    fn test_dead_player_cannot_take_more_damage() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        player.is_alive = false;
        player.health = 0;

        player.take_damage(50);
        assert_eq!(player.health, 0); // No change
    }

    #[test]
    fn test_player_serialization() {
        let id = Uuid::new_v4();
        let player = Player::new(id, Position::new(5, 5));

        let json = serde_json::to_string(&player).expect("Failed to serialize");
        let deserialized: Player = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.id, id);
        assert_eq!(deserialized.position, Position::new(5, 5));
        assert_eq!(deserialized.health, 100);
        assert!(deserialized.is_alive);
    }

    // Bomb placement tests
    #[test]
    fn test_can_place_bomb_initially() {
        let player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        let cooldown_ticks = 20;

        assert!(player.can_place_bomb(0, cooldown_ticks));
    }

    #[test]
    fn test_cannot_place_bomb_when_dead() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        player.is_alive = false;
        let cooldown_ticks = 20;

        assert!(!player.can_place_bomb(100, cooldown_ticks));
    }

    #[test]
    fn test_cannot_place_bomb_without_inventory() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        player.bombs_remaining = 0;
        let cooldown_ticks = 20;

        assert!(!player.can_place_bomb(100, cooldown_ticks));
    }

    #[test]
    fn test_cannot_place_bomb_during_cooldown() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        player.last_bomb_placement_tick = 100;
        let cooldown_ticks = 20;

        // 10 ticks later - still in cooldown
        assert!(!player.can_place_bomb(110, cooldown_ticks));

        // 20 ticks later - cooldown elapsed
        assert!(player.can_place_bomb(120, cooldown_ticks));
    }

    #[test]
    fn test_place_bomb_success() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        let current_tick = 100;

        let initial_bombs = player.bombs_remaining;
        let result = player.place_bomb(current_tick);

        assert!(result.is_ok());
        assert_eq!(player.bombs_remaining, initial_bombs - 1);
        assert_eq!(player.last_bomb_placement_tick, current_tick);
    }

    #[test]
    fn test_place_bomb_when_dead() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        player.is_alive = false;

        let result = player.place_bomb(100);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BombPlacementError::PlayerDead);
    }

    #[test]
    fn test_place_bomb_without_inventory() {
        let mut player = Player::new(Uuid::new_v4(), Position::new(5, 5));
        player.bombs_remaining = 0;

        let result = player.place_bomb(100);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BombPlacementError::NoBombsRemaining);
    }
}
