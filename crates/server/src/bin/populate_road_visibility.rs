// Utilitaire pour peupler la table road_chunk_visibility
// pour les routes existantes

use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;

    // Utiliser les structures de server
    use server::database::tables::road_segments_table::RoadSegmentsTable;

    let road_table = RoadSegmentsTable::new(pool.clone());

    println!("Loading all road segments...");

    // Charger tous les segments existants
    let all_segments = sqlx::query(
        r#"
        SELECT id, start_q, start_r, end_q, end_r, points, cell_path, importance, road_type_id
        FROM terrain.road_segments
        "#,
    )
    .fetch_all(&pool)
    .await?;

    println!("Found {} road segments", all_segments.len());

    if all_segments.is_empty() {
        println!("No segments to process!");
        return Ok(());
    }

    // Pour chaque segment, calculer et insérer les entrées de visibilité
    for row in all_segments {
        use sqlx::Row;
        use server::road::RoadSegment;
        use shared::grid::GridCell;
        use bevy::math::Vec2;

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
                .map_err(|e| format!("Failed to decode points for segment {}: {}", id, e))?
                .0;

        let start_cell = GridCell { q: start_q, r: start_r };
        let end_cell = GridCell { q: end_q, r: end_r };

        // Décoder le cell_path
        let cell_path = if let Ok(cell_path_bytes) = row.try_get::<Vec<u8>, _>("cell_path") {
            let cell_path_tuples: Vec<(i32, i32)> =
                bincode::decode_from_slice(&cell_path_bytes[..], bincode::config::standard())
                    .map_err(|e| format!("Failed to decode cell_path for segment {}: {}", id, e))?
                    .0;
            cell_path_tuples.iter().map(|&(q, r)| GridCell { q, r }).collect()
        } else {
            // Fallback
            if start_cell == end_cell {
                vec![start_cell]
            } else {
                vec![start_cell, end_cell]
            }
        };

        // Construire le RoadType depuis l'ID
        let road_type = match road_type_id {
            1 => shared::RoadType::dirt_path(1),
            2 => shared::RoadType::paved_road(2),
            3 => shared::RoadType::highway(3),
            _ => shared::RoadType::default(),
        };

        let segment = RoadSegment {
            id,
            start_cell,
            end_cell,
            cell_path,
            points: points_vec.iter().map(|&p| Vec2::from(p)).collect(),
            importance: importance as u8,
            road_type,
        };

        println!("Processing segment {} (cells: {})", id, segment.cell_path.len());

        // Mettre à jour la visibilité
        road_table.update_chunk_visibility(&segment).await?;
    }

    println!("✓ Done! All road segments now have visibility entries.");

    Ok(())
}
