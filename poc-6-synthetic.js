/**
 * Synthetic Tile Generator for POC-6
 *
 * Generates wind tiles that match meteo-tiler-v3 production encoding.
 * Used for testing tile stitching without a server.
 *
 * Encoding (meteo-tiler-v3 compatible):
 *   R = normalize(u, -15, 15)
 *   G = normalize(v, -15, 15)
 *   B = normalize(magnitude, 0, 35)
 *   A = 0 if NaN/land, 255 if valid
 *
 * normalize(value, min, max) = round(((clamp(value, min, max) - min) / (max - min)) * 255)
 */

// =============================================================================
// Constants
// =============================================================================

const EARTH_RADIUS = 6378137.0;
const MERCATOR_EXTENT = 20037508.342789244;
const TILE_SIZE = 256;

// Indonesia region bounds (same as Python script)
const BOUNDS = {
    LON_MIN: 80.0,
    LON_MAX: 160.0,
    LAT_MIN: -20.0,
    LAT_MAX: 20.0,
};

// Encoding config (match meteo-tiler-v3)
const CHANNEL_RANGES = {
    u: { min: -15, max: 15 },
    v: { min: -15, max: 15 },
    mag: { min: 0, max: 35 },
};

// Vortex definitions
const VORTICES = [
    { lon: 110.0, lat: 0.0, strength: 12.0, clockwise: true },    // Cyclone di Laut Jawa
    { lon: 127.0, lat: -5.0, strength: 10.0, clockwise: false },  // Anticyclone di Banda
    { lon: 135.0, lat: 5.0, strength: 8.0, clockwise: true },     // Cyclone di Pasifik
];

const SINGLE_VORTEX = { lon: 127.0, lat: -5.0 };

// =============================================================================
// Simplified Land Polygons (for NaN regions)
// =============================================================================

const LAND_POLYGONS = {
    sumatra: [
        [95.3, 5.6], [97.5, 5.2], [99.0, 4.0], [100.5, 2.0],
        [102.0, 0.5], [104.0, -1.0], [105.5, -3.0], [106.0, -5.9],
        [104.5, -5.8], [103.0, -4.5], [101.0, -2.5], [99.0, -0.5],
        [97.0, 1.5], [95.0, 3.5], [95.3, 5.6]
    ],
    java: [
        [105.2, -5.9], [106.5, -6.0], [108.0, -6.2], [110.0, -6.9],
        [112.0, -7.2], [114.0, -7.8], [114.5, -8.2], [113.0, -8.5],
        [110.0, -8.0], [107.5, -7.5], [105.8, -6.8], [105.2, -5.9]
    ],
    kalimantan: [
        [108.8, 1.0], [110.0, 1.5], [112.0, 1.2], [115.0, 2.5],
        [117.5, 4.2], [118.0, 3.0], [117.5, 1.0], [117.0, -1.0],
        [116.0, -3.0], [114.5, -4.0], [112.0, -3.5], [110.5, -2.5],
        [109.5, -1.0], [108.8, 1.0]
    ],
    sulawesi: [
        [119.5, 1.5], [120.5, 0.5], [121.0, -1.0], [122.5, -2.0],
        [121.5, -3.5], [120.5, -5.5], [119.5, -5.0], [120.0, -3.0],
        [119.0, -1.5], [119.5, 1.5]
    ],
    papua: [
        [130.0, -2.5], [132.0, -2.0], [135.0, -3.5], [138.0, -5.0],
        [141.0, -5.5], [141.0, -8.5], [138.0, -8.0], [135.0, -6.0],
        [132.0, -4.5], [130.0, -2.5]
    ],
};

// =============================================================================
// Coordinate Conversions
// =============================================================================

function mercatorToLonLat(x, y) {
    const lon = (x / EARTH_RADIUS) * (180 / Math.PI);
    const lat = (2 * Math.atan(Math.exp(y / EARTH_RADIUS)) - Math.PI / 2) * (180 / Math.PI);
    return { lon, lat };
}

function tileToMercatorBounds(z, x, y) {
    const n = Math.pow(2, z);
    const worldSize = MERCATOR_EXTENT * 2;
    const tileSize = worldSize / n;

    const minX = -MERCATOR_EXTENT + x * tileSize;
    const maxX = minX + tileSize;

    // Y is flipped: y=0 is at top (north)
    const maxY = MERCATOR_EXTENT - y * tileSize;
    const minY = maxY - tileSize;

    return { minX, minY, maxX, maxY };
}

// =============================================================================
// Land Detection (Point-in-Polygon)
// =============================================================================

function pointInPolygon(lon, lat, polygon) {
    const n = polygon.length;
    let inside = false;
    let j = n - 1;

    for (let i = 0; i < n; i++) {
        const [xi, yi] = polygon[i];
        const [xj, yj] = polygon[j];

        if (((yi > lat) !== (yj > lat)) &&
            (lon < (xj - xi) * (lat - yi) / (yj - yi) + xi)) {
            inside = !inside;
        }
        j = i;
    }

    return inside;
}

function isLand(lon, lat) {
    for (const polygon of Object.values(LAND_POLYGONS)) {
        if (pointInPolygon(lon, lat, polygon)) {
            return true;
        }
    }
    return false;
}

function isOutOfBounds(lon, lat) {
    return lon < BOUNDS.LON_MIN || lon > BOUNDS.LON_MAX ||
           lat < BOUNDS.LAT_MIN || lat > BOUNDS.LAT_MAX;
}

