use bevy::math::Vec2;
use shared::{ContourSegment, ContourSegmentData, TerrainChunkId, TerritoryChunkData};
use sqlx::{PgPool, Row};

/// Handler for the territory_contours table in the organizations schema
pub struct TerritoryContoursTable {
    pool: PgPool,
}

impl TerritoryContoursTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Store contour segments for an organization in a specific chunk
    pub async fn store_contour(
        &self,
        organization_id: u64,
        chunk_x: i32,
        chunk_y: i32,
        contour_segments: &[ContourSegment],
    ) -> Result<(), String> {
        // Flatten ContourSegment into a flat array of f32 for PostgreSQL
        // Format: [start.x, start.y, end.x, end.y, normal.x, normal.y, ...]
        let segments_flat: Vec<f32> = contour_segments
            .iter()
            .flat_map(|seg| {
                vec![
                    seg.start.x,
                    seg.start.y,
                    seg.end.x,
                    seg.end.y,
                    seg.normal.x,
                    seg.normal.y,
                ]
            })
            .collect();

        // Calculate bounding box from all segment points
        let (min_x, max_x, min_y, max_y) = if contour_segments.is_empty() {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            for seg in contour_segments {
                min_x = min_x.min(seg.start.x).min(seg.end.x);
                max_x = max_x.max(seg.start.x).max(seg.end.x);
                min_y = min_y.min(seg.start.y).min(seg.end.y);
                max_y = max_y.max(seg.start.y).max(seg.end.y);
            }

            (min_x, max_x, min_y, max_y)
        };

        let segment_count = contour_segments.len() as i32;

        sqlx::query(
            r#"
            INSERT INTO organizations.territory_contours
                (organization_id, chunk_x, chunk_y, contour_segments, bbox_min_x, bbox_max_x, bbox_min_y, bbox_max_y, segment_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (organization_id, chunk_x, chunk_y)
            DO UPDATE SET
                contour_segments = EXCLUDED.contour_segments,
                bbox_min_x = EXCLUDED.bbox_min_x,
                bbox_max_x = EXCLUDED.bbox_max_x,
                bbox_min_y = EXCLUDED.bbox_min_y,
                bbox_max_y = EXCLUDED.bbox_max_y,
                segment_count = EXCLUDED.segment_count,
                updated_at = NOW()
            "#,
        )
        .bind(organization_id as i64)
        .bind(chunk_x)
        .bind(chunk_y)
        .bind(&segments_flat)
        .bind(min_x)
        .bind(max_x)
        .bind(min_y)
        .bind(max_y)
        .bind(segment_count)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store territory contour: {}", e))?;

        Ok(())
    }

    /// Load all contours for a specific chunk as raw segment data
    /// Returns a Vec of (organization_id, segments_flat)
    /// Format: [start.x, start.y, end.x, end.y, normal.x, normal.y, ...]
    pub async fn load_chunk_contours(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> Result<Vec<TerritoryChunkData>, String> {
        let rows = sqlx::query(
            r#"
            SELECT organization_id, contour_segments
            FROM organizations.territory_contours
            WHERE chunk_x = $1 AND chunk_y = $2
            "#,
        )
        .bind(chunk_id.x)
        .bind(chunk_id.y)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load chunk contours: {}", e))?;

        let mut result = Vec::new();

        for row in rows {
            let organization_id: i64 = row.get("organization_id");
            let segments_flat: Vec<f32> = row.get("contour_segments");

            result.push(TerritoryChunkData {
                organization_id: organization_id as u64,
                segments: segments_flat
                    .chunks_exact(6)
                    .map(|chunk| ContourSegmentData {
                        start: [chunk[0], chunk[1]],
                        end: [chunk[2], chunk[3]],
                        normal: [chunk[4], chunk[5]],
                    })
                    .collect(),
            });
        }

        Ok(result)
    }

    /// Delete all contours for a specific organization
    pub async fn delete_organization_contours(&self, organization_id: u64) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM organizations.territory_contours
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete organization contours: {}", e))?;

        Ok(())
    }

    /// Delete contour for a specific chunk and organization
    pub async fn delete_chunk_contour(
        &self,
        organization_id: u64,
        chunk_x: i32,
        chunk_y: i32,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM organizations.territory_contours
            WHERE organization_id = $1 AND chunk_x = $2 AND chunk_y = $3
            "#,
        )
        .bind(organization_id as i64)
        .bind(chunk_x)
        .bind(chunk_y)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete chunk contour: {}", e))?;

        Ok(())
    }

    /// Get all chunks that have contours for a specific organization
    pub async fn get_organization_chunks(
        &self,
        organization_id: u64,
    ) -> Result<Vec<(i32, i32)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT chunk_x, chunk_y
            FROM organizations.territory_contours
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get organization chunks: {}", e))?;

        let result: Vec<(i32, i32)> = rows
            .iter()
            .map(|row| {
                let chunk_x: i32 = row.get("chunk_x");
                let chunk_y: i32 = row.get("chunk_y");
                (chunk_x, chunk_y)
            })
            .collect();

        Ok(result)
    }
}
