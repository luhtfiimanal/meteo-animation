/// Random function matching WGSL compute shader
/// fract(sin(seed * 12.9898 + 78.233) * 43758.5453)
pub fn random(seed: f32) -> f32 {
    let x = (seed * 12.9898 + 78.233).sin() * 43758.5453;
    x - x.floor()
}

/// Linear interpolation (same as GLSL mix)
pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Check if position is within bounds
pub fn in_bounds(x: f32, y: f32, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> bool {
    x >= min_x && x <= max_x && y >= min_y && y <= max_y
}

/// Clamp value to range
pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.max(min).min(max)
}

/// Bilinear interpolation
pub fn bilerp(v00: f32, v10: f32, v01: f32, v11: f32, tx: f32, ty: f32) -> f32 {
    let v0 = mix(v00, v10, tx);
    let v1 = mix(v01, v11, tx);
    mix(v0, v1, ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_deterministic() {
        // Same seed should produce same result
        assert_eq!(random(1.0), random(1.0));
        assert_eq!(random(42.0), random(42.0));
    }

    #[test]
    fn test_random_different_seeds() {
        // Different seeds should produce different results
        assert_ne!(random(1.0), random(2.0));
    }

    #[test]
    fn test_random_range() {
        // Output should be in [0, 1)
        for i in 0..100 {
            let r = random(i as f32);
            assert!(r >= 0.0 && r < 1.0, "random({}) = {} not in [0,1)", i, r);
        }
    }

    #[test]
    fn test_mix_midpoint() {
        assert_eq!(mix(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_mix_endpoints() {
        assert_eq!(mix(0.0, 10.0, 0.0), 0.0);
        assert_eq!(mix(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_mix_negative() {
        assert_eq!(mix(-10.0, 10.0, 0.5), 0.0);
    }

    #[test]
    fn test_in_bounds_inside() {
        assert!(in_bounds(50.0, 50.0, 0.0, 0.0, 100.0, 100.0));
    }

    #[test]
    fn test_in_bounds_edge() {
        assert!(in_bounds(0.0, 0.0, 0.0, 0.0, 100.0, 100.0));
        assert!(in_bounds(100.0, 100.0, 0.0, 0.0, 100.0, 100.0));
    }

    #[test]
    fn test_in_bounds_outside() {
        assert!(!in_bounds(-1.0, 50.0, 0.0, 0.0, 100.0, 100.0));
        assert!(!in_bounds(101.0, 50.0, 0.0, 0.0, 100.0, 100.0));
        assert!(!in_bounds(50.0, -1.0, 0.0, 0.0, 100.0, 100.0));
        assert!(!in_bounds(50.0, 101.0, 0.0, 0.0, 100.0, 100.0));
    }

    #[test]
    fn test_clamp_in_range() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
    }

    #[test]
    fn test_clamp_below() {
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn test_clamp_above() {
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_bilerp_corners() {
        // All same values
        assert_eq!(bilerp(1.0, 1.0, 1.0, 1.0, 0.5, 0.5), 1.0);
    }

    #[test]
    fn test_bilerp_horizontal_gradient() {
        // 0 on left, 10 on right
        let result = bilerp(0.0, 10.0, 0.0, 10.0, 0.5, 0.5);
        assert!((result - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_bilerp_center() {
        // Different values at corners
        let result = bilerp(0.0, 10.0, 20.0, 30.0, 0.5, 0.5);
        // Expected: avg of (0+10)/2=5 and (20+30)/2=25 = 15
        assert!((result - 15.0).abs() < 0.001);
    }
}
