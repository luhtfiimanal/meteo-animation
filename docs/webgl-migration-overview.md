# WebGL Migration Overview

## Keputusan

| Aspek | Keputusan |
|-------|-----------|
| Target | **WebGL 2** |
| Urutan | **POC 1 → 7** (Sequential, sync ke POC 7) |
| Pendekatan | **WASM (Rust) + WebGL** |
| Bahasa | **Rust** + wasm-bindgen + wasm-pack |
| Arsitektur | **Single WASM crate** untuk semua POC |

---

## Mengapa WASM + WebGL?

WebGL tidak memiliki **compute shader** seperti WebGPU. Alternatif:

| Pendekatan | Performance | Complexity | Max Particles |
|------------|-------------|------------|---------------|
| Pure JavaScript | Slow | Low | ~5,000-10,000 |
| **WASM (Rust)** | Fast | Medium | **~50,000-100,000** |
| Transform Feedback | Fast | High | ~50,000+ |
| Ping-pong Texture | Fastest | Very High | ~1,000,000+ |

**WASM dipilih karena:**
1. 10-50x lebih cepat dari JavaScript
2. SIMD support untuk parallel processing
3. Predictable memory layout (no GC pauses)
4. Logic di Rust = type-safe, reusable untuk WebGPU
5. Debugging masih possible (console.log dari JS side)

---

## Arsitektur

```
┌─────────────────────────────────────────────────────────────────┐
│                    meteo-animation-wasm                         │
│                    (Single Rust Crate)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  POC 1-2    │  │   POC 3     │  │  POC 5-6    │             │
│  │  Particle   │──│   Trail     │──│   Wind      │             │
│  │  Simulator  │  │   Manager   │  │   Sampler   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         │                │                │                     │
│         └────────────────┼────────────────┘                     │
│                          │                                      │
│                          ▼                                      │
│                 ┌─────────────────┐                            │
│                 │     POC 7       │                            │
│                 │  WindAnimation  │  ← Final Library API       │
│                 │   (unified)     │                            │
│                 └─────────────────┘                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### POC → Module Mapping

| POC | Rust Module | Fitur |
|-----|-------------|-------|
| POC 1 | `particle.rs` | Boundary detection, respawn |
| POC 2 | `wind.rs` | Wind sampling, NaN handling |
| POC 3 | `trail.rs` | Trail history (shift approach) |
| POC 5 | `wind.rs` | Bilinear interpolation |
| POC 6 | `tile.rs` | Multi-tile stitching |
| POC 7 | `animation.rs` | Unified WindAnimation API |

---

## File Structure

```
meteo-animation/
├── crates/
│   └── meteo-animation-wasm/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs              # Entry point
│       │   ├── utils.rs            # Shared utilities
│       │   ├── particle.rs         # ParticleSimulator
│       │   ├── wind.rs             # WindSampler
│       │   ├── trail.rs            # TrailManager
│       │   ├── tile.rs             # TileStitcher
│       │   ├── vertex.rs           # Trail vertex builder
│       │   └── animation.rs        # WindAnimation (POC 7)
│       └── pkg/                    # wasm-pack output
│
├── demo/
│   ├── poc-1-webgl.html
│   ├── poc-2-webgl.html
│   ├── poc-3-webgl.html
│   ├── poc-5-webgl.html
│   ├── poc-6-webgl.html
│   └── poc-7-webgl.html
│
├── docs/
│   ├── webgl-migration-overview.md  # This file
│   ├── webgl-rust-crate.md          # Rust module specs
│   ├── webgl-shaders.md             # GLSL shaders
│   └── webgl-poc-analysis.md        # POC analysis
│
└── src/                             # Existing WebGPU code
```

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  Frame Loop                                                      │
│                                                                  │
│  1. JavaScript: Get map bounds, delta time                      │
│                        │                                        │
│                        ▼                                        │
│  2. WASM: animation.update(dt, bounds)                          │
│     ├── ParticleSimulator.update()                              │
│     │   ├── Sample wind at each particle position               │
│     │   ├── Update velocity & position                          │
│     │   ├── Check bounds, respawn if needed                     │
│     │   └── Update trail history                                │
│     └── TrailVertexBuilder.build()                              │
│         └── Generate triangle vertices for all trails           │
│                        │                                        │
│                        ▼                                        │
│  3. JavaScript: Get vertices from WASM memory                   │
│     const ptr = animation.get_vertices();                       │
│     const vertices = new Float32Array(wasm.memory, ptr, len);   │
│                        │                                        │
│                        ▼                                        │
│  4. WebGL: Upload & render                                      │
│     gl.bufferData(vertices);                                    │
│     gl.drawArrays(gl.TRIANGLES, 0, count);                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Related Docs

- [Rust Crate Specification](./webgl-rust-crate.md)
- [GLSL Shaders](./webgl-shaders.md)
- [POC Analysis](./webgl-poc-analysis.md)
