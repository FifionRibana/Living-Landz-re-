use bevy::math::Vec2;

/// Convert screen coordinates (top-left origin, Y-down) to 
/// cell scene world coordinates (center origin, Y-up)
pub fn screen_to_world(screen_x: f32, screen_y: f32, screen_w: f32, screen_h: f32) -> Vec2 {
    Vec2::new(
        screen_x - screen_w / 2.0,
        screen_h / 2.0 - screen_y,
    )
}