use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use shared::TerrainChunkId;
use shared::grid::GridConfig;
use shared::GameState;

use crate::database::client::DatabaseTables;
use crate::world::resources::WorldGlobalState;

// ─── State ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BulkState {
    pub db_tables: Arc<DatabaseTables>,
    pub world_global_state: Arc<WorldGlobalState>,
    pub game_state: Arc<GameState>,
    pub grid_config: Arc<GridConfig>,
}

// ─── Request / Response types ───────────────────────────────────────

#[derive(Deserialize)]
pub struct TerrainChunksRequest {
    pub terrain_name: String,
    pub chunk_ids: Vec<[i32; 2]>,
}

#[derive(Deserialize)]
pub struct ExplorationQuery {
    #[allow(dead_code)]
    pub player_id: Option<u64>,
}

#[derive(serde::Serialize)]
pub struct ExplorationResponse {
    pub width: i32,
    pub height: i32,
    pub n_chunk_x: i32,
    pub n_chunk_y: i32,
    pub data: String, // base64 encoded LZ4 compressed
}

// ─── POST /api/terrain/chunks ───────────────────────────────────────

pub async fn terrain_chunks(
    State(state): State<BulkState>,
    Json(body): Json<TerrainChunksRequest>,
) -> impl IntoResponse {
    let chunk_ids: Vec<TerrainChunkId> = body
        .chunk_ids
        .iter()
        .map(|[x, y]| TerrainChunkId { x: *x, y: *y })
        .collect();

    let terrain_name = &body.terrain_name;
    let db_tables = &state.db_tables;
    let world_global_state = &state.world_global_state;
    let game_state = &state.game_state;

    let batch_start = std::time::Instant::now();
    let total = chunk_ids.len();

    // Phase 1: Try loading all chunks from DB in parallel
    let mut load_futures = Vec::with_capacity(total);
    for chunk_id in &chunk_ids {
        let db = db_tables.clone();
        let name = terrain_name.clone();
        let cid = *chunk_id;
        load_futures.push(async move {
            let terrain_result = db.terrains.load_terrain(&name, &cid).await;
            let cell_data = db.cells.load_chunk_cells(&cid).await.unwrap_or_default();
            let building_data = db.buildings.load_chunk_buildings(&cid).await.unwrap_or_default();
            let unit_data = db.units.load_chunk_units(cid).await.unwrap_or_default();
            (cid, terrain_result, cell_data, building_data, unit_data)
        });
    }

    let loaded = futures::future::join_all(load_futures).await;

    // Separate cached vs to-generate
    let mut chunks_data: Vec<(TerrainChunkId, Vec<u8>)> = Vec::with_capacity(total);
    let mut to_generate: Vec<TerrainChunkId> = Vec::new();

    for (cid, terrain_result, cell_data, building_data, unit_data) in loaded {
        match terrain_result {
            Ok((Some(terrain_data), Some(biome_data))) => {
                let compressed = encode_terrain_chunk(&terrain_data, &biome_data, &cell_data, &building_data, &unit_data);
                chunks_data.push((cid, compressed));
            }
            Ok((Some(terrain_data), None)) => {
                let compressed = encode_terrain_chunk(&terrain_data, &[], &cell_data, &building_data, &unit_data);
                chunks_data.push((cid, compressed));
            }
            Ok((None, _)) => {
                to_generate.push(cid);
            }
            Err(e) => {
                tracing::error!("DB error for chunk ({},{}): {}", cid.x, cid.y, e);
            }
        }
    }

    // Phase 2: Generate missing chunks
    for chunk_id in &to_generate {
        let (terrain_data, cell_data, building_data) =
            crate::world::systems::generate_chunk_data(chunk_id, world_global_state, db_tables, game_state).await;
        let unit_data = db_tables.units.load_chunk_units(*chunk_id).await.unwrap_or_default();
        let compressed = encode_terrain_chunk(&terrain_data, &[], &cell_data, &building_data, &unit_data);
        chunks_data.push((*chunk_id, compressed));
    }

    let cached = total - to_generate.len();
    tracing::info!(
        "📦 HTTP terrain batch: {} chunks ({} cached, {} generated) in {:.0}ms",
        total, cached, to_generate.len(),
        batch_start.elapsed().as_secs_f64() * 1000.0
    );

    // Build binary response
    let mut response_bytes: Vec<u8> = Vec::new();
    let chunk_count = chunks_data.len() as u32;
    response_bytes.extend_from_slice(&chunk_count.to_le_bytes());

    for (cid, data) in &chunks_data {
        response_bytes.extend_from_slice(&cid.x.to_le_bytes());
        response_bytes.extend_from_slice(&cid.y.to_le_bytes());
        let data_len = data.len() as u32;
        response_bytes.extend_from_slice(&data_len.to_le_bytes());
        response_bytes.extend_from_slice(data);
    }

    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/octet-stream".parse().unwrap());
    (StatusCode::OK, headers, response_bytes)
}

