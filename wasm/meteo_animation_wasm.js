let wasm;

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getUint32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

const AtlasBoundsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_atlasbounds_free(ptr >>> 0, 1));

const DynamicAtlasFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dynamicatlas_free(ptr >>> 0, 1));

const FixedAtlasFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fixedatlas_free(ptr >>> 0, 1));

const ParticleSimulatorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_particlesimulator_free(ptr >>> 0, 1));

const TileCoordFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_tilecoord_free(ptr >>> 0, 1));

const TileCoordinatorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_tilecoordinator_free(ptr >>> 0, 1));

const TrailManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_trailmanager_free(ptr >>> 0, 1));

const TrailVertexBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_trailvertexbuilder_free(ptr >>> 0, 1));

const WindSamplerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_windsampler_free(ptr >>> 0, 1));

/**
 * Atlas bounds in Web Mercator coordinates
 */
export class AtlasBounds {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AtlasBounds.prototype);
        obj.__wbg_ptr = ptr;
        AtlasBoundsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AtlasBoundsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_atlasbounds_free(ptr, 0);
    }
    /**
     * @param {number} min_x
     * @param {number} min_y
     * @param {number} max_x
     * @param {number} max_y
     */
    constructor(min_x, min_y, max_x, max_y) {
        const ret = wasm.atlasbounds_new(min_x, min_y, max_x, max_y);
        this.__wbg_ptr = ret >>> 0;
        AtlasBoundsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    width() {
        const ret = wasm.atlasbounds_width(this.__wbg_ptr);
        return ret;
    }
    /**
     * Expand bounds by ratio (1.2 = 20% buffer on each side)
     * @param {number} ratio
     * @returns {AtlasBounds}
     */
    expand(ratio) {
        const ret = wasm.atlasbounds_expand(this.__wbg_ptr, ratio);
        return AtlasBounds.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    height() {
        const ret = wasm.atlasbounds_height(this.__wbg_ptr);
        return ret;
    }
    /**
     * Check if point is inside bounds
     * @param {number} x
     * @param {number} y
     * @returns {boolean}
     */
    contains(x, y) {
        const ret = wasm.atlasbounds_contains(this.__wbg_ptr, x, y);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    get min_x() {
        const ret = wasm.__wbg_get_atlasbounds_min_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set min_x(arg0) {
        wasm.__wbg_set_atlasbounds_min_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get min_y() {
        const ret = wasm.__wbg_get_atlasbounds_min_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set min_y(arg0) {
        wasm.__wbg_set_atlasbounds_min_y(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get max_x() {
        const ret = wasm.__wbg_get_atlasbounds_max_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set max_x(arg0) {
        wasm.__wbg_set_atlasbounds_max_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get max_y() {
        const ret = wasm.__wbg_get_atlasbounds_max_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set max_y(arg0) {
        wasm.__wbg_set_atlasbounds_max_y(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) AtlasBounds.prototype[Symbol.dispose] = AtlasBounds.prototype.free;

/**
 * Dynamic atlas that resizes based on viewport
 */
export class DynamicAtlas {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DynamicAtlasFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dynamicatlas_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get_height() {
        const ret = wasm.dynamicatlas_get_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get atlas data length
     * @returns {number}
     */
    get_data_len() {
        const ret = wasm.dynamicatlas_get_data_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get atlas data pointer for GPU upload
     * @returns {number}
     */
    get_data_ptr() {
        const ret = wasm.dynamicatlas_get_data_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Update atlas to cover new viewport
     * Returns array of tile coords that need to be loaded [z, x, y, z, x, y, ...]
     * @param {number} viewport_min_x
     * @param {number} viewport_min_y
     * @param {number} viewport_max_x
     * @param {number} viewport_max_y
     * @param {number} data_zoom
     * @returns {Uint32Array}
     */
    update_viewport(viewport_min_x, viewport_min_y, viewport_max_x, viewport_max_y, data_zoom) {
        const ret = wasm.dynamicatlas_update_viewport(this.__wbg_ptr, viewport_min_x, viewport_min_y, viewport_max_x, viewport_max_y, data_zoom);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    get_bounds_max_x() {
        const ret = wasm.dynamicatlas_get_bounds_max_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_bounds_max_y() {
        const ret = wasm.dynamicatlas_get_bounds_max_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_bounds_min_x() {
        const ret = wasm.dynamicatlas_get_bounds_min_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_bounds_min_y() {
        const ret = wasm.dynamicatlas_get_bounds_min_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} buffer_ratio
     */
    constructor(buffer_ratio) {
        const ret = wasm.dynamicatlas_new(buffer_ratio);
        this.__wbg_ptr = ret >>> 0;
        DynamicAtlasFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Sample wind at world coordinate (returns [u, v, valid] as f32)
     * @param {number} world_x
     * @param {number} world_y
     * @returns {Float32Array}
     */
    sample(world_x, world_y) {
        const ret = wasm.dynamicatlas_sample(this.__wbg_ptr, world_x, world_y);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Set tile data at specific tile coordinate
     * @param {number} z
     * @param {number} x
     * @param {number} y
     * @param {Uint8Array} data
     */
    set_tile(z, x, y, data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.dynamicatlas_set_tile(this.__wbg_ptr, z, x, y, ptr0, len0);
    }
    /**
     * @returns {number}
     */
    get_width() {
        const ret = wasm.dynamicatlas_get_width(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) DynamicAtlas.prototype[Symbol.dispose] = DynamicAtlas.prototype.free;

/**
 * Fixed-size atlas that never resizes during pan/zoom.
 * Only the bounds (georeferencing) change.
 * This prevents particle "blink" during viewport changes.
 */
export class FixedAtlas {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FixedAtlasFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fixedatlas_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get_height() {
        const ret = wasm.fixedatlas_get_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    clear_dirty() {
        wasm.fixedatlas_clear_dirty(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    get_data_len() {
        const ret = wasm.fixedatlas_get_data_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_data_ptr() {
        const ret = wasm.fixedatlas_get_data_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get a copy of the atlas data as a Vec<u8>.
     * This is safer than get_data_ptr() as it avoids detached ArrayBuffer issues.
     * @returns {Uint8Array}
     */
    get_data_copy() {
        const ret = wasm.fixedatlas_get_data_copy(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    get_tile_zoom() {
        const ret = wasm.fixedatlas_get_tile_zoom(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Update bounds without resizing.
     * Called on every pan/zoom. Does NOT clear data.
     * @param {number} min_x
     * @param {number} min_y
     * @param {number} max_x
     * @param {number} max_y
     * @param {number} zoom
     */
    update_bounds(min_x, min_y, max_x, max_y, zoom) {
        wasm.fixedatlas_update_bounds(this.__wbg_ptr, min_x, min_y, max_x, max_y, zoom);
    }
    /**
     * @returns {number}
     */
    get_bounds_max_x() {
        const ret = wasm.fixedatlas_get_bounds_max_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_bounds_max_y() {
        const ret = wasm.fixedatlas_get_bounds_max_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_bounds_min_x() {
        const ret = wasm.fixedatlas_get_bounds_min_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_bounds_min_y() {
        const ret = wasm.fixedatlas_get_bounds_min_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * Place tile data with bilinear resampling.
     * The tile is resampled to fit the current atlas bounds.
     * @param {number} z
     * @param {number} x
     * @param {number} y
     * @param {Uint8Array} tile_data
     */
    set_tile_resampled(z, x, y, tile_data) {
        const ptr0 = passArray8ToWasm0(tile_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.fixedatlas_set_tile_resampled(this.__wbg_ptr, z, x, y, ptr0, len0);
    }
    /**
     * Create a new fixed-size atlas.
     * Size should be based on device screen resolution / 2^|zoom_offset|.
     * @param {number} width
     * @param {number} height
     */
    constructor(width, height) {
        const ret = wasm.fixedatlas_new(width, height);
        this.__wbg_ptr = ret >>> 0;
        FixedAtlasFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Clear atlas data (fill with zeros).
     * Use when data zoom changes and all tiles need to be reloaded.
     */
    clear() {
        wasm.fixedatlas_clear(this.__wbg_ptr);
    }
    /**
     * Sample wind at world coordinate (returns [u, v, valid] as f32)
     * @param {number} world_x
     * @param {number} world_y
     * @returns {Float32Array}
     */
    sample(world_x, world_y) {
        const ret = wasm.fixedatlas_sample(this.__wbg_ptr, world_x, world_y);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {boolean}
     */
    is_dirty() {
        const ret = wasm.fixedatlas_is_dirty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    get_width() {
        const ret = wasm.fixedatlas_get_width(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) FixedAtlas.prototype[Symbol.dispose] = FixedAtlas.prototype.free;

/**
 * Particle simulator for WebGL rendering
 */
export class ParticleSimulator {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ParticleSimulatorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_particlesimulator_free(ptr, 0);
    }
    /**
     * POC 6: Set position of particle at index
     * @param {number} idx
     * @param {number} x
     * @param {number} y
     */
    set_position(idx, x, y) {
        wasm.particlesimulator_set_position(this.__wbg_ptr, idx, x, y);
    }
    /**
     * POC 6: Get state of particle at index (WASM-exported version)
     * @param {number} idx
     * @returns {number}
     */
    get_state(idx) {
        const ret = wasm.particlesimulator_get_state(this.__wbg_ptr, idx);
        return ret;
    }
    /**
     * POC 2: Update with wind texture sampling + NaN handling
     * @param {WindSampler} wind
     * @param {number} bounds_min_x
     * @param {number} bounds_min_y
     * @param {number} bounds_max_x
     * @param {number} bounds_max_y
     * @param {number} delta_time
     * @param {number} speed_factor
     * @param {number} max_age
     * @param {number} random_seed
     * @param {number} target_count
     */
    update_with_wind(wind, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, delta_time, speed_factor, max_age, random_seed, target_count) {
        _assertClass(wind, WindSampler);
        wasm.particlesimulator_update_with_wind(this.__wbg_ptr, wind.__wbg_ptr, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, delta_time, speed_factor, max_age, random_seed, target_count);
    }
    /**
     * Get max particles count
     * @returns {number}
     */
    get_max_particles() {
        const ret = wasm.particlesimulator_get_max_particles(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get length of particle data array
     * @returns {number}
     */
    get_particles_len() {
        const ret = wasm.particlesimulator_get_particles_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get pointer to particle data for WebGL upload
     * @returns {number}
     */
    get_particles_ptr() {
        const ret = wasm.particlesimulator_get_particles_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * POC 3: Update particles with wind sampling AND sync trails
     * @param {WindSampler} wind
     * @param {TrailManager} trails
     * @param {number} bounds_min_x
     * @param {number} bounds_min_y
     * @param {number} bounds_max_x
     * @param {number} bounds_max_y
     * @param {number} delta_time
     * @param {number} speed_factor
     * @param {number} max_age
     * @param {number} random_seed
     * @param {number} target_count
     */
    update_with_trails(wind, trails, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, delta_time, speed_factor, max_age, random_seed, target_count) {
        _assertClass(wind, WindSampler);
        _assertClass(trails, TrailManager);
        wasm.particlesimulator_update_with_trails(this.__wbg_ptr, wind.__wbg_ptr, trails.__wbg_ptr, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, delta_time, speed_factor, max_age, random_seed, target_count);
    }
    /**
     * Create new simulator with all particles inactive
     * @param {number} max_particles
     */
    constructor(max_particles) {
        const ret = wasm.particlesimulator_new(max_particles);
        this.__wbg_ptr = ret >>> 0;
        ParticleSimulatorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * POC 6: Get X position of particle at index
     * @param {number} idx
     * @returns {number}
     */
    get_x(idx) {
        const ret = wasm.particlesimulator_get_x(this.__wbg_ptr, idx);
        return ret;
    }
    /**
     * POC 6: Get Y position of particle at index
     * @param {number} idx
     * @returns {number}
     */
    get_y(idx) {
        const ret = wasm.particlesimulator_get_y(this.__wbg_ptr, idx);
        return ret;
    }
    /**
     * Update all particles
     * @param {number} bounds_min_x
     * @param {number} bounds_min_y
     * @param {number} bounds_max_x
     * @param {number} bounds_max_y
     * @param {number} wind_x
     * @param {number} wind_y
     * @param {number} delta_time
     * @param {number} max_age
     * @param {number} random_seed
     * @param {number} target_count
     */
    update(bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, wind_x, wind_y, delta_time, max_age, random_seed, target_count) {
        wasm.particlesimulator_update(this.__wbg_ptr, bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y, wind_x, wind_y, delta_time, max_age, random_seed, target_count);
    }
    /**
     * POC 6: Set state of particle at index
     * @param {number} idx
     * @param {number} state
     */
    set_state(idx, state) {
        wasm.particlesimulator_set_state(this.__wbg_ptr, idx, state);
    }
}
if (Symbol.dispose) ParticleSimulator.prototype[Symbol.dispose] = ParticleSimulator.prototype.free;

/**
 * Tile coordinate (z, x, y)
 */
export class TileCoord {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TileCoordFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_tilecoord_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.__wbg_get_tilecoord_z(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set z(arg0) {
        wasm.__wbg_set_tilecoord_z(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.__wbg_get_tilecoord_x(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set x(arg0) {
        wasm.__wbg_set_tilecoord_x(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.__wbg_get_tilecoord_y(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set y(arg0) {
        wasm.__wbg_set_tilecoord_y(this.__wbg_ptr, arg0);
    }
    /**
     * Get geographic bounds of this tile (lon/lat)
     * @returns {Float64Array}
     */
    get_geo_bounds() {
        const ret = wasm.tilecoord_get_geo_bounds(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * Get Web Mercator bounds of this tile
     * @returns {Float64Array}
     */
    get_mercator_bounds() {
        const ret = wasm.tilecoord_get_mercator_bounds(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @param {number} z
     * @param {number} x
     * @param {number} y
     */
    constructor(z, x, y) {
        const ret = wasm.tilecoord_new(z, x, y);
        this.__wbg_ptr = ret >>> 0;
        TileCoordFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) TileCoord.prototype[Symbol.dispose] = TileCoord.prototype.free;

/**
 * Tile coordinator for managing tile requests with zoom offset
 */
export class TileCoordinator {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TileCoordinatorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_tilecoordinator_free(ptr, 0);
    }
    /**
     * Clear a specific tile (for LRU eviction sync)
     * @param {number} z
     * @param {number} x
     * @param {number} y
     */
    clear_tile(z, x, y) {
        wasm.tilecoordinator_clear_tile(this.__wbg_ptr, z, x, y);
    }
    /**
     * Clear tiles at specific zoom level
     * @param {number} z
     */
    clear_zoom(z) {
        wasm.tilecoordinator_clear_zoom(this.__wbg_ptr, z);
    }
    /**
     * Mark tile as loaded
     * @param {number} z
     * @param {number} x
     * @param {number} y
     */
    mark_loaded(z, x, y) {
        wasm.tilecoordinator_mark_loaded(this.__wbg_ptr, z, x, y);
    }
    /**
     * Get count of loaded tiles
     * @returns {number}
     */
    loaded_count() {
        const ret = wasm.tilecoordinator_loaded_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Calculate effective data zoom from view zoom
     * @param {number} view_zoom
     * @returns {number}
     */
    get_data_zoom(view_zoom) {
        const ret = wasm.tilecoordinator_get_data_zoom(this.__wbg_ptr, view_zoom);
        return ret >>> 0;
    }
    /**
     * Set minimum and maximum data zoom levels
     * @param {number} min_zoom
     * @param {number} max_zoom
     */
    set_zoom_range(min_zoom, max_zoom) {
        wasm.tilecoordinator_set_zoom_range(this.__wbg_ptr, min_zoom, max_zoom);
    }
    /**
     * Filter tile list to only unloaded tiles
     * Input: flat array [z, x, y, z, x, y, ...]
     * Output: flat array of unloaded tiles
     * @param {Uint32Array} tiles
     * @returns {Uint32Array}
     */
    filter_unloaded(tiles) {
        const ptr0 = passArray32ToWasm0(tiles, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tilecoordinator_filter_unloaded(this.__wbg_ptr, ptr0, len0);
        var v2 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v2;
    }
    /**
     * @param {number} zoom_offset
     */
    constructor(zoom_offset) {
        const ret = wasm.tilecoordinator_new(zoom_offset);
        this.__wbg_ptr = ret >>> 0;
        TileCoordinatorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Clear loaded tiles (e.g., when zoom changes significantly)
     */
    clear() {
        wasm.tilecoordinator_clear(this.__wbg_ptr);
    }
    /**
     * Check if tile is already loaded
     * @param {number} z
     * @param {number} x
     * @param {number} y
     * @returns {boolean}
     */
    is_loaded(z, x, y) {
        const ret = wasm.tilecoordinator_is_loaded(this.__wbg_ptr, z, x, y);
        return ret !== 0;
    }
}
if (Symbol.dispose) TileCoordinator.prototype[Symbol.dispose] = TileCoordinator.prototype.free;

/**
 * Trail manager using shift approach
 * Index 0 = HEAD (newest), Index N-1 = TAIL (oldest)
 */
export class TrailManager {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TrailManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_trailmanager_free(ptr, 0);
    }
    /**
     * Reset all trail positions to the same point (for spawn/respawn)
     * @param {number} particle_id
     * @param {number} x
     * @param {number} y
     */
    reset_trail(particle_id, x, y) {
        wasm.trailmanager_reset_trail(this.__wbg_ptr, particle_id, x, y);
    }
    /**
     * @returns {number}
     */
    trail_length() {
        const ret = wasm.trailmanager_trail_length(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    max_particles() {
        const ret = wasm.trailmanager_max_particles(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Push new position: shift all positions toward tail, store new at head
     * @param {number} particle_id
     * @param {number} x
     * @param {number} y
     */
    push_position(particle_id, x, y) {
        wasm.trailmanager_push_position(this.__wbg_ptr, particle_id, x, y);
    }
    /**
     * Get total size of trails data
     * @returns {number}
     */
    get_trails_len() {
        const ret = wasm.trailmanager_get_trails_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get raw pointer for WebGL upload
     * @returns {number}
     */
    get_trails_ptr() {
        const ret = wasm.trailmanager_get_trails_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} max_particles
     * @param {number} trail_length
     */
    constructor(max_particles, trail_length) {
        const ret = wasm.trailmanager_new(max_particles, trail_length);
        this.__wbg_ptr = ret >>> 0;
        TrailManagerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) TrailManager.prototype[Symbol.dispose] = TrailManager.prototype.free;

/**
 * Trail vertex builder - generates quads from trail positions
 */
export class TrailVertexBuilder {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TrailVertexBuilderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_trailvertexbuilder_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get_vertex_count() {
        const ret = wasm.trailvertexbuilder_get_vertex_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_vertices_ptr() {
        const ret = wasm.trailvertexbuilder_get_vertices_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_floats_per_vertex() {
        const ret = wasm.trailvertexbuilder_get_floats_per_vertex(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} max_particles
     * @param {number} trail_length
     */
    constructor(max_particles, trail_length) {
        const ret = wasm.trailvertexbuilder_new(max_particles, trail_length);
        this.__wbg_ptr = ret >>> 0;
        TrailVertexBuilderFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Build vertex data from trails
     * color: base RGB color (0-1 range)
     * @param {TrailManager} trails
     * @param {number} particle_count
     * @param {number} trail_width
     * @param {number} opacity
     * @param {number} color_r
     * @param {number} color_g
     * @param {number} color_b
     */
    build(trails, particle_count, trail_width, opacity, color_r, color_g, color_b) {
        _assertClass(trails, TrailManager);
        wasm.trailvertexbuilder_build(this.__wbg_ptr, trails.__wbg_ptr, particle_count, trail_width, opacity, color_r, color_g, color_b);
    }
}
if (Symbol.dispose) TrailVertexBuilder.prototype[Symbol.dispose] = TrailVertexBuilder.prototype.free;

/**
 * Wind texture sampler with bilinear interpolation
 */
export class WindSampler {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WindSampler.prototype);
        obj.__wbg_ptr = ptr;
        WindSamplerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WindSamplerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_windsampler_free(ptr, 0);
    }
    /**
     * Create WindSampler from DynamicAtlas
     * Copies atlas data and bounds for particle simulation
     * @param {DynamicAtlas} atlas
     * @returns {WindSampler}
     */
    static from_atlas(atlas) {
        _assertClass(atlas, DynamicAtlas);
        const ret = wasm.windsampler_from_atlas(atlas.__wbg_ptr);
        return WindSampler.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    get_height() {
        const ret = wasm.windsampler_get_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} min_x
     * @param {number} min_y
     * @param {number} max_x
     * @param {number} max_y
     */
    set_bounds(min_x, min_y, max_x, max_y) {
        wasm.windsampler_set_bounds(this.__wbg_ptr, min_x, min_y, max_x, max_y);
    }
    /**
     * @returns {number}
     */
    get_data_len() {
        const ret = wasm.windsampler_get_data_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_data_ptr() {
        const ret = wasm.windsampler_get_data_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Update WindSampler from DynamicAtlas
     * Call this when atlas data or bounds change
     * @param {DynamicAtlas} atlas
     */
    update_from_atlas(atlas) {
        _assertClass(atlas, DynamicAtlas);
        wasm.windsampler_update_from_atlas(this.__wbg_ptr, atlas.__wbg_ptr);
    }
    /**
     * @param {number} width
     * @param {number} height
     */
    constructor(width, height) {
        const ret = wasm.windsampler_new(width, height);
        this.__wbg_ptr = ret >>> 0;
        WindSamplerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    width() {
        const ret = wasm.windsampler_get_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    height() {
        const ret = wasm.windsampler_get_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Check if position has valid wind data (alpha > 127)
     * @param {number} x
     * @param {number} y
     * @returns {boolean}
     */
    is_valid(x, y) {
        const ret = wasm.windsampler_is_valid(this.__wbg_ptr, x, y);
        return ret !== 0;
    }
    /**
     * @param {Uint8Array} data
     */
    set_data(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.windsampler_set_data(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {number}
     */
    get_max_x() {
        const ret = wasm.windsampler_get_max_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_max_y() {
        const ret = wasm.windsampler_get_max_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_min_x() {
        const ret = wasm.windsampler_get_min_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_min_y() {
        const ret = wasm.windsampler_get_min_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_width() {
        const ret = wasm.windsampler_get_width(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) WindSampler.prototype[Symbol.dispose] = WindSampler.prototype.free;

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedFloat32ArrayMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('meteo_animation_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
