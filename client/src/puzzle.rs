use bevy::prelude::Resource;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PuzzlePoint {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PuzzleLevel {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub player_spawn: PuzzlePoint,
    pub goal: PuzzlePoint,
    pub walls: Vec<PuzzlePoint>,
}

#[derive(Resource, Clone)]
pub struct PuzzleLevelResource {
    pub level: PuzzleLevel,
    wall_positions: HashSet<(usize, usize)>,
}

impl PuzzleLevelResource {
    fn from_level(level: PuzzleLevel) -> Self {
        let wall_positions = level.walls.iter().map(|point| (point.x, point.y)).collect();

        Self {
            level,
            wall_positions,
        }
    }

    pub fn fallback() -> Self {
        Self::from_level(PuzzleLevel {
            id: "fallback_puzzle".to_string(),
            name: "Fallback Puzzle".to_string(),
            width: 20,
            height: 20,
            player_spawn: PuzzlePoint { x: 1, y: 1 },
            goal: PuzzlePoint { x: 18, y: 18 },
            walls: vec![],
        })
    }

    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        self.wall_positions.contains(&(x, y))
    }
}

pub fn load_default_level() -> Result<PuzzleLevelResource, serde_json::Error> {
    let level_json = include_str!("../../maps/puzzle_01.json");
    let level: PuzzleLevel = serde_json::from_str(level_json)?;
    Ok(PuzzleLevelResource::from_level(level))
}
