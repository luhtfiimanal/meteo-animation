# WebGL POC Analysis

Analisis detail dari setiap POC WebGPU dan tantangan konversi ke WebGL + WASM.

---

## POC 1: Particle Boundary Behavior

**File:** `demo/poc-particle-boundary.html`
**Fokus:** Lifecycle partikel, boundary detection, respawn logic

### Komponen WebGPU

| Komponen | Lokasi | Deskripsi |
|----------|--------|-----------|
| Compute Shader | Line 321-437 | Update posisi, state management, respawn |
| Render Shader | Line 439-533 | Point sprites sebagai quad (6 vertices/particle) |
| Particle Buffer | Line 648-654 | Storage buffer: `{pos, age, state}` per particle |
| Stats Buffer | Line 668-676 | Atomic counters untuk respawn/exit events |

### Key Learnings

1. **Fixed buffer size** dengan `target_count` uniform untuk control aktif/inaktif
2. **Particle state machine**:
   - 0 = active (bergerak normal)
   - 1 = exiting (keluar boundary)
   - 2 = respawning (sedang respawn)
   - 3 = inactive (tidak aktif)
3. **Random number generator** (sama di WGSL dan GLSL):
   ```
   fract(sin(seed * 12.9898 + time * 78.233) * 43758.5453)
   ```
4. **GPU readback** via staging buffer dengan `mapAsync()`

### Tantangan WebGL

| WebGPU Feature | WebGL Equivalent | Solusi |
|----------------|------------------|--------|
| `atomic<u32>` untuk stats | Tidak ada | CPU counter di WASM |
| Storage buffer read/write | Tidak ada | CPU array + VBO upload |
| Compute dispatch | Tidak ada | WASM update loop |

### WASM Module Mapping

```
POC 1 → ParticleSimulator
├── new(max_particles)
├── update_basic(uniforms)
├── get_particles_ptr() → *const f32
└── get_particle_count() → usize
```

---

## POC 2: NaN/NoData Handling

**File:** `demo/poc-nan-handling.html`
**Fokus:** Wind sampling, invalid region detection, valid spawn

### Komponen WebGPU

| Komponen | Lokasi | Deskripsi |
|----------|--------|-----------|
| Compute Shader | Line 378-525 | Wind sampling + NaN detection + respawn |
| NaN Region Shader | Line 618-668 | Background visualization |
| Wind Texture | Line 727-757 | RGBA: R=u, G=v, B=magnitude, A=validity |
| Texture Sampler | Line 760-765 | Linear filtering, clamp-to-edge |

### Wind Texture Encoding

```
┌─────────────────────────────────────────┐
│  RGBA Wind Texture Encoding             │
├─────────────────────────────────────────┤
│  R = (u_component + 15) / 30 * 255      │
│  G = (v_component + 15) / 30 * 255      │
│  B = speed / max_speed * 255            │
│  A = validity (0 = NaN, 255 = valid)    │
├─────────────────────────────────────────┤
│  Decode (di WASM):                      │
│  u = (R/255 - 0.5) * 2.0 * 15.0         │
│  v = (G/255 - 0.5) * 2.0 * 15.0         │
│  valid = A > 127                        │
└─────────────────────────────────────────┘
```

### Key Learnings

1. **Alpha channel untuk validity**: A < 0.5 = NaN/NoData
2. **NaN detection di WASM**: Sample texture, check alpha
3. **`find_valid_spawn()`**: Loop max 10 attempts untuk cari posisi valid
4. **Bind group timing**: Harus dibuat SETELAH texture siap

### Tantangan WebGL

| WebGPU Feature | WebGL Equivalent | Solusi |
|----------------|------------------|--------|
| `textureSampleLevel()` | CPU sampling | WASM WindSampler |
| Compute shader sampling | Tidak ada | Pass texture data ke WASM |

### WASM Module Mapping

```
POC 2 → WindSampler
├── new(width, height)
├── set_data(&[u8])        // RGBA texture data
├── set_bounds(min, max)   // Mercator bounds
├── sample(x, y) → WindSample { u, v, speed, valid }
└── is_valid(x, y) → bool

POC 2 → ParticleSimulator (extended)
├── update_with_wind(&WindSampler, uniforms)
└── find_valid_spawn(&WindSampler, bounds) → (x, y)
```

---

## POC 3: Wind Trail Rendering

**File:** `demo/poc-wind-trail.html`
**Fokus:** Trail history, fading effect, efficient rendering

### Komponen WebGPU

