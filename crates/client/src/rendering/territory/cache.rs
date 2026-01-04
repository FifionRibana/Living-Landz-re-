use bevy::prelude::*;
use std::collections::HashMap;

/// Resource to cache territory contours received from the server
#[derive(Resource, Default)]
pub struct TerritoryContourCache {
    /// Map of (chunk_x, chunk_y) -> list of organization contours in that chunk
    pub contours: HashMap<(i32, i32), Vec<OrganizationContour>>,
}

/// Contour data for a single organization in a specific chunk
#[derive(Debug, Clone)]
pub struct OrganizationContour {
    pub organization_id: u64,
    pub points: Vec<Vec2>,
    pub border_color: Color,
    pub fill_color: Color,
}

impl TerritoryContourCache {
    /// Add a contour to the cache
    pub fn add_contour(
        &mut self,
        chunk_id: shared::TerrainChunkId,
        organization_id: u64,
        contour_points: Vec<(f32, f32)>,
        border_color: (f32, f32, f32, f32),
        fill_color: (f32, f32, f32, f32),
    ) {
        let chunk_key = (chunk_id.x, chunk_id.y);

        // Convert points to Vec2
        let points: Vec<Vec2> = contour_points
            .into_iter()
            .map(|(x, y)| Vec2::new(x, y))
            .collect();

        // Convert colors
        let border_color = Color::linear_rgba(
            border_color.0,
            border_color.1,
            border_color.2,
            border_color.3,
        );

        let fill_color = Color::linear_rgba(
            fill_color.0,
            fill_color.1,
            fill_color.2,
            fill_color.3,
        );

        let contour = OrganizationContour {
            organization_id,
            points,
            border_color,
            fill_color,
        };

        self.contours
            .entry(chunk_key)
            .or_insert_with(Vec::new)
            .push(contour);
    }

    /// Get all contours for a specific chunk
    pub fn get_chunk_contours(&self, chunk_x: i32, chunk_y: i32) -> Option<&Vec<OrganizationContour>> {
        self.contours.get(&(chunk_x, chunk_y))
    }

    /// Clear all cached contours
    pub fn clear(&mut self) {
        self.contours.clear();
    }

    /// Remove contours for a specific chunk
    pub fn remove_chunk(&mut self, chunk_x: i32, chunk_y: i32) {
        self.contours.remove(&(chunk_x, chunk_y));
    }
}
