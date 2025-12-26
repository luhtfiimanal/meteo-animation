use wasm_bindgen::prelude::*;
use std::f64::consts::PI;

/// Earth radius in meters (WGS84)
const EARTH_RADIUS: f64 = 6378137.0;
/// Maximum latitude for Web Mercator
const MAX_LAT: f64 = 85.051129;
/// Tile size in pixels
const TILE_SIZE: u32 = 256;

/// Tile coordinate (z, x, y)
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

#[wasm_bindgen]
impl TileCoord {
    #[wasm_bindgen(constructor)]
    pub fn new(z: u32, x: u32, y: u32) -> Self {
        Self { z, x, y }
    }

    /// Get geographic bounds of this tile (lon/lat)
    pub fn get_geo_bounds(&self) -> Vec<f64> {
        let n = 2_u32.pow(self.z) as f64;

        let lon_min = (self.x as f64 / n) * 360.0 - 180.0;
        let lon_max = ((self.x + 1) as f64 / n) * 360.0 - 180.0;

        let lat_max = (PI * (1.0 - 2.0 * self.y as f64 / n)).sinh().atan() * 180.0 / PI;
        let lat_min = (PI * (1.0 - 2.0 * (self.y + 1) as f64 / n)).sinh().atan() * 180.0 / PI;

        vec![lon_min, lat_min, lon_max, lat_max]
    }

    /// Get Web Mercator bounds of this tile
    pub fn get_mercator_bounds(&self) -> Vec<f64> {
        let geo = self.get_geo_bounds();
        let min = lat_lng_to_mercator(geo[1], geo[0]);
        let max = lat_lng_to_mercator(geo[3], geo[2]);
        vec![min.0, min.1, max.0, max.1]
    }
}

/// Atlas bounds in Web Mercator coordinates
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct AtlasBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[wasm_bindgen]
impl AtlasBounds {
    #[wasm_bindgen(constructor)]
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Expand bounds by ratio (1.2 = 20% buffer on each side)
    pub fn expand(&self, ratio: f64) -> Self {
        let expand_x = self.width() * (ratio - 1.0) / 2.0;
        let expand_y = self.height() * (ratio - 1.0) / 2.0;
        Self {
            min_x: self.min_x - expand_x,
            min_y: self.min_y - expand_y,
            max_x: self.max_x + expand_x,
            max_y: self.max_y + expand_y,
        }
    }

