use bevy::prelude::*;

use shared::grid::CellData;
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
    
    pub async fn save_cells(
        &self,
        cells: &[CellData],
    ) -> Result<(), sqlx::Error> {

        let mut tx = self.pool.begin().await?;

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO cells (q, r, chunk_x, chunk_y, biome) "
        );

        query_builder.push_values(cells.iter(), |mut b, cell_data| {
            b.push_bind(cell_data.cell.q)
            .push_bind(cell_data.cell.r)
            .push_bind(cell_data.chunk.x)
            .push_bind(cell_data.chunk.y)
            .push_bind(cell_data.biome);
        });

        query_builder.build().execute(&mut *tx).await?;
        
        tx.commit().await?;
        Ok(())
    }
}