| Komponen | Lokasi | Deskripsi |
|----------|--------|-----------|
| Compute Shader | Line 364-514 | Particle update + trail history shift |
| Background Shader | Line 517-556 | NaN region visualization |
| Render Shader | Line 559-679 | Line strip sebagai instanced quads |
| Trail Buffer | Line 968-975 | `vec2 * MAX_TRAIL_LENGTH * MAX_PARTICLES` |

### Trail Buffer Layout

```
┌───────────────────────────────────────────────────────────────────┐
│  Trail Buffer (SHIFT approach, bukan ring buffer)                  │
├───────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Particle 0:  [x₀,y₀] [x₁,y₁] [x₂,y₂] ... [x₂₉,y₂₉]               │
│               HEAD ←────────────────────────→ TAIL                 │
│               newest                          oldest               │
│                                                                    │
│  Particle 1:  [x₀,y₀] [x₁,y₁] [x₂,y₂] ... [x₂₉,y₂₉]               │
│               ...                                                  │
│                                                                    │
│  Memory: trail_length * 2 * sizeof(f32) * max_particles            │
│          30 * 2 * 4 * 10000 = 2.4 MB                               │
│                                                                    │
└───────────────────────────────────────────────────────────────────┘
```

### Critical Order: MOVE → SHIFT → STORE

```
❌ WRONG ORDER (causes backward visual):
   STORE position → SHIFT → MOVE

✅ CORRECT ORDER:
   1. MOVE particle (update position)
   2. SHIFT trail array (drop oldest)
   3. STORE new position at index 0
```

### Key Learnings

1. **SHIFT approach** lebih simple dari ring buffer:
   - Index 0 = newest (HEAD)
   - Index N-1 = oldest (TAIL)
   - Tidak perlu track `trail_head` index
2. **Trail topology**: 6 vertices per segment (2 triangles = quad)
3. **Instance encoding** (WebGPU):
   ```
   particle_id = instance / segments
   segment_id = instance % segments
   ```
4. **Width taper**: `width * (1.0 - t * 0.7)` menuju tail
5. **Alpha fade**: `alpha = (1.0 - t) * 0.8`

### Tantangan WebGL

| WebGPU Feature | WebGL Equivalent | Solusi |
|----------------|------------------|--------|
| Trail buffer (15+ MB storage) | CPU array + VBO | WASM TrailManager |
| Compute shader untuk shift | Tidak ada | WASM shift loop |
| Instanced rendering | WebGL 2 instancing | Single batched draw |

### WASM Module Mapping

```
POC 3 → TrailManager
├── new(max_particles, trail_length)
├── push_position(particle_id, x, y)  // SHIFT then store
├── reset_trail(particle_id, x, y)    // On respawn
├── get_trail(particle_id) → &[f32]
└── get_trails_ptr() → *const f32

POC 3 → TrailVertexBuilder
├── build(&TrailManager, &ParticleSimulator, trail_width, active_count)
├── get_vertices_ptr() → *const f32
└── get_vertex_count() → usize
```

### Vertex Format (output dari TrailVertexBuilder)

```
Per vertex: [x, y, alpha, r, g, b] = 6 floats = 24 bytes
Per segment: 6 vertices = 36 floats = 144 bytes (2 triangles)
Per particle (30 segments): 29 segments = 4176 bytes
Total (10000 particles): ~40 MB vertex data
```

---

## POC 5: Minimal Complete (Reference)

**File:** `demo/poc-5-minimal.html`
**Fokus:** Integration dengan Leaflet, zoom scaling

### Key Features

1. **Zoom-based scaling**:
   ```javascript
   const PARTICLE_COUNTS = { 3: 1000, 5: 2000, 7: 3000, 10: 5000 };
   const SPEED_MULTIPLIERS = { 3: 2.0, 7: 1.0, 10: 0.5 };
   ```
2. **Leaflet fractional zoom**:
   ```javascript
   L.map('map', { zoomSnap: 0, zoomDelta: 0.5 });
   ```
3. **Real tile fetching** dari meteo-tiler API

### Zoom Scaling Logic

```
┌─────────────────────────────────────────────────────────────────┐
│  Zoom Level │ Particle Count │ Speed Multiplier │ Trail Width  │
├─────────────┼────────────────┼──────────────────┼──────────────┤
│      3      │     1,000      │       2.0        │     2.0      │
│      5      │     2,000      │       1.5        │     1.8      │
│      7      │     3,000      │       1.0        │     1.5      │
│     10      │     5,000      │       0.5        │     1.2      │
│     12      │     8,000      │       0.3        │     1.0      │
└─────────────────────────────────────────────────────────────────┘

Higher zoom = MORE particles, SLOWER movement, THINNER trails
```

---

## POC 6: Multi-Tile Stitching

**Fokus:** Handling multiple wind tiles, seamless boundaries

### Tile Stitching Logic