    /// Check if point is inside bounds
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

/// Convert lat/lng to Web Mercator
pub fn lat_lng_to_mercator(lat: f64, lng: f64) -> (f64, f64) {
    let x = lng * PI / 180.0 * EARTH_RADIUS;
    let lat_clamped = lat.max(-MAX_LAT).min(MAX_LAT);
    let lat_rad = lat_clamped * PI / 180.0;
    let y = (PI / 4.0 + lat_rad / 2.0).tan().ln() * EARTH_RADIUS;
    (x, y)
}

/// Convert Web Mercator to lat/lng
pub fn mercator_to_lat_lng(x: f64, y: f64) -> (f64, f64) {
    let lng = x / EARTH_RADIUS * 180.0 / PI;
    let lat = (2.0 * (y / EARTH_RADIUS).exp().atan() - PI / 2.0) * 180.0 / PI;
    (lat, lng)
}

/// Get tiles covering bounds at given zoom
pub fn get_tiles_for_bounds(bounds: &AtlasBounds, zoom: u32) -> Vec<TileCoord> {
    let n = 2_u32.pow(zoom) as f64;

    // Convert mercator bounds to lat/lng
    let (lat_min, lon_min) = mercator_to_lat_lng(bounds.min_x, bounds.min_y);
    let (lat_max, lon_max) = mercator_to_lat_lng(bounds.max_x, bounds.max_y);

    // Convert to tile coordinates
    let x_min = ((lon_min + 180.0) / 360.0 * n).floor() as i32;
    let x_max = ((lon_max + 180.0) / 360.0 * n).floor() as i32;

    let y_min = ((1.0 - (lat_max * PI / 180.0).tan().asinh() / PI) / 2.0 * n).floor() as i32;
    let y_max = ((1.0 - (lat_min * PI / 180.0).tan().asinh() / PI) / 2.0 * n).floor() as i32;

    let n_int = n as i32;
    let mut tiles = Vec::new();

    for y in y_min.max(0)..=y_max.min(n_int - 1) {
        for x in x_min.max(0)..=x_max.min(n_int - 1) {
            tiles.push(TileCoord::new(zoom, x as u32, y as u32));
        }
    }

    tiles
}

/// Dynamic atlas that resizes based on viewport
#[wasm_bindgen]
pub struct DynamicAtlas {
    data: Vec<u8>,
    width: u32,
    height: u32,
    bounds: AtlasBounds,
    buffer_ratio: f64,
    tile_zoom: u32,
    // Track which tiles are placed (for incremental updates)
    tiles_min_x: u32,
    tiles_min_y: u32,
    tiles_cols: u32,
    tiles_rows: u32,
}

#[wasm_bindgen]
impl DynamicAtlas {
    #[wasm_bindgen(constructor)]
    pub fn new(buffer_ratio: f64) -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
            bounds: AtlasBounds::new(0.0, 0.0, 1.0, 1.0),
            buffer_ratio,
            tile_zoom: 0,
            tiles_min_x: 0,
            tiles_min_y: 0,
            tiles_cols: 0,
            tiles_rows: 0,
        }
    }

    /// Update atlas to cover new viewport
    /// Returns array of tile coords that need to be loaded [z, x, y, z, x, y, ...]
    #[wasm_bindgen]
    pub fn update_viewport(
        &mut self,
        viewport_min_x: f64,
        viewport_min_y: f64,
        viewport_max_x: f64,
        viewport_max_y: f64,
        data_zoom: u32,
    ) -> Vec<u32> {
        let viewport = AtlasBounds::new(viewport_min_x, viewport_min_y, viewport_max_x, viewport_max_y);
        let expanded = viewport.expand(self.buffer_ratio);

        // Get tiles covering expanded bounds
        let tiles = get_tiles_for_bounds(&expanded, data_zoom);

        if tiles.is_empty() {
            return Vec::new();
        }

        // Find tile grid bounds
        let min_x = tiles.iter().map(|t| t.x).min().unwrap();
        let max_x = tiles.iter().map(|t| t.x).max().unwrap();
        let min_y = tiles.iter().map(|t| t.y).min().unwrap();
        let max_y = tiles.iter().map(|t| t.y).max().unwrap();

        let cols = max_x - min_x + 1;
        let rows = max_y - min_y + 1;

        // Calculate atlas dimensions
        let new_width = cols * TILE_SIZE;
        let new_height = rows * TILE_SIZE;

        // Calculate bounds from tile grid
        let first_tile = TileCoord::new(data_zoom, min_x, min_y);
        let last_tile = TileCoord::new(data_zoom, max_x, max_y);
        let first_bounds = first_tile.get_mercator_bounds();
        let last_bounds = last_tile.get_mercator_bounds();

        let new_bounds = AtlasBounds::new(
            first_bounds[0],  // min_x from first tile
            last_bounds[1],   // min_y from last tile (bottom)
            last_bounds[2],   // max_x from last tile
            first_bounds[3],  // max_y from first tile (top)
        );

        // Resize if needed
        if new_width != self.width || new_height != self.height || data_zoom != self.tile_zoom {
            self.width = new_width;
            self.height = new_height;
            self.data = vec![0; (new_width * new_height * 4) as usize];
            self.tile_zoom = data_zoom;
            self.tiles_min_x = min_x;
            self.tiles_min_y = min_y;
            self.tiles_cols = cols;
            self.tiles_rows = rows;
        }

        self.bounds = new_bounds;

        // Return flat array of tile coords
        tiles.iter().flat_map(|t| vec![t.z, t.x, t.y]).collect()
    }

    /// Set tile data at specific tile coordinate
    #[wasm_bindgen]
    pub fn set_tile(&mut self, z: u32, x: u32, y: u32, data: &[u8]) {
        if z != self.tile_zoom {
            return;
        }

        if data.len() != (TILE_SIZE * TILE_SIZE * 4) as usize {
            return;
        }

        // Calculate position in atlas
        if x < self.tiles_min_x || y < self.tiles_min_y {
            return;
        }

        let col = x - self.tiles_min_x;
        let row = y - self.tiles_min_y;

        if col >= self.tiles_cols || row >= self.tiles_rows {
            return;
        }

        let offset_x = col * TILE_SIZE;
        let offset_y = row * TILE_SIZE;

        // Copy tile data to atlas
        for ty in 0..TILE_SIZE {
            for tx in 0..TILE_SIZE {
                let src_idx = ((ty * TILE_SIZE + tx) * 4) as usize;
                let dst_x = offset_x + tx;
                let dst_y = offset_y + ty;
                let dst_idx = ((dst_y * self.width + dst_x) * 4) as usize;

                if dst_idx + 3 < self.data.len() && src_idx + 3 < data.len() {
                    self.data[dst_idx] = data[src_idx];
                    self.data[dst_idx + 1] = data[src_idx + 1];
                    self.data[dst_idx + 2] = data[src_idx + 2];
                    self.data[dst_idx + 3] = data[src_idx + 3];
                }
            }
        }
    }

    /// Get atlas data pointer for GPU upload
    #[wasm_bindgen]
    pub fn get_data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get atlas data length
    #[wasm_bindgen]
    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    #[wasm_bindgen]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen]
    pub fn get_height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen]
    pub fn get_bounds_min_x(&self) -> f64 {
        self.bounds.min_x
    }

    #[wasm_bindgen]
    pub fn get_bounds_min_y(&self) -> f64 {
        self.bounds.min_y
    }

    #[wasm_bindgen]
    pub fn get_bounds_max_x(&self) -> f64 {
        self.bounds.max_x
    }

    #[wasm_bindgen]
    pub fn get_bounds_max_y(&self) -> f64 {
        self.bounds.max_y
    }

    /// Sample wind at world coordinate (returns [u, v, valid] as f32)
    #[wasm_bindgen]
    pub fn sample(&self, world_x: f64, world_y: f64) -> Vec<f32> {
        if self.width == 0 || self.height == 0 {
            return vec![0.0, 0.0, 0.0];
        }

        if !self.bounds.contains(world_x, world_y) {
            return vec![0.0, 0.0, 0.0];
        }

        // World to pixel
        let u = (world_x - self.bounds.min_x) / self.bounds.width();
        let v = (world_y - self.bounds.min_y) / self.bounds.height();

        let px = (u * (self.width - 1) as f64).max(0.0).min((self.width - 1) as f64);
        // Y is flipped: atlas row 0 = NORTH (max_y), so invert v
        // v=0 (south/min_y) → py = height-1, v=1 (north/max_y) → py = 0
        let py = ((1.0 - v) * (self.height - 1) as f64).max(0.0).min((self.height - 1) as f64);

        // Bilinear interpolation
        let x0 = px.floor() as u32;
        let y0 = py.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let tx = (px - px.floor()) as f32;
        let ty = (py - py.floor()) as f32;

        let sample_pixel = |ix: u32, iy: u32| -> (f32, f32, bool) {
            let idx = ((iy * self.width + ix) * 4) as usize;
            if idx + 3 >= self.data.len() {
                return (0.0, 0.0, false);
            }
            let r = self.data[idx] as f32 / 255.0;
            let g = self.data[idx + 1] as f32 / 255.0;
            let a = self.data[idx + 3];
            (r, g, a > 127)
        };

        let (r00, g00, v00) = sample_pixel(x0, y0);
        let (r10, g10, v10) = sample_pixel(x1, y0);
        let (r01, g01, v01) = sample_pixel(x0, y1);
        let (r11, g11, v11) = sample_pixel(x1, y1);

        // All corners must be valid
        if !v00 || !v10 || !v01 || !v11 {
            return vec![0.0, 0.0, 0.0];
        }

        // Bilinear interpolation
        let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let bilerp = |v00: f32, v10: f32, v01: f32, v11: f32| {
            mix(mix(v00, v10, tx), mix(v01, v11, tx), ty)
        };

        let r = bilerp(r00, r10, r01, r11);
        let g = bilerp(g00, g10, g01, g11);

        // Decode: 0-1 → -15..15 (standard wind encoding)
        let wind_u = r * 30.0 - 15.0;
        let wind_v = g * 30.0 - 15.0;

        vec![wind_u, wind_v, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === TileCoord Tests ===

    #[test]
    fn test_tile_coord_new() {
        let coord = TileCoord::new(5, 26, 15);
        assert_eq!(coord.z, 5);
        assert_eq!(coord.x, 26);
        assert_eq!(coord.y, 15);
    }

    #[test]
    fn test_tile_coord_geo_bounds_z0() {
        let coord = TileCoord::new(0, 0, 0);
        let bounds = coord.get_geo_bounds();
        assert!((bounds[0] - (-180.0)).abs() < 0.01, "lon_min = {}", bounds[0]);
        assert!((bounds[2] - 180.0).abs() < 0.01, "lon_max = {}", bounds[2]);
    }

    #[test]
    fn test_tile_coord_geo_bounds_z1() {
        // z=1, x=0, y=0 should be top-left quadrant
        let coord = TileCoord::new(1, 0, 0);
        let bounds = coord.get_geo_bounds();
        assert!((bounds[0] - (-180.0)).abs() < 0.01, "lon_min = {}", bounds[0]);
        assert!((bounds[2] - 0.0).abs() < 0.01, "lon_max = {}", bounds[2]);
        assert!(bounds[3] > 80.0, "lat_max = {}", bounds[3]); // top of map
    }

    #[test]
    fn test_tile_coord_mercator_bounds() {
        let coord = TileCoord::new(5, 26, 15);
        let bounds = coord.get_mercator_bounds();
        // Should be 4 values: min_x, min_y, max_x, max_y
        assert_eq!(bounds.len(), 4);
        assert!(bounds[0] < bounds[2], "min_x < max_x");
        assert!(bounds[1] < bounds[3], "min_y < max_y");
    }

    // === AtlasBounds Tests ===

    #[test]
    fn test_atlas_bounds_new() {
        let bounds = AtlasBounds::new(0.0, 0.0, 100.0, 50.0);
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.min_y, 0.0);
        assert_eq!(bounds.max_x, 100.0);
        assert_eq!(bounds.max_y, 50.0);
    }

    #[test]
    fn test_atlas_bounds_width_height() {
        let bounds = AtlasBounds::new(10.0, 20.0, 110.0, 70.0);
        assert_eq!(bounds.width(), 100.0);
        assert_eq!(bounds.height(), 50.0);
    }

    #[test]
    fn test_atlas_bounds_expand() {
        let bounds = AtlasBounds::new(0.0, 0.0, 100.0, 100.0);
        let expanded = bounds.expand(1.2); // 20% buffer

        // Each side should expand by 10 (10% of 100)
        assert!((expanded.min_x - (-10.0)).abs() < 0.01);
        assert!((expanded.min_y - (-10.0)).abs() < 0.01);
        assert!((expanded.max_x - 110.0).abs() < 0.01);
        assert!((expanded.max_y - 110.0).abs() < 0.01);
    }

    #[test]
    fn test_atlas_bounds_contains() {
        let bounds = AtlasBounds::new(0.0, 0.0, 100.0, 100.0);
        assert!(bounds.contains(50.0, 50.0));
        assert!(bounds.contains(0.0, 0.0));
        assert!(bounds.contains(100.0, 100.0));
        assert!(!bounds.contains(-1.0, 50.0));
        assert!(!bounds.contains(101.0, 50.0));
    }

    // === Coordinate Conversion Tests ===

    #[test]
    fn test_lat_lng_to_mercator_origin() {
        let (x, y) = lat_lng_to_mercator(0.0, 0.0);
        assert!((x - 0.0).abs() < 1.0, "x = {}", x);
        assert!((y - 0.0).abs() < 1.0, "y = {}", y);
    }

    #[test]
    fn test_lat_lng_to_mercator_positive() {
        let (x, y) = lat_lng_to_mercator(45.0, 90.0);
        assert!(x > 0.0, "x should be positive for positive lng");
        assert!(y > 0.0, "y should be positive for positive lat");
    }

    #[test]
    fn test_mercator_roundtrip() {
        let lat = 5.0;
        let lng = 120.0;
        let (x, y) = lat_lng_to_mercator(lat, lng);
        let (lat2, lng2) = mercator_to_lat_lng(x, y);
        assert!((lat - lat2).abs() < 0.0001, "lat roundtrip: {} vs {}", lat, lat2);
        assert!((lng - lng2).abs() < 0.0001, "lng roundtrip: {} vs {}", lng, lng2);
    }

    // === get_tiles_for_bounds Tests ===

    #[test]
    fn test_get_tiles_for_bounds_single() {
        // Small area that fits in one tile
        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 119.0);
        let (max_x, max_y) = lat_lng_to_mercator(-4.0, 120.0);
        let bounds = AtlasBounds::new(min_x, min_y, max_x, max_y);

        let tiles = get_tiles_for_bounds(&bounds, 5);
        assert!(!tiles.is_empty(), "Should have at least one tile");
    }

    #[test]
    fn test_get_tiles_for_bounds_multiple() {
        // Large area covering multiple tiles
        let (min_x, min_y) = lat_lng_to_mercator(-10.0, 100.0);
        let (max_x, max_y) = lat_lng_to_mercator(10.0, 140.0);
        let bounds = AtlasBounds::new(min_x, min_y, max_x, max_y);

        let tiles = get_tiles_for_bounds(&bounds, 5);
        assert!(tiles.len() > 1, "Should have multiple tiles, got {}", tiles.len());
    }

    #[test]
    fn test_get_tiles_all_same_zoom() {
        let (min_x, min_y) = lat_lng_to_mercator(-10.0, 100.0);
        let (max_x, max_y) = lat_lng_to_mercator(10.0, 140.0);
        let bounds = AtlasBounds::new(min_x, min_y, max_x, max_y);

        let tiles = get_tiles_for_bounds(&bounds, 4);
        for tile in &tiles {
            assert_eq!(tile.z, 4, "All tiles should be zoom 4");
        }
    }

    // === DynamicAtlas Tests ===

    #[test]
    fn test_dynamic_atlas_new() {
        let atlas = DynamicAtlas::new(1.2);
        assert_eq!(atlas.width, 0);
        assert_eq!(atlas.height, 0);
        assert!((atlas.buffer_ratio - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_dynamic_atlas_update_viewport() {
        let mut atlas = DynamicAtlas::new(1.2);

        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 115.0);
        let (max_x, max_y) = lat_lng_to_mercator(5.0, 125.0);

        let tiles = atlas.update_viewport(min_x, min_y, max_x, max_y, 5);

        assert!(atlas.width > 0, "Width should be set");
        assert!(atlas.height > 0, "Height should be set");
        assert!(!tiles.is_empty(), "Should return tile coords");
        assert_eq!(tiles.len() % 3, 0, "Tile coords should be triplets");
    }

    #[test]
    fn test_dynamic_atlas_dimensions_match_tiles() {
        let mut atlas = DynamicAtlas::new(1.0); // No buffer for easier testing

        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 115.0);
        let (max_x, max_y) = lat_lng_to_mercator(5.0, 125.0);

        atlas.update_viewport(min_x, min_y, max_x, max_y, 5);

        // Width and height should be multiples of 256
        assert_eq!(atlas.width % 256, 0, "Width should be multiple of 256");
        assert_eq!(atlas.height % 256, 0, "Height should be multiple of 256");
    }

    #[test]
    fn test_dynamic_atlas_set_tile() {
        let mut atlas = DynamicAtlas::new(1.0);

        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 115.0);
        let (max_x, max_y) = lat_lng_to_mercator(5.0, 125.0);

        let tile_coords = atlas.update_viewport(min_x, min_y, max_x, max_y, 5);

        // Create fake tile data
        let tile_data: Vec<u8> = (0..256*256*4).map(|i| ((i % 256) as u8)).collect();

        // Set first tile
        if tile_coords.len() >= 3 {
            atlas.set_tile(tile_coords[0], tile_coords[1], tile_coords[2], &tile_data);
        }

        // Data should be non-empty
        assert!(atlas.data.iter().any(|&b| b != 0), "Atlas should have data");
    }

    #[test]
    fn test_dynamic_atlas_sample_empty() {
        let atlas = DynamicAtlas::new(1.2);
        let result = atlas.sample(0.0, 0.0);
        assert_eq!(result, vec![0.0, 0.0, 0.0], "Empty atlas should return invalid");
    }

    #[test]
    fn test_dynamic_atlas_sample_out_of_bounds() {
        let mut atlas = DynamicAtlas::new(1.0);

        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 115.0);
        let (max_x, max_y) = lat_lng_to_mercator(5.0, 125.0);

        atlas.update_viewport(min_x, min_y, max_x, max_y, 5);

        // Sample way outside bounds
        let (far_x, far_y) = lat_lng_to_mercator(50.0, 0.0);
        let result = atlas.sample(far_x, far_y);
        assert_eq!(result[2], 0.0, "Out of bounds should be invalid");
    }

    #[test]
    fn test_dynamic_atlas_sample_with_data() {
        let mut atlas = DynamicAtlas::new(1.0);

        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 119.0);
        let (max_x, max_y) = lat_lng_to_mercator(-4.0, 120.0);

        let tile_coords = atlas.update_viewport(min_x, min_y, max_x, max_y, 5);

        // Create tile with uniform wind: R=128 (u=0), G=128 (v=0), A=255 (valid)
        let tile_data: Vec<u8> = (0..256*256).flat_map(|_| vec![128u8, 128, 0, 255]).collect();

        // Set all tiles
        for chunk in tile_coords.chunks(3) {
            atlas.set_tile(chunk[0], chunk[1], chunk[2], &tile_data);
        }

        // Sample at center of viewport
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let result = atlas.sample(center_x, center_y);

        assert_eq!(result[2], 1.0, "Should be valid");
        assert!((result[0] - 0.0).abs() < 1.0, "u should be ~0, got {}", result[0]);
        assert!((result[1] - 0.0).abs() < 1.0, "v should be ~0, got {}", result[1]);
    }

    #[test]
    fn test_tile_3_7_3_covers_indonesia() {
        // Tile z=3, x=7, y=3
        // Should cover part of Southeast Asia region
        let tile = TileCoord::new(3, 7, 3);
        let geo = tile.get_geo_bounds();

        // geo = [lon_min, lat_min, lon_max, lat_max]
        let (lon_min, lat_min, lon_max, lat_max) = (geo[0], geo[1], geo[2], geo[3]);

        // At z=3, each tile is 45 degrees wide
        // x=7 means last column: 135E to 180E
        assert!((lon_min - 135.0).abs() < 1.0, "lon_min={} should be ~135", lon_min);
        assert!((lon_max - 180.0).abs() < 1.0, "lon_max={} should be ~180", lon_max);

        // y=3 means upper-middle (equator region)
        // At z=3, y=3 spans roughly 0 to 40.97N
        assert!(lat_min > -10.0, "lat_min={} should be > -10", lat_min);
        assert!(lat_max > 0.0, "lat_max={} should be > 0", lat_max);
    }

    #[test]
    fn test_viewport_indonesia_requests_tile_3_7_3() {
        let mut atlas = DynamicAtlas::new(1.0);

        // Indonesian viewport: roughly 95E-141E, 11S-6N
        // Let's use eastern Indonesia: 120E-141E, 5S-5N
        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 135.0);
        let (max_x, max_y) = lat_lng_to_mercator(5.0, 145.0);

        let tiles = atlas.update_viewport(min_x, min_y, max_x, max_y, 3);

        // Should include tile (3, 7, 3)
        let has_tile_7_3 = tiles.chunks(3).any(|c| c[0] == 3 && c[1] == 7 && c[2] == 3);
        assert!(has_tile_7_3, "Should request tile 3/7/3 for eastern Indonesia");
    }

    #[test]
    fn test_real_tile_wind_encoding() {
        // Test meteo-tiler wind encoding format:
        // R = u component: encoded as (u + 15) / 30 * 255
        // G = v component: encoded as (v + 15) / 30 * 255
        // A = validity: 255 = valid, 0 = NaN

        let mut atlas = DynamicAtlas::new(1.0);

        // Small viewport in eastern Indonesia
        let (min_x, min_y) = lat_lng_to_mercator(0.0, 136.0);
        let (max_x, max_y) = lat_lng_to_mercator(2.0, 138.0);

        let tiles = atlas.update_viewport(min_x, min_y, max_x, max_y, 3);

        // Create tile with realistic wind: u=5 m/s, v=-3 m/s
        // Encoded: R = (5+15)/30*255 = 170, G = (-3+15)/30*255 = 102
        let u_encoded = 170u8;
        let v_encoded = 102u8;
        let tile_data: Vec<u8> = (0..256*256).flat_map(|_| vec![u_encoded, v_encoded, 0, 255]).collect();

        // Set all tiles
        for chunk in tiles.chunks(3) {
            atlas.set_tile(chunk[0], chunk[1], chunk[2], &tile_data);
        }

        // Sample at center
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let result = atlas.sample(center_x, center_y);

        // Verify valid
        assert_eq!(result[2], 1.0, "Should be valid");

        // Decode: (sample/255 * 30) - 15
        // u = (170/255 * 30) - 15 = 5.0
        // v = (102/255 * 30) - 15 = -3.0
        let expected_u = 5.0;
        let expected_v = -3.0;

        assert!((result[0] - expected_u).abs() < 0.5, "u={} should be ~{}", result[0], expected_u);
        assert!((result[1] - expected_v).abs() < 0.5, "v={} should be ~{}", result[1], expected_v);
    }

    #[test]
    fn test_real_tile_nan_region() {
        // Test that NaN regions (land/no-data) are handled correctly
        let mut atlas = DynamicAtlas::new(1.0);

        let (min_x, min_y) = lat_lng_to_mercator(0.0, 136.0);
        let (max_x, max_y) = lat_lng_to_mercator(2.0, 138.0);

        let tiles = atlas.update_viewport(min_x, min_y, max_x, max_y, 3);

        // Create tile with NaN (alpha = 0)
        let tile_data: Vec<u8> = (0..256*256).flat_map(|_| vec![128u8, 128, 0, 0]).collect();

        for chunk in tiles.chunks(3) {
            atlas.set_tile(chunk[0], chunk[1], chunk[2], &tile_data);
        }

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let result = atlas.sample(center_x, center_y);

        // Should be invalid (validity = 0)
        assert_eq!(result[2], 0.0, "NaN region should be invalid");
    }

    #[test]
    fn test_dynamic_atlas_bounds_match_tiles() {
        let mut atlas = DynamicAtlas::new(1.0);

        let (min_x, min_y) = lat_lng_to_mercator(-5.0, 115.0);
        let (max_x, max_y) = lat_lng_to_mercator(5.0, 125.0);

        atlas.update_viewport(min_x, min_y, max_x, max_y, 5);

        // Bounds should encompass the viewport
        assert!(atlas.bounds.min_x <= min_x, "Atlas should cover viewport min_x");
        assert!(atlas.bounds.min_y <= min_y, "Atlas should cover viewport min_y");
        assert!(atlas.bounds.max_x >= max_x, "Atlas should cover viewport max_x");
        assert!(atlas.bounds.max_y >= max_y, "Atlas should cover viewport max_y");
    }
}
