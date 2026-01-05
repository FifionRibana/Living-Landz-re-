use bevy::prelude::*;
use shared::{ContourSegment, TerrainChunkId};
use std::collections::HashMap;
/// Resource to cache territory contours received from the server
#[derive(Resource, Default)]
pub struct TerritoryContourCache {
    /// Map of (chunk_x, chunk_y) -> list of organization contours in that chunk
    pub contours: HashMap<TerrainChunkId, Vec<OrganizationContour>>,
}

/// Contour data for a single organization in a specific chunk
#[derive(Debug, Clone)]
pub struct OrganizationContour {
    pub organization_id: u64,
    /// Raw segment data: [start.x, start.y, end.x, end.y, normal.x, normal.y, ...]
    pub segments: Vec<ContourSegment>,
    pub border_color: Color,
    pub fill_color: Color,
}

impl TerritoryContourCache {
    /// Add a contour to the cache
    pub fn add_contour(
        &mut self,
        chunk_id: TerrainChunkId,
        organization_id: u64,
        segments: Vec<ContourSegment>,
        border_color: Color,
        fill_color: Color,
    ) {
        // Convert colors
        let contour = OrganizationContour {
            organization_id,
            segments,
            border_color,
            fill_color,
        };

        self.contours
            .entry(chunk_id)
            .or_insert_with(Vec::new)
            .push(contour);
    }

    /// Get all contours for a specific chunk
    pub fn get_chunk_contours(&self, chunk_id: &TerrainChunkId) -> Option<&Vec<OrganizationContour>> {
        self.contours.get(chunk_id)
    }

    /// Clear all cached contours
    pub fn clear(&mut self) {
        self.contours.clear();
    }

    /// Remove contours for a specific chunk
    pub fn remove_chunk(&mut self, chunk_id: &TerrainChunkId) {
        self.contours.remove(&chunk_id);
    }
}
