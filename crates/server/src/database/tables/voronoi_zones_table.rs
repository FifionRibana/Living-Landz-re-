use shared::grid::GridCell;
use shared::BiomeTypeEnum;
use sqlx::{PgPool, Row};

/// Database handler for Voronoi zones.
/// Only stores seeds — zone cell membership is computed on demand via shared::voronoi.
pub struct VoronoiZonesTable {
    pool: PgPool,
}

impl VoronoiZonesTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new Voronoi zone (seed only — no cell assignments stored).
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

    /// Load all seeds from DB.
    pub async fn load_all_seeds(&self) -> Result<Vec<(GridCell, i64)>, String> {
        let rows = sqlx::query("SELECT id, seed_cell_q, seed_cell_r FROM terrain.voronoi_zones")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to load voronoi seeds: {}", e))?;

        Ok(rows
            .iter()
            .map(|r| {
                (
                    GridCell {
                        q: r.get::<i32, _>("seed_cell_q"),
                        r: r.get::<i32, _>("seed_cell_r"),
                    },
                    r.get::<i64, _>("id"),
                )
            })
            .collect())
    }

    /// Get the zone ID at a specific cell (computed from seeds, not stored).
    pub async fn get_zone_at_cell(&self, cell: GridCell) -> Result<Option<i64>, String> {
        let seeds = self.load_all_seeds().await?;
        Ok(shared::voronoi::find_closest_seed(cell, &seeds).map(|(id, _)| id))
    }

    /// Compute all cells belonging to a zone via BFS flood-fill.
    pub async fn compute_zone_cells(&self, zone_id: i64) -> Result<Vec<GridCell>, String> {
        let seeds = self.load_all_seeds().await?;
        let seed_cell = seeds
            .iter()
            .find(|(_, id)| *id == zone_id)
            .map(|(cell, _)| *cell)
            .ok_or_else(|| format!("Zone {} not found", zone_id))?;

        Ok(shared::voronoi::compute_zone_cells(zone_id, seed_cell, &seeds))
    }

    /// Check if a zone is available (no cells claimed by any organization).
    pub async fn is_zone_available(&self, zone_id: i64) -> Result<bool, String> {
        let zone_cells = self.compute_zone_cells(zone_id).await?;

        if zone_cells.is_empty() {
            return Ok(false);
        }

        // Build a single query to check all cells at once
        let qs: Vec<i32> = zone_cells.iter().map(|c| c.q).collect();
        let rs: Vec<i32> = zone_cells.iter().map(|c| c.r).collect();

        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM organizations.territory_cells tc
            WHERE EXISTS (
                SELECT 1 FROM unnest($1::int[], $2::int[]) AS t(q, r)
                WHERE tc.cell_q = t.q AND tc.cell_r = t.r
            )
            "#,
        )
        .bind(&qs)
        .bind(&rs)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(count == 0)
    }

    /// Get zone information (from voronoi_zones table only).
    pub async fn get_zone_info(&self, zone_id: i64) -> Result<Option<VoronoiZoneInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, seed_cell_q, seed_cell_r, biome_type
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
        }))
    }
}

/// Information about a Voronoi zone
#[derive(Debug, Clone)]
pub struct VoronoiZoneInfo {
    pub id: i64,
    pub seed_cell: GridCell,
    pub biome: BiomeTypeEnum,
}
