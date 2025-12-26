use wasm_bindgen::prelude::*;
use crate::atlas::{AtlasBounds, TileCoord};

/// Tile size in pixels
const TILE_SIZE: u32 = 256;

/// Fixed-size atlas that never resizes during pan/zoom.
/// Only the bounds (georeferencing) change.
/// This prevents particle "blink" during viewport changes.
#[wasm_bindgen]
pub struct FixedAtlas {
    data: Vec<u8>,
    width: u32,
    height: u32,
    bounds: AtlasBounds,
    tile_zoom: u32,
    dirty: bool,
}

#[wasm_bindgen]
impl FixedAtlas {
    /// Create a new fixed-size atlas.
    /// Size should be based on device screen resolution / 2^|zoom_offset|.
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![0; (width * height * 4) as usize],
            width,
            height,
            bounds: AtlasBounds::new(0.0, 0.0, 1.0, 1.0),
            tile_zoom: 0,
            dirty: true,
        }
    }

    /// Update bounds without resizing.
    /// Called on every pan/zoom. Does NOT clear data.
    #[wasm_bindgen]
    pub fn update_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, zoom: u32) {
        // Check if bounds actually changed
        let new_bounds = AtlasBounds::new(min_x, min_y, max_x, max_y);

        // If bounds changed, we need to resample existing data
        if self.bounds.min_x != new_bounds.min_x ||
           self.bounds.min_y != new_bounds.min_y ||
           self.bounds.max_x != new_bounds.max_x ||
           self.bounds.max_y != new_bounds.max_y {
            self.dirty = true;
        }

        self.bounds = new_bounds;
        self.tile_zoom = zoom;
    }

    /// Place tile data with bilinear resampling.
    /// The tile is resampled to fit the current atlas bounds.
    #[wasm_bindgen]
    pub fn set_tile_resampled(&mut self, z: u32, x: u32, y: u32, tile_data: &[u8]) {
        if z != self.tile_zoom {
            return;
        }

        if tile_data.len() != (TILE_SIZE * TILE_SIZE * 4) as usize {
            return;
        }

        let tile = TileCoord::new(z, x, y);
        let tile_bounds = tile.get_mercator_bounds();
        let tile_min_x = tile_bounds[0];
        let tile_min_y = tile_bounds[1];
        let tile_max_x = tile_bounds[2];
        let tile_max_y = tile_bounds[3];

        // Calculate atlas pixel range for this tile
        let (ax_min, ay_min) = self.world_to_pixel(tile_min_x, tile_max_y); // top-left
        let (ax_max, ay_max) = self.world_to_pixel(tile_max_x, tile_min_y); // bottom-right

        // Clamp to atlas bounds
        let ax_start = (ax_min.floor().max(0.0) as u32).min(self.width);
        let ax_end = (ax_max.ceil().max(0.0) as u32).min(self.width);
        let ay_start = (ay_min.floor().max(0.0) as u32).min(self.height);
        let ay_end = (ay_max.ceil().max(0.0) as u32).min(self.height);

        // Tile dimensions in world units
        let tile_width = tile_max_x - tile_min_x;
        let tile_height = tile_max_y - tile_min_y;

        if tile_width <= 0.0 || tile_height <= 0.0 {
            return;
        }

        // Iterate only pixels covered by this tile
        for py in ay_start..ay_end {
            for px in ax_start..ax_end {
                // Atlas pixel → World coordinate
                let (world_x, world_y) = self.pixel_to_world(px, py);

                // World coord → Tile UV (0..1)
                let tu = (world_x - tile_min_x) / tile_width;
                let tv = (tile_max_y - world_y) / tile_height; // Y flipped

                // Skip if outside tile bounds
                if tu < 0.0 || tu > 1.0 || tv < 0.0 || tv > 1.0 {
                    continue;
                }

                // Tile UV → Tile pixel (with bilinear sampling)
                let tx = tu * (TILE_SIZE - 1) as f64;
                let ty = tv * (TILE_SIZE - 1) as f64;

                // Bilinear sample from tile
                let rgba = bilinear_sample_tile(tile_data, tx as f32, ty as f32);

                // Write to atlas
                let idx = ((py * self.width + px) * 4) as usize;
                if idx + 3 < self.data.len() {
                    self.data[idx] = rgba[0];
                    self.data[idx + 1] = rgba[1];
                    self.data[idx + 2] = rgba[2];
                    self.data[idx + 3] = rgba[3];
                }
            }
        }

        self.dirty = true;
    }

    /// Clear atlas data (fill with zeros).
    /// Use when data zoom changes and all tiles need to be reloaded.
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.data.fill(0);
        self.dirty = true;
    }

    /// Convert atlas pixel to world coordinate
    fn pixel_to_world(&self, px: u32, py: u32) -> (f64, f64) {
        let u = (px as f64 + 0.5) / self.width as f64;
        let v = (py as f64 + 0.5) / self.height as f64;

        let world_x = self.bounds.min_x + u * self.bounds.width();
        // Y is flipped: py=0 is north (max_y), py=height is south (min_y)
        let world_y = self.bounds.max_y - v * self.bounds.height();

        (world_x, world_y)
    }

    /// Convert world coordinate to atlas pixel
    fn world_to_pixel(&self, x: f64, y: f64) -> (f64, f64) {
        let u = (x - self.bounds.min_x) / self.bounds.width();
        let v = (self.bounds.max_y - y) / self.bounds.height(); // Y flipped

        let px = u * self.width as f64;
        let py = v * self.height as f64;

        (px, py)
    }

    // === Getters ===

    #[wasm_bindgen]
    pub fn get_data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[wasm_bindgen]
    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    /// Get a copy of the atlas data as a Vec<u8>.
    /// This is safer than get_data_ptr() as it avoids detached ArrayBuffer issues.
    #[wasm_bindgen]
    pub fn get_data_copy(&self) -> Vec<u8> {
        self.data.clone()
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

    #[wasm_bindgen]
    pub fn get_tile_zoom(&self) -> u32 {
        self.tile_zoom
    }

    #[wasm_bindgen]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[wasm_bindgen]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
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
        let (px, py) = self.world_to_pixel(world_x, world_y);

        // Clamp to valid range
        let px = px.max(0.0).min((self.width - 1) as f64);
        let py = py.max(0.0).min((self.height - 1) as f64);

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

/// Bilinear sample from tile data (256x256 RGBA)
fn bilinear_sample_tile(tile_data: &[u8], tx: f32, ty: f32) -> [u8; 4] {
    let x0 = tx.floor() as u32;
    let y0 = ty.floor() as u32;
    let x1 = (x0 + 1).min(TILE_SIZE - 1);
    let y1 = (y0 + 1).min(TILE_SIZE - 1);

    let fx = tx.fract();
    let fy = ty.fract();

    let get_pixel = |x: u32, y: u32| -> [u8; 4] {
        let idx = ((y * TILE_SIZE + x) * 4) as usize;
        if idx + 3 < tile_data.len() {
            [tile_data[idx], tile_data[idx + 1], tile_data[idx + 2], tile_data[idx + 3]]
        } else {
            [0, 0, 0, 0]
        }
    };

    let p00 = get_pixel(x0, y0);
    let p10 = get_pixel(x1, y0);
    let p01 = get_pixel(x0, y1);
    let p11 = get_pixel(x1, y1);

    // If any corner is invalid (alpha=0), return invalid
    if p00[3] == 0 || p10[3] == 0 || p01[3] == 0 || p11[3] == 0 {
        return [0, 0, 0, 0];
    }

    // Bilinear interpolate each channel
    let bilerp = |v00: u8, v10: u8, v01: u8, v11: u8| -> u8 {
        let v00 = v00 as f32;
        let v10 = v10 as f32;
        let v01 = v01 as f32;
        let v11 = v11 as f32;

        let top = v00 + (v10 - v00) * fx;
        let bottom = v01 + (v11 - v01) * fx;
        let result = top + (bottom - top) * fy;

        result.round().max(0.0).min(255.0) as u8
    };

    [
        bilerp(p00[0], p10[0], p01[0], p11[0]),
        bilerp(p00[1], p10[1], p01[1], p11[1]),
        bilerp(p00[2], p10[2], p01[2], p11[2]),
        bilerp(p00[3], p10[3], p01[3], p11[3]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_atlas_new() {
        let atlas = FixedAtlas::new(512, 512);
        assert_eq!(atlas.width, 512);
        assert_eq!(atlas.height, 512);
        assert_eq!(atlas.data.len(), 512 * 512 * 4);
        assert!(atlas.dirty);
    }

    #[test]
    fn test_fixed_atlas_update_bounds_no_resize() {
        let mut atlas = FixedAtlas::new(512, 512);
        let original_len = atlas.data.len();

        atlas.update_bounds(100.0, 200.0, 300.0, 400.0, 5);

        // Data should NOT be reallocated
        assert_eq!(atlas.data.len(), original_len);
        assert_eq!(atlas.bounds.min_x, 100.0);
        assert_eq!(atlas.bounds.min_y, 200.0);
        assert_eq!(atlas.bounds.max_x, 300.0);
        assert_eq!(atlas.bounds.max_y, 400.0);
        assert_eq!(atlas.tile_zoom, 5);
    }

    #[test]
    fn test_pixel_to_world_roundtrip() {
        let mut atlas = FixedAtlas::new(256, 256);
        atlas.update_bounds(0.0, 0.0, 1000.0, 1000.0, 5);

        // Center pixel should map to center of bounds
        let (wx, wy) = atlas.pixel_to_world(128, 128);
        assert!((wx - 500.0).abs() < 10.0, "wx = {}", wx);
        assert!((wy - 500.0).abs() < 10.0, "wy = {}", wy);

        // And back
        let (px, py) = atlas.world_to_pixel(wx, wy);
        assert!((px - 128.0).abs() < 1.0, "px = {}", px);
        assert!((py - 128.0).abs() < 1.0, "py = {}", py);
    }

    #[test]
    fn test_bilinear_sample_tile_uniform() {
        // Create uniform tile: all pixels = (128, 128, 0, 255)
        let tile_data: Vec<u8> = (0..TILE_SIZE * TILE_SIZE)
            .flat_map(|_| vec![128u8, 128, 0, 255])
            .collect();

        let result = bilinear_sample_tile(&tile_data, 127.5, 127.5);
        assert_eq!(result[0], 128);
        assert_eq!(result[1], 128);
        assert_eq!(result[3], 255);
    }

    #[test]
    fn test_bilinear_sample_tile_gradient() {
        // Create gradient tile: R increases with X
        let mut tile_data = vec![0u8; (TILE_SIZE * TILE_SIZE * 4) as usize];
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let idx = ((y * TILE_SIZE + x) * 4) as usize;
                tile_data[idx] = x as u8;     // R = x
                tile_data[idx + 1] = 0;       // G = 0
                tile_data[idx + 2] = 0;       // B = 0
                tile_data[idx + 3] = 255;     // A = valid
            }
        }

        // Sample at x=100.5 should interpolate between 100 and 101
        let result = bilinear_sample_tile(&tile_data, 100.5, 0.0);
        assert!(result[0] >= 100 && result[0] <= 101, "r = {}", result[0]);
    }

    #[test]
    fn test_bilinear_sample_tile_invalid_corner() {
        // Create tile with one invalid corner
        let mut tile_data = vec![0u8; (TILE_SIZE * TILE_SIZE * 4) as usize];
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let idx = ((y * TILE_SIZE + x) * 4) as usize;
                tile_data[idx] = 128;
                tile_data[idx + 1] = 128;
                tile_data[idx + 2] = 0;
                // Make top-left corner invalid
                tile_data[idx + 3] = if x == 0 && y == 0 { 0 } else { 255 };
            }
        }

        // Sample near invalid corner should return invalid
        let result = bilinear_sample_tile(&tile_data, 0.5, 0.5);
        assert_eq!(result[3], 0, "should be invalid");
    }

    #[test]
    fn test_sample_empty_atlas() {
        let atlas = FixedAtlas::new(256, 256);
        let result = atlas.sample(0.0, 0.0);
        assert_eq!(result, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_sample_out_of_bounds() {
        let mut atlas = FixedAtlas::new(256, 256);
        atlas.update_bounds(0.0, 0.0, 100.0, 100.0, 5);

        let result = atlas.sample(200.0, 200.0);
        assert_eq!(result[2], 0.0, "should be invalid (out of bounds)");
    }

    #[test]
    fn test_dirty_flag() {
        let mut atlas = FixedAtlas::new(256, 256);
        assert!(atlas.is_dirty());

        atlas.clear_dirty();
        assert!(!atlas.is_dirty());

        atlas.update_bounds(0.0, 0.0, 100.0, 100.0, 5);
        assert!(atlas.is_dirty());
    }

    #[test]
    fn test_clear() {
        let mut atlas = FixedAtlas::new(256, 256);

        // Set some data
        atlas.data[0] = 255;
        atlas.data[1] = 255;

        atlas.clear();

        assert_eq!(atlas.data[0], 0);
        assert_eq!(atlas.data[1], 0);
        assert!(atlas.is_dirty());
    }
}
