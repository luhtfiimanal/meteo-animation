use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement,
    WebGl2RenderingContext,
    WebGlBuffer,
    WebGlProgram,
    WebGlShader,
    WebGlTexture,
    WebGlVertexArrayObject,
};
use crate::{
    ParticleSimulator,
    TrailManager,
    TrailVertexBuilder,
    FixedAtlas,
};

/// Hybrid WASM Renderer
///
/// Owns WebGL context and renders particles directly from WASM.
/// Eliminates JS ↔ WASM boundary for vertex data.
///
/// Architecture:
/// - JS: Leaflet map, tile fetching, view bounds
/// - WASM: Particle sim, vertex build, WebGL rendering (this module)
/// - GPU: Final output
///
/// Only 4 hops: JS → WASM → GPU → Screen
#[wasm_bindgen]
pub struct HybridRenderer {
    // WebGL context
    gl: WebGl2RenderingContext,

    // Shader programs
    trail_program: WebGlProgram,
    bg_program: WebGlProgram,

    // Buffers and VAO
    trail_vao: WebGlVertexArrayObject,
    trail_buffer: WebGlBuffer,
    wind_texture: WebGlTexture,

    // Particle system (WASM-owned)
    particles: ParticleSimulator,
    trails: TrailManager,
    vertex_builder: TrailVertexBuilder,
    atlas: FixedAtlas,

    // Render state
    last_wind_tex_width: u32,
    last_wind_tex_height: u32,

    // View projection
    clip_per_meter_x: f32,
    clip_per_meter_y: f32,
    translate_x: f32,
    translate_y: f32,
}

