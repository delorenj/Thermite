//! Map Loading and Caching System
//!
//! Provides efficient map template loading and caching for fast match creation.
//! Templates are loaded from disk at Game Server startup and cached in memory.
//! Each match gets a cloned Grid instance so modifications (destructible blocks)
//! don't affect the cached template.

use crate::map_system::Grid;
use crate::map_template::MapTemplate;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Thread-safe map template cache
///
/// Stores loaded map templates in memory for fast access during match creation.
/// Templates are keyed by their map_id (derived from filename without extension).
///
/// # Example
///
/// ```ignore
/// use thermite_server::map_cache::MapCache;
///
/// let cache = MapCache::new();
/// cache.load_all_from_directory("maps/")?;
///
/// // Get a template for inspection
/// let template = cache.get_template("factory_01")?;
///
/// // Clone a grid for a new match (with procedural variation)
/// let match_grid = cache.clone_for_match("factory_01", Some(12345))?;
/// ```
#[derive(Debug)]
pub struct MapCache {
    /// Cached templates keyed by map_id
    templates: RwLock<HashMap<String, MapTemplate>>,
    /// Directory where templates are loaded from (for hot-reload)
    maps_directory: RwLock<Option<PathBuf>>,
}

impl Default for MapCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MapCache {
    /// Create a new empty MapCache
    pub fn new() -> Self {
        MapCache {
            templates: RwLock::new(HashMap::new()),
            maps_directory: RwLock::new(None),
        }
    }

    /// Load all map templates from a directory
    ///
    /// Scans the directory for JSON files and loads each as a MapTemplate.
    /// The map_id is derived from the filename (e.g., "factory_01.json" -> "factory_01").
    ///
    /// # Arguments
    ///
    /// * `directory` - Path to the maps directory
    ///
    /// # Returns
    ///
    /// Number of templates successfully loaded
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be read or no valid templates found
    pub fn load_all_from_directory<P: AsRef<Path>>(
        &self,
        directory: P,
    ) -> Result<usize, CacheError> {
        let dir_path = directory.as_ref();

        if !dir_path.exists() {
            return Err(CacheError::DirectoryNotFound(
                dir_path.display().to_string(),
            ));
        }

        if !dir_path.is_dir() {
            return Err(CacheError::NotADirectory(dir_path.display().to_string()));
        }

        // Store directory path for potential hot-reload
        {
            let mut maps_dir = self.maps_directory.write().unwrap();
            *maps_dir = Some(dir_path.to_path_buf());
        }

        let entries = fs::read_dir(dir_path)
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        let mut loaded_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            // Extract map_id from filename
            let map_id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(id) => id.to_string(),
                None => continue,
            };

