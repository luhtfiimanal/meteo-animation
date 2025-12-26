# meteo-animation

Wind particle animation library using **Rust WASM + WebGL 2**. Visualize meteorological vector fields with smooth, GPU-accelerated particle trails.

[![Demo](https://img.shields.io/badge/demo-live-brightgreen)](https://luhtfiimanal.github.io/meteo-animation/)
[![Release](https://img.shields.io/github/v/release/luhtfiimanal/meteo-animation)](https://github.com/luhtfiimanal/meteo-animation/releases)
[![Tests](https://img.shields.io/badge/tests-106%20passed-success)](https://github.com/luhtfiimanal/meteo-animation)

**[Live Demo](https://luhtfiimanal.github.io/meteo-animation/)** | **[Download WASM](https://github.com/luhtfiimanal/meteo-animation/releases)**

---

## Features

- **High Performance** - Rust WASM for particle physics, WebGL 2 for rendering
- **Smooth Pan/Zoom** - FixedAtlas architecture prevents visual glitches
- **Tile Support** - Load wind tiles from any WMTS-compatible server
- **Trail Effects** - Configurable particle trails with alpha fade and width taper
- **NaN Handling** - Graceful handling of land/invalid data regions
- **Mobile Ready** - Responsive design with touch support

## Quick Start

### 1. Download WASM

```bash
# Download from GitHub Release
VERSION=v0.1.0
BASE_URL="https://github.com/luhtfiimanal/meteo-animation/releases/download/${VERSION}"

mkdir -p wasm
curl -L "${BASE_URL}/meteo_animation_wasm_bg.wasm" -o wasm/meteo_animation_wasm_bg.wasm
curl -L "${BASE_URL}/meteo_animation_wasm.js" -o wasm/meteo_animation_wasm.js
curl -L "${BASE_URL}/meteo_animation_wasm.d.ts" -o wasm/meteo_animation_wasm.d.ts
```

### 2. Initialize WASM

```javascript
import init, {
  ParticleSimulator,
  WindSampler,
  TrailManager,
  TrailVertexBuilder,
  FixedAtlas,
  TileCoordinator
} from './wasm/meteo_animation_wasm.js';

// Initialize WASM module
await init('./wasm/meteo_animation_wasm_bg.wasm');
```

### 3. Create Components

```javascript
// Define world bounds (Web Mercator)
const bounds = new Float64Array([
  -20037508, -20037508,  // min X, min Y
   20037508,  20037508   // max X, max Y
]);

// Particle simulator (handles physics)
const simulator = new ParticleSimulator(10000, bounds);

// Trail manager (stores particle history)
const trails = new TrailManager(10000, 12);  // 10k particles, 12 trail points

// Vertex builder (generates WebGL vertices)
const vertexBuilder = new TrailVertexBuilder(10000, 12);

// Wind sampler (samples wind from texture)
const windSampler = new WindSampler(
  windData,        // Uint8Array RGBA
  256, 256,        // texture size
  bounds,          // world bounds
  15.0             // max wind speed (m/s)
);
```

### 4. Animation Loop

```javascript
function animate(time) {
  const dt = 0.016;  // 60fps

  // Update particles with wind
  simulator.update_with_trails(windSampler, trails, dt, 50.0);

  // Build vertices for WebGL
  const vertices = vertexBuilder.build(
    simulator, trails,
    2.0,   // trail width
    1.0,   // alpha
    5.0    // max age
  );

  // Upload to WebGL and render...
  requestAnimationFrame(animate);
}
```

## Tile Loading (Advanced)

For loading tiles from a WMTS server:

```javascript
// Fixed-size atlas (matches screen resolution)
const atlas = new FixedAtlas(1920, 1080);

// Tile coordinator (manages tile requests)
const coordinator = new TileCoordinator(-2);  // zoom offset

// On viewport change
function onViewportChange(bounds, zoom) {
  const dataZoom = coordinator.get_data_zoom(zoom);

  // Update atlas bounds
  atlas.update_bounds(
    bounds.minX, bounds.minY,
    bounds.maxX, bounds.maxY,
    dataZoom
  );

  // Get required tiles
  const tiles = atlas.get_visible_tiles();
  const unloaded = coordinator.filter_unloaded(tiles);

  // Load and resample tiles
  for (const [z, x, y] of unloaded) {
    const tileData = await fetchTile(z, x, y);
    atlas.resample_tile(z, x, y, tileData);
    coordinator.mark_loaded(z, x, y);
  }

  // Create wind sampler from atlas
  const sampler = WindSampler.from_atlas(atlas);
}
```

## Wind Data Encoding

Expected RGBA texture format:

| Channel | Value | Formula |
|---------|-------|---------|
| R | u component | `(u + 15) / 30 * 255` |
| G | v component | `(v + 15) / 30 * 255` |
| B | speed | `speed / 35 * 255` |
| A | validity | `0` = invalid, `255` = valid |

Decoding in shader:
```glsl
vec2 wind = (texture.rg - 0.5) * 2.0 * 15.0;  // -15 to +15 m/s
float valid = texture.a > 0.5 ? 1.0 : 0.0;
```

## API Reference

### ParticleSimulator

| Method | Description |
|--------|-------------|
| `new(count, bounds)` | Create simulator with particle count and world bounds |
| `update(dt, speed)` | Update particles (no wind) |
| `update_with_wind(sampler, dt, speed)` | Update with wind sampling |
| `update_with_trails(sampler, trails, dt, speed)` | Update with wind + trails |
| `get_positions()` | Get Float32Array of [x, y, x, y, ...] |
| `get_ages()` | Get Float32Array of particle ages |

### WindSampler

| Method | Description |
|--------|-------------|
| `new(data, w, h, bounds, max_speed)` | Create from RGBA texture data |
| `from_atlas(atlas)` | Create from FixedAtlas |
| `sample(x, y)` | Sample wind at world coordinate → [u, v] |
| `is_valid(x, y)` | Check if coordinate has valid data |

### TrailManager

| Method | Description |
|--------|-------------|
| `new(count, length)` | Create trail storage |
| `push(simulator)` | Push current positions to trail history |
| `get_trail(index)` | Get trail points for particle |

### FixedAtlas

| Method | Description |
|--------|-------------|
| `new(width, height)` | Create fixed-size atlas |
| `update_bounds(...)` | Update world bounds (no resize) |
| `resample_tile(z, x, y, data)` | Resample tile into atlas |
| `get_data()` | Get atlas RGBA data |

## Building from Source

```bash
# Prerequisites
cargo install wasm-pack

# Clone
git clone https://github.com/luhtfiimanal/meteo-animation.git
cd meteo-animation

# Build WASM
cd crates/meteo-animation-wasm
wasm-pack build --target web --release

# Run tests
cargo test
```

## Project Structure

```
meteo-animation/
├── crates/meteo-animation-wasm/
│   └── src/
│       ├── lib.rs          # WASM entry point
│       ├── particle.rs     # ParticleSimulator
│       ├── wind.rs         # WindSampler
│       ├── trail.rs        # TrailManager
│       ├── vertex.rs       # TrailVertexBuilder
│       ├── atlas.rs        # DynamicAtlas
│       ├── fixed_atlas.rs  # FixedAtlas
│       ├── tile.rs         # TileCoordinator
│       └── utils.rs        # Helpers
├── demo/                   # Demo HTML files
└── docs/                   # Documentation
```

## License

MIT

## Credits

Developed for meteorological data visualization.