// =============================================================================
// Wind Patterns
// =============================================================================

function windUniformGradient(lon, lat) {
    const u = 5.0;  // Base eastward wind (m/s)
    const v = 2.0 * Math.sin(lat * Math.PI / 40);  // +/-2 m/s variation
    return { u, v };
}

function windSingleVortex(lon, lat) {
    const dx = lon - SINGLE_VORTEX.lon;
    const dy = lat - SINGLE_VORTEX.lat;
    const dist = Math.sqrt(dx * dx + dy * dy) + 0.01;

    // Tangential velocity (clockwise for Southern Hemisphere cyclone)
    const angle = Math.atan2(dy, dx) + Math.PI / 2;
    const radius = 5.0;  // degrees
    const strength = 10.0;  // m/s
    const magnitude = strength * Math.min(1.0, radius / dist) * Math.exp(-dist / radius);

    const u = Math.cos(angle) * magnitude;
    const v = Math.sin(angle) * magnitude;
    return { u, v };
}

function windMultiVortex(lon, lat) {
    let u = 3.0;  // Base easterly trade wind
    let v = 0.0;

    for (const vortex of VORTICES) {
        const dx = lon - vortex.lon;
        const dy = lat - vortex.lat;
        const dist = Math.sqrt(dx * dx + dy * dy) + 0.01;

        let angle = Math.atan2(dy, dx);
        if (vortex.clockwise) {
            angle += Math.PI / 2;
        } else {
            angle -= Math.PI / 2;
        }

        const falloff = Math.min(1.0, 5.0 / dist);
        u += Math.cos(angle) * vortex.strength * falloff;
        v += Math.sin(angle) * vortex.strength * falloff;
    }

    return { u, v };
}

const WIND_PATTERNS = {
    uniform: windUniformGradient,
    single: windSingleVortex,
    multi: windMultiVortex,
};

// =============================================================================
// Encoding (meteo-tiler-v3 compatible)
// =============================================================================

function normalizeToU8(value, min, max) {
    if (!Number.isFinite(value)) return 0;
    const clamped = Math.max(min, Math.min(max, value));
    const normalized = (clamped - min) / (max - min);
    return Math.round(normalized * 255);
}

function encodeWind(u, v, isValid) {
    if (!isValid) {
        return [0, 0, 0, 0];  // NaN region
    }

    const speed = Math.sqrt(u * u + v * v);
    return [
        normalizeToU8(u, CHANNEL_RANGES.u.min, CHANNEL_RANGES.u.max),       // R
        normalizeToU8(v, CHANNEL_RANGES.v.min, CHANNEL_RANGES.v.max),       // G
        normalizeToU8(speed, CHANNEL_RANGES.mag.min, CHANNEL_RANGES.mag.max), // B
        255  // A = valid
    ];
}

// =============================================================================
// Synthetic Tile Generator
// =============================================================================

export class SyntheticTileGenerator {
    constructor(pattern = 'multi') {
        this.pattern = pattern;
        this.patternFunc = WIND_PATTERNS[pattern] || WIND_PATTERNS.multi;
        // Reusable buffer to avoid allocations
        this._buffer = new Uint8Array(TILE_SIZE * TILE_SIZE * 4);
    }

    setPattern(pattern) {
        this.pattern = pattern;
        this.patternFunc = WIND_PATTERNS[pattern] || WIND_PATTERNS.multi;
    }

    generateTile(z, x, y) {
        const data = this._buffer;  // Reuse buffer
        const bounds = tileToMercatorBounds(z, x, y);

        for (let py = 0; py < TILE_SIZE; py++) {
            for (let px = 0; px < TILE_SIZE; px++) {
                // Pixel to Mercator (center of pixel)
                const u = (px + 0.5) / TILE_SIZE;
                const v = (py + 0.5) / TILE_SIZE;

                const mercX = bounds.minX + u * (bounds.maxX - bounds.minX);
                const mercY = bounds.maxY - v * (bounds.maxY - bounds.minY);  // Y flipped

                // Mercator to Lon/Lat
                const { lon, lat } = mercatorToLonLat(mercX, mercY);

                // Check if outside Indonesia bounds or on land (NaN region)
                const outOfBounds = isOutOfBounds(lon, lat);
                const onLand = !outOfBounds && isLand(lon, lat);
                const isValid = !outOfBounds && !onLand;

                // Sample wind at this location
                const wind = this.patternFunc(lon, lat);

                // Encode to RGBA
                const rgba = encodeWind(wind.u, wind.v, isValid);

                const idx = (py * TILE_SIZE + px) * 4;
                data[idx] = rgba[0];
                data[idx + 1] = rgba[1];
                data[idx + 2] = rgba[2];
                data[idx + 3] = rgba[3];
            }
        }

        return data;
    }

    /**
     * Generate tile and return as ImageData (for canvas)
     */
    generateTileImageData(z, x, y) {
        const data = this.generateTile(z, x, y);
        return new ImageData(new Uint8ClampedArray(data.buffer), TILE_SIZE, TILE_SIZE);
    }
}

// =============================================================================
// Exports
// =============================================================================

export { WIND_PATTERNS, CHANNEL_RANGES, LAND_POLYGONS, BOUNDS };
export { tileToMercatorBounds, mercatorToLonLat, isLand, isOutOfBounds };
export { normalizeToU8, encodeWind };