            // Load and cache the template
            match MapTemplate::load_from_file(&path) {
                Ok(template) => {
                    let mut templates = self.templates.write().unwrap();
                    templates.insert(map_id.clone(), template);
                    loaded_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{}: {}", map_id, e));
                }
            }
        }

        if loaded_count == 0 {
            if errors.is_empty() {
                return Err(CacheError::NoTemplatesFound(
                    dir_path.display().to_string(),
                ));
            } else {
                return Err(CacheError::AllTemplatesFailed(errors.join("; ")));
            }
        }

        Ok(loaded_count)
    }

    /// Get a reference to a cached template by map_id
    ///
    /// # Arguments
    ///
    /// * `map_id` - The map identifier (e.g., "factory_01")
    ///
    /// # Returns
    ///
    /// Cloned MapTemplate if found
    pub fn get_template(&self, map_id: &str) -> Result<MapTemplate, CacheError> {
        let templates = self.templates.read().unwrap();
        templates
            .get(map_id)
            .cloned()
            .ok_or_else(|| CacheError::TemplateNotFound(map_id.to_string()))
    }

    /// Clone a grid for a new match instance
    ///
    /// Generates a new Grid from the template with procedural variation.
    /// The returned Grid is independent and can be modified during the match
    /// without affecting the cached template.
    ///
    /// # Arguments
    ///
    /// * `map_id` - The map identifier
    /// * `seed` - Optional RNG seed for deterministic procedural generation
    ///
    /// # Returns
    ///
    /// A new Grid instance ready for match use
    pub fn clone_for_match(
        &self,
        map_id: &str,
        seed: Option<u64>,
    ) -> Result<Grid, CacheError> {
        let template = self.get_template(map_id)?;
        template
            .generate_grid(seed)
            .map_err(|e| CacheError::GenerationError(e.to_string()))
    }

    /// Get the number of cached templates
    pub fn template_count(&self) -> usize {
        self.templates.read().unwrap().len()
    }

    /// Get list of all cached map IDs
    pub fn list_map_ids(&self) -> Vec<String> {
        self.templates.read().unwrap().keys().cloned().collect()
    }

    /// Check if a map_id exists in the cache
    pub fn contains(&self, map_id: &str) -> bool {
        self.templates.read().unwrap().contains_key(map_id)
    }

    /// Reload all templates from the previously loaded directory
    ///
    /// This is the hot-reload functionality. It clears the cache and
    /// reloads all templates from the maps directory.
    ///
    /// # Returns
    ///
    /// Number of templates reloaded
    pub fn reload(&self) -> Result<usize, CacheError> {
        let maps_dir = {
            let dir = self.maps_directory.read().unwrap();
            dir.clone()
        };

        match maps_dir {
            Some(dir) => {
                // Clear existing cache
                {
                    let mut templates = self.templates.write().unwrap();
                    templates.clear();
                }
                // Reload from directory
                self.load_all_from_directory(&dir)
            }
            None => Err(CacheError::NoDirectoryConfigured),
        }
    }

    /// Reload a single template by map_id
    ///
    /// # Arguments
    ///
    /// * `map_id` - The map identifier to reload
    ///
    /// # Returns
    ///
    /// Ok(()) if successfully reloaded
    pub fn reload_single(&self, map_id: &str) -> Result<(), CacheError> {
        let maps_dir = {
            let dir = self.maps_directory.read().unwrap();
            dir.clone()
        };

        match maps_dir {
            Some(dir) => {
                let path = dir.join(format!("{}.json", map_id));
                if !path.exists() {
                    return Err(CacheError::TemplateNotFound(map_id.to_string()));
                }

                let template = MapTemplate::load_from_file(&path)
                    .map_err(|e| CacheError::LoadError(e.to_string()))?;

                let mut templates = self.templates.write().unwrap();
                templates.insert(map_id.to_string(), template);
                Ok(())
            }
            None => Err(CacheError::NoDirectoryConfigured),
        }
    }

    /// Insert a template directly (useful for testing)
    pub fn insert(&self, map_id: String, template: MapTemplate) {
        let mut templates = self.templates.write().unwrap();
        templates.insert(map_id, template);
    }
}

/// Thread-safe wrapper for sharing MapCache across async contexts
pub type SharedMapCache = Arc<MapCache>;

/// Create a shared map cache
pub fn create_shared_cache() -> SharedMapCache {
    Arc::new(MapCache::new())
}

