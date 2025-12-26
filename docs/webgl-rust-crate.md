# Rust WASM Crate Specification

## Setup

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack

# Build
cd crates/meteo-animation-wasm
wasm-pack build --target web --release

# Output: pkg/meteo_animation_wasm.js, pkg/meteo_animation_wasm_bg.wasm
```

## Cargo.toml

```toml
[package]
name = "meteo-animation-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"

[profile.release]
opt-level = 3
lto = true
```

---

## Module: lib.rs

```rust
mod utils;
mod particle;
mod wind;
mod trail;
mod tile;
mod vertex;
mod animation;

pub use particle::ParticleSimulator;
pub use wind::WindSampler;
pub use trail::TrailManager;
pub use tile::TileStitcher;
pub use vertex::TrailVertexBuilder;
pub use animation::WindAnimation;
```

---

## Module: utils.rs

Shared utilities matching WGSL compute shader behavior.

```rust
/// Same random function as WGSL compute shader
/// Deterministic based on seed
pub fn random(seed: f32) -> f32 {
    let x = (seed * 12.9898 + 78.233).sin() * 43758.5453;
    x - x.floor()
}

pub fn random2(seed: f32) -> (f32, f32) {
    (random(seed), random(seed + 1.0))
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.max(min).min(max)
}

