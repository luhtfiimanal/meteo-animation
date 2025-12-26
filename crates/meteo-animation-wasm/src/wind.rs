use wasm_bindgen::prelude::*;
use crate::utils::{clamp, bilerp};
use crate::atlas::DynamicAtlas;

/// Wind sample result (internal, not exported to WASM)
pub struct WindSample {
    pub u: f32,
    pub v: f32,
    pub speed: f32,
    pub valid: bool,
}

/// Wind texture sampler with bilinear interpolation
#[wasm_bindgen]
pub struct WindSampler {
    data: Vec<u8>,
    width: u32,
    height: u32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

#[wasm_bindgen]
impl WindSampler {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![0; (width * height * 4) as usize],
            width,
            height,
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        }
    }

    #[wasm_bindgen]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen]
    pub fn height(&self) -> u32 {
        self.height
    }

    // Getters for JS compatibility (get_* naming)
    #[wasm_bindgen]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen]
    pub fn get_height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen]
    pub fn get_min_x(&self) -> f32 {
        self.min_x
    }

    #[wasm_bindgen]
    pub fn get_min_y(&self) -> f32 {
        self.min_y
    }

    #[wasm_bindgen]
    pub fn get_max_x(&self) -> f32 {
        self.max_x
    }

    #[wasm_bindgen]
    pub fn get_max_y(&self) -> f32 {
        self.max_y
    }

    #[wasm_bindgen]
    pub fn get_data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[wasm_bindgen]
    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    #[wasm_bindgen]
    pub fn set_data(&mut self, data: &[u8]) {
        self.data = data.to_vec();
    }

    #[wasm_bindgen]
    pub fn set_bounds(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.min_x = min_x;
        self.min_y = min_y;
        self.max_x = max_x;
        self.max_y = max_y;
    }

    /// Create WindSampler from DynamicAtlas
    /// Copies atlas data and bounds for particle simulation
    #[wasm_bindgen]
    pub fn from_atlas(atlas: &DynamicAtlas) -> Self {
        let width = atlas.get_width();
        let height = atlas.get_height();

        // Copy data from atlas via pointer
        let data_len = atlas.get_data_len();
        let data_ptr = atlas.get_data_ptr();
        let data = unsafe {
            std::slice::from_raw_parts(data_ptr, data_len).to_vec()
        };

        Self {
            data,
            width,
            height,
            min_x: atlas.get_bounds_min_x() as f32,
            min_y: atlas.get_bounds_min_y() as f32,
            max_x: atlas.get_bounds_max_x() as f32,
            max_y: atlas.get_bounds_max_y() as f32,
        }
    }

    /// Update WindSampler from DynamicAtlas
    /// Call this when atlas data or bounds change
    #[wasm_bindgen]
    pub fn update_from_atlas(&mut self, atlas: &DynamicAtlas) {
        self.width = atlas.get_width();
        self.height = atlas.get_height();
        self.min_x = atlas.get_bounds_min_x() as f32;
        self.min_y = atlas.get_bounds_min_y() as f32;
        self.max_x = atlas.get_bounds_max_x() as f32;
        self.max_y = atlas.get_bounds_max_y() as f32;

        // Copy data from atlas via pointer
        let data_len = atlas.get_data_len();
        let data_ptr = atlas.get_data_ptr();
        self.data = unsafe {
            std::slice::from_raw_parts(data_ptr, data_len).to_vec()
        };
    }

    /// Check if position has valid wind data (alpha > 127)
    #[wasm_bindgen]
    pub fn is_valid(&self, x: f32, y: f32) -> bool {
        let (px, py) = self.world_to_pixel(x, y);
        let ix = px.floor() as u32;
        let iy = py.floor() as u32;

        if ix >= self.width || iy >= self.height {
            return false;
        }

        let idx = ((iy * self.width + ix) * 4 + 3) as usize;
        if idx >= self.data.len() {
            return false;
        }

        self.data[idx] > 127
    }
}

