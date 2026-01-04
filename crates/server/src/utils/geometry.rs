use bevy::prelude::*;

/// Intersection avec une ligne verticale x = x_line
pub fn intersect_vertical(start: Vec2, end: Vec2, x_line: f32) -> Option<f32> {
    let dx = end.x - start.x;
    if dx.abs() < 0.0001 {
        return None; // Segment vertical, pas d'intersection unique
    }
    let t = (x_line - start.x) / dx;
    if t > 0.0 && t < 1.0 { Some(t) } else { None }
}

/// Intersection avec une ligne horizontale y = y_line
pub fn intersect_horizontal(start: Vec2, end: Vec2, y_line: f32) -> Option<f32> {
    let dy = end.y - start.y;
    if dy.abs() < 0.0001 {
        return None; // Segment horizontal, pas d'intersection unique
    }
    let t = (y_line - start.y) / dy;
    if t > 0.0 && t < 1.0 { Some(t) } else { None }
}
