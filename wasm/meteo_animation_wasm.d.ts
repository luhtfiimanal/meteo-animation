/* tslint:disable */
/* eslint-disable */

export class AtlasBounds {
  free(): void;
  [Symbol.dispose](): void;
  constructor(min_x: number, min_y: number, max_x: number, max_y: number);
  width(): number;
  /**
   * Expand bounds by ratio (1.2 = 20% buffer on each side)
   */
  expand(ratio: number): AtlasBounds;
  height(): number;
  /**
   * Check if point is inside bounds
   */
  contains(x: number, y: number): boolean;
  min_x: number;
  min_y: number;
  max_x: number;
  max_y: number;
}

export class DynamicAtlas {
  free(): void;
  [Symbol.dispose](): void;
  get_height(): number;
  /**
   * Get atlas data length
   */
  get_data_len(): number;
  /**
   * Get atlas data pointer for GPU upload
   */
  get_data_ptr(): number;
  /**
   * Update atlas to cover new viewport
   * Returns array of tile coords that need to be loaded [z, x, y, z, x, y, ...]
   */
  update_viewport(viewport_min_x: number, viewport_min_y: number, viewport_max_x: number, viewport_max_y: number, data_zoom: number): Uint32Array;
  get_bounds_max_x(): number;
  get_bounds_max_y(): number;
  get_bounds_min_x(): number;
  get_bounds_min_y(): number;
  constructor(buffer_ratio: number);
  /**
   * Sample wind at world coordinate (returns [u, v, valid] as f32)
   */
  sample(world_x: number, world_y: number): Float32Array;
  /**
   * Set tile data at specific tile coordinate
   */
  set_tile(z: number, x: number, y: number, data: Uint8Array): void;
  get_width(): number;
}

export class FixedAtlas {
  free(): void;
  [Symbol.dispose](): void;
  get_height(): number;
  clear_dirty(): void;
  get_data_len(): number;
  get_data_ptr(): number;
  /**
   * Get a copy of the atlas data as a Vec<u8>.
   * This is safer than get_data_ptr() as it avoids detached ArrayBuffer issues.
   */
  get_data_copy(): Uint8Array;
  get_tile_zoom(): number;
  /**
   * Update bounds without resizing.
   * Called on every pan/zoom. Does NOT clear data.
   */
  update_bounds(min_x: number, min_y: number, max_x: number, max_y: number, zoom: number): void;
  get_bounds_max_x(): number;
  get_bounds_max_y(): number;
  get_bounds_min_x(): number;
  get_bounds_min_y(): number;
  /**
   * Place tile data with bilinear resampling.
   * The tile is resampled to fit the current atlas bounds.
   */
  set_tile_resampled(z: number, x: number, y: number, tile_data: Uint8Array): void;
  /**
   * Create a new fixed-size atlas.
   * Size should be based on device screen resolution / 2^|zoom_offset|.
   */
  constructor(width: number, height: number);
  /**
   * Clear atlas data (fill with zeros).
   * Use when data zoom changes and all tiles need to be reloaded.
   */
  clear(): void;
  /**
   * Sample wind at world coordinate (returns [u, v, valid] as f32)
   */
  sample(world_x: number, world_y: number): Float32Array;
  is_dirty(): boolean;
  get_width(): number;
}