pub fn in_bounds(x: f32, y: f32, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> bool {
    x >= min_x && x <= max_x && y >= min_y && y <= max_y
}

/// Bilinear interpolation
pub fn bilerp(v00: f32, v10: f32, v01: f32, v11: f32, tx: f32, ty: f32) -> f32 {
    let v0 = lerp(v00, v10, tx);
    let v1 = lerp(v01, v11, tx);
    lerp(v0, v1, ty)
}
```

---

## Module: particle.rs (POC 1-2)

```rust
use wasm_bindgen::prelude::*;
use crate::utils::*;
use crate::wind::WindSampler;

/// Particle data layout per particle (8 floats):
/// [x, y, age, state, velocity_x, velocity_y, speed, seed]
const FLOATS_PER_PARTICLE: usize = 8;

/// Particle states
const STATE_ACTIVE: f32 = 0.0;
const STATE_EXITING: f32 = 1.0;
const STATE_RESPAWNING: f32 = 2.0;
const STATE_INACTIVE: f32 = 3.0;

#[wasm_bindgen]
pub struct ParticleSimulator {
    particles: Vec<f32>,
    max_particles: usize,
}

#[wasm_bindgen]
impl ParticleSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new(max_particles: usize) -> Self {
        let mut particles = vec![0.0; max_particles * FLOATS_PER_PARTICLE];

        // Initialize all particles as inactive
        for i in 0..max_particles {
            let base = i * FLOATS_PER_PARTICLE;
            particles[base + 0] = -1000.0;  // x (off-screen)
            particles[base + 1] = -1000.0;  // y
            particles[base + 2] = 0.0;      // age
            particles[base + 3] = STATE_INACTIVE;
            particles[base + 4] = 0.0;      // velocity_x
            particles[base + 5] = 0.0;      // velocity_y
            particles[base + 6] = 0.0;      // speed
            particles[base + 7] = i as f32; // seed
        }

        Self { particles, max_particles }
    }

    /// POC 1: Basic update with fixed wind velocity
    #[wasm_bindgen]
    pub fn update_basic(
        &mut self,
        bounds_min_x: f32, bounds_min_y: f32,
        bounds_max_x: f32, bounds_max_y: f32,
        wind_x: f32, wind_y: f32,
        delta_time: f32,
        max_age: f32,
        target_count: u32,
        random_seed: f32,
    ) {
        for i in 0..self.max_particles {
            let base = i * FLOATS_PER_PARTICLE;
            let should_be_active = (i as u32) < target_count;

            if !should_be_active {
                self.particles[base + 3] = STATE_INACTIVE;
                continue;
            }

            let state = self.particles[base + 3];

            // Activate if inactive
            if state == STATE_INACTIVE {
                self.respawn(i, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, random_seed);
                continue;
            }

            // Update position
            let x = self.particles[base + 0] + wind_x * delta_time;
            let y = self.particles[base + 1] + wind_y * delta_time;
            let age = self.particles[base + 2] + 1.0;

            // Check respawn conditions
            let out_of_bounds = !in_bounds(x, y, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y);
            let too_old = age > max_age;

            if out_of_bounds || too_old {
                self.respawn(i, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, random_seed + age);
            } else {
                self.particles[base + 0] = x;
                self.particles[base + 1] = y;
                self.particles[base + 2] = age;
                self.particles[base + 4] = wind_x;
                self.particles[base + 5] = wind_y;
                self.particles[base + 6] = (wind_x * wind_x + wind_y * wind_y).sqrt();
            }
        }
    }

    /// POC 2: Update with wind texture sampling + NaN handling
    #[wasm_bindgen]
    pub fn update_with_wind(
        &mut self,
        wind: &WindSampler,
        bounds_min_x: f32, bounds_min_y: f32,
        bounds_max_x: f32, bounds_max_y: f32,
        delta_time: f32,
        speed_factor: f32,
        max_age: f32,
        target_count: u32,
        random_seed: f32,
    ) {
        for i in 0..self.max_particles {
            let base = i * FLOATS_PER_PARTICLE;
            let should_be_active = (i as u32) < target_count;

            if !should_be_active {
                self.particles[base + 3] = STATE_INACTIVE;
                continue;
            }

            let state = self.particles[base + 3];

            // Activate if inactive
            if state == STATE_INACTIVE {
                self.respawn_valid(i, wind, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, random_seed);
                continue;
            }

            let x = self.particles[base + 0];
            let y = self.particles[base + 1];

            // Sample wind
            let sample = wind.sample(x, y);

            // NaN handling - respawn if invalid
            if !sample.valid {
                self.respawn_valid(i, wind, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, random_seed);
                continue;
            }

            // Update position
            let new_x = x + sample.u * delta_time * speed_factor;
            let new_y = y + sample.v * delta_time * speed_factor;
            let age = self.particles[base + 2] + 1.0;

            // Check respawn conditions
            let out_of_bounds = !in_bounds(new_x, new_y, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y);
            let too_old = age > max_age;

            if out_of_bounds || too_old {
                self.respawn_valid(i, wind, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, random_seed + age);
            } else {
                self.particles[base + 0] = new_x;
                self.particles[base + 1] = new_y;
                self.particles[base + 2] = age;
                self.particles[base + 3] = STATE_ACTIVE;
                self.particles[base + 4] = sample.u;
                self.particles[base + 5] = sample.v;
                self.particles[base + 6] = sample.speed;
            }
        }
    }

    fn respawn(&mut self, i: usize, min_x: f32, min_y: f32, max_x: f32, max_y: f32, seed: f32) {
        let base = i * FLOATS_PER_PARTICLE;
        let particle_seed = self.particles[base + 7] + seed;

        self.particles[base + 0] = min_x + random(particle_seed) * (max_x - min_x);
        self.particles[base + 1] = min_y + random(particle_seed + 1.0) * (max_y - min_y);
        self.particles[base + 2] = random(particle_seed + 2.0) * 20.0;  // stagger ages
        self.particles[base + 3] = STATE_RESPAWNING;
    }

    fn respawn_valid(&mut self, i: usize, wind: &WindSampler, min_x: f32, min_y: f32, max_x: f32, max_y: f32, seed: f32) {
        let base = i * FLOATS_PER_PARTICLE;
        let particle_seed = self.particles[base + 7] + seed;

        // Try up to 10 times to find valid position
        for attempt in 0..10 {
            let x = min_x + random(particle_seed + attempt as f32 * 7.13) * (max_x - min_x);
            let y = min_y + random(particle_seed + attempt as f32 * 7.13 + 1.0) * (max_y - min_y);

            if wind.is_valid(x, y) {
                self.particles[base + 0] = x;
                self.particles[base + 1] = y;
                self.particles[base + 2] = random(particle_seed + 2.0) * 20.0;
                self.particles[base + 3] = STATE_RESPAWNING;
                return;
            }
        }

        // Fallback to center
        self.particles[base + 0] = (min_x + max_x) * 0.5;
        self.particles[base + 1] = (min_y + max_y) * 0.5;
        self.particles[base + 2] = 0.0;
        self.particles[base + 3] = STATE_RESPAWNING;
    }

    #[wasm_bindgen]
    pub fn get_particles_ptr(&self) -> *const f32 {
        self.particles.as_ptr()
    }

    #[wasm_bindgen]
    pub fn get_particles_len(&self) -> usize {
        self.particles.len()
    }

    #[wasm_bindgen]
    pub fn get_max_particles(&self) -> usize {
        self.max_particles
    }

    pub fn get_particle(&self, i: usize) -> (f32, f32, f32, f32, f32) {
        let base = i * FLOATS_PER_PARTICLE;
        (
            self.particles[base + 0],  // x
            self.particles[base + 1],  // y
            self.particles[base + 2],  // age
            self.particles[base + 3],  // state
            self.particles[base + 6],  // speed
        )
    }
}
```

---

## Module: wind.rs (POC 2, 5)

```rust
use wasm_bindgen::prelude::*;
use crate::utils::*;

