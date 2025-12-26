# meteo-animation Package

## ⚠️ PRODUCTION CONTEXT

**This is NOT a throwaway POC.** This library is designed for production use in meteorological visualization applications.

Handle with **extra care**. Each POC is a production building block. The POC approach is designed to make AI-assisted development comfortable with incremental, testable steps.

---

## Quick Reference

| Task | Command |
|------|---------|
| Run demo server | `bun run demo` |
| Build library | `bun run build` |
| Run tests | `bun run test` |
| Type check | `bun run type-check` |

## Demo Server

```bash
bun run demo
# Server runs at http://localhost:5180/
```

Demo files are in `demo/` directory:
- `index.html` - Demo index page
- `poc-5-minimal.html` - Minimal WebGPU + Leaflet integration (REFERENCE IMPLEMENTATION)
- `poc-1-webgl.html` - **WebGL + WASM** particle boundary (POC 1 migrasi)
- `poc-2-webgl.html` - **WebGL + WASM** NaN handling (POC 2 migrasi)
- `poc-3-webgl.html` - **WebGL + WASM** wind trails (POC 3 migrasi)
- `poc-4-webgl.html` - **WebGL + WASM** map coordinate sync (POC 4 migrasi)
- `poc-5-webgl.html` - **WebGL + WASM** Leaflet integration (POC 5 migrasi)
- `poc-6-webgl.html` - **WebGL + WASM** dynamic atlas with tile loading (POC 6 migrasi)

## POC 5 Key Findings

### 1. Trail Rendering Order (Critical!)
```
MOVE particle → SHIFT trail array → STORE position
```
Wrong order causes "backward" visual (tail in front).

### 2. Trail Buffer: SHIFT Approach
- Index 0 = newest (HEAD), Index N-1 = oldest (TAIL)
- Simpler than ring buffer, no trail_head tracking in render

### 3. Trail Fade
```wgsl
trail_t = 1.0 - vertex_idx / (len-1)
// HEAD (vertex 0) = opaque, TAIL = transparent
```

### 4. Zoom Scaling
- Use lookup tables + linear interpolation
- Scale both COUNT and SPEED at high zoom
- High zoom = fewer + slower particles

### 5. Leaflet Fractional Zoom
```javascript
L.map('map', {
  zoomSnap: 0,
  zoomDelta: 0.5,
  wheelPxPerZoomLevel: 120,
})
```

### 6. Wind Texture Encoding
- RGBA: R=u, G=v, B=speed, A=validity
- Encode: `(value + 15) / 30 * 255`
- Decode: `(sample - 0.5) * 2.0 * maxRange`

### 7. Bind Group Timing
Create AFTER all resources ready. POC 5 failed due to constructor timing.

---

## WebGL Migration (WASM + Rust)

Target migrasi dari WebGPU ke WebGL 2 menggunakan WASM (Rust) untuk simulasi partikel.

### ⚠️ CRITICAL: Incremental Development Rule

**POC 1-7 MUST be sequential and incremental!**

```
POC 1 → POC 2 → POC 3 → ... → POC 7
  │       │       │
  │       │       └── REUSE particle.rs, wind.rs, ADD trail.rs
  │       └── REUSE particle.rs, ADD wind.rs
  └── CREATE particle.rs
```

**Rules:**
1. Each POC builds on top of previous POCs
2. NEVER rewrite existing modules - only extend them
3. New POC = add new module OR add new method to existing module
4. All previous tests MUST still pass
5. Demo files can import ALL modules from previous POCs

**Example - POC 3 extends POC 2:**
```rust
// particle.rs - ADD method, don't rewrite
impl ParticleSimulator {
    // POC 1: existing method
    pub fn update(&mut self, ...) { ... }

    // POC 2: added method
    pub fn update_with_wind(&mut self, wind: &WindSampler, ...) { ... }

    // POC 3: added method (reuses WindSampler from POC 2)
    pub fn update_with_trails(&mut self, wind: &WindSampler, trails: &mut TrailManager, ...) { ... }
}
```

**Why this matters:**
- WebGPU implementation rewrote code at each POC = frustrating & wasteful
- Incremental approach = less code, more reuse, easier maintenance
- Final POC 7 should just compose existing modules

### Quick Reference