#[wasm_bindgen]
impl HybridRenderer {
    /// Create a new hybrid renderer
    ///
    /// # Arguments
    /// * `canvas` - HTML canvas element
    /// * `max_particles` - Maximum number of particles
    /// * `trail_length` - Trail history length
    /// * `atlas_width` - Fixed atlas width in pixels
    /// * `atlas_height` - Fixed atlas height in pixels
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: HtmlCanvasElement,
        max_particles: u32,
        trail_length: u32,
        atlas_width: u32,
        atlas_height: u32,
    ) -> Result<HybridRenderer, JsValue> {
        // Get WebGL2 context
        let gl = canvas
            .get_context("webgl2")?
            .ok_or("Failed to get WebGL2 context")?
            .dyn_into::<WebGl2RenderingContext>()?;

        // Enable blending for particle trails
        gl.enable(WebGl2RenderingContext::BLEND);
        gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        // Create shader programs
        let trail_program = create_trail_program(&gl)?;
        let bg_program = create_background_program(&gl)?;

        // Create trail VAO and buffer
        let trail_vao = gl.create_vertex_array()
            .ok_or("Failed to create VAO")?;
        let trail_buffer = gl.create_buffer()
            .ok_or("Failed to create buffer")?;

        gl.bind_vertex_array(Some(&trail_vao));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&trail_buffer));

        // Vertex layout: [x, y, alpha, r, g, b] = 6 floats per vertex
        let stride = 6 * 4; // 6 floats * 4 bytes
        let max_vertices = max_particles * (trail_length - 1) * 6; // Quads as triangles
        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            (max_vertices * stride) as i32,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        // Setup vertex attributes
        let pos_location = gl.get_attrib_location(&trail_program, "a_position") as u32;
        let alpha_location = gl.get_attrib_location(&trail_program, "a_alpha") as u32;
        let color_location = gl.get_attrib_location(&trail_program, "a_color") as u32;

        gl.enable_vertex_attrib_array(pos_location);
        gl.vertex_attrib_pointer_with_i32(
            pos_location, 2, WebGl2RenderingContext::FLOAT, false, stride, 0
        );

        gl.enable_vertex_attrib_array(alpha_location);
        gl.vertex_attrib_pointer_with_i32(
            alpha_location, 1, WebGl2RenderingContext::FLOAT, false, stride, 8
        );

        gl.enable_vertex_attrib_array(color_location);
        gl.vertex_attrib_pointer_with_i32(
            color_location, 3, WebGl2RenderingContext::FLOAT, false, stride, 12
        );

        // Create wind texture
        let wind_texture = gl.create_texture()
            .ok_or("Failed to create texture")?;

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&wind_texture));
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );

        // Create particle system
        let particles = ParticleSimulator::new(max_particles);
        let trails = TrailManager::new(max_particles, trail_length);
        let vertex_builder = TrailVertexBuilder::new(max_particles, trail_length);
        let atlas = FixedAtlas::new(atlas_width, atlas_height);

        Ok(HybridRenderer {
            gl,
            trail_program,
            bg_program,
            trail_vao,
            trail_buffer,
            wind_texture,
            particles,
            trails,
            vertex_builder,
            atlas,
            last_wind_tex_width: 0,
            last_wind_tex_height: 0,
            clip_per_meter_x: 1.0,
            clip_per_meter_y: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
        })
    }

    /// Update view projection matrix
    /// Called from JS when map pans/zooms
    pub fn set_projection(
        &mut self,
        clip_per_meter_x: f32,
        clip_per_meter_y: f32,
        translate_x: f32,
        translate_y: f32,
    ) {
        self.clip_per_meter_x = clip_per_meter_x;
        self.clip_per_meter_y = clip_per_meter_y;
        self.translate_x = translate_x;
        self.translate_y = translate_y;
    }

    /// Update atlas bounds
    /// Called from JS when viewport changes
    pub fn update_atlas_bounds(
        &mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        zoom: u32,
    ) {
        self.atlas.update_bounds(min_x, min_y, max_x, max_y, zoom);
    }

    /// Set atlas tile data
    /// Called from JS after tile fetch
    pub fn set_atlas_tile(
        &mut self,
        z: u32,
        x: u32,
        y: u32,
        tile_data: &[u8],
    ) {
        self.atlas.set_tile_resampled(z, x, y, tile_data);
    }

    /// Clear atlas (e.g., on zoom level change)
    pub fn clear_atlas(&mut self) {
        self.atlas.clear();
    }

    /// Main render function - called every frame from JS
    ///
    /// Does EVERYTHING in WASM:
    /// 1. Update particles
    /// 2. Build vertices
    /// 3. Upload to GPU
    /// 4. Draw
    ///
    /// Zero copies to JS!
    pub fn render(
        &mut self,
        delta_time: f32,
        particle_count: u32,
        speed_factor: f32,
        max_age: u32,
        time: f32,
        trail_width: f32,
        trail_opacity: f32,
    ) {
        // Get atlas bounds for particle simulation
        let bounds_min_x = self.atlas.get_bounds_min_x() as f32;
        let bounds_min_y = self.atlas.get_bounds_min_y() as f32;
        let bounds_max_x = self.atlas.get_bounds_max_x() as f32;
        let bounds_max_y = self.atlas.get_bounds_max_y() as f32;

        // Update particles (WASM, zero copy)
        // TODO: Create WindSampler from atlas for wind sampling
        // For now, this is placeholder - will integrate with existing WindSampler

        // Build vertices (WASM, zero copy)
        self.vertex_builder.build(
            &self.trails,
            particle_count,
            trail_width,
            trail_opacity,
            0.376, 0.647, 0.980, // Light blue color
        );

        let vertex_count = self.vertex_builder.get_vertex_count();
        if vertex_count == 0 {
            return;
        }

        // Get vertex data pointer (still in WASM memory)
        let vertex_ptr = self.vertex_builder.get_vertices_ptr();
        let vertex_len = vertex_count * 6; // 6 floats per vertex

        // Upload to GPU (WASM → GPU direct, no JS!)
        unsafe {
            let vertex_slice = std::slice::from_raw_parts(
                vertex_ptr as *const f32,
                vertex_len as usize,
            );
            let vertex_bytes = std::slice::from_raw_parts(
                vertex_slice.as_ptr() as *const u8,
                vertex_slice.len() * 4,
            );

            self.gl.bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&self.trail_buffer),
            );
            self.gl.buffer_sub_data_with_i32_and_u8_array(
                WebGl2RenderingContext::ARRAY_BUFFER,
                0,
                vertex_bytes,
            );
        }

        // Update wind texture if atlas is dirty
        if self.atlas.is_dirty() {
            self.update_wind_texture();
            self.atlas.clear_dirty();
        }

        // Clear screen
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        // Draw background
        self.draw_background(bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y);

        // Draw trails
        self.draw_trails(vertex_count);
    }

    // Private methods

    fn update_wind_texture(&mut self) {
        let width = self.atlas.get_width();
        let height = self.atlas.get_height();
        let data_ptr = self.atlas.get_data_ptr();
        let data_len = self.atlas.get_data_len();

        if data_len == 0 {
            return;
        }

        unsafe {
            let data_slice = std::slice::from_raw_parts(data_ptr, data_len);

            self.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.wind_texture),
            );

            if width != self.last_wind_tex_width || height != self.last_wind_tex_height {
                // Reallocate
                self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    WebGl2RenderingContext::RGBA as i32,
                    width as i32,
                    height as i32,
                    0,
                    WebGl2RenderingContext::RGBA,
                    WebGl2RenderingContext::UNSIGNED_BYTE,
                    Some(data_slice),
                ).unwrap();

                self.last_wind_tex_width = width;
                self.last_wind_tex_height = height;
            } else {
                // Update existing
                self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    0,
                    0,
                    width as i32,
                    height as i32,
                    WebGl2RenderingContext::RGBA,
                    WebGl2RenderingContext::UNSIGNED_BYTE,
                    Some(data_slice),
                ).unwrap();
            }
        }
    }

    fn draw_background(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        // TODO: Implement background shader
        // For now, skip to focus on particle rendering
    }

    fn draw_trails(&self, vertex_count: u32) {
        self.gl.use_program(Some(&self.trail_program));

        // Set uniforms
        let clip_loc = self.gl.get_uniform_location(&self.trail_program, "u_clip_per_meter");
        self.gl.uniform2f(
            clip_loc.as_ref(),
            self.clip_per_meter_x,
            self.clip_per_meter_y,
        );

        let translate_loc = self.gl.get_uniform_location(&self.trail_program, "u_translate");
        self.gl.uniform2f(
            translate_loc.as_ref(),
            self.translate_x,
            self.translate_y,
        );

        // Draw
        self.gl.bind_vertex_array(Some(&self.trail_vao));
        self.gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            vertex_count as i32,
        );
    }
}

