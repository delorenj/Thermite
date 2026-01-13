use serde::{Deserialize, Serialize};

/// Represents different tile types on the game map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    /// Solid wall - blocks movement and blast propagation
    Wall,
    /// Open floor - walkable
    Floor,
    /// Destructible block - can be destroyed by bombs
    Destructible,
    /// Loot spawn point
    Loot,
    /// Extraction zone
    Extraction,
}

impl Tile {
    /// Returns true if the tile can be walked on
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor | Tile::Loot | Tile::Extraction)
    }

    /// Returns true if the tile blocks movement
    pub fn blocks_movement(&self) -> bool {
        matches!(self, Tile::Wall | Tile::Destructible)
    }
}

/// Represents a 2D grid-based game map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Vec<Tile>>,
}

impl Grid {
    /// Creates a new grid with the specified dimensions, filled with Floor tiles
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the grid (number of columns)
    /// * `height` - Height of the grid (number of rows)
    ///
    /// # Examples
    ///
    /// ```
    /// use thermite_server::map_system::Grid;
    ///
    /// let grid = Grid::new(20, 20);
    /// assert_eq!(grid.width(), 20);
    /// assert_eq!(grid.height(), 20);
    /// ```
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![Tile::Floor; width]; height];
        Grid {
            width,
            height,
            tiles,
        }
    }

    /// Returns the width of the grid
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the grid
    pub fn height(&self) -> usize {
        self.height
    }

    /// Gets a tile at the specified coordinates
    ///
    /// Returns None if coordinates are out of bounds
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-based)
    /// * `y` - Row index (0-based)
    pub fn get_tile_at(&self, x: usize, y: usize) -> Option<Tile> {
        if x < self.width && y < self.height {
            Some(self.tiles[y][x])
        } else {
            None
        }
    }

    /// Sets a tile at the specified coordinates
    ///
    /// Returns true if successful, false if out of bounds
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-based)
    /// * `y` - Row index (0-based)
    /// * `tile` - The tile type to set
    pub fn set_tile_at(&mut self, x: usize, y: usize, tile: Tile) -> bool {
        if x < self.width && y < self.height {
            self.tiles[y][x] = tile;
            true
        } else {
            false
        }
    }

    /// Checks if a tile at the given coordinates is walkable
    ///
    /// Returns false if coordinates are out of bounds or tile blocks movement
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        self.get_tile_at(x, y)
            .map(|tile| tile.is_walkable())
            .unwrap_or(false)
    }

    /// Checks if a tile at the given coordinates is occupied (blocks movement)
    ///
    /// Returns true if coordinates are out of bounds or tile blocks movement
    pub fn is_occupied(&self, x: usize, y: usize) -> bool {
        !self.is_walkable(x, y)
    }

    /// Checks if the given coordinates are within grid bounds
    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    /// Find all positions of a specific tile type
    ///
    /// # Arguments
    ///
    /// * `tile_type` - The tile type to search for
    ///
    /// # Returns
    ///
    /// A vector of positions where the tile type is found
    pub fn find_tiles(&self, tile_type: Tile) -> Vec<crate::player::Position> {
        use crate::player::Position;
        let mut positions = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.tiles[y][x] == tile_type {
                    positions.push(Position::new(x, y));
                }
            }
        }
        positions
    }

    /// Returns the coordinates of all valid neighboring tiles (cardinal directions)
    ///
    /// Returns up to 4 neighbors (north, south, east, west) that are within grid bounds.
    /// Does not check walkability - use is_walkable() on results if needed.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index of the center tile
    /// * `y` - Row index of the center tile
    ///
    /// # Examples
    ///
    /// ```
    /// use thermite_server::map_system::Grid;
    ///
    /// let grid = Grid::new(10, 10);
    /// let neighbors = grid.neighbors(5, 5);
    /// assert_eq!(neighbors.len(), 4); // All 4 directions valid
    ///
    /// let corner_neighbors = grid.neighbors(0, 0);
    /// assert_eq!(corner_neighbors.len(), 2); // Only east and south valid
    /// ```
    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(4);

        // North (y - 1)
        if y > 0 {
            result.push((x, y - 1));
        }

        // South (y + 1)
        if y + 1 < self.height {
            result.push((x, y + 1));
        }

        // West (x - 1)
        if x > 0 {
            result.push((x - 1, y));
        }

        // East (x + 1)
        if x + 1 < self.width {
            result.push((x + 1, y));
        }

        result
    }

    /// Returns walkable neighboring tiles (cardinal directions)
    ///
    /// Same as neighbors() but filters to only include tiles where is_walkable() returns true.
    /// Useful for pathfinding and movement validation.
    pub fn walkable_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        self.neighbors(x, y)
            .into_iter()
            .filter(|(nx, ny)| self.is_walkable(*nx, *ny))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_walkability() {
        assert!(Tile::Floor.is_walkable());
        assert!(Tile::Loot.is_walkable());
        assert!(Tile::Extraction.is_walkable());
        assert!(!Tile::Wall.is_walkable());
        assert!(!Tile::Destructible.is_walkable());
    }

    #[test]
    fn test_tile_blocks_movement() {
        assert!(Tile::Wall.blocks_movement());
        assert!(Tile::Destructible.blocks_movement());
        assert!(!Tile::Floor.blocks_movement());
        assert!(!Tile::Loot.blocks_movement());
        assert!(!Tile::Extraction.blocks_movement());
    }

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new(20, 20);
        assert_eq!(grid.width(), 20);
        assert_eq!(grid.height(), 20);
    }

    #[test]
    fn test_grid_zero_dimensions() {
        let grid = Grid::new(0, 0);
        assert_eq!(grid.width(), 0);
        assert_eq!(grid.height(), 0);
        assert!(grid.get_tile_at(0, 0).is_none());
    }

    #[test]
    fn test_grid_1x1() {
        let grid = Grid::new(1, 1);
        assert_eq!(grid.width(), 1);
        assert_eq!(grid.height(), 1);
        assert_eq!(grid.get_tile_at(0, 0), Some(Tile::Floor));
        assert!(grid.get_tile_at(1, 0).is_none());
        assert!(grid.get_tile_at(0, 1).is_none());
    }

    #[test]
    fn test_grid_large_dimensions() {
        let grid = Grid::new(100, 100);
        assert_eq!(grid.width(), 100);
        assert_eq!(grid.height(), 100);
        // Test corners
        assert_eq!(grid.get_tile_at(0, 0), Some(Tile::Floor));
        assert_eq!(grid.get_tile_at(99, 99), Some(Tile::Floor));
        // Test out of bounds
        assert!(grid.get_tile_at(100, 99).is_none());
        assert!(grid.get_tile_at(99, 100).is_none());
    }

    #[test]
    fn test_get_tile_at() {
        let grid = Grid::new(10, 10);
        // Valid coordinates
        assert_eq!(grid.get_tile_at(0, 0), Some(Tile::Floor));
        assert_eq!(grid.get_tile_at(9, 9), Some(Tile::Floor));
        assert_eq!(grid.get_tile_at(5, 5), Some(Tile::Floor));
        // Out of bounds
        assert!(grid.get_tile_at(10, 5).is_none());
        assert!(grid.get_tile_at(5, 10).is_none());
        assert!(grid.get_tile_at(10, 10).is_none());
    }

    #[test]
    fn test_set_tile_at() {
        let mut grid = Grid::new(10, 10);
        
        // Valid set operations
        assert!(grid.set_tile_at(0, 0, Tile::Wall));
        assert_eq!(grid.get_tile_at(0, 0), Some(Tile::Wall));
        
        assert!(grid.set_tile_at(5, 5, Tile::Destructible));
        assert_eq!(grid.get_tile_at(5, 5), Some(Tile::Destructible));
        
        assert!(grid.set_tile_at(9, 9, Tile::Extraction));
        assert_eq!(grid.get_tile_at(9, 9), Some(Tile::Extraction));
        
        // Out of bounds set operations
        assert!(!grid.set_tile_at(10, 5, Tile::Wall));
        assert!(!grid.set_tile_at(5, 10, Tile::Wall));
        assert!(!grid.set_tile_at(10, 10, Tile::Wall));
    }

    #[test]
    fn test_is_walkable() {
        let mut grid = Grid::new(10, 10);
        
        // Floor is walkable
        assert!(grid.is_walkable(0, 0));
        
        // Wall is not walkable
        grid.set_tile_at(1, 1, Tile::Wall);
        assert!(!grid.is_walkable(1, 1));
        
        // Destructible is not walkable
        grid.set_tile_at(2, 2, Tile::Destructible);
        assert!(!grid.is_walkable(2, 2));
        
        // Loot is walkable
        grid.set_tile_at(3, 3, Tile::Loot);
        assert!(grid.is_walkable(3, 3));
        
        // Extraction is walkable
        grid.set_tile_at(4, 4, Tile::Extraction);
        assert!(grid.is_walkable(4, 4));
        
        // Out of bounds is not walkable
        assert!(!grid.is_walkable(10, 10));
    }

    #[test]
    fn test_is_occupied() {
        let mut grid = Grid::new(10, 10);
        
        // Floor is not occupied
        assert!(!grid.is_occupied(0, 0));
        
        // Wall is occupied
        grid.set_tile_at(1, 1, Tile::Wall);
        assert!(grid.is_occupied(1, 1));
        
        // Destructible is occupied
        grid.set_tile_at(2, 2, Tile::Destructible);
        assert!(grid.is_occupied(2, 2));
        
        // Out of bounds is occupied
        assert!(grid.is_occupied(10, 10));
    }

    #[test]
    fn test_in_bounds() {
        let grid = Grid::new(10, 10);
        
        // Valid bounds
        assert!(grid.in_bounds(0, 0));
        assert!(grid.in_bounds(9, 9));
        assert!(grid.in_bounds(5, 5));
        
        // Invalid bounds
        assert!(!grid.in_bounds(10, 0));
        assert!(!grid.in_bounds(0, 10));
        assert!(!grid.in_bounds(10, 10));
        assert!(!grid.in_bounds(100, 100));
    }

    #[test]
    fn test_serialization() {
        let mut grid = Grid::new(3, 3);
        grid.set_tile_at(0, 0, Tile::Wall);
        grid.set_tile_at(1, 1, Tile::Destructible);
        grid.set_tile_at(2, 2, Tile::Extraction);
        
        // Serialize to JSON
        let json = serde_json::to_string(&grid).expect("Failed to serialize");
        
        // Deserialize back
        let deserialized: Grid = serde_json::from_str(&json).expect("Failed to deserialize");
        
        // Verify structure
        assert_eq!(deserialized.width(), 3);
        assert_eq!(deserialized.height(), 3);
        assert_eq!(deserialized.get_tile_at(0, 0), Some(Tile::Wall));
        assert_eq!(deserialized.get_tile_at(1, 1), Some(Tile::Destructible));
        assert_eq!(deserialized.get_tile_at(2, 2), Some(Tile::Extraction));
    }

    #[test]
    fn test_asymmetric_grid() {
        let grid = Grid::new(5, 10);
        assert_eq!(grid.width(), 5);
        assert_eq!(grid.height(), 10);
        
        // Test bounds for asymmetric dimensions
        assert!(grid.in_bounds(4, 9));
        assert!(!grid.in_bounds(5, 9));
        assert!(!grid.in_bounds(4, 10));
    }

    #[test]
    fn test_bounds_checking_comprehensive() {
        let grid = Grid::new(10, 15);

        // Test all corners
        assert!(grid.in_bounds(0, 0));  // top-left
        assert!(grid.in_bounds(9, 0));  // top-right
        assert!(grid.in_bounds(0, 14)); // bottom-left
        assert!(grid.in_bounds(9, 14)); // bottom-right

        // Test just outside bounds
        assert!(!grid.in_bounds(10, 0));  // right edge
        assert!(!grid.in_bounds(0, 15));  // bottom edge
        assert!(!grid.in_bounds(10, 15)); // bottom-right corner
    }

    #[test]
    fn test_neighbors_center() {
        let grid = Grid::new(10, 10);
        let neighbors = grid.neighbors(5, 5);

        // Center tile should have 4 neighbors
        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&(5, 4))); // north
        assert!(neighbors.contains(&(5, 6))); // south
        assert!(neighbors.contains(&(4, 5))); // west
        assert!(neighbors.contains(&(6, 5))); // east
    }

    #[test]
    fn test_neighbors_corner_top_left() {
        let grid = Grid::new(10, 10);
        let neighbors = grid.neighbors(0, 0);

        // Top-left corner should have 2 neighbors
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&(0, 1))); // south
        assert!(neighbors.contains(&(1, 0))); // east
    }

    #[test]
    fn test_neighbors_corner_bottom_right() {
        let grid = Grid::new(10, 10);
        let neighbors = grid.neighbors(9, 9);

        // Bottom-right corner should have 2 neighbors
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&(9, 8))); // north
        assert!(neighbors.contains(&(8, 9))); // west
    }

    #[test]
    fn test_neighbors_edge_top() {
        let grid = Grid::new(10, 10);
        let neighbors = grid.neighbors(5, 0);

        // Top edge (not corner) should have 3 neighbors
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&(5, 1))); // south
        assert!(neighbors.contains(&(4, 0))); // west
        assert!(neighbors.contains(&(6, 0))); // east
    }

    #[test]
    fn test_neighbors_edge_left() {
        let grid = Grid::new(10, 10);
        let neighbors = grid.neighbors(0, 5);

        // Left edge (not corner) should have 3 neighbors
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&(0, 4))); // north
        assert!(neighbors.contains(&(0, 6))); // south
        assert!(neighbors.contains(&(1, 5))); // east
    }

    #[test]
    fn test_walkable_neighbors() {
        let mut grid = Grid::new(5, 5);

        // Place walls around center (2, 2)
        grid.set_tile_at(2, 1, Tile::Wall);   // north
        grid.set_tile_at(1, 2, Tile::Destructible); // west
        // south and east remain floor (walkable)

        let walkable = grid.walkable_neighbors(2, 2);

        // Only 2 walkable neighbors (south and east)
        assert_eq!(walkable.len(), 2);
        assert!(walkable.contains(&(2, 3))); // south
        assert!(walkable.contains(&(3, 2))); // east

        // Wall and destructible should not be included
        assert!(!walkable.contains(&(2, 1))); // north (wall)
        assert!(!walkable.contains(&(1, 2))); // west (destructible)
    }

    #[test]
    fn test_walkable_neighbors_all_blocked() {
        let mut grid = Grid::new(3, 3);

        // Surround center with walls
        grid.set_tile_at(1, 0, Tile::Wall);
        grid.set_tile_at(1, 2, Tile::Wall);
        grid.set_tile_at(0, 1, Tile::Wall);
        grid.set_tile_at(2, 1, Tile::Wall);

        let walkable = grid.walkable_neighbors(1, 1);
        assert_eq!(walkable.len(), 0);
    }

    #[test]
    fn test_neighbors_1x1_grid() {
        let grid = Grid::new(1, 1);
        let neighbors = grid.neighbors(0, 0);

        // 1x1 grid has no neighbors
        assert_eq!(neighbors.len(), 0);
    }
}