/// Errors that can occur during map cache operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("Path is not a directory: {0}")]
    NotADirectory(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("No templates found in directory: {0}")]
    NoTemplatesFound(String),

    #[error("All templates failed to load: {0}")]
    AllTemplatesFailed(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Failed to load template: {0}")]
    LoadError(String),

    #[error("Failed to generate grid: {0}")]
    GenerationError(String),

    #[error("No directory configured for reload")]
    NoDirectoryConfigured,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map_template::{LootTier, Point, Zone};

    fn create_test_template(name: &str) -> MapTemplate {
        MapTemplate {
            name: name.to_string(),
            width: 10,
            height: 10,
            walls: vec![],
            spawn_points: vec![Point::new(0, 0), Point::new(9, 9)],
            extraction_points: vec![Point::new(0, 9), Point::new(9, 0)],
            loot_spawns: vec![Point::new(5, 5)],
            zones: vec![Zone {
                id: "center".to_string(),
                loot_tier: LootTier::Rare,
                area: vec![Point::new(4, 4), Point::new(5, 5)],
            }],
            destructible_zones: vec![Point::new(2, 2), Point::new(7, 7)],
            variation_percentage: 0.25,
            raid_duration_seconds: 300,
        }
    }

    #[test]
    fn test_new_cache_is_empty() {
        let cache = MapCache::new();
        assert_eq!(cache.template_count(), 0);
        assert!(cache.list_map_ids().is_empty());
    }

    #[test]
    fn test_insert_and_get_template() {
        let cache = MapCache::new();
        let template = create_test_template("test_map");

        cache.insert("test_map".to_string(), template.clone());

        assert_eq!(cache.template_count(), 1);
        assert!(cache.contains("test_map"));

        let retrieved = cache.get_template("test_map").unwrap();
        assert_eq!(retrieved.name, "test_map");
        assert_eq!(retrieved.width, 10);
        assert_eq!(retrieved.height, 10);
    }

    #[test]
    fn test_get_nonexistent_template() {
        let cache = MapCache::new();
        let result = cache.get_template("nonexistent");
        assert!(matches!(result, Err(CacheError::TemplateNotFound(_))));
    }

    #[test]
    fn test_clone_for_match() {
        let cache = MapCache::new();
        let template = create_test_template("test_map");
        cache.insert("test_map".to_string(), template);

        // Generate two grids with same seed - should be identical
        let grid1 = cache.clone_for_match("test_map", Some(42)).unwrap();
        let grid2 = cache.clone_for_match("test_map", Some(42)).unwrap();

        assert_eq!(grid1.width(), 10);
        assert_eq!(grid1.height(), 10);

        // Both grids should be identical with same seed
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    grid1.get_tile_at(x, y),
                    grid2.get_tile_at(x, y),
                    "Mismatch at ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_clone_for_match_different_seeds() {
        let cache = MapCache::new();
        let mut template = create_test_template("test_map");
        // Add more destructible zones to increase chance of difference
        template.destructible_zones = (0..8)
            .map(|i| Point::new(1 + i % 8, 1 + i / 8))
            .collect();
        template.variation_percentage = 0.5;
        cache.insert("test_map".to_string(), template);

        // Generate grids with different seeds - may differ due to procedural variation
        let grid1 = cache.clone_for_match("test_map", Some(1)).unwrap();
        let grid2 = cache.clone_for_match("test_map", Some(999)).unwrap();

        // Both should have same dimensions
        assert_eq!(grid1.width(), grid2.width());
        assert_eq!(grid1.height(), grid2.height());
    }

    #[test]
    fn test_clone_for_match_nonexistent() {
        let cache = MapCache::new();
        let result = cache.clone_for_match("nonexistent", None);
        assert!(matches!(result, Err(CacheError::TemplateNotFound(_))));
    }

    #[test]
    fn test_list_map_ids() {
        let cache = MapCache::new();
        cache.insert("map_a".to_string(), create_test_template("Map A"));
        cache.insert("map_b".to_string(), create_test_template("Map B"));
        cache.insert("map_c".to_string(), create_test_template("Map C"));

        let ids = cache.list_map_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"map_a".to_string()));
        assert!(ids.contains(&"map_b".to_string()));
        assert!(ids.contains(&"map_c".to_string()));
    }

    #[test]
    fn test_contains() {
        let cache = MapCache::new();
        cache.insert("existing".to_string(), create_test_template("Existing"));

        assert!(cache.contains("existing"));
        assert!(!cache.contains("nonexistent"));
    }

    #[test]
    fn test_shared_cache() {
        let shared = create_shared_cache();
        shared.insert("test".to_string(), create_test_template("Test"));

        // Can clone Arc and access from multiple references
        let shared2 = Arc::clone(&shared);
        assert!(shared2.contains("test"));
        assert_eq!(shared2.template_count(), 1);
    }

    #[test]
    fn test_load_from_nonexistent_directory() {
        let cache = MapCache::new();
        let result = cache.load_all_from_directory("/nonexistent/path");
        assert!(matches!(result, Err(CacheError::DirectoryNotFound(_))));
    }

    #[test]
    fn test_reload_without_directory() {
        let cache = MapCache::new();
        let result = cache.reload();
        assert!(matches!(result, Err(CacheError::NoDirectoryConfigured)));
    }

    #[test]
    fn test_load_factory_01_from_maps_directory() {
        // Integration test: Load the actual factory_01.json
        let maps_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("maps");

        let cache = MapCache::new();
        let count = cache.load_all_from_directory(&maps_dir).unwrap();

        // Should load at least factory_01
        assert!(count >= 1);
        assert!(cache.contains("factory_01"));

        // Verify we can generate a grid
        let grid = cache.clone_for_match("factory_01", Some(42)).unwrap();
        assert_eq!(grid.width(), 20);
        assert_eq!(grid.height(), 20);
    }

    #[test]
    fn test_grid_independence_from_template() {
        // Verify that modifying a cloned grid doesn't affect the template
        let cache = MapCache::new();
        cache.insert("test".to_string(), create_test_template("Test"));

        // Clone a grid
        let mut grid = cache.clone_for_match("test", Some(42)).unwrap();

        // Modify the grid
        use crate::map_system::Tile;
        grid.set_tile_at(5, 5, Tile::Wall);

        // Clone another grid - should not have our modification
        let grid2 = cache.clone_for_match("test", Some(42)).unwrap();

        // The original template generates Floor at (5,5), not Wall
        // (unless procedurally placed as destructible)
        assert_eq!(grid.get_tile_at(5, 5), Some(Tile::Wall));
        // grid2 should match what template generates, not our modification
        assert_ne!(
            grid2.get_tile_at(5, 5),
            Some(Tile::Wall),
            "Template should be unaffected by grid modification"
        );
    }
}
