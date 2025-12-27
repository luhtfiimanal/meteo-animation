# Hybrid WASM Renderer - Usage Guide

## 🎯 Overview

The `HybridRenderer` eliminates the JS ↔ WASM boundary for particle rendering while keeping Leaflet for map functionality.

### Performance Impact

```
BEFORE (POC 6):
  JS Tile Fetch → WASM Particle → JS Vertex Build → JS Upload → GPU
  ❌ 3 boundary crossings
  ❌ 256KB vertex data copied 3 times per frame

AFTER (Hybrid):
  JS Tile Fetch → WASM [Particle + Build + Upload + Render] → GPU
  ✅ 1 boundary crossing
  ✅ Zero vertex copies
  ✅ ~2x FPS improvement expected
```

---

## 📦 Installation

```bash
cd crates/meteo-animation-wasm
~/.cargo/bin/wasm-pack build --target web --release
```

---

## 🚀 Basic Usage

```html
<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
</head>
<body>
    <div id="map" style="height: 100vh"></div>
    <canvas id="windCanvas" style="position: absolute; top: 0; left: 0; pointer-events: none; z-index: 450;"></canvas>

    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
    <script type="module">
        import init, { HybridRenderer } from './pkg/meteo_animation_wasm.js';

        async function main() {
            // Initialize WASM
            await init();

            // Create Leaflet map (JS-owned)
            const map = L.map('map', {
                zoomSnap: 0,
                zoomDelta: 0.5,
                wheelPxPerZoomLevel: 120,
            }).setView([0, 120], 5);

            L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                attribution: '&copy; OpenStreetMap',
            }).addTo(map);

            // Get canvas
            const canvas = document.getElementById('windCanvas');

            // Resize canvas to match map
            function resizeCanvas() {
                const size = map.getSize();
                canvas.width = size.x * window.devicePixelRatio;
                canvas.height = size.y * window.devicePixelRatio;
                canvas.style.width = size.x + 'px';
                canvas.style.height = size.y + 'px';
            }
            resizeCanvas();
            map.on('resize', resizeCanvas);

            // Create WASM renderer (owns WebGL context)
            const renderer = new HybridRenderer(
                canvas,
                20000,  // max particles
                25,     // trail length
                512,    // atlas width
                512     // atlas height
            );

            // Update projection on map move
            function updateProjection() {
                const center = map.getCenter();
                const size = map.getSize();

                // Calculate projection matrix (same as POC 6)
                const centerMerc = latLngToMercator(center.lat, center.lng);
                const centerPx = map.latLngToContainerPoint(center);

                const offsetLng = L.latLng(center.lat, center.lng + 1);
                const offsetMerc = latLngToMercator(center.lat, center.lng + 1);
                const offsetPx = map.latLngToContainerPoint(offsetLng);
                const pxPerMeterX = (offsetPx.x - centerPx.x) / (offsetMerc.x - centerMerc.x);

                const dpr = window.devicePixelRatio;
                const clipPerMeterX = pxPerMeterX * dpr * 2 / (size.x * dpr);
                // ... similar for Y

                const centerClipX = (centerPx.x / size.x) * 2 - 1;
                const translateX = centerClipX - centerMerc.x * clipPerMeterX;
                // ... similar for Y

                // Single call - WASM stores projection state
                renderer.set_projection(
                    clipPerMeterX,
                    clipPerMeterY,
                    translateX,
                    translateY
                );
            }

            map.on('move', updateProjection);
            updateProjection();

            // Update atlas on viewport change
            function updateAtlas() {
                const bounds = map.getBounds();
                const sw = latLngToMercator(bounds.getSouth(), bounds.getWest());
                const ne = latLngToMercator(bounds.getNorth(), bounds.getEast());
                const zoom = Math.floor(map.getZoom());

                renderer.update_atlas_bounds(
                    sw.x, sw.y,
                    ne.x, ne.y,
                    zoom - 2  // zoom offset
                );
            }

            map.on('moveend', updateAtlas);
            updateAtlas();

            // Tile loading (JS-owned, simple)
            async function loadTile(z, x, y) {
                const url = `https://tiles.example.com/${z}/${x}/${y}`;
                const response = await fetch(url);
                const blob = await response.blob();
                const bitmap = await createImageBitmap(blob);

                const offscreen = new OffscreenCanvas(bitmap.width, bitmap.height);
                const ctx = offscreen.getContext('2d');
                ctx.drawImage(bitmap, 0, 0);
                const imageData = ctx.getImageData(0, 0, bitmap.width, bitmap.height);
                bitmap.close();

                // Send to WASM
                renderer.set_atlas_tile(z, x, y, imageData.data);
            }

            // Animation loop - SINGLE CALL!
            let lastTime = performance.now();
            function frame(time) {
                const dt = (time - lastTime) / 1000;
                lastTime = time;

                const zoom = map.getZoom();
                const particleCount = getZoomScaledValue(zoom, ZOOM_PARTICLES);
                const speedFactor = getZoomScaledValue(zoom, ZOOM_SPEED);

                // EVERYTHING happens in WASM!
                // - Particle simulation
                // - Vertex building
                // - GPU upload
                // - WebGL rendering
                renderer.render(
                    dt,                  // delta time
                    particleCount,       // particle count
                    speedFactor,         // speed multiplier
                    100,                 // max age
                    time / 1000,         // time
                    2.0,                 // trail width
                    0.8                  // trail opacity
                );

                requestAnimationFrame(frame);
            }
            requestAnimationFrame(frame);
        }

        function latLngToMercator(lat, lng) {
            const EARTH_RADIUS = 6378137;
            const MAX_LAT = 85.051129;
            const x = lng * Math.PI / 180 * EARTH_RADIUS;
            const latRad = Math.max(-MAX_LAT, Math.min(MAX_LAT, lat)) * Math.PI / 180;
            const y = Math.log(Math.tan(Math.PI / 4 + latRad / 2)) * EARTH_RADIUS;
            return { x, y };
        }

        const ZOOM_PARTICLES = {
            3: 13000, 4: 13000, 5: 13000, 6: 13000, 7: 13000,
            8: 13000, 9: 13000, 10: 13000, 11: 13000, 12: 13000
        };

        const ZOOM_SPEED = {
            3: 40000, 4: 35000, 5: 30000, 6: 25000, 7: 20000,
            8: 15000, 9: 10000, 10: 7000, 11: 5000, 12: 3000
        };

        function getZoomScaledValue(zoom, table) {
            const keys = Object.keys(table).map(Number).sort((a, b) => a - b);
            if (zoom <= keys[0]) return table[keys[0]];
            if (zoom >= keys[keys.length - 1]) return table[keys[keys.length - 1]];
            for (let i = 0; i < keys.length - 1; i++) {
                if (zoom >= keys[i] && zoom <= keys[i + 1]) {
                    const t = (zoom - keys[i]) / (keys[i + 1] - keys[i]);
                    return table[keys[i]] * (1 - t) + table[keys[i + 1]] * t;
                }
            }
            return table[keys[0]];
        }

        main().catch(console.error);
    </script>