// Internal methods (not exported to WASM)
impl WindSampler {
    fn world_to_pixel(&self, x: f32, y: f32) -> (f32, f32) {
        let u = (x - self.min_x) / (self.max_x - self.min_x);
        let v = (y - self.min_y) / (self.max_y - self.min_y);

        let px = clamp(u * (self.width - 1) as f32, 0.0, (self.width - 1) as f32);
        // Y is flipped: atlas row 0 = NORTH (max_y), so invert v
        // v=0 (south/min_y) → py = height-1, v=1 (north/max_y) → py = 0
        let py = clamp((1.0 - v) * (self.height - 1) as f32, 0.0, (self.height - 1) as f32);

        (px, py)
    }

    fn sample_pixel(&self, ix: u32, iy: u32) -> (f32, f32, f32, bool) {
        if ix >= self.width || iy >= self.height {
            return (0.0, 0.0, 0.0, false);
        }

        let idx = ((iy * self.width + ix) * 4) as usize;
        if idx + 3 >= self.data.len() {
            return (0.0, 0.0, 0.0, false);
        }

        let r = self.data[idx] as f32 / 255.0;
        let g = self.data[idx + 1] as f32 / 255.0;
        let b = self.data[idx + 2] as f32 / 255.0;
        let a = self.data[idx + 3];

        (r, g, b, a > 127)
    }

    /// Sample wind with bilinear interpolation
    /// Decodes: u = r * 30 - 15, v = g * 30 - 15
    pub fn sample(&self, x: f32, y: f32) -> WindSample {
        let (px, py) = self.world_to_pixel(x, y);

        let x0 = px.floor() as u32;
        let y0 = py.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let tx = px - px.floor();
        let ty = py - py.floor();

        let (r00, g00, _, v00) = self.sample_pixel(x0, y0);
        let (r10, g10, _, v10) = self.sample_pixel(x1, y0);
        let (r01, g01, _, v01) = self.sample_pixel(x0, y1);
        let (r11, g11, _, v11) = self.sample_pixel(x1, y1);

        // All corners must be valid for interpolation
        if !v00 || !v10 || !v01 || !v11 {
            return WindSample {
                u: 0.0,
                v: 0.0,
                speed: 0.0,
                valid: false,
            };
        }

        let r = bilerp(r00, r10, r01, r11, tx, ty);
        let g = bilerp(g00, g10, g01, g11, tx, ty);

        // Decode: 0-1 → -15..15
        let u = r * 30.0 - 15.0;
        let v = g * 30.0 - 15.0;
        let speed = (u * u + v * v).sqrt();

        WindSample {
            u,
            v,
            speed,
            valid: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_sampler() {
        let sampler = WindSampler::new(256, 256);
        assert_eq!(sampler.width(), 256);
        assert_eq!(sampler.height(), 256);
    }

    #[test]
    fn test_new_allocates_data() {
        let sampler = WindSampler::new(4, 4);
        assert_eq!(sampler.data.len(), 4 * 4 * 4); // 64 bytes
    }

    #[test]
    fn test_set_bounds() {
        let mut sampler = WindSampler::new(256, 256);
        sampler.set_bounds(0.0, 0.0, 100.0, 100.0);
        assert_eq!(sampler.min_x, 0.0);
        assert_eq!(sampler.max_x, 100.0);
        assert_eq!(sampler.min_y, 0.0);
        assert_eq!(sampler.max_y, 100.0);
    }

    #[test]
    fn test_is_valid_returns_false_for_nan() {
        let mut sampler = WindSampler::new(4, 4);
        // All pixels with alpha = 0
        let data: Vec<u8> = (0..16).flat_map(|_| [128, 128, 128, 0]).collect();
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 4.0, 4.0);

        assert!(!sampler.is_valid(0.5, 0.5));
        assert!(!sampler.is_valid(2.0, 2.0));
    }

    #[test]
    fn test_is_valid_returns_true_for_valid() {
        let mut sampler = WindSampler::new(4, 4);
        // All pixels with alpha = 255
        let data: Vec<u8> = (0..16).flat_map(|_| [128, 128, 128, 255]).collect();
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 4.0, 4.0);

        assert!(sampler.is_valid(0.5, 0.5));
        assert!(sampler.is_valid(2.0, 2.0));
    }

