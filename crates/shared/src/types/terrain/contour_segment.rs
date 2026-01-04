use bevy::prelude::*;

/// Un segment du contour avec sa normale (pointant vers l'intérieur)
#[derive(Clone, Copy, Debug)]
pub struct ContourSegment {
    pub start: Vec2,
    pub end: Vec2,
    pub normal: Vec2, // Normale unitaire pointant vers l'intérieur
}

impl ContourSegment {
    pub fn new(start: Vec2, end: Vec2, interior_side: Vec2) -> Self {
        let dir = (end - start).normalize();
        // Normale perpendiculaire
        let perp = Vec2::new(-dir.y, dir.x);

        // Choisir le sens qui pointe vers l'intérieur
        let midpoint = (start + end) * 0.5;
        let normal = if (midpoint + perp - interior_side).length()
            < (midpoint - perp - interior_side).length()
        {
            perp
        } else {
            -perp
        };

        Self { start, end, normal }
    }

    /// Créer un segment à partir d'un contour ordonné (sens horaire = intérieur à droite)
    pub fn from_contour_points(start: Vec2, end: Vec2) -> Self {
        let dir = (end - start).normalize();
        // Pour un contour sens horaire, l'intérieur est à droite
        let normal = Vec2::new(dir.y, -dir.x);
        Self { start, end, normal }
    }
}