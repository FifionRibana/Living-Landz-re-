use sqlx::{PgPool, Row};
use shared::grid::GridCell;
use shared::BiomeTypeEnum;

/// Database handler for Voronoi zones
pub struct VoronoiZonesTable {
    pool: PgPool,
}

impl VoronoiZonesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new Voronoi zone
    pub async fn create_zone(
        &self,
        seed_cell: GridCell,
        biome: BiomeTypeEnum,
    ) -> Result<i64, String> {
        let zone_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO terrain.voronoi_zones (seed_cell_q, seed_cell_r, biome_type)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(seed_cell.q)
        .bind(seed_cell.r)
        .bind(biome as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create voronoi zone: {}", e))?;

        Ok(zone_id)
    }

    /// Add cells to a zone (bulk insert for performance)
    pub async fn add_cells_to_zone(
        &self,
        zone_id: i64,
        cells: &[GridCell],
    ) -> Result<(), String> {
        if cells.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO terrain.voronoi_zone_cells (zone_id, cell_q, cell_r)"
        );

        query_builder.push_values(cells.iter(), |mut b, cell| {
            b.push_bind(zone_id).push_bind(cell.q).push_bind(cell.r);
        });

        query_builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to add cells to zone: {}", e))?;

        Ok(())
    }

    /// Get the zone ID at a specific cell
    pub async fn get_zone_at_cell(&self, cell: GridCell) -> Result<Option<i64>, String> {
        let zone_id = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT zone_id FROM terrain.voronoi_zone_cells
            WHERE cell_q = $1 AND cell_r = $2
            "#,
        )
        .bind(cell.q)
        .bind(cell.r)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get zone at cell: {}", e))?;

        Ok(zone_id)
    }

    /// Get all cells belonging to a zone
    pub async fn get_zone_cells(&self, zone_id: i64) -> Result<Vec<GridCell>, String> {
        let rows = sqlx::query(
            r#"
            SELECT cell_q, cell_r FROM terrain.voronoi_zone_cells
            WHERE zone_id = $1
            "#,
        )
        .bind(zone_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get zone cells: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| GridCell {
                q: row.get("cell_q"),
                r: row.get("cell_r"),
            })
            .collect())
    }

    /// Check if a zone is available (no cells claimed by another organization)
    pub async fn is_zone_available(&self, zone_id: i64) -> Result<bool, String> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT tc.organization_id)
            FROM terrain.voronoi_zone_cells vzc
            LEFT JOIN organizations.territory_cells tc
                ON vzc.cell_q = tc.cell_q AND vzc.cell_r = tc.cell_r
            WHERE vzc.zone_id = $1
                AND tc.organization_id IS NOT NULL
            "#,
        )
        .bind(zone_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check zone availability: {}", e))?;

        // Zone is available if no organization has claimed any cells
        Ok(count == 0)
    }

    /// Get zone information
    pub async fn get_zone_info(&self, zone_id: i64) -> Result<Option<VoronoiZoneInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, seed_cell_q, seed_cell_r, biome_type, cell_count, area_m2
            FROM terrain.voronoi_zones
            WHERE id = $1
            "#,
        )
        .bind(zone_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get zone info: {}", e))?;

        Ok(row.map(|r| VoronoiZoneInfo {
            id: r.get::<i64, _>("id"),
            seed_cell: GridCell {
                q: r.get("seed_cell_q"),
                r: r.get("seed_cell_r"),
            },
            biome: BiomeTypeEnum::from_id(r.get::<i32, _>("biome_type") as i16)
                .unwrap_or(BiomeTypeEnum::Grassland),
            cell_count: r.get("cell_count"),
            area_m2: r.get("area_m2"),
        }))
    }
}

/// Information about a Voronoi zone
#[derive(Debug, Clone)]
pub struct VoronoiZoneInfo {
    pub id: i64,
    pub seed_cell: GridCell,
    pub biome: BiomeTypeEnum,
    pub cell_count: i32,
    pub area_m2: f32,
}
