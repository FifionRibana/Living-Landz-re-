mod sdf_generation;
mod territory_contours;
pub mod contour_generation;

pub use sdf_generation::*;
pub use territory_contours::*;
pub use contour_generation::*;

use std::hash::{Hash, Hasher};

/// Generate pseudo-random colors for an organization based on its ID
/// Returns (border_color, fill_color) as RGBA tuples
pub fn generate_org_colors(org_id: u64) -> ([f32; 4], [f32; 4]) {
    // Use a simple hash function to generate deterministic pseudo-random colors
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    org_id.hash(&mut hasher);
    let hash = hasher.finish();

    // Generate vibrant hue from hash (0-360 degrees)
    let hue = (hash % 360) as f32;
    let saturation = 0.7;
    let lightness = 0.5;

    // Convert HSL to RGB for border (dark, saturated)
    let border_rgb = hsl_to_rgb(hue, saturation, lightness * 0.4);
    let border_color = [border_rgb.0, border_rgb.1, border_rgb.2, 1.0f32];

    // Fill color (lighter, semi-transparent)
    let fill_rgb = hsl_to_rgb(hue, saturation * 0.6, lightness * 1.2);
    let fill_color = [fill_rgb.0, fill_rgb.1, fill_rgb.2, 0.3f32];

    (border_color, fill_color)
}

/// Convert HSL color to RGB
/// h: hue in degrees (0-360)
/// s: saturation (0-1)
/// l: lightness (0-1)
/// Returns (r, g, b) in range 0-1
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let h = h / 360.0;
    let s = s.clamp(0.0, 1.0);
    let l = l.clamp(0.0, 1.0);

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (r + m, g + m, b + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsl_to_rgb() {
        // Red
        let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
        assert!((r - 1.0).abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!(b.abs() < 0.01);

        // Green
        let (r, g, b) = hsl_to_rgb(120.0, 1.0, 0.5);
        assert!(r.abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!(b.abs() < 0.01);

        // Blue
        let (r, g, b) = hsl_to_rgb(240.0, 1.0, 0.5);
        assert!(r.abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_org_colors_deterministic() {
        // Same org ID should generate same colors
        let colors1 = generate_org_colors(42);
        let colors2 = generate_org_colors(42);
        assert_eq!(colors1, colors2);

        // Different org IDs should generate different colors
        let colors3 = generate_org_colors(43);
        assert_ne!(colors1, colors3);
    }

    #[test]
    fn test_generate_org_colors_alpha() {
        let (border, fill) = generate_org_colors(123);

        // Border should be opaque
        assert_eq!(border.3, 1.0);

        // Fill should be semi-transparent
        assert_eq!(fill.3, 0.3);
    }
}
