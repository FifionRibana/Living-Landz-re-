use bevy::prelude::*;
use shared::OceanData;
use sqlx::PgPool;
use sqlx::Row;

#[derive(Resource, Clone)]
pub struct OceanDataTable {
    pool: PgPool,
}

impl OceanDataTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Sauvegarde les données globales de l'océan (SDF + heightmap)
    pub async fn save_ocean_data(&self, ocean_data: OceanData) -> Result<(), sqlx::Error> {
        let expected_size = ocean_data.width * ocean_data.height;
        if ocean_data.sdf_values.len() != expected_size {
            return Err(sqlx::Error::Protocol("SDF size mismatch".to_string()));
        }
        if ocean_data.heightmap_values.len() != expected_size {
            return Err(sqlx::Error::Protocol("Heightmap size mismatch".to_string()));
        }

        sqlx::query(
            r#"
            INSERT INTO terrain.ocean_data (name, width, height, max_distance, sdf_data, heightmap_data, generated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (name)
            DO UPDATE SET
                width = $2,
                height = $3,
                max_distance = $4,
                sdf_data = $5,
                heightmap_data = $6,
                generated_at = $7
            "#,
        )
        .bind(&ocean_data.name)
        .bind(ocean_data.width as i32)
        .bind(ocean_data.height as i32)
        .bind(ocean_data.max_distance)
        .bind(&ocean_data.sdf_values)
        .bind(&ocean_data.heightmap_values)
        .bind(ocean_data.generated_at as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Charge les données globales de l'océan
    pub async fn load_ocean_data(&self, name: &str) -> Result<Option<OceanData>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT width, height, max_distance, sdf_data, heightmap_data, generated_at
             FROM terrain.ocean_data
             WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            OceanData {
                name: name.to_string(),
                width: r.get::<i32, _>("width") as usize,
                height: r.get::<i32, _>("height") as usize,
                max_distance: r.get("max_distance"),
                sdf_values: r.get("sdf_data"),
                heightmap_values: r.get("heightmap_data"),
                generated_at: r.get::<i64, _>("generated_at") as u64,
            }
        }))
    }

    /// Supprime les données d'océan pour un monde
    pub async fn clear_ocean_data(&self, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM terrain.ocean_data WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Vérifie si des données d'océan existent pour un monde
    pub async fn ocean_data_exists(&self, name: &str) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM terrain.ocean_data WHERE name = $1)")
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get(0))
    }
}
