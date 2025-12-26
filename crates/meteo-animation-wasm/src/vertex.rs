use wasm_bindgen::prelude::*;
use crate::trail::TrailManager;

/// Vertex format: [x, y, alpha, r, g, b] per vertex
const FLOATS_PER_VERTEX: usize = 6;
/// 6 vertices per quad (2 triangles)
const VERTICES_PER_SEGMENT: usize = 6;

/// Trail vertex builder - generates quads from trail positions
#[wasm_bindgen]
pub struct TrailVertexBuilder {
    vertices: Vec<f32>,
    vertex_count: usize,
    max_vertices: usize,
}

#[wasm_bindgen]
impl TrailVertexBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(max_particles: usize, trail_length: usize) -> Self {
        // (trail_length - 1) segments per particle
        let segments_per_particle = if trail_length > 1 { trail_length - 1 } else { 0 };
        let max_vertices = max_particles * segments_per_particle * VERTICES_PER_SEGMENT;
        let size = max_vertices * FLOATS_PER_VERTEX;

        Self {
            vertices: vec![0.0; size],
            vertex_count: 0,
            max_vertices,
        }
    }

    /// Build vertex data from trails
    /// color: base RGB color (0-1 range)
    #[wasm_bindgen]
    pub fn build(
        &mut self,
        trails: &TrailManager,
        particle_count: usize,
        trail_width: f32,
        opacity: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
    ) {
        self.vertex_count = 0;

        let trail_length = trails.trail_length();
        if trail_length < 2 {
            return;
        }

        let segments = trail_length - 1;

        for p in 0..particle_count.min(trails.max_particles()) {
            for s in 0..segments {
                // Get two consecutive positions
                let (x0, y0) = trails.get_position(p, s);
                let (x1, y1) = trails.get_position(p, s + 1);

                // Skip degenerate segments
                let dx = x1 - x0;
                let dy = y1 - y0;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 0.0001 {
                    self.emit_degenerate_quad();
                    continue;
                }

                // Direction and normal
                let dir_x = dx / len;
                let dir_y = dy / len;
                let norm_x = -dir_y;
                let norm_y = dir_x;

                // Alpha fade: HEAD (s=0) = 1.0, TAIL (s=segments-1) = 0.0
                let t0 = s as f32 / segments as f32;
                let t1 = (s + 1) as f32 / segments as f32;
                let alpha0 = (1.0 - t0) * opacity;
                let alpha1 = (1.0 - t1) * opacity;

                // Width taper: HEAD = full, TAIL = 30%
                let width0 = trail_width * (1.0 - t0 * 0.7);
                let width1 = trail_width * (1.0 - t1 * 0.7);

                // Calculate quad corners
                // P0 side
                let v0_x = x0 + norm_x * width0 * 0.5;
                let v0_y = y0 + norm_y * width0 * 0.5;
                let v3_x = x0 - norm_x * width0 * 0.5;
                let v3_y = y0 - norm_y * width0 * 0.5;

                // P1 side
                let v1_x = x1 + norm_x * width1 * 0.5;
                let v1_y = y1 + norm_y * width1 * 0.5;
                let v2_x = x1 - norm_x * width1 * 0.5;
                let v2_y = y1 - norm_y * width1 * 0.5;

                // Emit 6 vertices (2 triangles)
                // Triangle 1: v0, v1, v2
                self.emit_vertex(v0_x, v0_y, alpha0, color_r, color_g, color_b);
                self.emit_vertex(v1_x, v1_y, alpha1, color_r, color_g, color_b);
                self.emit_vertex(v2_x, v2_y, alpha1, color_r, color_g, color_b);

                // Triangle 2: v0, v2, v3
                self.emit_vertex(v0_x, v0_y, alpha0, color_r, color_g, color_b);
                self.emit_vertex(v2_x, v2_y, alpha1, color_r, color_g, color_b);
                self.emit_vertex(v3_x, v3_y, alpha0, color_r, color_g, color_b);
            }
        }
    }

    #[wasm_bindgen]
    pub fn get_vertices_ptr(&self) -> *const f32 {
        self.vertices.as_ptr()
    }

    #[wasm_bindgen]
    pub fn get_vertex_count(&self) -> usize {
        self.vertex_count
    }

    #[wasm_bindgen]
    pub fn get_floats_per_vertex(&self) -> usize {
        FLOATS_PER_VERTEX
    }
}

impl TrailVertexBuilder {
    fn emit_vertex(&mut self, x: f32, y: f32, alpha: f32, r: f32, g: f32, b: f32) {
        if self.vertex_count >= self.max_vertices {
            return;
        }

        let base = self.vertex_count * FLOATS_PER_VERTEX;
        self.vertices[base] = x;
        self.vertices[base + 1] = y;
        self.vertices[base + 2] = alpha;
        self.vertices[base + 3] = r;
        self.vertices[base + 4] = g;
        self.vertices[base + 5] = b;
        self.vertex_count += 1;
    }

