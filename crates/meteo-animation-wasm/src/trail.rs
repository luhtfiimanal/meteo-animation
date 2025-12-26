use wasm_bindgen::prelude::*;

/// Trail manager using shift approach
/// Index 0 = HEAD (newest), Index N-1 = TAIL (oldest)
#[wasm_bindgen]
pub struct TrailManager {
    trails: Vec<f32>,      // [x, y] * trail_length * max_particles
    trail_length: usize,
    max_particles: usize,
}

#[wasm_bindgen]
impl TrailManager {
    #[wasm_bindgen(constructor)]
    pub fn new(max_particles: usize, trail_length: usize) -> Self {
        let size = max_particles * trail_length * 2; // 2 floats per position
        Self {
            trails: vec![0.0; size],
            trail_length,
            max_particles,
        }
    }

    #[wasm_bindgen]
    pub fn trail_length(&self) -> usize {
        self.trail_length
    }

    #[wasm_bindgen]
    pub fn max_particles(&self) -> usize {
        self.max_particles
    }

    /// Reset all trail positions to the same point (for spawn/respawn)
    #[wasm_bindgen]
    pub fn reset_trail(&mut self, particle_id: usize, x: f32, y: f32) {
        if particle_id >= self.max_particles {
            return;
        }

        let base = particle_id * self.trail_length * 2;
        for i in 0..self.trail_length {
            let idx = base + i * 2;
            self.trails[idx] = x;
            self.trails[idx + 1] = y;
        }
    }

    /// Push new position: shift all positions toward tail, store new at head
    #[wasm_bindgen]
    pub fn push_position(&mut self, particle_id: usize, x: f32, y: f32) {
        if particle_id >= self.max_particles {
            return;
        }

        let base = particle_id * self.trail_length * 2;

        // Shift positions toward tail (from end to start)
        // Index N-1 gets value from N-2, etc.
        for i in (1..self.trail_length).rev() {
            let src = base + (i - 1) * 2;
            let dst = base + i * 2;
            self.trails[dst] = self.trails[src];
            self.trails[dst + 1] = self.trails[src + 1];
        }

        // Store new position at head (index 0)
        self.trails[base] = x;
        self.trails[base + 1] = y;
    }

    /// Get raw pointer for WebGL upload
    #[wasm_bindgen]
    pub fn get_trails_ptr(&self) -> *const f32 {
        self.trails.as_ptr()
    }

    /// Get total size of trails data
    #[wasm_bindgen]
    pub fn get_trails_len(&self) -> usize {
        self.trails.len()
    }
}

// Internal methods (not exported to WASM)
impl TrailManager {
    /// Get position at specific segment (for testing)
    pub fn get_position(&self, particle_id: usize, segment: usize) -> (f32, f32) {
        if particle_id >= self.max_particles || segment >= self.trail_length {
            return (0.0, 0.0);
        }

        let base = particle_id * self.trail_length * 2;
        let idx = base + segment * 2;
        (self.trails[idx], self.trails[idx + 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_trails() {
        let tm = TrailManager::new(100, 16);
        assert_eq!(tm.trail_length(), 16);
        assert_eq!(tm.max_particles(), 100);
    }

    #[test]
    fn test_new_allocates_correct_size() {
        let tm = TrailManager::new(10, 8);
        // 10 particles * 8 segments * 2 floats = 160
        assert_eq!(tm.get_trails_len(), 160);
    }

    #[test]
    fn test_reset_fills_all_same() {
        let mut tm = TrailManager::new(10, 4);
        tm.reset_trail(0, 50.0, 50.0);

        for i in 0..4 {
            let (x, y) = tm.get_position(0, i);
            assert_eq!((x, y), (50.0, 50.0), "Segment {} should be (50, 50)", i);
        }
    }

    #[test]
    fn test_reset_different_particles() {
        let mut tm = TrailManager::new(10, 4);
        tm.reset_trail(0, 10.0, 10.0);
        tm.reset_trail(1, 20.0, 20.0);

        assert_eq!(tm.get_position(0, 0), (10.0, 10.0));
        assert_eq!(tm.get_position(1, 0), (20.0, 20.0));
    }

    #[test]
    fn test_push_stores_at_head() {
        let mut tm = TrailManager::new(10, 4);
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 10.0, 10.0);

        // HEAD (index 0) should be new position
        assert_eq!(tm.get_position(0, 0), (10.0, 10.0));
    }

    #[test]
    fn test_push_shifts_positions() {
        let mut tm = TrailManager::new(10, 4);
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 10.0, 10.0);

        // HEAD should be new position
        assert_eq!(tm.get_position(0, 0), (10.0, 10.0));
        // Previous HEAD shifted to index 1
        assert_eq!(tm.get_position(0, 1), (0.0, 0.0));
        // Index 2 and 3 should still be original
        assert_eq!(tm.get_position(0, 2), (0.0, 0.0));
        assert_eq!(tm.get_position(0, 3), (0.0, 0.0));
    }

    #[test]
    fn test_push_drops_oldest() {
        let mut tm = TrailManager::new(10, 3);
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 1.0, 1.0);
        tm.push_position(0, 2.0, 2.0);
        tm.push_position(0, 3.0, 3.0);

        // Trail should be [3, 2, 1], oldest (0,0) dropped
        assert_eq!(tm.get_position(0, 0), (3.0, 3.0));
        assert_eq!(tm.get_position(0, 1), (2.0, 2.0));
        assert_eq!(tm.get_position(0, 2), (1.0, 1.0));
    }

    #[test]
    fn test_multiple_particles_independent() {
        let mut tm = TrailManager::new(10, 3);
        tm.reset_trail(0, 0.0, 0.0);
        tm.reset_trail(1, 100.0, 100.0);

        tm.push_position(0, 5.0, 5.0);
        tm.push_position(1, 105.0, 105.0);

        // Particle 0
        assert_eq!(tm.get_position(0, 0), (5.0, 5.0));
        assert_eq!(tm.get_position(0, 1), (0.0, 0.0));

        // Particle 1
        assert_eq!(tm.get_position(1, 0), (105.0, 105.0));
        assert_eq!(tm.get_position(1, 1), (100.0, 100.0));
    }

    #[test]
    fn test_out_of_bounds_particle_id() {
        let mut tm = TrailManager::new(5, 4);
        // Should not panic
        tm.reset_trail(10, 50.0, 50.0);
        tm.push_position(10, 60.0, 60.0);
        let pos = tm.get_position(10, 0);
        assert_eq!(pos, (0.0, 0.0));
    }
}
