use std::collections::HashMap;

use bevy::prelude::*;
use hexx::HexLayout;
use shared::{ContourSegment, TerrainChunkId, constants, grid::GridCell};
use sqlx::{PgPool, Row};

use crate::utils;

/// Résultat du découpage : segments par chunk
pub type ChunkContours = HashMap<TerrainChunkId, Vec<ContourSegment>>;

/// Découpe un contour en segments par chunk
pub fn split_contour_into_chunks(contour_points: &[Vec2]) -> ChunkContours {
    let mut result: ChunkContours = HashMap::new();

    if contour_points.len() < 2 {
        return result;
    }

    let n = contour_points.len();

    for i in 0..n {
        let start = contour_points[i];
        let end = contour_points[(i + 1) % n];

        // Découper ce segment selon les chunks qu'il traverse
        let segments = clip_segment_to_chunks(start, end);

        for (chunk, clipped_start, clipped_end) in segments {
            let segment = ContourSegment::from_contour_points(clipped_start, clipped_end);
            result.entry(chunk).or_default().push(segment);
        }
    }

    result
}

/// Découpe un segment aux frontières des chunks
/// Retourne une liste de (chunk, start, end) pour chaque portion du segment
fn clip_segment_to_chunks(start: Vec2, end: Vec2) -> Vec<(TerrainChunkId, Vec2, Vec2)> {
    let mut result = Vec::new();

    // Collecter tous les points d'intersection avec les bordures de chunks
    let mut points = vec![(0.0f32, start)];

    let dir = end - start;
    let length = dir.length();

    if length < 0.0001 {
        // Segment dégénéré
        let chunk = TerrainChunkId::from_world_pos(start);
        return vec![(chunk, start, end)];
    }

    // Intersections avec les lignes verticales (bordures X des chunks)
    let min_x = start.x.min(end.x);
    let max_x = start.x.max(end.x);
    let first_chunk_x = (min_x / constants::CHUNK_SIZE.x).floor() as i32;
    let last_chunk_x = (max_x / constants::CHUNK_SIZE.x).floor() as i32;

    for chunk_x in first_chunk_x..=last_chunk_x + 1 {
        let x = chunk_x as f32 * constants::CHUNK_SIZE.x;
        if x > min_x
            && x < max_x
            && let Some(t) = utils::geometry::intersect_vertical(start, end, x)
        {
            let point = start + dir * t;
            points.push((t, point));
        }
    }

    // Intersections avec les lignes horizontales (bordures Y des chunks)
    let min_y = start.y.min(end.y);
    let max_y = start.y.max(end.y);
    let first_chunk_y = (min_y / constants::CHUNK_SIZE.y).floor() as i32;
    let last_chunk_y = (max_y / constants::CHUNK_SIZE.y).floor() as i32;

    for chunk_y in first_chunk_y..=last_chunk_y + 1 {
        let y = chunk_y as f32 * constants::CHUNK_SIZE.y;
        if y > min_y
            && y < max_y
            && let Some(t) = utils::geometry::intersect_horizontal(start, end, y)
        {
            let point = start + dir * t;
            points.push((t, point));
        }
    }

    // Ajouter le point final
    points.push((1.0, end));

    // Trier par t
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Dédupliquer les points très proches
    points.dedup_by(|a, b| (a.0 - b.0).abs() < 0.0001);

    // Créer les segments
    for i in 0..points.len() - 1 {
        let seg_start = points[i].1;
        let seg_end = points[i + 1].1;

        // Déterminer le chunk de ce segment (utiliser le milieu)
        let midpoint = (seg_start + seg_end) * 0.5;
        let chunk = TerrainChunkId::from_world_pos(midpoint);

        result.push((chunk, seg_start, seg_end));
    }

    result
}


pub async fn fix_chunk_assignments(pool: &PgPool, layout: &HexLayout) {
    // buildings_base
    let rows = sqlx::query("SELECT id, cell_q, cell_r, chunk_x, chunk_y FROM buildings.buildings_base")
        .fetch_all(pool).await.unwrap();
    
    for row in &rows {
        let id: i64 = row.get("id");
        let cell = GridCell { q: row.get("cell_q"), r: row.get("cell_r") };
        let correct = cell.to_chunk_id(layout);
        let current_x: i32 = row.get("chunk_x");
        let current_y: i32 = row.get("chunk_y");
        
        if current_x != correct.x || current_y != correct.y {
            sqlx::query("UPDATE buildings.buildings_base SET chunk_x = $1, chunk_y = $2 WHERE id = $3")
                .bind(correct.x).bind(correct.y).bind(id)
                .execute(pool).await.ok();
            tracing::warn!(
                "Fixed building {} chunk ({},{}) → ({},{})",
                id, current_x, current_y, correct.x, correct.y
            );
        }
    }

    // units
    let rows = sqlx::query("SELECT id, current_cell_q, current_cell_r, current_chunk_x, current_chunk_y FROM units.units")
        .fetch_all(pool).await.unwrap();
    
    for row in &rows {
        let id: i64 = row.get("id");
        let cell = GridCell { q: row.get("current_cell_q"), r: row.get("current_cell_r") };
        let correct = cell.to_chunk_id(layout);
        let current_x: i32 = row.get("current_chunk_x");
        let current_y: i32 = row.get("current_chunk_y");
        
        if current_x != correct.x || current_y != correct.y {
            sqlx::query("UPDATE units.units SET current_chunk_x = $1, current_chunk_y = $2 WHERE id = $3")
                .bind(correct.x).bind(correct.y).bind(id)
                .execute(pool).await.ok();
            tracing::warn!(
                "Fixed unit {} chunk ({},{}) → ({},{})",
                id, current_x, current_y, correct.x, correct.y
            );
        }
    }

    // scheduled_actions (actives seulement)
    let rows = sqlx::query("SELECT id, cell_q, cell_r, chunk_x, chunk_y FROM actions.scheduled_actions WHERE status_id < 3")
        .fetch_all(pool).await.unwrap();
    
    for row in &rows {
        let id: i64 = row.get("id");
        let cell = GridCell { q: row.get("cell_q"), r: row.get("cell_r") };
        let correct = cell.to_chunk_id(layout);
        let current_x: i32 = row.get("chunk_x");
        let current_y: i32 = row.get("chunk_y");
        
        if current_x != correct.x || current_y != correct.y {
            sqlx::query("UPDATE actions.scheduled_actions SET chunk_x = $1, chunk_y = $2 WHERE id = $3")
                .bind(correct.x).bind(correct.y).bind(id)
                .execute(pool).await.ok();
            tracing::warn!(
                "Fixed action {} chunk ({},{}) → ({},{})",
                id, current_x, current_y, correct.x, correct.y
            );
        }
    }
}