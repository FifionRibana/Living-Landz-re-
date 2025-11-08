use bevy::prelude::*;

use shared::{
    TerrainChunkId,
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

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cells (
                q INT NOT NULL,
                r INT NOT NULL,
                
                biome biome_type NOT NULL,
                terrain_type VARCHAR,
                
                building_id BIGINT REFERENCES buildings(id) ON DELETE SET NULL,
                
                chunk_x INT NOT NULL,
                chunk_y INT NOT NULL,
                
                UNIQUE(q, r),
                UNIQUE(chunk_x, chunk_y, q, r),
                PRIMARY KEY (q, r)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_cells_chunk ON cells(chunk_x, chunk_y)")
            .execute(&self.pool)
            .await?;

        tracing::info!("✓ Cells Database schema ready");
        Ok(())
    }

    pub async fn save_cells(&self, cells: &[CellData]) -> Result<(), sqlx::Error> {
        const BATCH_SIZE: usize = 1000;
        let chunks: Vec<_> = cells.chunks(BATCH_SIZE).collect();
        println!("Inserting {} cells in {} chunks", cells.len(), chunks.len());
        // Écrire les données

        let mut tx = self.pool.begin().await?;
        for chunk in chunks {
            let mut query_builder =
                sqlx::QueryBuilder::new("INSERT INTO cells (q, r, chunk_x, chunk_y, biome)");

            query_builder.push_values(chunk.iter(), |mut b, cell_data| {
                b.push_bind(cell_data.cell.q)
                    .push_bind(cell_data.cell.r)
                    .push_bind(cell_data.chunk.x)
                    .push_bind(cell_data.chunk.y)
                    .push_bind(cell_data.biome);
            });

            query_builder.push(
                "ON CONFLICT (q, r) DO UPDATE SET chunk_x = EXCLUDED.chunk_x, chunk_y = EXCLUDED.chunk_y, biome = EXCLUDED.biome"
            );

            query_builder.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn load_chunk_cells(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> Result<(Vec<CellData>), sqlx::Error> {
        let row = sqlx::query("SELECT q, r, biome FROM cells WHERE chunk_x = $1 AND chunk_y = $2")
            .bind(chunk_id.x)
            .bind(chunk_id.y)
            .fetch_all(&self.pool)
            .await?;

        let cells = row
            .iter()
            .map(|r| CellData {
                biome: r.get("biome"),
                cell: GridCell {
                    q: r.get("q"),
                    r: r.get("r"),
                },
                chunk: chunk_id.clone(),
            })
            .collect::<Vec<_>>();

        Ok(cells)
    }
}
