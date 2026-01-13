use crate::map_system::{Grid, Tile};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::Path;

/// Represents a 2D coordinate point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }
}

/// Defines a zone on the map with risk tier and loot quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub loot_tier: LootTier,
    pub area: Vec<Point>,
}

/// Loot quality tiers based on zone risk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LootTier {
    /// Safe outer edges - 250-350 credits per raid average
    Common,
    /// Moderate risk middle zones - 500-700 credits
    Uncommon,
    /// High risk center zones - 1000-1500 credits
    Rare,
}

/// Map template defining static layout and procedural variation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTemplate {
    pub name: String,
    pub width: usize,
    pub height: usize,
    /// Static wall positions (never change)
    pub walls: Vec<Point>,
    /// Spawn point positions
    pub spawn_points: Vec<Point>,
    /// Extraction zone positions
    pub extraction_points: Vec<Point>,
    /// Loot spawn positions
    pub loot_spawns: Vec<Point>,
    /// Zones for risk-tiered loot
    pub zones: Vec<Zone>,
    /// Areas where destructible blocks can spawn (procedural variation)
    pub destructible_zones: Vec<Point>,
    /// Percentage of destructible zones to fill (0.0-1.0)
    #[serde(default = "default_variation_percentage")]
    pub variation_percentage: f32,
    /// Raid duration in seconds (defaults to 300 = 5 minutes)
    #[serde(default = "default_raid_duration_seconds")]
    pub raid_duration_seconds: u64,
}

fn default_variation_percentage() -> f32 {
    0.25 // 25% by default (within 20-30% requirement)
}

fn default_raid_duration_seconds() -> u64 {
    300 // 5 minutes by default
}

impl MapTemplate {
    /// Load a map template from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON template file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, TemplateError> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|e| TemplateError::IoError(e.to_string()))?;
        
        let template: MapTemplate = serde_json::from_str(&contents)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;
        
