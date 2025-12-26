# WebGL Shaders (GLSL)

Konversi dari WGSL (WebGPU) ke GLSL ES 3.0 (WebGL 2).

---

## POC 1: Particle Point Sprites

### Vertex Shader

```glsl
#version 300 es
precision highp float;

// Particle attributes dari WASM
in vec2 a_position;
in float a_age;
in float a_state;

// Uniforms
uniform vec2 u_bounds_min;
uniform vec2 u_bounds_max;
uniform vec2 u_canvas_size;
uniform float u_point_size;

// Output ke fragment shader
out vec4 v_color;

void main() {
    // Skip inactive particles (state == 3)
    if (a_state == 3.0) {
        gl_Position = vec4(2.0, 2.0, 0.0, 1.0); // Off-screen
        gl_PointSize = 0.0;
        return;
    }

    // Transform Mercator coords to clip space [-1, 1]
    vec2 normalized = (a_position - u_bounds_min) / (u_bounds_max - u_bounds_min);
    vec2 clip_pos = normalized * 2.0 - 1.0;

    // Y-flip for screen coordinates
    gl_Position = vec4(clip_pos.x, -clip_pos.y, 0.0, 1.0);
    gl_PointSize = u_point_size;

    // Color based on particle state
    if (a_state == 2.0) {
        // Respawning - yellow
        v_color = vec4(0.984, 0.749, 0.145, 1.0);
    } else if (a_state == 1.0) {
        // Exiting - red
        v_color = vec4(0.973, 0.443, 0.443, 1.0);
    } else {
        // Active - green with age fade
        float age_fade = 1.0 - (a_age / 100.0);
        v_color = vec4(0.290, 0.871, 0.502, age_fade);
    }
}
```

### Fragment Shader

```glsl
#version 300 es
precision mediump float;

in vec4 v_color;
out vec4 fragColor;

void main() {
    // gl_PointCoord: [0,1] within point sprite
    vec2 coord = gl_PointCoord * 2.0 - 1.0;
    float dist = length(coord);

    // Discard pixels outside circle
    if (dist > 1.0) discard;

    // Soft edge with distance fade
    float alpha = v_color.a * (1.0 - dist * 0.5);
    fragColor = vec4(v_color.rgb, alpha);
}
```

---

## POC 2: NaN Region Visualization

### Full-Screen Quad Vertex Shader

```glsl
#version 300 es
precision highp float;

in vec2 a_position;  // Quad vertices: [-1,-1], [1,-1], [-1,1], [1,1]

out vec2 v_uv;

void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);

    // Convert clip space to UV [0, 1]
    v_uv = (a_position + 1.0) * 0.5;
}
```

### NaN Region Fragment Shader

```glsl
#version 300 es
precision mediump float;

uniform sampler2D u_wind_texture;

in vec2 v_uv;
out vec4 fragColor;

void main() {
    vec4 wind = texture(u_wind_texture, v_uv);

    // Alpha < 0.5 indicates NaN/NoData region
    if (wind.a < 0.5) {
        // Dark background for invalid regions
        fragColor = vec4(0.15, 0.15, 0.2, 1.0);
    } else {
        // Subtle background showing wind magnitude
        float magnitude = wind.b;
        fragColor = vec4(
            0.04 + magnitude * 0.04,
            0.04 + magnitude * 0.04,
            0.08 + magnitude * 0.06,
            1.0
        );
    }
}
```

---

## POC 3: Trail Rendering

### Trail Vertex Shader

```glsl
#version 300 es
precision highp float;

// Per-vertex attributes (pre-built by WASM TrailVertexBuilder)
in vec2 a_position;    // World position
in float a_alpha;      // Trail fade (1.0 at head, 0.0 at tail)
in vec3 a_color;       // RGB color

// Uniforms
uniform vec2 u_bounds_min;
uniform vec2 u_bounds_max;
uniform float u_global_opacity;

// Output
out vec4 v_color;

void main() {
    // Transform to clip space
    vec2 normalized = (a_position - u_bounds_min) / (u_bounds_max - u_bounds_min);
    vec2 clip_pos = normalized * 2.0 - 1.0;

    gl_Position = vec4(clip_pos.x, -clip_pos.y, 0.0, 1.0);

    // Apply alpha fade and global opacity
    v_color = vec4(a_color, a_alpha * u_global_opacity);
}
```

### Trail Fragment Shader

```glsl
#version 300 es
precision mediump float;

in vec4 v_color;
out vec4 fragColor;

void main() {
    fragColor = v_color;
}
```

---

## POC 7: Production Trail Shader

### Vertex Shader (dengan speed-based color)

