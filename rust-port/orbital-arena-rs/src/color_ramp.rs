//! Speed color ramp (sandbox-visual-identity.spec.md §3) — the signature
//! blue → cyan → magenta → white palette shared by free-particle bloom/trails
//! (§4) and anything else that colors motion by speed. A pure function, unlike
//! most of this port's render code, so it gets real unit tests.

use bevy::prelude::*;

/// Map a speed in m/s (world units/s) to the reference's 3-segment smoothstep gradient
/// over `[0, 8]` m/s (§3): deep blue → cyan → magenta → white. `alpha` is passed through
/// unchanged so callers can combine it with their own fade/pulse alpha.
pub fn color_from_speed(speed: f64, alpha: f32) -> Color {
    let t = (speed / 8.0).clamp(0.0, 1.0);
    let t2 = t * t * (3.0 - 2.0 * t); // smoothstep

    let (r, g, b) = if t2 < 0.33 {
        let u = t2 / 0.33;
        (0.1 * u, 0.3 + 0.5 * u, 0.8 + 0.2 * u)
    } else if t2 < 0.66 {
        let u = (t2 - 0.33) / 0.33;
        (0.1 + 0.8 * u, 0.8 - 0.5 * u, 1.0 - 0.2 * u)
    } else {
        let u = (t2 - 0.66) / 0.34;
        (0.9 + 0.1 * u, 0.3 + 0.7 * u, 0.8 + 0.2 * u)
    };

    Color::srgba(r as f32, g as f32, b as f32, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(color: Color) -> [f32; 4] {
        color.to_srgba().to_f32_array()
    }

    #[test]
    fn zero_speed_is_deep_blue() {
        let [r, g, b, a] = rgba(color_from_speed(0.0, 1.0));
        assert!((r - 0.0).abs() < 1e-6);
        assert!((g - 0.3).abs() < 1e-6);
        assert!((b - 0.8).abs() < 1e-6);
        assert!((a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn max_speed_is_near_white() {
        let [r, g, b, _] = rgba(color_from_speed(8.0, 1.0));
        assert!((r - 1.0).abs() < 1e-5);
        assert!((g - 1.0).abs() < 1e-5);
        assert!((b - 1.0).abs() < 1e-5);
    }

    #[test]
    fn speed_above_range_clamps_to_max() {
        assert_eq!(
            rgba(color_from_speed(100.0, 1.0)),
            rgba(color_from_speed(8.0, 1.0))
        );
    }

    #[test]
    fn alpha_passes_through_unchanged() {
        let [.., a] = rgba(color_from_speed(3.0, 0.42));
        assert!((a - 0.42).abs() < 1e-6);
    }
}