    #[test]
    fn test_is_valid_mixed_regions() {
        let mut sampler = WindSampler::new(2, 2);
        // Data layout (row-major):
        //   Row 0 (NORTH): pixel (0,0) valid, pixel (1,0) NaN
        //   Row 1 (SOUTH): pixel (0,1) valid, pixel (1,1) valid
        let data = vec![
            128, 128, 128, 255,  // pixel (0,0) = NORTH-WEST - valid
            128, 128, 128, 0,    // pixel (1,0) = NORTH-EAST - NaN
            128, 128, 128, 255,  // pixel (0,1) = SOUTH-WEST - valid
            128, 128, 128, 255,  // pixel (1,1) = SOUTH-EAST - valid
        ];
        sampler.set_data(&data);
        // bounds: min_y=0 (SOUTH), max_y=2 (NORTH)
        sampler.set_bounds(0.0, 0.0, 2.0, 2.0);

        // World (0, 0) = SOUTH-WEST → pixel (0,1) = valid
        assert!(sampler.is_valid(0.0, 0.0));
        // World (2, 2) = NORTH-EAST → pixel (1,0) = NaN
        assert!(!sampler.is_valid(2.0, 2.0));
        // World (2, 0) = SOUTH-EAST → pixel (1,1) = valid
        assert!(sampler.is_valid(2.0, 0.0));
    }

    #[test]
    fn test_sample_decodes_wind_zero() {
        let mut sampler = WindSampler::new(2, 2);
        // R=128 → u = 0.5*30-15 = 0
        // G=128 → v = 0.5*30-15 = 0
        let data = vec![
            128, 128, 128, 255,
            128, 128, 128, 255,
            128, 128, 128, 255,
            128, 128, 128, 255,
        ];
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 2.0, 2.0);

