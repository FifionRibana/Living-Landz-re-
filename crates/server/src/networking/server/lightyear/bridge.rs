use shared::{TerrainChunkId, grid::GridCell};

/// Events sent from tokio (tungstenite handlers, ActionProcessor) → Bevy ECS
pub enum BridgeEvent {
    /// Player logged in — spawn their lord as a replicated entity
    SpawnLord {
        player_id: u64,
        chunk: TerrainChunkId,
        cell: GridCell,
    },
    /// MoveUnit action completed — update lord position in ECS
    UpdateLordPosition {
        player_id: u64,
        to_chunk: TerrainChunkId,
        to_cell: GridCell,
    },
    /// Player disconnected — despawn their lord entity
    DespawnLord {
        player_id: u64,
    },
}

/// Bevy Resource — lives in the ECS, polled by Bevy systems
#[derive(bevy::prelude::Resource)]
pub struct LightyearBridge {
    rx: std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<BridgeEvent>>,
}

impl LightyearBridge {
    /// Non-blocking drain of all pending events. Safe to call from Bevy systems.
    pub fn drain(&self) -> Vec<BridgeEvent> {
        let Ok(mut rx) = self.rx.try_lock() else {
            return vec![];
        };
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        events
    }
}

/// Sender half — cloned into tungstenite handlers and ActionProcessor.
/// Cheap to clone (Arc internally).
#[derive(Clone)]
pub struct BridgeSender {
    tx: tokio::sync::mpsc::UnboundedSender<BridgeEvent>,
}

impl BridgeSender {
    pub fn send(&self, event: BridgeEvent) {
        if self.tx.send(event).is_err() {
            tracing::warn!("LightyearBridge receiver dropped — event lost");
        }
    }
}

/// Create the bridge pair. Call once at startup.
pub fn create_bridge() -> (LightyearBridge, BridgeSender) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (
        LightyearBridge {
            rx: std::sync::Mutex::new(rx),
        },
        BridgeSender { tx },
    )
}