pub struct WindSample {
    pub u: f32,
    pub v: f32,
    pub speed: f32,
    pub valid: bool,
}

#[wasm_bindgen]
pub struct WindSampler {
    data: Vec<u8>,
    width: u32,
    height: u32,
    // Bounds in world coordinates (Mercator)
    min_x: f32, min_y: f32,
    max_x: f32, max_y: f32,
    // Encoding range
    u_min: f32, u_max: f32,
    v_min: f32, v_max: f32,
}

#[wasm_bindgen]
impl WindSampler {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![0; (width * height * 4) as usize],
            width,
            height,
            min_x: 0.0, min_y: 0.0,
            max_x: 1.0, max_y: 1.0,
            u_min: -15.0, u_max: 15.0,
            v_min: -15.0, v_max: 15.0,
        }
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

    #[wasm_bindgen]
    pub fn set_encoding(&mut self, u_min: f32, u_max: f32, v_min: f32, v_max: f32) {
        self.u_min = u_min;
        self.u_max = u_max;
        self.v_min = v_min;
        self.v_max = v_max;
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.data = vec![0; (width * height * 4) as usize];
    }

    /// Check if position has valid wind data (alpha > 0.5)
    #[wasm_bindgen]
    pub fn is_valid(&self, x: f32, y: f32) -> bool {
        let (px, py) = self.world_to_pixel(x, y);
        let idx = ((py as u32) * self.width + (px as u32)) as usize * 4;

        if idx + 3 >= self.data.len() {
            return false;
        }

        self.data[idx + 3] > 127  // alpha > 0.5
    }

    fn world_to_pixel(&self, x: f32, y: f32) -> (f32, f32) {
        let u = (x - self.min_x) / (self.max_x - self.min_x);
        let v = (y - self.min_y) / (self.max_y - self.min_y);

        let px = clamp(u * self.width as f32, 0.0, (self.width - 1) as f32);
        let py = clamp(v * self.height as f32, 0.0, (self.height - 1) as f32);

        (px, py)
    }

    fn sample_pixel(&self, px: u32, py: u32) -> (f32, f32, f32, bool) {
        let idx = (py * self.width + px) as usize * 4;

        if idx + 3 >= self.data.len() {
            return (0.0, 0.0, 0.0, false);
        }

        let r = self.data[idx] as f32 / 255.0;
        let g = self.data[idx + 1] as f32 / 255.0;
        let b = self.data[idx + 2] as f32 / 255.0;
        let a = self.data[idx + 3] as f32 / 255.0;

        (r, g, b, a > 0.5)
    }

    /// Sample wind with bilinear interpolation
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

        // All corners must be valid
        if !v00 || !v10 || !v01 || !v11 {
            return WindSample { u: 0.0, v: 0.0, speed: 0.0, valid: false };
        }

        let r = bilerp(r00, r10, r01, r11, tx, ty);
        let g = bilerp(g00, g10, g01, g11, tx, ty);

        // Decode: 0-1 → u_min..u_max
        let u = self.u_min + r * (self.u_max - self.u_min);
        let v = self.v_min + g * (self.v_max - self.v_min);
        let speed = (u * u + v * v).sqrt();

        WindSample { u, v, speed, valid: true }
    }
}
```

---

## Module: trail.rs (POC 3)

```rust
use wasm_bindgen::prelude::*;

