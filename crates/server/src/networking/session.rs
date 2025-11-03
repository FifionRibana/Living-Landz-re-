use bevy::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Resource, Clone)]
pub struct Sessions {
    inner: Arc<RwLock<HashMap<u64, SocketAddr>>>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Sessions {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn inner(&self) -> Arc<RwLock<HashMap<u64, SocketAddr>>> {
        self.inner.clone()
    }

    pub async fn insert(&self, player_id: u64, addr: SocketAddr) {
        self.inner.write().await.insert(player_id, addr);
    }

    pub async fn remove(&self, player_id: &u64) {
        self.inner.write().await.remove(player_id);
    }

    pub async fn count(&self) -> usize {
        self.inner.read().await.len()
    }
}
