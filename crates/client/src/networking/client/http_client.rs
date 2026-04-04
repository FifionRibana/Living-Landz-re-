use bevy::prelude::*;
use serde::Deserialize;
use shared::TerrainChunkId;

#[derive(Resource, Clone)]
pub struct HttpBulkClient {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl HttpBulkClient {
    pub fn new() -> Self {
        let base_url = std::env::var("AUTH_HTTP_HOST")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn fetch_terrain_chunks(
        &self,
        terrain_name: &str,
        chunk_ids: &[(i32, i32)],
    ) -> Result<Vec<(TerrainChunkId, Vec<u8>)>, String> {
        let body = serde_json::json!({
            "terrain_name": terrain_name,
            "chunk_ids": chunk_ids,
        });

        let response = self
            .client
            .post(format!("{}/api/terrain/chunks", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP terrain request error: {}", e))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("HTTP terrain response error: {}", e))?;

        parse_terrain_response(&bytes)
    }

    pub async fn fetch_ocean(&self, world_name: &str) -> Result<Vec<u8>, String> {
        let response = self
            .client
            .get(format!("{}/api/world/{}/ocean", self.base_url, world_name))
            .send()
            .await
            .map_err(|e| format!("HTTP ocean request error: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err("No ocean data found".to_string());
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("HTTP ocean response error: {}", e))
    }

    pub async fn fetch_lake(&self, world_name: &str) -> Result<Vec<u8>, String> {
        let response = self
            .client
            .get(format!("{}/api/world/{}/lake", self.base_url, world_name))
            .send()
            .await
            .map_err(|e| format!("HTTP lake request error: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err("No lake data found".to_string());
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("HTTP lake response error: {}", e))
    }

    pub async fn fetch_terrain_global(&self, world_name: &str) -> Result<Vec<u8>, String> {
        let response = self
            .client
            .get(format!(
                "{}/api/world/{}/terrain-global",
                self.base_url, world_name
            ))
            .send()
            .await
            .map_err(|e| format!("HTTP terrain-global request error: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err("No terrain global data found".to_string());
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("HTTP terrain-global response error: {}", e))
    }

    pub async fn fetch_exploration(&self, world_name: &str) -> Result<ExplorationResponse, String> {
        let response = self
            .client
            .get(format!(
                "{}/api/world/{}/exploration",
                self.base_url, world_name
            ))
            .send()
            .await
            .map_err(|e| format!("HTTP exploration request error: {}", e))?;

        let json: ExplorationJsonResponse = response
            .json()
            .await
            .map_err(|e| format!("HTTP exploration response error: {}", e))?;

        // Decode base64 → compressed bytes
        use base64::Engine;
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(&json.data)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        Ok(ExplorationResponse {
            width: json.width,
            height: json.height,
            n_chunk_x: json.n_chunk_x,
            n_chunk_y: json.n_chunk_y,
            compressed_data: compressed,
        })
    }
}

pub struct ExplorationResponse {
    pub width: i32,
    pub height: i32,
    pub n_chunk_x: i32,
    pub n_chunk_y: i32,
    pub compressed_data: Vec<u8>,
}

#[derive(Deserialize)]
struct ExplorationJsonResponse {
    width: i32,
    height: i32,
    n_chunk_x: i32,
    n_chunk_y: i32,
    data: String,
}

/// Public alias for use from streaming.rs async tasks.
pub fn parse_terrain_response_pub(bytes: &[u8]) -> Result<Vec<(TerrainChunkId, Vec<u8>)>, String> {
    parse_terrain_response(bytes)
}

fn parse_terrain_response(bytes: &[u8]) -> Result<Vec<(TerrainChunkId, Vec<u8>)>, String> {
    if bytes.len() < 4 {
        return Err("Response too short".to_string());
    }

    let chunk_count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let mut offset = 4;
    let mut result = Vec::with_capacity(chunk_count);

    for _ in 0..chunk_count {
        if offset + 12 > bytes.len() {
            return Err("Truncated response header".to_string());
        }

        let chunk_x = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let chunk_y = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let data_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + data_len > bytes.len() {
            return Err("Truncated response data".to_string());
        }

        let data = bytes[offset..offset + data_len].to_vec();
        offset += data_len;

        result.push((TerrainChunkId { x: chunk_x, y: chunk_y }, data));
    }

    Ok(result)
}

// ─── Channel resources for async results ────────────────────────────

/// Channel for terrain chunk results from HTTP requests.
#[derive(Resource)]
pub struct HttpTerrainReceiver {
    rx: std::sync::Mutex<std::sync::mpsc::Receiver<Vec<(TerrainChunkId, Vec<u8>)>>>,
}

impl HttpTerrainReceiver {
    pub fn new(rx: std::sync::mpsc::Receiver<Vec<(TerrainChunkId, Vec<u8>)>>) -> Self {
        Self { rx: std::sync::Mutex::new(rx) }
    }

    pub fn try_recv(&self) -> Option<Vec<(TerrainChunkId, Vec<u8>)>> {
        self.rx.lock().ok()?.try_recv().ok()
    }
}

#[derive(Resource, Clone)]
pub struct HttpTerrainSender {
    pub tx: std::sync::mpsc::Sender<Vec<(TerrainChunkId, Vec<u8>)>>,
}

/// Enum of global data results from HTTP requests.
pub enum HttpGlobalResult {
    Ocean(Vec<u8>),
    Lake(Vec<u8>),
    TerrainGlobal(Vec<u8>),
    Exploration(ExplorationResponse),
}

#[derive(Resource)]
pub struct HttpGlobalReceiver {
    rx: std::sync::Mutex<std::sync::mpsc::Receiver<HttpGlobalResult>>,
}

impl HttpGlobalReceiver {
    pub fn new(rx: std::sync::mpsc::Receiver<HttpGlobalResult>) -> Self {
        Self { rx: std::sync::Mutex::new(rx) }
    }

    pub fn try_recv(&self) -> Option<HttpGlobalResult> {
        self.rx.lock().ok()?.try_recv().ok()
    }
}

#[derive(Resource, Clone)]
pub struct HttpGlobalSender {
    pub tx: std::sync::mpsc::Sender<HttpGlobalResult>,
}