/// Trail data layout: [x, y] * trail_length * max_particles
/// Index 0 = HEAD (newest), Index N-1 = TAIL (oldest)
#[wasm_bindgen]
pub struct TrailManager {
    trails: Vec<f32>,
    trail_length: usize,
    max_particles: usize,
}

#[wasm_bindgen]
impl TrailManager {
    #[wasm_bindgen(constructor)]
    pub fn new(max_particles: usize, trail_length: usize) -> Self {
        let trails = vec![0.0; max_particles * trail_length * 2];
        Self { trails, trail_length, max_particles }
    }

    /// CRITICAL: Must be called AFTER particle position update
    /// Order: MOVE particle → SHIFT trail → STORE new position
    #[wasm_bindgen]
    pub fn push_position(&mut self, particle_id: usize, x: f32, y: f32) {
        if particle_id >= self.max_particles {
            return;
        }

        let base = particle_id * self.trail_length * 2;

        // Shift all positions down (oldest gets dropped)
        for i in (1..self.trail_length).rev() {
            let dst = base + i * 2;
            let src = base + (i - 1) * 2;
            self.trails[dst] = self.trails[src];
            self.trails[dst + 1] = self.trails[src + 1];
        }

        // Store new position at HEAD (index 0)
        self.trails[base] = x;
        self.trails[base + 1] = y;
    }

    /// Reset all trail positions to a single point (on respawn)
    #[wasm_bindgen]
    pub fn reset_trail(&mut self, particle_id: usize, x: f32, y: f32) {
        if particle_id >= self.max_particles {
            return;
        }

        let base = particle_id * self.trail_length * 2;

        for i in 0..self.trail_length {
            self.trails[base + i * 2] = x;
            self.trails[base + i * 2 + 1] = y;
        }
    }

    #[wasm_bindgen]
    pub fn get_trails_ptr(&self) -> *const f32 {
        self.trails.as_ptr()
    }

    #[wasm_bindgen]
    pub fn get_trails_len(&self) -> usize {
        self.trails.len()
    }

    pub fn get_position(&self, particle_id: usize, trail_idx: usize) -> (f32, f32) {
        let base = particle_id * self.trail_length * 2;
        let idx = base + trail_idx * 2;
        (self.trails[idx], self.trails[idx + 1])
    }
}
```

---

## Module: vertex.rs

```rust
use wasm_bindgen::prelude::*;
use crate::particle::ParticleSimulator;
use crate::trail::TrailManager;

/// Vertex layout: [x, y, alpha, r, g, b] per vertex
const FLOATS_PER_VERTEX: usize = 6;

#[wasm_bindgen]
pub struct TrailVertexBuilder {
    vertices: Vec<f32>,
    vertex_count: usize,
}

#[wasm_bindgen]
impl TrailVertexBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(max_particles: usize, trail_length: usize) -> Self {
        // Each trail segment = 6 vertices (2 triangles)
        let max_segments = max_particles * (trail_length - 1);
        let max_vertices = max_segments * 6;

