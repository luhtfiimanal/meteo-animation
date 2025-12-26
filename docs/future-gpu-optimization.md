# Future: GPU-Centric Particle Animation

## Overview

POC 6.1 uses WASM for particle physics (CPU-based). This document outlines
a future GPU-native approach using WebGL 2 Transform Feedback.

## Current Architecture (WASM)

```
Tiles → WASM Atlas → CPU Resample → GPU Texture
                ↓
        CPU: WindSampler.sample()
                ↓
        CPU: ParticleSimulator.update()
                ↓
        Upload vertex buffer every frame
                ↓
        GPU: Render only
```

## Future Architecture (GPU Transform Feedback)

```
Tiles → GPU Tile Cache Texture
                ↓
        GPU: Resample shader (render-to-texture)
                ↓
        GPU: Atlas Texture
                ↓
        GPU: Transform Feedback (particle update)
             - Sample wind from atlas
             - Update positions
             - Handle respawn
                ↓
        GPU: Render (no upload needed!)
```

## Key WebGL 2 Features

### Transform Feedback

Update vertex attributes in GPU without CPU roundtrip:

```javascript
// Setup
const tf = gl.createTransformFeedback();
gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);

// Link output varyings
gl.transformFeedbackVaryings(program,
    ['v_position', 'v_age'], gl.SEPARATE_ATTRIBS);

// Update loop
gl.beginTransformFeedback(gl.POINTS);
gl.drawArrays(gl.POINTS, 0, particleCount);
gl.endTransformFeedback();

// Swap buffers (ping-pong)
[inputBuffer, outputBuffer] = [outputBuffer, inputBuffer];
```

### Particle Update Shader

```glsl
#version 300 es
precision highp float;

// Input attributes (from previous frame)
in vec2 a_position;
in float a_age;
in float a_seed;

// Output (transform feedback)
out vec2 v_position;
out float v_age;
out float v_seed;

// Uniforms
uniform sampler2D u_wind;
uniform vec2 u_bounds_min;
uniform vec2 u_bounds_max;
uniform float u_dt;
uniform float u_speed;
uniform float u_max_age;
uniform float u_time;

// Pseudo-random
float random(vec2 st) {
    return fract(sin(dot(st, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    // Sample wind (GPU bilinear via GL_LINEAR)
    vec2 uv = (a_position - u_bounds_min) / (u_bounds_max - u_bounds_min);
    vec4 wind = texture(u_wind, uv);

    float valid = wind.a > 0.5 ? 1.0 : 0.0;
    float u = (wind.r - 0.5) * 30.0;
    float v = (wind.g - 0.5) * 30.0;

    // Update position
    vec2 newPos = a_position + vec2(u, v) * u_dt * u_speed;
    float newAge = a_age + u_dt;

    // Check bounds and validity
    bool outOfBounds = newPos.x < u_bounds_min.x || newPos.x > u_bounds_max.x ||
                       newPos.y < u_bounds_min.y || newPos.y > u_bounds_max.y;
    bool tooOld = newAge > u_max_age;
    bool invalid = valid < 0.5;

    // Respawn if needed
    if (outOfBounds || tooOld || invalid) {
        vec2 seed = vec2(a_seed, u_time);
        newPos.x = mix(u_bounds_min.x, u_bounds_max.x, random(seed));
        newPos.y = mix(u_bounds_min.y, u_bounds_max.y, random(seed + 1.0));
        newAge = random(seed + 2.0) * u_max_age;
    }

    v_position = newPos;
    v_age = newAge;
    v_seed = a_seed;
}
```

### Render-to-Texture (Atlas Resample)

```javascript
// Setup framebuffer for atlas
const atlasFBO = gl.createFramebuffer();
const atlasTexture = gl.createTexture();
gl.bindTexture(gl.TEXTURE_2D, atlasTexture);
gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, atlasWidth, atlasHeight,
              0, gl.RGBA, gl.UNSIGNED_BYTE, null);
gl.bindFramebuffer(gl.FRAMEBUFFER, atlasFBO);
gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                        gl.TEXTURE_2D, atlasTexture, 0);

// On bounds change - resample tiles to atlas
function resampleAtlas() {
    gl.bindFramebuffer(gl.FRAMEBUFFER, atlasFBO);
    gl.viewport(0, 0, atlasWidth, atlasHeight);
    gl.useProgram(resampleProgram);
    gl.bindTexture(gl.TEXTURE_2D, tileCacheTexture);
    gl.uniform2f(u_view_min, viewBounds.minX, viewBounds.minY);
    gl.uniform2f(u_view_max, viewBounds.maxX, viewBounds.maxY);
    // ... set tile bounds uniforms
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
}
```

## Benefits

| Metric | WASM (Current) | GPU (Future) |
|--------|----------------|--------------|
| Particle update | ~2ms CPU | ~0.1ms GPU |
| Buffer upload | Every frame | Never |
| Wind sampling | CPU bilinear | GPU GL_LINEAR |
| Power usage | Higher | Lower |
| Mobile perf | Good | Better |

## Implementation Priority

1. First: Fix POC 6.1 (WASM, no blink)
2. Later: POC 7 with Transform Feedback
3. Future: WebGPU compute shaders (when widely supported)

## References

- [WebGL 2 Transform Feedback](https://webgl2fundamentals.org/webgl/lessons/webgl-transform-feedback.html)
- [Particle Systems with Transform Feedback](https://www.khronos.org/opengl/wiki/Transform_Feedback)