```
┌─────────────────────────────────────────────────────────────────┐
│  Tile Stitching (TileStitcher in Rust)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────┬─────┬─────┐                                            │
│  │ T00 │ T01 │ T02 │                                            │
│  ├─────┼─────┼─────┤  →  Stitch into single large texture       │
│  │ T10 │ T11 │ T12 │                                            │
│  └─────┴─────┴─────┘                                            │
│                                                                  │
│  Each tile: 256x256 RGBA                                        │
│  Stitched: (3*256) x (2*256) = 768x512                          │
│                                                                  │
│  TileStitcher:                                                  │
│  - stitch(tiles: &[TileData], layout: TileLayout) → Vec<u8>     │
│  - Handles edge blending at tile boundaries                     │
│  - Returns unified wind texture for WindSampler                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## POC 7: Unified API (Final)

**Fokus:** Production-ready API, semua fitur unified

### WindAnimation API

```rust
#[wasm_bindgen]
pub struct WindAnimation {
    particles: ParticleSimulator,
    trails: TrailManager,
    wind: WindSampler,
    vertex_builder: TrailVertexBuilder,

    // Runtime config
    speed_multiplier: f32,
    opacity: f32,
    paused: bool,
}

#[wasm_bindgen]
impl WindAnimation {
    // Construction
    pub fn new(config: &WindAnimationConfig) -> Self;

    // Frame update
    pub fn update(&mut self, delta_time: f32, view_bounds: &ViewBounds);

    // Get vertices for WebGL
    pub fn get_vertices(&self) -> *const f32;
    pub fn get_vertex_count(&self) -> usize;

    // Runtime controls
    pub fn set_speed_multiplier(&mut self, mult: f32);
    pub fn set_opacity(&mut self, opacity: f32);
    pub fn set_particle_count(&mut self, count: u32);
    pub fn pause(&mut self);
    pub fn resume(&mut self);

    // Wind data
    pub fn set_wind_data(&mut self, data: &[u8], width: u32, height: u32);
    pub fn set_wind_bounds(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32);
}
```

### JavaScript Integration

```javascript
import init, { WindAnimation, WindAnimationConfig } from './pkg/meteo_animation_wasm.js';

async function createWindLayer(map) {
    const wasm = await init();

    // Create animation
    const config = new WindAnimationConfig();
    config.max_particles = 10000;
    config.trail_length = 30;
    config.speed_multiplier = 1.0;
    config.opacity = 0.8;

    const animation = new WindAnimation(config);

    // Setup WebGL overlay (Leaflet custom layer)
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl2');
    const program = createShaderProgram(gl, TRAIL_VS, TRAIL_FS);
    const vao = setupTrailVAO(gl, program);

    // Animation loop
    function frame() {
        // Get map bounds
        const bounds = map.getBounds();
        const zoom = map.getZoom();

        // Adjust particle count based on zoom
        animation.set_particle_count(getParticleCount(zoom));
        animation.set_speed_multiplier(getSpeedMultiplier(zoom));

        // Update simulation
        animation.update(deltaTime, {
            min_x: bounds.getWest(),
            min_y: bounds.getSouth(),
            max_x: bounds.getEast(),
            max_y: bounds.getNorth(),
        });

        // Get vertices from WASM
        const ptr = animation.get_vertices();
        const count = animation.get_vertex_count();
        const vertices = new Float32Array(wasm.memory.buffer, ptr, count * 6);

        // Upload & render
        gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.DYNAMIC_DRAW);
        gl.drawArrays(gl.TRIANGLES, 0, count);

        requestAnimationFrame(frame);
    }

    frame();
}
```

---

## Memory Comparison

| Component | WebGPU | WebGL + WASM |
|-----------|--------|--------------|
| Particle data | GPU Storage ~400KB | WASM heap ~400KB |
| Trail data | GPU Storage ~2.4MB | WASM heap ~2.4MB |
| Vertex data | GPU generated | WASM heap ~40MB |
| Wind texture | GPU Texture | JS + WASM ~1MB |
| **Total** | ~3MB GPU | ~44MB RAM |

**Note:** WebGL + WASM menggunakan lebih banyak RAM karena vertex data di-generate di CPU.

---

## Performance Targets

| Metric | WebGPU | WebGL + WASM Target |
|--------|--------|---------------------|
| Particle count | 100,000+ | 10,000-50,000 |
| FPS | 60 | 60 |
| Frame time | <5ms | <16ms |
| CPU usage | ~5% | ~30% |

---

## Related Docs

- [Migration Overview](./webgl-migration-overview.md)
- [Rust Crate Specification](./webgl-rust-crate.md)
- [GLSL Shaders](./webgl-shaders.md)
