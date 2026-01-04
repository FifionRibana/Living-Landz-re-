use sqlx::{PgPool, Row};
use bevy::math::Vec2;

/// Handler for the territory_contours table in the organizations schema
pub struct TerritoryContoursTable {
    pool: PgPool,
}

impl TerritoryContoursTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Store contour points for an organization in a specific chunk
    pub async fn store_contour(
        &self,
        organization_id: u64,
        chunk_x: i32,
        chunk_y: i32,
        contour_points: &[Vec2],
    ) -> Result<(), String> {
        // Flatten Vec2 points into a flat array of f32 for PostgreSQL
        let points_flat: Vec<f32> = contour_points
            .iter()
            .flat_map(|p| vec![p.x, p.y])
            .collect();

        // Calculate bounding box
        let (min_x, max_x, min_y, max_y) = if contour_points.is_empty() {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            for p in contour_points {
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);
                min_y = min_y.min(p.y);
                max_y = max_y.max(p.y);
            }

            (min_x, max_x, min_y, max_y)
        };

        let point_count = contour_points.len() as i32;

        sqlx::query(
            r#"
            INSERT INTO organizations.territory_contours
                (organization_id, chunk_x, chunk_y, contour_points, bbox_min_x, bbox_max_x, bbox_min_y, bbox_max_y, point_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (organization_id, chunk_x, chunk_y)
            DO UPDATE SET
                contour_points = EXCLUDED.contour_points,
                bbox_min_x = EXCLUDED.bbox_min_x,
                bbox_max_x = EXCLUDED.bbox_max_x,
                bbox_min_y = EXCLUDED.bbox_min_y,
                bbox_max_y = EXCLUDED.bbox_max_y,
                point_count = EXCLUDED.point_count,
                updated_at = NOW()
            "#,
        )
        .bind(organization_id as i64)
        .bind(chunk_x)
        .bind(chunk_y)
        .bind(&points_flat)
        .bind(min_x)
        .bind(max_x)
        .bind(min_y)
        .bind(max_y)
        .bind(point_count)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store territory contour: {}", e))?;

        Ok(())
    }

    /// Load all contours for a specific chunk
    /// Returns a Vec of (organization_id, contour_points)
    pub async fn load_chunk_contours(
        &self,
        chunk_x: i32,
        chunk_y: i32,
    ) -> Result<Vec<(u64, Vec<Vec2>)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT organization_id, contour_points
            FROM organizations.territory_contours
            WHERE chunk_x = $1 AND chunk_y = $2
            "#,
        )
        .bind(chunk_x)
        .bind(chunk_y)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load chunk contours: {}", e))?;

        let mut result = Vec::new();

        for row in rows {
            let organization_id: i64 = row.get("organization_id");
            let points_flat: Vec<f32> = row.get("contour_points");

            // Convert flat array back to Vec2 points
            let mut contour_points = Vec::new();
            for i in (0..points_flat.len()).step_by(2) {
                if i + 1 < points_flat.len() {
                    contour_points.push(Vec2::new(points_flat[i], points_flat[i + 1]));
                }
            }

            result.push((organization_id as u64, contour_points));
        }

        Ok(result)
    }

    /// Delete all contours for a specific organization
    pub async fn delete_organization_contours(
        &self,
        organization_id: u64,
    ) -> Result<(), String> {
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