        template.validate()?;
        Ok(template)
    }

    /// Validate the template structure
    fn validate(&self) -> Result<(), TemplateError> {
        // Check dimensions are valid
        if self.width == 0 || self.height == 0 {
            return Err(TemplateError::InvalidTemplate(
                "Map dimensions must be greater than zero".to_string(),
            ));
        }

        // Check we have at least one spawn point
        if self.spawn_points.is_empty() {
            return Err(TemplateError::InvalidTemplate(
                "Template must have at least one spawn point".to_string(),
            ));
        }

        // Check we have at least one extraction point
        if self.extraction_points.is_empty() {
            return Err(TemplateError::InvalidTemplate(
                "Template must have at least one extraction point".to_string(),
            ));
        }

        // Validate all points are within bounds
        for point in self.walls.iter()
            .chain(self.spawn_points.iter())
            .chain(self.extraction_points.iter())
            .chain(self.loot_spawns.iter())
            .chain(self.destructible_zones.iter())
        {
            if point.x >= self.width || point.y >= self.height {
                return Err(TemplateError::InvalidTemplate(
                    format!("Point ({}, {}) is out of bounds", point.x, point.y),
                ));
            }
        }

        // Validate variation percentage is within range
        if self.variation_percentage < 0.0 || self.variation_percentage > 1.0 {
            return Err(TemplateError::InvalidTemplate(
                "Variation percentage must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate a grid from this template with procedural variation
    ///
    /// # Arguments
    ///
    /// * `seed` - Optional RNG seed for deterministic generation
    pub fn generate_grid(&self, seed: Option<u64>) -> Result<Grid, TemplateError> {
        let mut rng: Box<dyn rand::RngCore> = if let Some(s) = seed {
            Box::new(rand::rngs::StdRng::seed_from_u64(s))
        } else {
            Box::new(rand::thread_rng())
        };

        // Create base grid
        let mut grid = Grid::new(self.width, self.height);

        // Place static walls
        for wall in &self.walls {
            grid.set_tile_at(wall.x, wall.y, Tile::Wall);
        }

        // Place extraction zones
        for extract in &self.extraction_points {
            grid.set_tile_at(extract.x, extract.y, Tile::Extraction);
        }

        // Place loot spawns
        for loot in &self.loot_spawns {
            grid.set_tile_at(loot.x, loot.y, Tile::Loot);
        }

        // Procedurally place destructible blocks
        let num_destructibles = (self.destructible_zones.len() as f32 * self.variation_percentage).round() as usize;
        let mut available_zones: Vec<_> = self.destructible_zones.clone();
        available_zones.shuffle(&mut *rng);

        for point in available_zones.iter().take(num_destructibles) {
            grid.set_tile_at(point.x, point.y, Tile::Destructible);
        }

        // Validate map connectivity
        self.validate_connectivity(&grid)?;

        Ok(grid)
    }

    /// Validate that all spawn points can reach all extraction points
    fn validate_connectivity(&self, grid: &Grid) -> Result<(), TemplateError> {
        for spawn in &self.spawn_points {
            for extract in &self.extraction_points {
                if !self.has_path(grid, *spawn, *extract) {
                    return Err(TemplateError::ConnectivityError(
                        format!(
                            "Spawn ({}, {}) cannot reach Extract ({}, {})",
                            spawn.x, spawn.y, extract.x, extract.y
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// BFS pathfinding to check if a path exists between two points
    fn has_path(&self, grid: &Grid, start: Point, end: Point) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if current == end {
                return true;
            }

            // Check all 4 cardinal directions
            let neighbors = [
                (current.x.wrapping_sub(1), current.y), // left
                (current.x + 1, current.y),             // right
                (current.x, current.y.wrapping_sub(1)), // up
                (current.x, current.y + 1),             // down
            ];

            for (nx, ny) in neighbors {
                if grid.in_bounds(nx, ny) && grid.is_walkable(nx, ny) {
                    let neighbor = Point::new(nx, ny);
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        false
    }
}

/// Errors that can occur during template loading and generation
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
    
    #[error("Connectivity error: {0}")]
    ConnectivityError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(5, 10);
        assert_eq!(point.x, 5);
        assert_eq!(point.y, 10);
    }

    #[test]
    fn test_loot_tier_serialization() {
        let tier = LootTier::Rare;
        let json = serde_json::to_string(&tier).unwrap();
        let deserialized: LootTier = serde_json::from_str(&json).unwrap();
        assert_eq!(tier, deserialized);
    }

    #[test]
    fn test_template_validation_empty_dimensions() {
        let template = MapTemplate {
            name: "test".to_string(),
            width: 0,
            height: 10,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(1, 1)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.25,
            raid_duration_seconds: 300,
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn test_template_validation_no_spawns() {
        let template = MapTemplate {
            name: "test".to_string(),
            width: 10,
            height: 10,
            walls: vec![],
            spawn_points: vec![],
            extraction_points: vec![Point::new(1, 1)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.25,
            raid_duration_seconds: 300,
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn test_template_validation_no_extracts() {
        let template = MapTemplate {
            name: "test".to_string(),
            width: 10,
            height: 10,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.25,
            raid_duration_seconds: 300,
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn test_template_validation_out_of_bounds() {
        let template = MapTemplate {
            name: "test".to_string(),
            width: 10,
            height: 10,
            walls: vec![Point::new(15, 15)], // Out of bounds
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(1, 1)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.25,
            raid_duration_seconds: 300,
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn test_template_validation_invalid_variation() {
        let template = MapTemplate {
            name: "test".to_string(),
            width: 10,
            height: 10,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(1, 1)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 1.5, // Invalid
            raid_duration_seconds: 300,
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn test_simple_grid_generation() {
        let template = MapTemplate {
            name: "simple".to_string(),
            width: 5,
            height: 5,
            walls: vec![Point::new(2, 2)],
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(4, 4)],
            loot_spawns: vec![Point::new(2, 0)],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.0,
            raid_duration_seconds: 300,
        };

        let grid = template.generate_grid(Some(42)).unwrap();
        assert_eq!(grid.width(), 5);
        assert_eq!(grid.height(), 5);
        assert_eq!(grid.get_tile_at(2, 2), Some(Tile::Wall));
        assert_eq!(grid.get_tile_at(4, 4), Some(Tile::Extraction));
        assert_eq!(grid.get_tile_at(2, 0), Some(Tile::Loot));
    }

    #[test]
    fn test_procedural_destructibles() {
        let template = MapTemplate {
            name: "procedural".to_string(),
            width: 10,
            height: 10,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(9, 9)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![
                Point::new(1, 1),
                Point::new(2, 2),
                Point::new(3, 3),
                Point::new(4, 4),
            ],
            variation_percentage: 0.5, // 50% of 4 = 2 destructibles
            raid_duration_seconds: 300,
        };

        let grid = template.generate_grid(Some(42)).unwrap();
        
        // Count destructibles
        let mut destructible_count = 0;
        for point in &template.destructible_zones {
            if grid.get_tile_at(point.x, point.y) == Some(Tile::Destructible) {
                destructible_count += 1;
            }
        }
        
        // Should place approximately 2 destructibles (50% of 4)
        assert_eq!(destructible_count, 2);
    }

    #[test]
    fn test_pathfinding_simple() {
        let template = MapTemplate {
            name: "path_test".to_string(),
            width: 5,
            height: 5,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(4, 4)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.0,
            raid_duration_seconds: 300,
        };

        let grid = template.generate_grid(Some(42)).unwrap();
        assert!(template.has_path(&grid, Point::new(0, 0), Point::new(4, 4)));
    }

    #[test]
    fn test_pathfinding_blocked() {
        let template = MapTemplate {
            name: "blocked".to_string(),
            width: 5,
            height: 5,
            walls: vec![
                Point::new(2, 0),
                Point::new(2, 1),
                Point::new(2, 2),
                Point::new(2, 3),
                Point::new(2, 4),
            ], // Vertical wall blocking left from right
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(4, 4)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.0,
            raid_duration_seconds: 300,
        };

        // Should fail connectivity validation
        assert!(template.generate_grid(Some(42)).is_err());
    }

    #[test]
    fn test_connectivity_validation_success() {
        let template = MapTemplate {
            name: "connected".to_string(),
            width: 10,
            height: 10,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0), Point::new(9, 0)],
            extraction_points: vec![Point::new(0, 9), Point::new(9, 9)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![],
            variation_percentage: 0.0,
            raid_duration_seconds: 300,
        };

        // All spawns should reach all extracts
        let result = template.generate_grid(Some(42));
        assert!(result.is_ok());
    }

    #[test]
    fn test_deterministic_generation() {
        let template = MapTemplate {
            name: "deterministic".to_string(),
            width: 10,
            height: 10,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0)],
            extraction_points: vec![Point::new(9, 9)],
            loot_spawns: vec![],
            zones: vec![],
            destructible_zones: vec![
                Point::new(1, 1),
                Point::new(2, 2),
                Point::new(3, 3),
                Point::new(4, 4),
            ],
            variation_percentage: 0.5,
            raid_duration_seconds: 300,
        };

        // Generate with same seed twice
        let grid1 = template.generate_grid(Some(123)).unwrap();
        let grid2 = template.generate_grid(Some(123)).unwrap();

        // Should be identical
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(grid1.get_tile_at(x, y), grid2.get_tile_at(x, y));
            }
        }
    }

    #[test]
    fn test_zone_serialization() {
        let zone = Zone {
            id: "hot_center".to_string(),
            loot_tier: LootTier::Rare,
            area: vec![Point::new(5, 5), Point::new(6, 6)],
        };

        let json = serde_json::to_string(&zone).unwrap();
        let deserialized: Zone = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "hot_center");
        assert_eq!(deserialized.loot_tier, LootTier::Rare);
        assert_eq!(deserialized.area.len(), 2);
    }

    #[test]
    fn test_load_factory_01_template() {
        // Load the factory_01.json template from the maps directory
        let template_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("maps/factory_01.json");

        let template = MapTemplate::load_from_file(&template_path)
            .expect("Failed to load factory_01.json");

        // Verify basic structure
        assert_eq!(template.name, "Factory Floor 01");
        assert_eq!(template.width, 20);
        assert_eq!(template.height, 20);

        // Verify spawn points (8 total: 4 corners + 4 mid-edges)
        assert_eq!(template.spawn_points.len(), 8);
        assert!(template.spawn_points.contains(&Point::new(2, 2)));
        assert!(template.spawn_points.contains(&Point::new(17, 2)));
        assert!(template.spawn_points.contains(&Point::new(2, 17)));
        assert!(template.spawn_points.contains(&Point::new(17, 17)));
        assert!(template.spawn_points.contains(&Point::new(6, 3)));
        assert!(template.spawn_points.contains(&Point::new(13, 3)));
        assert!(template.spawn_points.contains(&Point::new(6, 16)));
        assert!(template.spawn_points.contains(&Point::new(13, 16)));

        // Verify extraction points (4 cardinal directions from center)
        assert_eq!(template.extraction_points.len(), 4);

        // Verify zones exist
        assert_eq!(template.zones.len(), 7);

        // Verify hot_center zone is Rare tier
        let hot_center = template.zones.iter().find(|z| z.id == "hot_center");
        assert!(hot_center.is_some());
        assert_eq!(hot_center.unwrap().loot_tier, LootTier::Rare);

        // Verify edge zones are Common tier
        let edge_nw = template.zones.iter().find(|z| z.id == "edge_nw");
        assert!(edge_nw.is_some());
        assert_eq!(edge_nw.unwrap().loot_tier, LootTier::Common);

        // Verify variation percentage is within 20-30% range
        assert!(template.variation_percentage >= 0.2 && template.variation_percentage <= 0.3);
    }

    #[test]
    fn test_generate_grid_from_factory_01() {
        let template_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("maps/factory_01.json");

        let template = MapTemplate::load_from_file(&template_path)
            .expect("Failed to load factory_01.json");

        // Generate a grid with a fixed seed for reproducibility
        let grid = template.generate_grid(Some(42))
            .expect("Failed to generate grid from factory_01");

        // Verify grid dimensions
        assert_eq!(grid.width(), 20);
        assert_eq!(grid.height(), 20);

        // Verify border walls exist (top row)
        for x in 0..20 {
            assert_eq!(grid.get_tile_at(x, 0), Some(Tile::Wall), "Top wall missing at x={}", x);
            assert_eq!(grid.get_tile_at(x, 19), Some(Tile::Wall), "Bottom wall missing at x={}", x);
        }

        // Verify border walls exist (left and right columns)
        for y in 0..20 {
            assert_eq!(grid.get_tile_at(0, y), Some(Tile::Wall), "Left wall missing at y={}", y);
            assert_eq!(grid.get_tile_at(19, y), Some(Tile::Wall), "Right wall missing at y={}", y);
        }

        // Verify extraction points are walkable
        assert!(grid.is_walkable(9, 2));   // north extraction
        assert!(grid.is_walkable(2, 9));   // west extraction
        assert!(grid.is_walkable(17, 9));  // east extraction
        assert!(grid.is_walkable(9, 17));  // south extraction

        // Verify spawn points are walkable
        assert!(grid.is_walkable(2, 2));   // NW spawn
        assert!(grid.is_walkable(17, 2));  // NE spawn
        assert!(grid.is_walkable(2, 17));  // SW spawn
        assert!(grid.is_walkable(17, 17)); // SE spawn

        // Verify all spawns can reach all extractions (pathfinding validation passes)
        // This is implicitly tested since generate_grid() succeeded
    }
}
