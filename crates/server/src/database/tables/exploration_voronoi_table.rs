use sqlx::{PgPool, Row};

/// Database handler for exploration Voronoi — only persists which zones are explored.
/// Seeds are computed deterministically, never stored.
#[derive(Clone)]
pub struct ExplorationVoronoiTable {
    pool: PgPool,
}

impl ExplorationVoronoiTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Load the set of explored zone IDs.
    pub async fn load_explored_set(&self) -> Result<std::collections::HashSet<i64>, sqlx::Error> {
        let rows = sqlx::query("SELECT zone_id FROM terrain.explored_voronoi")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(|row| row.get::<i64, _>("zone_id")).collect())
    }

    /// Mark zones as explored. Returns the newly explored zone IDs.
    pub async fn mark_explored(
        &self,
        zone_ids: &[i64],
        player_id: i64,
    ) -> Result<Vec<i64>, sqlx::Error> {
        if zone_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut newly_explored = Vec::new();

        for batch in zone_ids.chunks(500) {
            let mut qb = sqlx::QueryBuilder::new(
                "INSERT INTO terrain.explored_voronoi (zone_id, explored_by) ",
            );

            qb.push_values(batch.iter(), |mut b, zone_id| {
                b.push_bind(*zone_id).push_bind(player_id);
            });

            qb.push(" ON CONFLICT (zone_id) DO NOTHING RETURNING zone_id");

            let rows = qb.build().fetch_all(&self.pool).await?;
            for row in rows {
                newly_explored.push(row.get::<i64, _>("zone_id"));
            }
        }

        Ok(newly_explored)
    }

    /// Clear all exploration data.
    pub async fn clear_all(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM terrain.explored_voronoi")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}