fn encode_terrain_chunk(
    terrain_data: &shared::TerrainChunkData,
    biome_data: &[shared::BiomeChunkData],
    cell_data: &[shared::grid::CellData],
    building_data: &[shared::BuildingData],
    unit_data: &[shared::UnitData],
) -> Vec<u8> {
    let payload = (terrain_data, biome_data, cell_data, building_data, unit_data);
    let raw = bincode::encode_to_vec(&payload, bincode::config::standard())
        .expect("Failed to encode terrain chunk payload");
    shared::protocol::bulk_compress::compress(&raw)
}

// ─── GET /api/world/:name/ocean ─────────────────────────────────────

pub async fn ocean_data(
    State(state): State<BulkState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.db_tables.ocean_data.load_ocean_data(&name).await {
        Ok(Some(ocean_data)) => {
            let raw = bincode::encode_to_vec(&ocean_data, bincode::config::standard())
                .expect("Failed to encode ocean data");
            let compressed = shared::protocol::bulk_compress::compress(&raw);
            tracing::info!("📦 HTTP ocean data: {} → {} bytes", raw.len(), compressed.len());

            let mut headers = HeaderMap::new();
            headers.insert("content-type", "application/octet-stream".parse().unwrap());
            (StatusCode::OK, headers, compressed).into_response()
        }
        Ok(None) => {
            tracing::warn!("No ocean data for {}", name);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to load ocean data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ─── GET /api/world/:name/lake ──────────────────────────────────────

pub async fn lake_data(
    State(state): State<BulkState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.db_tables.lake_data.load_lake_data(&name).await {
        Ok(Some(lake_data)) => {
            let raw = bincode::encode_to_vec(&lake_data, bincode::config::standard())
                .expect("Failed to encode lake data");
            let compressed = shared::protocol::bulk_compress::compress(&raw);
            tracing::info!("📦 HTTP lake data: {} → {} bytes", raw.len(), compressed.len());

            let mut headers = HeaderMap::new();
            headers.insert("content-type", "application/octet-stream".parse().unwrap());
            (StatusCode::OK, headers, compressed).into_response()
        }
        Ok(None) => {
            tracing::warn!("No lake data for {}", name);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to load lake data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ─── GET /api/world/:name/terrain-global ────────────────────────────

pub async fn terrain_global_data(
    State(state): State<BulkState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.db_tables.terrain_global_data.load_terrain_global_data(&name).await {
        Ok(Some(data)) => {
            let raw = bincode::encode_to_vec(&data, bincode::config::standard())
                .expect("Failed to encode terrain global data");
            let compressed = shared::protocol::bulk_compress::compress(&raw);
            tracing::info!("📦 HTTP terrain global: {} → {} bytes", raw.len(), compressed.len());

            let mut headers = HeaderMap::new();
            headers.insert("content-type", "application/octet-stream".parse().unwrap());
            (StatusCode::OK, headers, compressed).into_response()
        }
        Ok(None) => {
            tracing::warn!("No terrain global data for {}", name);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to load terrain global data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ─── GET /api/world/:name/exploration ───────────────────────────────

pub async fn exploration_map(
    State(state): State<BulkState>,
    Path(name): Path<String>,
    Query(_query): Query<ExplorationQuery>,
) -> impl IntoResponse {
    let _ = name; // world name not used for exploration query yet

    let n_chunk_x = state.world_global_state.n_chunk_x;
    let n_chunk_y = state.world_global_state.n_chunk_y;
    let chunk_w = shared::constants::CHUNK_SIZE.x;
    let chunk_h = shared::constants::CHUNK_SIZE.y;
    let world_seed = crate::exploration::EXPLORATION_WORLD_SEED;
    let seeds = crate::exploration::compute_all_seeds(
        n_chunk_x, n_chunk_y, &state.grid_config.layout, world_seed,
    );

    match state.db_tables.exploration_voronoi.load_explored_set().await {
        Ok(explored) => {
            let (width, height, data) = crate::exploration::rasterize_exploration(
                &seeds, &explored, n_chunk_x, n_chunk_y, chunk_w, chunk_h,
            );
            let compressed = shared::protocol::bulk_compress::compress(&data);
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &compressed,
            );
            tracing::info!(
                "📦 HTTP exploration map {}×{}: {} → {} bytes",
                width, height, data.len(), compressed.len()
            );

            Json(ExplorationResponse {
                width,
                height,
                n_chunk_x,
                n_chunk_y,
                data: encoded,
            }).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to load exploration data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