        let sample = sampler.sample(1.0, 1.0);
        assert!(sample.valid);
        assert!((sample.u - 0.0).abs() < 0.1, "u={}", sample.u);
        assert!((sample.v - 0.0).abs() < 0.1, "v={}", sample.v);
    }

    #[test]
    fn test_sample_decodes_wind_positive() {
        let mut sampler = WindSampler::new(2, 2);
        // R=255 → u = 1.0*30-15 = 15
        // G=255 → v = 15
        let data = vec![
            255, 255, 128, 255,
            255, 255, 128, 255,
            255, 255, 128, 255,
            255, 255, 128, 255,
        ];
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 2.0, 2.0);

        let sample = sampler.sample(1.0, 1.0);
        assert!(sample.valid);
        assert!((sample.u - 15.0).abs() < 0.5, "u={}", sample.u);
        assert!((sample.v - 15.0).abs() < 0.5, "v={}", sample.v);
    }

    #[test]
    fn test_sample_decodes_wind_negative() {
        let mut sampler = WindSampler::new(2, 2);
        // R=0 → u = 0*30-15 = -15
        // G=0 → v = -15
        let data = vec![
            0, 0, 128, 255,
            0, 0, 128, 255,
            0, 0, 128, 255,
            0, 0, 128, 255,
        ];
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 2.0, 2.0);

        let sample = sampler.sample(1.0, 1.0);
        assert!(sample.valid);
        assert!((sample.u - (-15.0)).abs() < 0.5, "u={}", sample.u);
        assert!((sample.v - (-15.0)).abs() < 0.5, "v={}", sample.v);
    }

    #[test]
    fn test_sample_invalid_at_nan_region() {
        let mut sampler = WindSampler::new(2, 2);
        // One corner is NaN
        let data = vec![
            128, 128, 128, 0,    // NaN at (0,0)
            128, 128, 128, 255,
            128, 128, 128, 255,
            128, 128, 128, 255,
        ];
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 2.0, 2.0);

        // Sample near (0,0) - bilinear needs all 4 corners valid
        let sample = sampler.sample(0.25, 0.25);
        assert!(!sample.valid);
    }

    #[test]
    fn test_bilinear_interpolation() {
        let mut sampler = WindSampler::new(2, 2);
        // Left side: R=128 (u=0), Right side: R=255 (u=15)
        let data = vec![
            128, 128, 128, 255,  // (0,0) u=0
            255, 128, 128, 255,  // (1,0) u=15
            128, 128, 128, 255,  // (0,1) u=0
            255, 128, 128, 255,  // (1,1) u=15
        ];
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 2.0, 2.0);

        // Sample at center - should interpolate to ~7.5
        let sample = sampler.sample(1.0, 1.0);
        assert!(sample.valid);
        assert!((sample.u - 7.5).abs() < 1.0, "u={} expected ~7.5", sample.u);
    }

    #[test]
    fn test_sample_calculates_speed() {
        let mut sampler = WindSampler::new(2, 2);
        // R=178 → u = (178/255)*30-15 = 5.94
        // G=178 → v = 5.94
        // speed = sqrt(5.94^2 + 5.94^2) = 8.4
        let data = vec![
            178, 178, 128, 255,
            178, 178, 128, 255,
            178, 178, 128, 255,
            178, 178, 128, 255,
        ];
        sampler.set_data(&data);
        sampler.set_bounds(0.0, 0.0, 2.0, 2.0);

        let sample = sampler.sample(1.0, 1.0);
        assert!(sample.valid);
        assert!(sample.speed > 5.0, "speed={} should be > 5", sample.speed);
    }

    #[test]
    fn test_world_to_pixel_mapping() {
        let mut sampler = WindSampler::new(100, 100);
        sampler.set_bounds(0.0, 0.0, 100.0, 100.0);

        // Sample at world (50, 50) should map to pixel (49.5, 49.5)
        let (px, py) = sampler.world_to_pixel(50.0, 50.0);
        assert!((px - 49.5).abs() < 0.1, "px={}", px);
        assert!((py - 49.5).abs() < 0.1, "py={}", py);
    }

    #[test]
    fn test_from_atlas_copies_dimensions() {
        use crate::atlas::DynamicAtlas;

        let mut atlas = DynamicAtlas::new(1.2);
        // Update viewport to initialize atlas
        atlas.update_viewport(0.0, 0.0, 1000.0, 1000.0, 3);

        let sampler = WindSampler::from_atlas(&atlas);

        assert_eq!(sampler.width(), atlas.get_width());
        assert_eq!(sampler.height(), atlas.get_height());
    }

    #[test]
    fn test_from_atlas_copies_bounds() {
        use crate::atlas::DynamicAtlas;

        let mut atlas = DynamicAtlas::new(1.2);
        atlas.update_viewport(1000.0, 2000.0, 3000.0, 4000.0, 3);

        let sampler = WindSampler::from_atlas(&atlas);

        assert!((sampler.min_x - atlas.get_bounds_min_x() as f32).abs() < 1.0);
        assert!((sampler.min_y - atlas.get_bounds_min_y() as f32).abs() < 1.0);
        assert!((sampler.max_x - atlas.get_bounds_max_x() as f32).abs() < 1.0);
        assert!((sampler.max_y - atlas.get_bounds_max_y() as f32).abs() < 1.0);
    }

    #[test]
    fn test_from_atlas_copies_data() {
        use crate::atlas::DynamicAtlas;

        let mut atlas = DynamicAtlas::new(1.2);
        atlas.update_viewport(0.0, 0.0, 1000.0, 1000.0, 3);

        let sampler = WindSampler::from_atlas(&atlas);

        // Data should be same length
        let expected_len = (atlas.get_width() * atlas.get_height() * 4) as usize;
        assert_eq!(sampler.data.len(), expected_len);
    }

    #[test]
    fn test_update_from_atlas() {
        use crate::atlas::DynamicAtlas;

        let mut sampler = WindSampler::new(10, 10);
        sampler.set_bounds(0.0, 0.0, 10.0, 10.0);

        let mut atlas = DynamicAtlas::new(1.2);
        atlas.update_viewport(1000.0, 2000.0, 3000.0, 4000.0, 4);

        sampler.update_from_atlas(&atlas);

        // Dimensions should be updated
        assert_eq!(sampler.width(), atlas.get_width());
        assert_eq!(sampler.height(), atlas.get_height());

        // Bounds should be updated
        assert!((sampler.min_x - atlas.get_bounds_min_x() as f32).abs() < 1.0);
    }
}