// Shader creation helpers

fn create_trail_program(gl: &WebGl2RenderingContext) -> Result<WebGlProgram, JsValue> {
    let vertex_shader = compile_shader(
        gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        TRAIL_VERTEX_SHADER,
    )?;

    let fragment_shader = compile_shader(
        gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        TRAIL_FRAGMENT_SHADER,
    )?;

    link_program(gl, &vertex_shader, &fragment_shader)
}

fn create_background_program(gl: &WebGl2RenderingContext) -> Result<WebGlProgram, JsValue> {
    // TODO: Implement background shaders
    // For now, create dummy program
    create_trail_program(gl)
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl.create_shader(shader_type)
        .ok_or("Failed to create shader")?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if !gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let log = gl.get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(JsValue::from_str(&format!("Shader compile error: {}", log)));
    }

    Ok(shader)
}

fn link_program(
    gl: &WebGl2RenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, JsValue> {
    let program = gl.create_program()
        .ok_or("Failed to create program")?;

    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    gl.link_program(&program);

    if !gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let log = gl.get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(JsValue::from_str(&format!("Program link error: {}", log)));
    }

    Ok(program)
}

// Shaders (same as POC 6)

const TRAIL_VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;

in vec2 a_position;
in float a_alpha;
in vec3 a_color;

uniform vec2 u_clip_per_meter;
uniform vec2 u_translate;

out vec4 v_color;

void main() {
    vec2 clip = a_position * u_clip_per_meter + u_translate;
    gl_Position = vec4(clip, 0.0, 1.0);
    v_color = vec4(a_color, a_alpha);
}
"#;

const TRAIL_FRAGMENT_SHADER: &str = r#"#version 300 es
precision mediump float;

in vec4 v_color;
out vec4 fragColor;

void main() {
    fragColor = v_color;
}
"#;
