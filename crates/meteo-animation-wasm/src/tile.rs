use wasm_bindgen::prelude::*;
use std::collections::HashSet;

/// Tile coordinator for managing tile requests with zoom offset
#[wasm_bindgen]
pub struct TileCoordinator {
    loaded: HashSet<(u32, u32, u32)>,  // (z, x, y) of loaded tiles
    zoom_offset: i32,                   // Offset from view zoom (e.g., -2)
    min_zoom: u32,                      // Minimum data zoom
    max_zoom: u32,                      // Maximum data zoom
}

#[wasm_bindgen]
impl TileCoordinator {
    #[wasm_bindgen(constructor)]
    pub fn new(zoom_offset: i32) -> Self {
        Self {
            loaded: HashSet::new(),
            zoom_offset,
            min_zoom: 2,
            max_zoom: 8,
        }
    }

    /// Set minimum and maximum data zoom levels
    #[wasm_bindgen]
    pub fn set_zoom_range(&mut self, min_zoom: u32, max_zoom: u32) {
        self.min_zoom = min_zoom;
        self.max_zoom = max_zoom;
    }

    /// Calculate effective data zoom from view zoom
    #[wasm_bindgen]
    pub fn get_data_zoom(&self, view_zoom: u32) -> u32 {
        let adjusted = (view_zoom as i32 + self.zoom_offset).max(0) as u32;
        adjusted.max(self.min_zoom).min(self.max_zoom)
    }

    /// Check if tile is already loaded
    #[wasm_bindgen]
    pub fn is_loaded(&self, z: u32, x: u32, y: u32) -> bool {
        self.loaded.contains(&(z, x, y))
    }

    /// Mark tile as loaded
    #[wasm_bindgen]
    pub fn mark_loaded(&mut self, z: u32, x: u32, y: u32) {
        self.loaded.insert((z, x, y));
    }

    /// Clear loaded tiles (e.g., when zoom changes significantly)
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.loaded.clear();
    }

    /// Clear tiles at specific zoom level
    #[wasm_bindgen]
    pub fn clear_zoom(&mut self, z: u32) {
        self.loaded.retain(|&(tz, _, _)| tz != z);
    }

    /// Clear a specific tile (for LRU eviction sync)
    #[wasm_bindgen]
    pub fn clear_tile(&mut self, z: u32, x: u32, y: u32) {
        self.loaded.remove(&(z, x, y));
    }

    /// Get count of loaded tiles
    #[wasm_bindgen]
    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    /// Filter tile list to only unloaded tiles
    /// Input: flat array [z, x, y, z, x, y, ...]
    /// Output: flat array of unloaded tiles
    #[wasm_bindgen]
    pub fn filter_unloaded(&self, tiles: &[u32]) -> Vec<u32> {
        let mut result = Vec::new();
        for chunk in tiles.chunks(3) {
            if chunk.len() == 3 {
                let (z, x, y) = (chunk[0], chunk[1], chunk[2]);
                if !self.is_loaded(z, x, y) {
                    result.push(z);
                    result.push(x);
                    result.push(y);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let coord = TileCoordinator::new(-2);
        assert_eq!(coord.zoom_offset, -2);
        assert_eq!(coord.loaded_count(), 0);
    }

    #[test]
    fn test_get_data_zoom_offset() {
        let coord = TileCoordinator::new(-2);

        // View zoom 7 with offset -2 = data zoom 5
        assert_eq!(coord.get_data_zoom(7), 5);

        // View zoom 5 with offset -2 = data zoom 3
        assert_eq!(coord.get_data_zoom(5), 3);
    }

    #[test]
    fn test_get_data_zoom_min_clamp() {
        let coord = TileCoordinator::new(-2);

        // View zoom 3 with offset -2 = 1, but min is 2
        assert_eq!(coord.get_data_zoom(3), 2);

        // View zoom 1 with offset -2 = 0, but min is 2
        assert_eq!(coord.get_data_zoom(1), 2);
    }

    #[test]
    fn test_get_data_zoom_max_clamp() {
        let coord = TileCoordinator::new(0);

        // View zoom 10 with offset 0 = 10, but max is 8
        assert_eq!(coord.get_data_zoom(10), 8);
    }

    #[test]
    fn test_set_zoom_range() {
        let mut coord = TileCoordinator::new(-2);
        coord.set_zoom_range(3, 6);

        assert_eq!(coord.get_data_zoom(10), 6); // Max clamp
        assert_eq!(coord.get_data_zoom(3), 3);  // Min clamp
    }

    #[test]
    fn test_is_loaded_empty() {
        let coord = TileCoordinator::new(-2);
        assert!(!coord.is_loaded(5, 10, 15));
    }

    #[test]
    fn test_mark_loaded() {
        let mut coord = TileCoordinator::new(-2);
        coord.mark_loaded(5, 10, 15);

        assert!(coord.is_loaded(5, 10, 15));
        assert!(!coord.is_loaded(5, 10, 16));
        assert_eq!(coord.loaded_count(), 1);
    }

    #[test]
    fn test_mark_loaded_multiple() {
        let mut coord = TileCoordinator::new(-2);
        coord.mark_loaded(5, 10, 15);
        coord.mark_loaded(5, 11, 15);
        coord.mark_loaded(5, 10, 16);

        assert_eq!(coord.loaded_count(), 3);
    }

    #[test]
    fn test_mark_loaded_duplicate() {
        let mut coord = TileCoordinator::new(-2);
        coord.mark_loaded(5, 10, 15);
        coord.mark_loaded(5, 10, 15); // Duplicate

        assert_eq!(coord.loaded_count(), 1);
    }

    #[test]
    fn test_clear() {
        let mut coord = TileCoordinator::new(-2);
        coord.mark_loaded(5, 10, 15);
        coord.mark_loaded(5, 11, 15);

        coord.clear();

        assert_eq!(coord.loaded_count(), 0);
        assert!(!coord.is_loaded(5, 10, 15));
    }

    #[test]
    fn test_clear_zoom() {
        let mut coord = TileCoordinator::new(-2);
        coord.mark_loaded(5, 10, 15);
        coord.mark_loaded(5, 11, 15);
        coord.mark_loaded(6, 20, 30);

        coord.clear_zoom(5);

        assert_eq!(coord.loaded_count(), 1);
        assert!(!coord.is_loaded(5, 10, 15));
        assert!(coord.is_loaded(6, 20, 30));
    }

    #[test]
    fn test_filter_unloaded_all_new() {
        let coord = TileCoordinator::new(-2);
        let tiles = vec![5, 10, 15, 5, 11, 15];

        let result = coord.filter_unloaded(&tiles);

        assert_eq!(result, vec![5, 10, 15, 5, 11, 15]);
    }

    #[test]
    fn test_filter_unloaded_some_loaded() {
        let mut coord = TileCoordinator::new(-2);
        coord.mark_loaded(5, 10, 15);

        let tiles = vec![5, 10, 15, 5, 11, 15, 5, 12, 15];
        let result = coord.filter_unloaded(&tiles);

        // Should only return the two unloaded tiles
        assert_eq!(result, vec![5, 11, 15, 5, 12, 15]);
    }

    #[test]
    fn test_filter_unloaded_all_loaded() {
        let mut coord = TileCoordinator::new(-2);
        coord.mark_loaded(5, 10, 15);
        coord.mark_loaded(5, 11, 15);

        let tiles = vec![5, 10, 15, 5, 11, 15];
        let result = coord.filter_unloaded(&tiles);

        assert!(result.is_empty());
    }
}