export class ParticleSimulator {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * POC 6: Set position of particle at index
   */
  set_position(idx: number, x: number, y: number): void;
  /**
   * POC 6: Get state of particle at index (WASM-exported version)
   */
  get_state(idx: number): number;
  /**
   * POC 2: Update with wind texture sampling + NaN handling
   */
  update_with_wind(wind: WindSampler, bounds_min_x: number, bounds_min_y: number, bounds_max_x: number, bounds_max_y: number, delta_time: number, speed_factor: number, max_age: number, random_seed: number, target_count: number): void;
  /**
   * Get max particles count
   */
  get_max_particles(): number;
  /**
   * Get length of particle data array
   */
  get_particles_len(): number;
  /**
   * Get pointer to particle data for WebGL upload
   */
  get_particles_ptr(): number;
  /**
   * POC 3: Update particles with wind sampling AND sync trails
   */
  update_with_trails(wind: WindSampler, trails: TrailManager, bounds_min_x: number, bounds_min_y: number, bounds_max_x: number, bounds_max_y: number, delta_time: number, speed_factor: number, max_age: number, random_seed: number, target_count: number): void;
  /**
   * Create new simulator with all particles inactive
   */
  constructor(max_particles: number);
  /**
   * POC 6: Get X position of particle at index
   */
  get_x(idx: number): number;
  /**
   * POC 6: Get Y position of particle at index
   */
  get_y(idx: number): number;
  /**
   * Update all particles
   */
  update(bounds_min_x: number, bounds_min_y: number, bounds_max_x: number, bounds_max_y: number, wind_x: number, wind_y: number, delta_time: number, max_age: number, random_seed: number, target_count: number): void;
  /**
   * POC 6: Set state of particle at index
   */
  set_state(idx: number, state: number): void;
}

export class TileCoord {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get geographic bounds of this tile (lon/lat)
   */
  get_geo_bounds(): Float64Array;
  /**
   * Get Web Mercator bounds of this tile
   */
  get_mercator_bounds(): Float64Array;
  constructor(z: number, x: number, y: number);
  z: number;
  x: number;
  y: number;
}

export class TileCoordinator {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Clear a specific tile (for LRU eviction sync)
   */
  clear_tile(z: number, x: number, y: number): void;
  /**
   * Clear tiles at specific zoom level
   */
  clear_zoom(z: number): void;
  /**
   * Mark tile as loaded
   */
  mark_loaded(z: number, x: number, y: number): void;
  /**
   * Get count of loaded tiles
   */
  loaded_count(): number;
  /**
   * Calculate effective data zoom from view zoom
   */
  get_data_zoom(view_zoom: number): number;
  /**
   * Set minimum and maximum data zoom levels
   */
  set_zoom_range(min_zoom: number, max_zoom: number): void;
  /**
   * Filter tile list to only unloaded tiles
   * Input: flat array [z, x, y, z, x, y, ...]
   * Output: flat array of unloaded tiles
   */
  filter_unloaded(tiles: Uint32Array): Uint32Array;
  constructor(zoom_offset: number);
  /**
   * Clear loaded tiles (e.g., when zoom changes significantly)
   */
  clear(): void;
  /**
   * Check if tile is already loaded
   */
  is_loaded(z: number, x: number, y: number): boolean;
}

export class TrailManager {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Reset all trail positions to the same point (for spawn/respawn)
   */
  reset_trail(particle_id: number, x: number, y: number): void;
  trail_length(): number;
  max_particles(): number;
  /**
   * Push new position: shift all positions toward tail, store new at head
   */
  push_position(particle_id: number, x: number, y: number): void;
  /**
   * Get total size of trails data
   */
  get_trails_len(): number;
  /**
   * Get raw pointer for WebGL upload
   */
  get_trails_ptr(): number;
  constructor(max_particles: number, trail_length: number);
}

export class TrailVertexBuilder {
  free(): void;
  [Symbol.dispose](): void;
  get_vertex_count(): number;
  get_vertices_ptr(): number;
  get_floats_per_vertex(): number;
  constructor(max_particles: number, trail_length: number);
  /**
   * Build vertex data from trails
   * color: base RGB color (0-1 range)
   */
  build(trails: TrailManager, particle_count: number, trail_width: number, opacity: number, color_r: number, color_g: number, color_b: number): void;
}