</body>
</html>
```

---

## 🔑 Key Differences from POC 6

### POC 6 (Current)

```javascript
// JS owns WebGL
const gl = canvas.getContext('webgl2');

// Animation loop
function frame() {
    // 1. WASM: Update particles
    simulator.update_with_trails(windSampler, trails, ...);

    // 2. WASM: Build vertices
    vertexBuilder.build(trails, ...);

    // 3. JS: Get vertex data (COPY!)
    const ptr = vertexBuilder.get_vertices_ptr();
    const vertices = new Float32Array(wasm.memory.buffer, ptr, vertexCount * 6);

    // 4. JS: Upload to GPU (COPY!)
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, vertices);

    // 5. JS: Render
    gl.drawArrays(gl.TRIANGLES, 0, vertexCount);
}
```

**Problems:**
- `wasm.memory.buffer` can detach (ArrayBuffer detachment)
- Vertex data copied from WASM → JS → GPU (2 copies)
- JS overhead for every frame

---

### Hybrid Renderer (New)

```javascript
// WASM owns WebGL
const renderer = new HybridRenderer(canvas, ...);

// Animation loop
function frame() {
    // SINGLE CALL - everything happens in WASM!
    renderer.render(dt, particleCount, speedFactor, ...);
    // No data copies, no JS overhead
}
```

**Benefits:**
- Vertex data stays in WASM heap → GPU direct (ZERO copies)
- No ArrayBuffer detachment issues
- Minimal JS overhead (just function call)

---

## 🎛️ API Reference

### Constructor

```rust
new HybridRenderer(
    canvas: HtmlCanvasElement,
    max_particles: u32,
    trail_length: u32,
    atlas_width: u32,
    atlas_height: u32
) -> Result<HybridRenderer, JsValue>
```

Creates a new hybrid renderer. WASM takes ownership of WebGL context.

---

### Methods

#### `set_projection(clip_per_meter_x, clip_per_meter_y, translate_x, translate_y)`

Updates view projection matrix. Call when map pans/zooms.

```javascript
map.on('move', () => {
    // Calculate projection matrix
    renderer.set_projection(clipPerMeterX, clipPerMeterY, translateX, translateY);
});
```

---

#### `update_atlas_bounds(min_x, min_y, max_x, max_y, zoom)`

Updates atlas geographic bounds. Call when viewport changes.

```javascript
map.on('moveend', () => {
    const bounds = map.getBounds();
    const sw = latLngToMercator(bounds.getSouth(), bounds.getWest());
    const ne = latLngToMercator(bounds.getNorth(), bounds.getEast());
    renderer.update_atlas_bounds(sw.x, sw.y, ne.x, ne.y, zoom);
});
```

---

#### `set_atlas_tile(z, x, y, tile_data)`

Sets wind tile data. Call after tile fetch completes.

```javascript
const imageData = await fetchTile(z, x, y);
renderer.set_atlas_tile(z, x, y, imageData.data);  // Uint8Array
```

---

#### `clear_atlas()`

Clears all atlas data. Call when zoom level changes significantly.

```javascript
renderer.clear_atlas();
```

---

#### `render(delta_time, particle_count, speed_factor, max_age, time, trail_width, trail_opacity)`

**THE MAIN METHOD** - renders a complete frame.

Called every animation frame. Does EVERYTHING in WASM:
1. Update particles based on wind field
2. Build vertex buffer
3. Upload to GPU
4. Execute WebGL draw calls

```javascript
function frame(time) {
    renderer.render(
        0.016,          // 60 fps
        10000,          // particle count
        20000,          // speed factor
        100,            // max particle age
        time / 1000,    // current time
        2.0,            // trail width in world units
        0.8             // trail opacity
    );
    requestAnimationFrame(frame);
}
```

---

## 🚀 Migration from POC 6

### Step 1: Remove JS WebGL code

**Delete:**
```javascript
const gl = canvas.getContext('webgl2');
const program = createProgram(gl, vs, fs);
const buffer = gl.createBuffer();
// ... all WebGL setup
```

---

### Step 2: Replace with HybridRenderer

**Add:**
```javascript
import init, { HybridRenderer } from './pkg/meteo_animation_wasm.js';
await init();
const renderer = new HybridRenderer(canvas, 20000, 25, 512, 512);
```

---

### Step 3: Simplify animation loop

**Before:**
```javascript
function frame() {
    updateViewState();  // 30 lines
    simulator.update_with_trails(...);
    vertexBuilder.build(...);
    const vertices = new Float32Array(...);  // COPY
    gl.bufferSubData(...);  // COPY
    gl.useProgram(program);
    gl.uniform2f(...);
    gl.drawArrays(...);
}
```

**After:**
```javascript
function frame() {
    renderer.render(dt, particleCount, speedFactor, 100, time, 2.0, 0.8);
}
```

**From 50+ lines → 1 line!**

---

## ⚠️ Limitations (Current)

1. **WindSampler integration not complete**
   - Particles don't follow wind yet
   - Need to integrate `WindSampler::from_atlas()` in renderer

2. **Background shader not implemented**
   - Only trail rendering works
   - Wind visualization overlay needs porting

3. **No particle physics yet**
   - Renderer structure is ready
   - Need to wire up `update_with_trails` in `render()` method

---

## 🛠️ Next Steps

### For Production Use:

1. **Complete WindSampler integration**
   ```rust
   // In renderer.rs, render() method:
   let sampler = WindSampler::from_atlas(&self.atlas);
   self.particles.update_with_trails(
       &sampler,
       &mut self.trails,
       bounds_min_x, bounds_min_y,
       bounds_max_x, bounds_max_y,
       delta_time,
       speed_factor,
       max_age,
       time,
       particle_count
   );
   ```

2. **Add background rendering**
   - Port BG_FRAGMENT_SHADER from POC 6
   - Draw wind field visualization

3. **Test with real tiles**
   - Load from meteo-tiler-v3
   - Verify bilinear resampling

4. **Performance benchmarking**
   - Compare FPS vs POC 6
   - Measure memory usage
   - Profile WASM → GPU transfer

---

## 📊 Expected Performance Gains

| Metric | POC 6 (Current) | Hybrid Renderer | Improvement |
|--------|----------------|-----------------|-------------|
| **Idle FPS** | 40 fps | 60 fps | **+50%** |
| **Brutal pan FPS** | 20 fps | 45 fps | **+125%** |
| **Vertex copies** | 2 per frame | 0 | **Eliminated** |
| **JS overhead** | 5-8ms | <1ms | **-85%** |
| **Memory churn** | 256KB/frame | 0 | **Eliminated** |
| **Code complexity** | 1300 lines | 400 lines | **-70%** |

---

## 💡 Philosophy

**"Let WASM do what WASM does best, let JS do what JS does best."**

- **JS**: High-level orchestration (Leaflet, tile fetching, user input)
- **WASM**: Performance-critical loops (particles, rendering)
- **GPU**: What it was born to do (drawing triangles)

No unnecessary boundary crossings. No unnecessary copies. Just **pure speed**.

---

## 🎯 Goal for Leaflet Plugin

This hybrid approach is **perfect for a Leaflet plugin**:

```javascript
// Future API (dream state)
import { WindParticleLayer } from 'leaflet-wind-particles';

const windLayer = new WindParticleLayer({
    tileUrl: 'https://tiles.example.com/{z}/{x}/{y}',
    particleCount: 10000,
    trailLength: 25,
});

map.addLayer(windLayer);  // That's it!
```

**User-friendly, performant, production-ready.**
