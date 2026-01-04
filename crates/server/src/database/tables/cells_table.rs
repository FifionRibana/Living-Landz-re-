use bevy::prelude::*;

use shared::{
    BiomeTypeEnum, TerrainChunkId,
    grid::{CellData, GridCell},
};
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct CellsTable {
    pool: PgPool,
}

impl CellsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_cells(&self, cells: &[CellData]) -> Result<(), sqlx::Error> {
        const BATCH_SIZE: usize = 1000;
        let chunks: Vec<_> = cells.chunks(BATCH_SIZE).collect();
        println!("Inserting {} cells in {} chunks", cells.len(), chunks.len());
        // Écrire les données

        let mut tx = self.pool.begin().await?;
        for chunk in chunks {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO terrain.cells (q, r, chunk_x, chunk_y, biome_id)",
            );

            query_builder.push_values(chunk.iter(), |mut b, cell_data| {
                b.push_bind(cell_data.cell.q)
                    .push_bind(cell_data.cell.r)
                    .push_bind(cell_data.chunk.x)
                    .push_bind(cell_data.chunk.y)
                    .push_bind(cell_data.biome.to_id());
            });

            query_builder.push(
                "ON CONFLICT (q, r) DO UPDATE SET chunk_x = EXCLUDED.chunk_x, chunk_y = EXCLUDED.chunk_y, biome_id = EXCLUDED.biome_id"
            );

            query_builder.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn load_chunk_cells(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> Result<Vec<CellData>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT q, r, biome_id FROM terrain.cells WHERE chunk_x = $1 AND chunk_y = $2",
        )
        .bind(chunk_id.x)
        .bind(chunk_id.y)
        .fetch_all(&self.pool)
        .await?;

        let cells = row
            .iter()
            .map(|r| CellData {
                biome: BiomeTypeEnum::from_id(r.get("biome_id"))
                    .unwrap_or(BiomeTypeEnum::Undefined),
                cell: GridCell {
                    q: r.get("q"),
                    r: r.get("r"),
                },
                chunk: *chunk_id,
            })
            .collect::<Vec<_>>();

        Ok(cells)
    }

    /// Get biome type at a specific cell
    pub async fn get_biome_at_cell(&self, cell: &GridCell) -> Result<Option<BiomeTypeEnum>, String> {
        let result = sqlx::query(
            r#"
            SELECT biome_id
            FROM terrain.cells
            WHERE q = $1 AND r = $2
            "#,
        )
        .bind(cell.q)
        .bind(cell.r)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get biome at cell: {}", e))?;

        if let Some(row) = result {
            let biome_id = row.get::<i16, _>("biome_id");
            Ok(BiomeTypeEnum::from_id(biome_id))
        } else {
            Ok(None)
        }
    }
}
