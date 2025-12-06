use bevy::prelude::*;
use shared::{grid::GridCell, RoadSegmentData};
use sqlx::{PgPool, Row};
use crate::road::RoadSegment;

#[derive(Resource, Clone)]
pub struct RoadSegmentsTable {
    pool: PgPool,
}

impl RoadSegmentsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Sauvegarde un segment de route dans la base de données
    pub async fn save_road_segment(&self, segment: &RoadSegment) -> Result<i64, sqlx::Error> {
        // Encoder les points en bincode
        let points_vec: Vec<[f32; 2]> = segment.points.iter().map(|p| p.to_array()).collect();
        let points_bytes = bincode::encode_to_vec(&points_vec, bincode::config::standard())
            .expect("Failed to encode road points");

        // Encoder le cell_path en bincode
        let cell_path_vec: Vec<(i32, i32)> = segment.cell_path.iter().map(|c| (c.q, c.r)).collect();
        let cell_path_bytes = bincode::encode_to_vec(&cell_path_vec, bincode::config::standard())
            .expect("Failed to encode cell path");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Calculer le chunk principal (basé sur le start_cell)
        let (chunk_x, chunk_y) = Self::cell_to_chunk(&segment.start_cell);

        let result = if segment.id > 0 {
            // Update
            sqlx::query(
                r#"
                UPDATE terrain.road_segments
                SET points = $1, cell_path = $2, importance = $3, road_type_id = $4, updated_at = $5
                WHERE id = $6
                RETURNING id
                "#,
            )
            .bind(&points_bytes)
            .bind(&cell_path_bytes)
            .bind(segment.importance as i16)
            .bind(segment.road_type.id)
            .bind(now)
            .bind(segment.id)
            .fetch_one(&self.pool)
            .await?
        } else {
            // Insert
            sqlx::query(
                r#"
                INSERT INTO terrain.road_segments
                (start_q, start_r, end_q, end_r, points, cell_path, importance, road_type_id, chunk_x, chunk_y, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (start_q, start_r, end_q, end_r)
                DO UPDATE SET points = $5, cell_path = $6, importance = $7, road_type_id = $8, updated_at = $12
                RETURNING id
                "#,
            )
            .bind(segment.start_cell.q)
            .bind(segment.start_cell.r)
            .bind(segment.end_cell.q)
            .bind(segment.end_cell.r)
            .bind(&points_bytes)
            .bind(&cell_path_bytes)
            .bind(segment.importance as i16)
            .bind(segment.road_type.id)
            .bind(chunk_x)
            .bind(chunk_y)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(result.get("id"))
    }

    /// Sauvegarde un segment de route avec des coordonnées de chunk explicites
    /// (au lieu de les calculer à partir de la cellule)
    pub async fn save_road_segment_with_chunk(&self, segment: &RoadSegment, chunk_x: i32, chunk_y: i32) -> Result<i64, sqlx::Error> {
        // Encoder les points en bincode
        let points_vec: Vec<[f32; 2]> = segment.points.iter().map(|p| p.to_array()).collect();
        let points_bytes = bincode::encode_to_vec(&points_vec, bincode::config::standard())
            .expect("Failed to encode road points");

        // Encoder le cell_path en bincode
        let cell_path_vec: Vec<(i32, i32)> = segment.cell_path.iter().map(|c| (c.q, c.r)).collect();
        let cell_path_bytes = bincode::encode_to_vec(&cell_path_vec, bincode::config::standard())
            .expect("Failed to encode cell path");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let result = if segment.id > 0 {
            // Update
            sqlx::query(
                r#"
                UPDATE terrain.road_segments
                SET points = $1, cell_path = $2, importance = $3, road_type_id = $4, updated_at = $5
                WHERE id = $6
                RETURNING id
                "#,
            )
            .bind(&points_bytes)
            .bind(&cell_path_bytes)
            .bind(segment.importance as i16)
            .bind(segment.road_type.id)
            .bind(now)
            .bind(segment.id)
            .fetch_one(&self.pool)
            .await?
        } else {
            // Insert
            sqlx::query(
                r#"
                INSERT INTO terrain.road_segments
                (start_q, start_r, end_q, end_r, points, cell_path, importance, road_type_id, chunk_x, chunk_y, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (start_q, start_r, end_q, end_r)
                DO UPDATE SET points = $5, cell_path = $6, importance = $7, road_type_id = $8, updated_at = $12
                RETURNING id
                "#,
            )
            .bind(segment.start_cell.q)
            .bind(segment.start_cell.r)
            .bind(segment.end_cell.q)
            .bind(segment.end_cell.r)
            .bind(&points_bytes)
            .bind(&cell_path_bytes)
            .bind(segment.importance as i16)
            .bind(segment.road_type.id)
            .bind(chunk_x)
            .bind(chunk_y)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(result.get("id"))
    }

    /// Charge tous les segments de route d'un chunk
    pub async fn load_road_segments_by_chunk(
        &self,
        chunk_x: i32,
        chunk_y: i32,
    ) -> Result<Vec<RoadSegment>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, start_q, start_r, end_q, end_r, points, cell_path, importance, road_type_id
            FROM terrain.road_segments
            WHERE chunk_x = $1 AND chunk_y = $2
            "#,
        )
        .bind(chunk_x)
        .bind(chunk_y)
        .fetch_all(&self.pool)
        .await?;

        let segments: Vec<RoadSegment> = rows
            .iter()
            .filter_map(|row| {
                let id: i64 = row.get("id");
                let start_q: i32 = row.get("start_q");
                let start_r: i32 = row.get("start_r");
                let end_q: i32 = row.get("end_q");
                let end_r: i32 = row.get("end_r");
                let importance: i16 = row.get("importance");
                let road_type_id: i32 = row.get("road_type_id");
                let points_bytes: Vec<u8> = row.get("points");

                // Décoder les points
                let points_vec: Vec<[f32; 2]> =
                    bincode::decode_from_slice(&points_bytes[..], bincode::config::standard())
                        .ok()?
                        .0;

                let start_cell = GridCell { q: start_q, r: start_r };
                let end_cell = GridCell { q: end_q, r: end_r };

                // Décoder le cell_path depuis la DB, ou reconstruire si absent (anciens segments)
                let cell_path = if let Ok(cell_path_bytes) = row.try_get::<Vec<u8>, _>("cell_path") {
                    // Décoder le cell_path depuis la DB
                    let cell_path_tuples: Vec<(i32, i32)> =
                        bincode::decode_from_slice(&cell_path_bytes[..], bincode::config::standard())
                            .ok()?
                            .0;
                    cell_path_tuples.iter().map(|&(q, r)| GridCell { q, r }).collect()
                } else {
                    // Fallback pour les anciens segments sans cell_path
                    if start_cell == end_cell {
                        vec![start_cell]
                    } else {
                        vec![start_cell, end_cell]
                    }
                };

                // Construire le RoadType depuis l'ID (TODO: charger category/variant depuis lookup)
                let road_type = match road_type_id {
                    1 => shared::RoadType::dirt_path(1),
                    2 => shared::RoadType::paved_road(2),
                    3 => shared::RoadType::highway(3),
                    _ => shared::RoadType::default(),
                };

                Some(RoadSegment {
                    id,
                    start_cell,
                    end_cell,
                    cell_path,
                    points: points_vec.iter().map(|&p| Vec2::from(p)).collect(),
                    importance: importance as u8,
                    road_type,
                })
            })
            .collect();

        Ok(segments)
    }

    /// Charge un segment de route par son ID
    pub async fn load_road_segment(&self, id: i64) -> Result<Option<RoadSegment>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, start_q, start_r, end_q, end_r, points, cell_path, importance, road_type_id
            FROM terrain.road_segments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let segment = row.and_then(|r| {
            let id: i64 = r.get("id");
            let start_q: i32 = r.get("start_q");
            let start_r: i32 = r.get("start_r");
            let end_q: i32 = r.get("end_q");
            let end_r: i32 = r.get("end_r");
            let importance: i16 = r.get("importance");
            let road_type_id: i32 = r.get("road_type_id");
            let points_bytes: Vec<u8> = r.get("points");

            let points_vec: Vec<[f32; 2]> =
                bincode::decode_from_slice(&points_bytes[..], bincode::config::standard())
                    .ok()?
                    .0;

            let start_cell = GridCell { q: start_q, r: start_r };
            let end_cell = GridCell { q: end_q, r: end_r };

            // Décoder le cell_path depuis la DB, ou reconstruire si absent (anciens segments)
            let cell_path = if let Ok(cell_path_bytes) = r.try_get::<Vec<u8>, _>("cell_path") {
                // Décoder le cell_path depuis la DB
                let cell_path_tuples: Vec<(i32, i32)> =
                    bincode::decode_from_slice(&cell_path_bytes[..], bincode::config::standard())
                        .ok()?
                        .0;
                cell_path_tuples.iter().map(|&(q, r)| GridCell { q, r }).collect()
            } else {
                // Fallback pour les anciens segments sans cell_path
                if start_cell == end_cell {
                    vec![start_cell]
                } else {
                    vec![start_cell, end_cell]
                }
            };

            // Construire le RoadType depuis l'ID (TODO: charger category/variant depuis lookup)
            let road_type = match road_type_id {
                1 => shared::RoadType::dirt_path(1),
                2 => shared::RoadType::paved_road(2),
                3 => shared::RoadType::highway(3),
                _ => shared::RoadType::default(),
            };

            Some(RoadSegment {
                id,
                start_cell,
                end_cell,
                cell_path,
                points: points_vec.iter().map(|&p| Vec2::from(p)).collect(),
                importance: importance as u8,
                road_type,
            })
        });

        Ok(segment)
    }

    /// Supprime un segment de route
    pub async fn delete_road_segment(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM terrain.road_segments WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Charge tous les segments connectés à une cellule hexagonale
    pub async fn load_connected_segments(
        &self,
        cell: &GridCell,
    ) -> Result<Vec<RoadSegment>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, start_q, start_r, end_q, end_r, points, cell_path, importance, road_type_id
            FROM terrain.road_segments
            WHERE (start_q = $1 AND start_r = $2) OR (end_q = $1 AND end_r = $2)
            "#,
        )
        .bind(cell.q)
        .bind(cell.r)
        .fetch_all(&self.pool)
        .await?;

        let segments: Vec<RoadSegment> = rows
            .iter()
            .filter_map(|row| {
                let id: i64 = row.get("id");
                let start_q: i32 = row.get("start_q");
                let start_r: i32 = row.get("start_r");
                let end_q: i32 = row.get("end_q");
                let end_r: i32 = row.get("end_r");
                let importance: i16 = row.get("importance");
                let road_type_id: i32 = row.get("road_type_id");
                let points_bytes: Vec<u8> = row.get("points");

                let points_vec: Vec<[f32; 2]> =
                    bincode::decode_from_slice(&points_bytes[..], bincode::config::standard())
                        .ok()?
                        .0;

                let start_cell = GridCell { q: start_q, r: start_r };
                let end_cell = GridCell { q: end_q, r: end_r };

                // Décoder le cell_path depuis la DB, ou reconstruire si absent (anciens segments)
                let cell_path = if let Ok(cell_path_bytes) = row.try_get::<Vec<u8>, _>("cell_path") {
                    // Décoder le cell_path depuis la DB
                    let cell_path_tuples: Vec<(i32, i32)> =
                        bincode::decode_from_slice(&cell_path_bytes[..], bincode::config::standard())
                            .ok()?
                            .0;
                    cell_path_tuples.iter().map(|&(q, r)| GridCell { q, r }).collect()
                } else {
                    // Fallback pour les anciens segments sans cell_path
                    if start_cell == end_cell {
                        vec![start_cell]
                    } else {
                        vec![start_cell, end_cell]
                    }
                };

                // Construire le RoadType depuis l'ID (TODO: charger category/variant depuis lookup)
                let road_type = match road_type_id {
                    1 => shared::RoadType::dirt_path(1),
                    2 => shared::RoadType::paved_road(2),
                    3 => shared::RoadType::highway(3),
                    _ => shared::RoadType::default(),
                };

                Some(RoadSegment {
                    id,
                    start_cell,
                    end_cell,
                    cell_path,
                    points: points_vec.iter().map(|&p| Vec2::from(p)).collect(),
                    importance: importance as u8,
                    road_type,
                })
            })
            .collect();

        Ok(segments)
    }

    /// Convertit une cellule hexagonale en coordonnées de chunk
    /// Utilise les constantes du projet pour calculer le chunk
    fn cell_to_chunk(cell: &GridCell) -> (i32, i32) {
        // Convertir les coordonnées axiales en position monde
        use shared::constants::{CHUNK_SIZE, HEX_SIZE, HEX_RATIO};
        use hexx::{Hex, HexLayout};

        // Utiliser le même HexLayout que le terrain pour garantir la cohérence
        let layout = HexLayout::flat()
            .with_hex_size(HEX_SIZE)
            .with_scale(Vec2::new(HEX_RATIO.x * HEX_SIZE, HEX_RATIO.y * HEX_SIZE));

        let hex = Hex::new(cell.q, cell.r);
        let world_pos = layout.hex_to_world_pos(hex);

        // Calculer le chunk
        let chunk_x = (world_pos.x / CHUNK_SIZE.x).floor() as i32;
        let chunk_y = (world_pos.y / CHUNK_SIZE.y).floor() as i32;

        tracing::info!(
            "cell_to_chunk: cell ({},{}) -> world ({:.1},{:.1}) -> chunk ({},{})",
            cell.q, cell.r, world_pos.x, world_pos.y, chunk_x, chunk_y
        );

        (chunk_x, chunk_y)
    }
}
