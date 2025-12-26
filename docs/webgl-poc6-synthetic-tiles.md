# POC 6: Synthetic Tile Generator

Dokumentasi detail untuk synthetic tile generation di POC-6.

---

## Overview

POC-6 fokus pada **tile stitching** - menggabungkan multiple tiles menjadi satu atlas texture.
Synthetic tiles harus **100% match** dengan output meteo-tiler-v3 production:
- Ter-georeferensi ke tile bounds (z, x, y)
- Encoding sama dengan production
- NaN region (land mask) untuk test validity handling
- **JavaScript implementation** (tidak membebani WASM production bundle)

---

## meteo-tiler-v3 Encoding Reference

Source: `meteo-tiler-v3/src/proc/tile/postprocessor/vector_tiles.rs`

```
┌─────────────────────────────────────────────────────────────┐
│  RGBA Encoding (meteo-tiler-v3 production)                  │
├─────────────────────────────────────────────────────────────┤
│  R = normalize(u, min, max)                                 │
│  G = normalize(v, min, max)                                 │
│  B = normalize(magnitude, 0, max_speed)                     │
│  A = 0 if NaN/invalid, 255 if valid                         │
├─────────────────────────────────────────────────────────────┤
│  normalize(value, min, max) =                               │
│      round(((clamp(value, min, max) - min) / (max - min)) * 255)
├─────────────────────────────────────────────────────────────┤
│  Config: channelRanges.r = {min: -15, max: 15}              │
│          channelRanges.g = {min: -15, max: 15}              │
│          channelRanges.b = {min: 0, max: 35}                │
└─────────────────────────────────────────────────────────────┘
```

**Decode formula:**
```javascript
u = (R / 255) * (max - min) + min  // e.g., (R / 255) * 30 - 15
v = (G / 255) * (max - min) + min
speed = (B / 255) * 35
valid = A > 127
```

---

## Wind Patterns

### Pattern A: Uniform + Gradient

Simplest pattern untuk verify tile stitching continuity.

```javascript
function windUniformGradient(lon, lat) {
    const u = 5.0;  // Base eastward wind (m/s)
    const v = 2.0 * Math.sin(lat * Math.PI / 40);  // +/-2 m/s variation
    return { u, v };
}
```

### Pattern B: Single Vortex

Single cyclone at Banda Sea (127E, 5S).

```javascript
function windSingleVortex(lon, lat) {
    const dx = lon - 127.0;
    const dy = lat - (-5.0);
    const dist = Math.sqrt(dx * dx + dy * dy) + 0.01;
    const angle = Math.atan2(dy, dx) + Math.PI / 2;
    const magnitude = 10.0 * Math.min(1.0, 5.0 / dist) * Math.exp(-dist / 5.0);
    return { u: Math.cos(angle) * magnitude, v: Math.sin(angle) * magnitude };
}
```

### Pattern C: Multi Vortex

Multiple vortices simulating realistic weather patterns.

```javascript
const VORTICES = [
    { lon: 110.0, lat: 0.0, strength: 12.0, clockwise: true },    // Cyclone di Laut Jawa
    { lon: 127.0, lat: -5.0, strength: 10.0, clockwise: false },  // Anticyclone di Banda
    { lon: 135.0, lat: 5.0, strength: 8.0, clockwise: true },     // Cyclone di Pasifik
];

function windMultiVortex(lon, lat) {
    let u = 3.0, v = 0.0;  // Base trade wind
    for (const vortex of VORTICES) {
        // ... add vortex contribution
    }
    return { u, v };
}
```

---

## NaN Region (Land Mask)

Simplified polygon land mask untuk major Indonesian islands:

```javascript
const LAND_POLYGONS = {
    sumatra: [[95.3, 5.6], [97.5, 5.2], ...],
    java: [[105.2, -5.9], [106.5, -6.0], ...],
    kalimantan: [[108.8, 1.0], [110.0, 1.5], ...],
    sulawesi: [[119.5, 1.5], [120.5, 0.5], ...],
    papua: [[130.0, -2.5], [132.0, -2.0], ...],
};

function isLand(lon, lat) {
    // Point-in-polygon test for each island
    return pointInPolygon(lon, lat, ...);
}
```

**Land pixels:** `R=0, G=0, B=0, A=0` (fully transparent/invalid)

---

## JavaScript Implementation

**File:** `demo/poc-6-synthetic.js`

```javascript
import { SyntheticTileGenerator } from './poc-6-synthetic.js';

// Create generator
const synth = new SyntheticTileGenerator('multi');

// Generate tiles for atlas
function fillAtlasWithSynthetic(tileCoords) {
    for (let i = 0; i < tileCoords.length; i += 3) {
        const z = tileCoords[i];
        const x = tileCoords[i + 1];
        const y = tileCoords[i + 2];

        const data = synth.generateTile(z, x, y);
        atlas.set_tile(z, x, y, data);
    }
}

// Change pattern
synth.setPattern('single');  // or 'uniform', 'multi'
```

---

## Verification

### Python Visualization

```bash
cd pytools
uv run python synthetic_wind.py multi 4
```

Output:
1. Streamline plot with Cartopy (coastlines)
2. NaN regions as gray hatched areas
3. Tile grid overlay at zoom 4

### Visual Checks

1. **Stitching Continuity**: No visible seams at tile boundaries
2. **Vortex Position**: Vortex stays at same lon/lat when zooming
3. **NaN Region**: Land areas shown as hatched, no wind flow
4. **Speed Gradient**: Color varies smoothly with wind speed

### POC-6 Demo

```bash
cd /home/sareep/project/wmts-nc/meteo-animation
bun run demo
# Open http://localhost:5180/demo/poc-6-webgl.html
```

UI Controls:
- **Pattern dropdown**: Switch between uniform/single/multi
- **Synthetic button**: Load synthetic tiles
- **Load Tiles button**: Load real tiles from server

---

## Files

| File | Purpose |
|------|---------|
| `demo/poc-6-synthetic.js` | JavaScript synthetic tile generator |
| `demo/poc-6-webgl.html` | POC-6 demo with pattern selector |
| `pytools/synthetic_wind.py` | Python visualization for verification |
| `docs/webgl-poc6-synthetic-tiles.md` | This documentation |

---

## Key Design Decisions

| Aspect | Decision | Reason |
|--------|----------|--------|
| Language | JavaScript | Zero WASM production bundle impact |
| Encoding | meteo-tiler-v3 formula | 100% match with production |
| Land mask | Simplified polygons | Good enough for testing, fast |
| Pattern selector | Dropdown UI | Easy visual verification |
