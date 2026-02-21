use serde::{Deserialize, Serialize};

/// Cardinal directions for grid-based movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Apply direction to coordinates, returning new position.
    /// Returns None if movement would result in negative coordinates.
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

/// Player position on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Position { x, y }
    }

    /// Apply a direction to get a new position.
    /// Returns None if movement would result in negative coordinates.
    pub fn step(&self, direction: Direction) -> Option<Position> {
        direction.apply(self.x, self.y).map(|(x, y)| Position { x, y })
    }
}