```glsl
#version 300 es
precision highp float;

// Vertex attributes dari WASM
in vec2 a_position;
in float a_alpha;
in float a_speed;     // Wind speed magnitude

// Uniforms
uniform vec2 u_bounds_min;
uniform vec2 u_bounds_max;
uniform float u_opacity;
uniform float u_max_speed;

// Output
out vec4 v_color;

// Speed to color mapping (viridis-like)
vec3 speedToColor(float speed) {
    float t = clamp(speed / u_max_speed, 0.0, 1.0);

    // Simplified viridis colormap
    vec3 c0 = vec3(0.267, 0.004, 0.329); // Purple (low)
    vec3 c1 = vec3(0.282, 0.471, 0.557); // Teal
    vec3 c2 = vec3(0.133, 0.658, 0.518); // Green
    vec3 c3 = vec3(0.993, 0.906, 0.144); // Yellow (high)

    if (t < 0.33) {
        return mix(c0, c1, t * 3.0);
    } else if (t < 0.66) {
        return mix(c1, c2, (t - 0.33) * 3.0);
    } else {
        return mix(c2, c3, (t - 0.66) * 3.0);
    }
}

void main() {
    // Transform to clip space
    vec2 normalized = (a_position - u_bounds_min) / (u_bounds_max - u_bounds_min);
    vec2 clip_pos = normalized * 2.0 - 1.0;

    gl_Position = vec4(clip_pos.x, -clip_pos.y, 0.0, 1.0);

    // Color based on wind speed
    vec3 color = speedToColor(a_speed);
    v_color = vec4(color, a_alpha * u_opacity);
}
```

### Fragment Shader (dengan smooth edges)

```glsl
#version 300 es
precision mediump float;

in vec4 v_color;
out vec4 fragColor;

void main() {
    // Optional: slight premultiplied alpha untuk smooth blending
    fragColor = vec4(v_color.rgb * v_color.a, v_color.a);
}
```

---

## WebGL 2 vs WebGL 1 Compatibility

### WebGL 2 (GLSL ES 3.0) - Recommended

```glsl
#version 300 es
in vec2 a_position;    // attribute → in
out vec4 v_color;      // varying → out
out vec4 fragColor;    // gl_FragColor → out variable
texture()              // texture2D → texture
```

### WebGL 1 Fallback (GLSL ES 1.0)

```glsl
// No #version directive
attribute vec2 a_position;
varying vec4 v_color;
// gl_FragColor instead of out
texture2D()            // texture2D instead of texture
```

---

## Shader Utility Functions

### Random Number Generator (match WGSL)

```glsl
float random(float seed) {
    return fract(sin(seed * 12.9898 + 78.233) * 43758.5453);
}

vec2 random2(float seed) {
    return vec2(random(seed), random(seed + 1.0));
}
```

### Bilinear Interpolation (for CPU wind sampling)

```glsl
// Note: Bilinear dilakukan di Rust/WASM, bukan di shader
// Tapi jika perlu di shader:
vec4 sampleBilinear(sampler2D tex, vec2 uv, vec2 texSize) {
    vec2 pixel = uv * texSize - 0.5;
    vec2 f = fract(pixel);
    vec2 base = (floor(pixel) + 0.5) / texSize;
    vec2 step = 1.0 / texSize;

    vec4 tl = texture(tex, base);
    vec4 tr = texture(tex, base + vec2(step.x, 0.0));
    vec4 bl = texture(tex, base + vec2(0.0, step.y));
    vec4 br = texture(tex, base + step);

    return mix(mix(tl, tr, f.x), mix(bl, br, f.x), f.y);
}
```

---

## JavaScript: Shader Loading & Compilation

```javascript
function createShader(gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error('Shader compile error:', gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
    }
    return shader;
}

function createProgram(gl, vertexSource, fragmentSource) {
    const vertexShader = createShader(gl, gl.VERTEX_SHADER, vertexSource);
    const fragmentShader = createShader(gl, gl.FRAGMENT_SHADER, fragmentSource);

    const program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error('Program link error:', gl.getProgramInfoLog(program));
        gl.deleteProgram(program);
        return null;
    }
    return program;
}
```

---

## Attribute & Uniform Setup

```javascript
// Setup particle rendering
function setupParticleRendering(gl, program) {
    // Get attribute locations
    const posLoc = gl.getAttribLocation(program, 'a_position');
    const ageLoc = gl.getAttribLocation(program, 'a_age');
    const stateLoc = gl.getAttribLocation(program, 'a_state');

    // Get uniform locations
    const boundsMinLoc = gl.getUniformLocation(program, 'u_bounds_min');
    const boundsMaxLoc = gl.getUniformLocation(program, 'u_bounds_max');
    const pointSizeLoc = gl.getUniformLocation(program, 'u_point_size');

    // Create VAO
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    // Setup vertex buffer (interleaved: [x, y, age, state] per particle)
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

    const STRIDE = 4 * 4; // 4 floats * 4 bytes

    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, STRIDE, 0);

    gl.enableVertexAttribArray(ageLoc);
    gl.vertexAttribPointer(ageLoc, 1, gl.FLOAT, false, STRIDE, 8);

    gl.enableVertexAttribArray(stateLoc);
    gl.vertexAttribPointer(stateLoc, 1, gl.FLOAT, false, STRIDE, 12);

    return { vao, buffer, boundsMinLoc, boundsMaxLoc, pointSizeLoc };
}
```

---

## Blending Setup

```javascript
// Enable alpha blending for trails
gl.enable(gl.BLEND);
gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

// Alternative: additive blending for glow effect
// gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
```

---

## Related Docs

- [Migration Overview](./webgl-migration-overview.md)
- [Rust Crate Specification](./webgl-rust-crate.md)
- [POC Analysis](./webgl-poc-analysis.md)