export class WindSampler {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create WindSampler from DynamicAtlas
   * Copies atlas data and bounds for particle simulation
   */
  static from_atlas(atlas: DynamicAtlas): WindSampler;
  get_height(): number;
  set_bounds(min_x: number, min_y: number, max_x: number, max_y: number): void;
  get_data_len(): number;
  get_data_ptr(): number;
  /**
   * Update WindSampler from DynamicAtlas
   * Call this when atlas data or bounds change
   */
  update_from_atlas(atlas: DynamicAtlas): void;
  constructor(width: number, height: number);
  width(): number;
  height(): number;
  /**
   * Check if position has valid wind data (alpha > 127)
   */
  is_valid(x: number, y: number): boolean;
  set_data(data: Uint8Array): void;
  get_max_x(): number;
  get_max_y(): number;
  get_min_x(): number;
  get_min_y(): number;
  get_width(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_atlasbounds_free: (a: number, b: number) => void;
  readonly __wbg_dynamicatlas_free: (a: number, b: number) => void;
  readonly __wbg_get_atlasbounds_max_x: (a: number) => number;
  readonly __wbg_get_atlasbounds_max_y: (a: number) => number;
  readonly __wbg_get_atlasbounds_min_x: (a: number) => number;
  readonly __wbg_get_atlasbounds_min_y: (a: number) => number;
  readonly __wbg_get_tilecoord_x: (a: number) => number;
  readonly __wbg_get_tilecoord_y: (a: number) => number;
  readonly __wbg_get_tilecoord_z: (a: number) => number;
  readonly __wbg_set_atlasbounds_max_x: (a: number, b: number) => void;
  readonly __wbg_set_atlasbounds_max_y: (a: number, b: number) => void;
  readonly __wbg_set_atlasbounds_min_x: (a: number, b: number) => void;
  readonly __wbg_set_atlasbounds_min_y: (a: number, b: number) => void;
  readonly __wbg_set_tilecoord_x: (a: number, b: number) => void;
  readonly __wbg_set_tilecoord_y: (a: number, b: number) => void;
  readonly __wbg_set_tilecoord_z: (a: number, b: number) => void;
  readonly __wbg_tilecoord_free: (a: number, b: number) => void;
  readonly atlasbounds_contains: (a: number, b: number, c: number) => number;
  readonly atlasbounds_expand: (a: number, b: number) => number;
  readonly atlasbounds_height: (a: number) => number;
  readonly atlasbounds_new: (a: number, b: number, c: number, d: number) => number;
  readonly atlasbounds_width: (a: number) => number;
  readonly dynamicatlas_get_bounds_max_x: (a: number) => number;
  readonly dynamicatlas_get_bounds_max_y: (a: number) => number;
  readonly dynamicatlas_get_bounds_min_x: (a: number) => number;
  readonly dynamicatlas_get_bounds_min_y: (a: number) => number;
  readonly dynamicatlas_get_data_len: (a: number) => number;
  readonly dynamicatlas_get_data_ptr: (a: number) => number;
  readonly dynamicatlas_get_height: (a: number) => number;
  readonly dynamicatlas_get_width: (a: number) => number;
  readonly dynamicatlas_new: (a: number) => number;
  readonly dynamicatlas_sample: (a: number, b: number, c: number) => [number, number];
  readonly dynamicatlas_set_tile: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly dynamicatlas_update_viewport: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly tilecoord_get_geo_bounds: (a: number) => [number, number];
  readonly tilecoord_get_mercator_bounds: (a: number) => [number, number];
  readonly tilecoord_new: (a: number, b: number, c: number) => number;
  readonly __wbg_trailmanager_free: (a: number, b: number) => void;
  readonly trailmanager_get_trails_len: (a: number) => number;
  readonly trailmanager_get_trails_ptr: (a: number) => number;
  readonly trailmanager_max_particles: (a: number) => number;
  readonly trailmanager_new: (a: number, b: number) => number;
  readonly trailmanager_push_position: (a: number, b: number, c: number, d: number) => void;
  readonly trailmanager_reset_trail: (a: number, b: number, c: number, d: number) => void;
  readonly trailmanager_trail_length: (a: number) => number;
  readonly __wbg_particlesimulator_free: (a: number, b: number) => void;
  readonly particlesimulator_get_max_particles: (a: number) => number;
  readonly particlesimulator_get_particles_len: (a: number) => number;
  readonly particlesimulator_get_particles_ptr: (a: number) => number;
  readonly particlesimulator_get_state: (a: number, b: number) => number;
  readonly particlesimulator_get_x: (a: number, b: number) => number;
  readonly particlesimulator_get_y: (a: number, b: number) => number;
  readonly particlesimulator_new: (a: number) => number;
  readonly particlesimulator_set_position: (a: number, b: number, c: number, d: number) => void;
  readonly particlesimulator_set_state: (a: number, b: number, c: number) => void;
  readonly particlesimulator_update: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
  readonly particlesimulator_update_with_trails: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
  readonly particlesimulator_update_with_wind: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
  readonly __wbg_trailvertexbuilder_free: (a: number, b: number) => void;
  readonly trailvertexbuilder_build: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly trailvertexbuilder_get_floats_per_vertex: (a: number) => number;
  readonly trailvertexbuilder_get_vertex_count: (a: number) => number;
  readonly trailvertexbuilder_get_vertices_ptr: (a: number) => number;
  readonly trailvertexbuilder_new: (a: number, b: number) => number;
  readonly __wbg_fixedatlas_free: (a: number, b: number) => void;
  readonly fixedatlas_clear: (a: number) => void;
  readonly fixedatlas_clear_dirty: (a: number) => void;
  readonly fixedatlas_get_bounds_max_x: (a: number) => number;
  readonly fixedatlas_get_bounds_max_y: (a: number) => number;
  readonly fixedatlas_get_bounds_min_x: (a: number) => number;
  readonly fixedatlas_get_bounds_min_y: (a: number) => number;
  readonly fixedatlas_get_data_copy: (a: number) => [number, number];
  readonly fixedatlas_get_data_len: (a: number) => number;
  readonly fixedatlas_get_data_ptr: (a: number) => number;
  readonly fixedatlas_get_height: (a: number) => number;
  readonly fixedatlas_get_tile_zoom: (a: number) => number;
  readonly fixedatlas_get_width: (a: number) => number;
  readonly fixedatlas_is_dirty: (a: number) => number;
  readonly fixedatlas_new: (a: number, b: number) => number;
  readonly fixedatlas_sample: (a: number, b: number, c: number) => [number, number];
  readonly fixedatlas_set_tile_resampled: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly fixedatlas_update_bounds: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly __wbg_windsampler_free: (a: number, b: number) => void;
  readonly windsampler_from_atlas: (a: number) => number;
  readonly windsampler_get_data_len: (a: number) => number;
  readonly windsampler_get_data_ptr: (a: number) => number;
  readonly windsampler_get_height: (a: number) => number;
  readonly windsampler_get_max_x: (a: number) => number;
  readonly windsampler_get_max_y: (a: number) => number;
  readonly windsampler_get_min_x: (a: number) => number;
  readonly windsampler_get_min_y: (a: number) => number;
  readonly windsampler_get_width: (a: number) => number;
  readonly windsampler_is_valid: (a: number, b: number, c: number) => number;
  readonly windsampler_new: (a: number, b: number) => number;
  readonly windsampler_set_bounds: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly windsampler_set_data: (a: number, b: number, c: number) => void;
  readonly windsampler_update_from_atlas: (a: number, b: number) => void;
  readonly windsampler_height: (a: number) => number;
  readonly windsampler_width: (a: number) => number;
  readonly __wbg_tilecoordinator_free: (a: number, b: number) => void;
  readonly tilecoordinator_clear: (a: number) => void;
  readonly tilecoordinator_clear_tile: (a: number, b: number, c: number, d: number) => void;
  readonly tilecoordinator_clear_zoom: (a: number, b: number) => void;
  readonly tilecoordinator_filter_unloaded: (a: number, b: number, c: number) => [number, number];
  readonly tilecoordinator_get_data_zoom: (a: number, b: number) => number;
  readonly tilecoordinator_is_loaded: (a: number, b: number, c: number, d: number) => number;
  readonly tilecoordinator_loaded_count: (a: number) => number;
  readonly tilecoordinator_mark_loaded: (a: number, b: number, c: number, d: number) => void;
  readonly tilecoordinator_new: (a: number) => number;
  readonly tilecoordinator_set_zoom_range: (a: number, b: number, c: number) => void;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