        Self {
            vertices: vec![0.0; max_vertices * FLOATS_PER_VERTEX],
            vertex_count: 0,
        }
    }

    /// Build vertices for all active particles
    /// Returns number of vertices generated
    #[wasm_bindgen]
    pub fn build(
        &mut self,
        particles: &ParticleSimulator,
        trails: &TrailManager,
        active_count: usize,
        trail_length: usize,
        trail_width: f32,
        opacity: f32,
    ) -> usize {
        self.vertex_count = 0;

        for p in 0..active_count.min(particles.get_max_particles()) {
            let (_, _, age, state, speed) = particles.get_particle(p);

            // Skip inactive particles
            if state == 3.0 {
                continue;
            }

            // Age fade (fade out near max age)
            let age_fade = 1.0 - (age / 100.0).min(1.0).max(0.0);

            // Speed to color (blue gradient)
            let speed_norm = (speed / 15.0).min(1.0);
            let r = 0.2 + speed_norm * 0.3;
            let g = 0.4 + speed_norm * 0.4;
            let b = 0.8 + speed_norm * 0.2;

            // Build segments
            for s in 0..(trail_length - 1) {
                let (x0, y0) = trails.get_position(p, s);
                let (x1, y1) = trails.get_position(p, s + 1);

                // Skip zero-length segments
                let dx = x1 - x0;
                let dy = y1 - y0;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 0.001 {
                    continue;
                }

                // Normal perpendicular to line
                let nx = -dy / len;
                let ny = dx / len;

                // Trail fade (head=1, tail=0)
                let t0 = s as f32 / (trail_length - 1) as f32;
                let t1 = (s + 1) as f32 / (trail_length - 1) as f32;

                let alpha0 = (1.0 - t0) * age_fade * opacity;
                let alpha1 = (1.0 - t1) * age_fade * opacity;

                // Width taper
                let w0 = trail_width * (1.0 - t0 * 0.7);
                let w1 = trail_width * (1.0 - t1 * 0.7);

                // Generate 6 vertices for quad
                self.add_vertex(x0 - nx * w0, y0 - ny * w0, alpha0, r, g, b);
                self.add_vertex(x0 + nx * w0, y0 + ny * w0, alpha0, r, g, b);
                self.add_vertex(x1 - nx * w1, y1 - ny * w1, alpha1, r, g, b);

                self.add_vertex(x1 - nx * w1, y1 - ny * w1, alpha1, r, g, b);
                self.add_vertex(x0 + nx * w0, y0 + ny * w0, alpha0, r, g, b);
                self.add_vertex(x1 + nx * w1, y1 + ny * w1, alpha1, r, g, b);
            }
        }

        self.vertex_count
    }

    fn add_vertex(&mut self, x: f32, y: f32, alpha: f32, r: f32, g: f32, b: f32) {
        let idx = self.vertex_count * FLOATS_PER_VERTEX;
        if idx + FLOATS_PER_VERTEX <= self.vertices.len() {
            self.vertices[idx] = x;
            self.vertices[idx + 1] = y;
            self.vertices[idx + 2] = alpha;
            self.vertices[idx + 3] = r;
            self.vertices[idx + 4] = g;
            self.vertices[idx + 5] = b;
            self.vertex_count += 1;
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
}
```

---

## Module: animation.rs (POC 7)

```rust
use wasm_bindgen::prelude::*;
use crate::particle::ParticleSimulator;
use crate::wind::WindSampler;
use crate::trail::TrailManager;
use crate::vertex::TrailVertexBuilder;

#[wasm_bindgen]
pub struct WindAnimation {
    particles: ParticleSimulator,
    trails: TrailManager,
    wind: WindSampler,
    vertex_builder: TrailVertexBuilder,

    // Config
    max_particles: usize,
    trail_length: usize,
    speed_multiplier: f32,
    trail_width: f32,
    opacity: f32,
    paused: bool,
}

#[wasm_bindgen]
impl WindAnimation {
    #[wasm_bindgen(constructor)]
    pub fn new(
        max_particles: u32,
        trail_length: u32,
        speed_multiplier: f32,
        trail_width: f32,
        opacity: f32,
    ) -> Self {
        let max_p = max_particles as usize;
        let trail_l = trail_length as usize;

        Self {
            particles: ParticleSimulator::new(max_p),
            trails: TrailManager::new(max_p, trail_l),
            wind: WindSampler::new(256, 256),
            vertex_builder: TrailVertexBuilder::new(max_p, trail_l),
            max_particles: max_p,
            trail_length: trail_l,
            speed_multiplier,
            trail_width,
            opacity,
            paused: false,
        }
    }

    /// Main update loop - call every frame
    #[wasm_bindgen]
    pub fn update(
        &mut self,
        delta_time: f32,
        view_min_x: f32, view_min_y: f32,
        view_max_x: f32, view_max_y: f32,
        particle_count: u32,
        max_age: f32,
        random_seed: f32,
    ) {
        if self.paused {
            return;
        }

        // Update particles with wind
        self.particles.update_with_wind(
            &self.wind,
            view_min_x, view_min_y,
            view_max_x, view_max_y,
            delta_time,
            self.speed_multiplier,
            max_age,
            particle_count,
            random_seed,
        );

        // Update trails
        let count = particle_count.min(self.max_particles as u32) as usize;
        for i in 0..count {
            let (x, y, _, state, _) = self.particles.get_particle(i);

            if state == 3.0 {
                continue;
            }

            if state == 2.0 {
                // Respawning - reset trail
                self.trails.reset_trail(i, x, y);
            } else {
                // Active - push position
                self.trails.push_position(i, x, y);
            }
        }

        // Build vertices
        self.vertex_builder.build(
            &self.particles,
            &self.trails,
            count,
            self.trail_length,
            self.trail_width,
            self.opacity,
        );
    }

    #[wasm_bindgen]
    pub fn set_wind_data(&mut self, data: &[u8], width: u32, height: u32) {
        self.wind.resize(width, height);
        self.wind.set_data(data);
    }

    #[wasm_bindgen]
    pub fn set_wind_bounds(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.wind.set_bounds(min_x, min_y, max_x, max_y);
    }

    #[wasm_bindgen]
    pub fn set_wind_encoding(&mut self, u_min: f32, u_max: f32, v_min: f32, v_max: f32) {
        self.wind.set_encoding(u_min, u_max, v_min, v_max);
    }

    // Runtime controls
    #[wasm_bindgen]
    pub fn set_speed_multiplier(&mut self, mult: f32) {
        self.speed_multiplier = mult;
    }

    #[wasm_bindgen]
    pub fn set_trail_width(&mut self, width: f32) {
        self.trail_width = width;
    }

    #[wasm_bindgen]
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    #[wasm_bindgen]
    pub fn resume(&mut self) {
        self.paused = false;
    }

    #[wasm_bindgen]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    // Vertex access
    #[wasm_bindgen]
    pub fn get_vertices_ptr(&self) -> *const f32 {
        self.vertex_builder.get_vertices_ptr()
    }

    #[wasm_bindgen]
    pub fn get_vertex_count(&self) -> usize {
        self.vertex_builder.get_vertex_count()
    }
}
```

---

## JavaScript Integration

```javascript
import init, { WindAnimation } from './pkg/meteo_animation_wasm.js';

async function main() {
    const wasm = await init();

    const animation = new WindAnimation(
        10000,  // max_particles
        30,     // trail_length
        1.0,    // speed_multiplier
        2.0,    // trail_width
        0.8     // opacity
    );

    // Set wind data (from tile fetch)
    animation.set_wind_data(tileData, width, height);
    animation.set_wind_bounds(minX, minY, maxX, maxY);
    animation.set_wind_encoding(-15, 15, -15, 15);

    function frame() {
        const dt = getDeltaTime();
        const bounds = getViewBounds();

        animation.update(
            dt,
            bounds.minX, bounds.minY,
            bounds.maxX, bounds.maxY,
            getParticleCount(zoom),
            100,  // max_age
            Math.random()
        );

        // Get vertices from WASM
        const ptr = animation.get_vertices_ptr();
        const count = animation.get_vertex_count();
        const vertices = new Float32Array(wasm.memory.buffer, ptr, count * 6);

        // Upload to WebGL and render
        gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.DYNAMIC_DRAW);
        gl.drawArrays(gl.TRIANGLES, 0, count);

        requestAnimationFrame(frame);
    }

    frame();
}
```