| Keputusan | Pilihan |
|-----------|---------|
| Target | WebGL 2 |
| Pendekatan | WASM (Rust) + WebGL |
| Bahasa | Rust + wasm-bindgen + wasm-pack |
| Arsitektur | Single WASM crate untuk semua POC |

### Documentation

- [Migration Overview](./docs/webgl-migration-overview.md) - Arsitektur dan keputusan
- [Rust Crate Spec](./docs/webgl-rust-crate.md) - Module dan API Rust
- [GLSL Shaders](./docs/webgl-shaders.md) - Konversi shader WGSL → GLSL
- [POC Analysis](./docs/webgl-poc-analysis.md) - Analisis detail setiap POC

### WASM Crate Structure

```
crates/meteo-animation-wasm/
├── Cargo.toml
└── src/
    ├── lib.rs              # Entry point
    ├── utils.rs            # random(), lerp(), clamp()
    ├── particle.rs         # ParticleSimulator
    ├── wind.rs             # WindSampler (+ from_atlas)
    ├── trail.rs            # TrailManager
    ├── atlas.rs            # DynamicAtlas, AtlasBounds, TileCoord (POC 6)
    ├── tile.rs             # TileCoordinator (POC 6)
    ├── vertex.rs           # TrailVertexBuilder
    └── animation.rs        # WindAnimation (POC 7 API)
```

### Build Commands

```bash
# Install Rust (jika belum)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack

# Build WASM
cd crates/meteo-animation-wasm
~/.cargo/bin/wasm-pack build --target web --release
```

### TDD (Test-Driven Development)

Rust crate menggunakan TDD approach:

| Task | Command |
|------|---------|
| Run tests | `cd crates/meteo-animation-wasm && cargo test` |
| Run with output | `cargo test -- --nocapture` |
| Build WASM | `~/.cargo/bin/wasm-pack build --target web --release` |

**TDD Cycle:**
1. Write failing test (RED)
2. Write minimal code to pass (GREEN)
3. Refactor if needed (REFACTOR)

**Current Test Coverage (106 tests):**
- `utils.rs`: 15 tests (random, mix, in_bounds, clamp, bilerp)
- `particle.rs`: 16 tests (new, update, respawn, deactivate, update_with_wind, update_with_trails)
- `wind.rs`: 17 tests (WindSampler, sample, is_valid, bilinear, from_atlas)
- `trail.rs`: 9 tests (TrailManager, push, shift, reset)
- `vertex.rs`: 9 tests (TrailVertexBuilder, alpha fade, width taper)
- `atlas.rs`: 26 tests (DynamicAtlas, AtlasBounds, TileCoord, viewport, sample, real tiles)
- `tile.rs`: 14 tests (TileCoordinator, zoom offset, filter_unloaded)

### POC → Module Mapping

| POC | Rust Module | Fitur |
|-----|-------------|-------|
| POC 1 | `particle.rs` | Boundary detection, respawn |
| POC 2 | `wind.rs` | Wind sampling, NaN handling |
| POC 3 | `trail.rs`, `vertex.rs` | Trail history (shift approach) |
| POC 4 | **NONE (reuse)** | View transform (shader only) |
| POC 5 | **NONE (reuse)** | Leaflet integration, zoom scaling (JS only) |
| POC 6 | `atlas.rs`, `tile.rs`, `wind.rs` | Dynamic atlas, viewport-based tiles, zoom-2 strategy |
| POC 7 | `animation.rs` | Unified WindAnimation API |

## POC 6 Key Findings

### 1. Dynamic Atlas Architecture
```
Viewport → +20% buffer → Atlas bounds → Tile requests at zoom-2
```
Atlas is a "physics layer" - resolution independent of visual zoom.

### 2. Zoom-2 Strategy
At view zoom 7, request tiles at zoom 5 → 4x fewer tiles per zoom level.
Saves bandwidth, acceptable for particle physics simulation.

### 3. TileCoordinator
Manages which tiles are loaded, prevents duplicate requests:
```javascript
const neededTiles = atlas.update_viewport(bounds, dataZoom);
const unloaded = coordinator.filter_unloaded(neededTiles);
```

### 4. WindSampler Integration
```rust
// Create sampler from atlas
let sampler = WindSampler::from_atlas(&atlas);
// Or update existing
sampler.update_from_atlas(&atlas);
```