    fn emit_degenerate_quad(&mut self) {
        // Emit 6 zero-area vertices (will be culled by GPU)
        for _ in 0..6 {
            self.emit_vertex(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
    }

    /// Get vertex data for testing
    pub fn get_vertex(&self, index: usize) -> (f32, f32, f32, f32, f32, f32) {
        if index >= self.vertex_count {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
        let base = index * FLOATS_PER_VERTEX;
        (
            self.vertices[base],
            self.vertices[base + 1],
            self.vertices[base + 2],
            self.vertices[base + 3],
            self.vertices[base + 4],
            self.vertices[base + 5],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_builder() {
        let vb = TrailVertexBuilder::new(100, 16);
        assert_eq!(vb.get_vertex_count(), 0);
    }

    #[test]
    fn test_new_allocates_correct_size() {
        let vb = TrailVertexBuilder::new(10, 4);
        // 10 particles * 3 segments * 6 vertices * 6 floats = 1080
        assert_eq!(vb.vertices.len(), 10 * 3 * 6 * 6);
    }

    #[test]
    fn test_build_generates_vertices() {
        let mut tm = TrailManager::new(10, 4);
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 10.0, 0.0);
        tm.push_position(0, 20.0, 0.0);

        let mut vb = TrailVertexBuilder::new(10, 4);
        vb.build(&tm, 1, 2.0, 1.0, 1.0, 1.0, 1.0);

        // 3 segments * 6 vertices = 18 vertices
        assert_eq!(vb.get_vertex_count(), 18);
    }

    #[test]
    fn test_build_empty_trails() {
        let tm = TrailManager::new(10, 4);
        let mut vb = TrailVertexBuilder::new(10, 4);
        vb.build(&tm, 0, 2.0, 1.0, 1.0, 1.0, 1.0);

        assert_eq!(vb.get_vertex_count(), 0);
    }

    #[test]
    fn test_alpha_fades_head_to_tail() {
        let mut tm = TrailManager::new(10, 3);
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 10.0, 0.0);
        tm.push_position(0, 20.0, 0.0);

        let mut vb = TrailVertexBuilder::new(10, 3);
        vb.build(&tm, 1, 2.0, 1.0, 1.0, 1.0, 1.0);

        // First segment (HEAD side) should have higher alpha
        let (_, _, alpha0, _, _, _) = vb.get_vertex(0);
        // Last segment (TAIL side) should have lower alpha
        let (_, _, alpha_last, _, _, _) = vb.get_vertex(11); // vertex at TAIL end

        assert!(alpha0 > alpha_last, "HEAD alpha {} should be > TAIL alpha {}", alpha0, alpha_last);
    }

    #[test]
    fn test_color_is_passed_through() {
        let mut tm = TrailManager::new(10, 3);
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 10.0, 0.0);

        let mut vb = TrailVertexBuilder::new(10, 3);
        vb.build(&tm, 1, 2.0, 1.0, 0.5, 0.6, 0.7);

        let (_, _, _, r, g, b) = vb.get_vertex(0);
        assert!((r - 0.5).abs() < 0.01, "R should be 0.5");
        assert!((g - 0.6).abs() < 0.01, "G should be 0.6");
        assert!((b - 0.7).abs() < 0.01, "B should be 0.7");
    }

    #[test]
    fn test_width_tapers() {
        let mut tm = TrailManager::new(10, 3);
        // Vertical trail for easy measurement
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 0.0, 10.0);
        tm.push_position(0, 0.0, 20.0);

        let mut vb = TrailVertexBuilder::new(10, 3);
        vb.build(&tm, 1, 4.0, 1.0, 1.0, 1.0, 1.0);

        // First segment vertices (HEAD side)
        let (x0, _, _, _, _, _) = vb.get_vertex(0);
        let (x3, _, _, _, _, _) = vb.get_vertex(5); // v3 of first segment

        // HEAD width = 4.0 (full width)
        let head_width = (x0 - x3).abs();
        assert!(head_width > 3.5, "HEAD width {} should be close to 4.0", head_width);

        // Last segment vertices (TAIL side)
        // segment 1 starts at vertex 6
        let (x_tail_0, _, _, _, _, _) = vb.get_vertex(7); // v1 of second segment
        let (x_tail_2, _, _, _, _, _) = vb.get_vertex(8); // v2 of second segment

        let tail_width = (x_tail_0 - x_tail_2).abs();
        // TAIL width should be smaller (30% taper at full)
        assert!(tail_width < head_width, "TAIL width {} should be < HEAD width {}", tail_width, head_width);
    }

    #[test]
    fn test_multiple_particles() {
        let mut tm = TrailManager::new(10, 3);
        tm.reset_trail(0, 0.0, 0.0);
        tm.push_position(0, 10.0, 0.0);
        tm.reset_trail(1, 100.0, 100.0);
        tm.push_position(1, 110.0, 100.0);

        let mut vb = TrailVertexBuilder::new(10, 3);
        vb.build(&tm, 2, 2.0, 1.0, 1.0, 1.0, 1.0);

        // 2 particles * 2 segments * 6 vertices = 24 vertices
        assert_eq!(vb.get_vertex_count(), 24);
    }

    #[test]
    fn test_degenerate_segments_handled() {
        let mut tm = TrailManager::new(10, 4);
        // All same position = degenerate segments
        tm.reset_trail(0, 50.0, 50.0);

        let mut vb = TrailVertexBuilder::new(10, 4);
        vb.build(&tm, 1, 2.0, 1.0, 1.0, 1.0, 1.0);

        // Should still emit vertices (degenerate quads)
        assert_eq!(vb.get_vertex_count(), 18);
    }
}
