use shared::grid::GridCell;
use shared::{
    ActionStatusEnum, ActionTypeEnum, BuildingTypeEnum, ProfessionEnum, ResourceSpecificTypeEnum,
    TerrainChunkId,
};
use shared::protocol::InventoryItemData;

// ─── tokio → Bevy events (existing + new) ───────────────────────────

pub enum BridgeEvent {
    /// Player logged in — spawn their lord as a replicated entity.
    SpawnLord {
        player_id: u64,
        chunk: TerrainChunkId,
        cell: GridCell,
    },
    /// ActionMoveUnit completed — update the lord's position in ECS.
    UpdateLordPosition {
        player_id: u64,
        to_chunk: TerrainChunkId,
        to_cell: GridCell,
    },
    /// Player disconnected — despawn their lord entity.
    DespawnLord { player_id: u64 },

    // ── NEW: Action responses to forward to client via lightyear ──

    /// Send an ActionStatusMsg to a specific player via lightyear.
    SendActionStatus {
        player_id: u64,
        action_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        status: ActionStatusEnum,
        action_type: ActionTypeEnum,
        completion_time: u64,
        action_name: Option<String>,
        unit_ids: Vec<u64>,
    },
    /// Send an ActionErrorMsg to a specific player via lightyear.
    SendActionError { player_id: u64, reason: String },

    /// A non-lord unit started moving — spawn a temporary replicated entity.
    SpawnMovingUnit {
        player_id: u64,
        unit_id: u64,
        chunk: TerrainChunkId,
        cell: GridCell,
    },
    /// A non-lord unit finished moving — despawn the temporary entity.
    DespawnMovingUnit {
        unit_id: u64,
    },

    /// Send a UnitPositionUpdatedMsg to a specific player via lightyear.
    SendUnitPositionUpdated {
        player_id: u64,
        unit_id: u64,
        from_cell: GridCell,
        from_chunk: TerrainChunkId,
        to_cell: GridCell,
        to_chunk: TerrainChunkId,
    },

    /// Broadcast an ActionCompletedMsg to all clients via lightyear.
    BroadcastActionCompleted {
        action_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        action_type: ActionTypeEnum,
    },

    /// All login data loaded from DB — send to client via lightyear Messages.
    SendLoginData {
        player_id: u64,
        player: shared::protocol::PlayerData,
        character: Option<shared::protocol::CharacterData>,
        lord: Option<shared::UnitData>,
        organization: Option<shared::OrganizationSummary>,
        game_data: shared::protocol::GameDataPayload,
    },

    // ── Bulk data responses (#137 Step 1) ──

    SendTerrainChunk {
        player_id: u64,
        chunk_id: TerrainChunkId,
        compressed_data: Vec<u8>,
    },
    SendOceanData {
        player_id: u64,
        compressed_data: Vec<u8>,
    },
    SendLakeData {
        player_id: u64,
        compressed_data: Vec<u8>,
    },
    SendTerrainGlobalData {
        player_id: u64,
        compressed_data: Vec<u8>,
    },
    SendExplorationMap {
        player_id: u64,
        width: i32,
        height: i32,
        n_chunk_x: i32,
        n_chunk_y: i32,
        compressed_data: Vec<u8>,
    },
    SendExplorationPatch {
        player_id: u64,
        patch_x: i32,
        patch_y: i32,
        patch_width: i32,
        patch_height: i32,
        compressed_data: Vec<u8>,
    },

    // ── Step 2 events: remaining server→client messages ──

    SendInventoryData {
        player_id: u64,
        unit_id: u64,
        items: Vec<InventoryItemData>,
    },
    SendInventoryUpdate {
        player_id: u64,
        unit_id: u64,
        item_id: i32,
        quantity_delta: i32,
        new_total: i32,
    },
    SendUnitProfessionChanged {
        player_id: u64,
        unit_id: u64,
        new_profession: ProfessionEnum,
        new_avatar_url: Option<String>,
    },
    SendUnitWorkStatusUpdate {
        player_id: u64,
        unit_id: u64,
        working_on_action_id: Option<u64>,
    },
    BroadcastRoadChunkSdfUpdate {
        terrain_name: String,
        chunk_id: TerrainChunkId,
        road_sdf_data: shared::RoadChunkSdfData,
    },
    BroadcastTerritoryContourUpdate {
        chunk_id: TerrainChunkId,
        contours: Vec<shared::protocol::TerritoryContourChunkData>,
    },
    BroadcastTerritoryBorderSdfUpdate {
        chunk_id: TerrainChunkId,
        border_sdf_data_list: Vec<shared::TerritoryBorderChunkSdfData>,
    },
    SendTerritoryBorderCells {
        player_id: u64,
        organization_id: u64,
        border_cells: Vec<GridCell>,
    },
    SendPopulationChanged {
        player_id: u64,
        organization_id: u64,
        new_population: i32,
        immigrant: Option<shared::UnitData>,
    },
    SendHamletFounded {
        player_id: u64,
        organization_id: u64,
        name: String,
        headquarters: GridCell,
        territory_cells: Vec<GridCell>,
    },
    SendHamletFoundError {
        player_id: u64,
        reason: String,
    },
    SendOrganizationAtCell {
        player_id: u64,
        cell: GridCell,
        organization: Option<shared::OrganizationSummary>,
    },
    SendUnitSlotUpdated {
        player_id: u64,
        unit_id: u64,
        cell: GridCell,
        slot_position: Option<shared::SlotPosition>,
    },
    SendDebugOrganizationCreated {
        player_id: u64,
        organization_id: u64,
        name: String,
    },
    SendDebugOrganizationDeleted {
        player_id: u64,
        organization_id: u64,
    },
    SendDebugUnitSpawned {
        player_id: u64,
        unit_data: shared::UnitData,
    },
    SendDebugError {
        player_id: u64,
        reason: String,
    },
}

// ─── Bevy → tokio: action requests ─────────────────────────────────

/// An action RPC from a lightyear client, forwarded to tokio for DB processing.
pub enum ActionRequest {
    MoveUnit {
        player_id: u64,
        unit_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
    },
    BuildBuilding {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        building_type: BuildingTypeEnum,
    },
    BuildRoad {
        player_id: u64,
        start_cell: GridCell,
        end_cell: GridCell,
    },
    HarvestResource {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        resource_specific_type: ResourceSpecificTypeEnum,
        unit_ids: Vec<u64>,
    },
    CraftResource {
        player_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        recipe_id: String,
        quantity: u32,
        unit_ids: Vec<u64>,
    },
    TrainUnit {
        player_id: u64,
        unit_id: u64,
        chunk_id: TerrainChunkId,
        cell: GridCell,
        target_profession: ProfessionEnum,
    },
    Explore {
        player_id: u64,
        cell: GridCell,
        radius: i32,
    },
    /// Load all login data for a newly connected player.
    LoadPlayerData {
        player_id: u64,
    },

    // ── Bulk data requests (#137 Step 1) ──

    LoadTerrainChunks {
        player_id: u64,
        terrain_name: String,
        chunk_ids: Vec<TerrainChunkId>,
    },
    LoadOceanData {
        player_id: u64,
        world_name: String,
    },
    LoadLakeData {
        player_id: u64,
        world_name: String,
    },
    LoadTerrainGlobalData {
        player_id: u64,
        world_name: String,
    },
    LoadExplorationMap {
        player_id: u64,
        terrain_name: String,
    },

    // ── Step 2 requests ──

    LoadInventory {
        player_id: u64,
        unit_id: u64,
    },
    LoadOrganizationAtCell {
        player_id: u64,
        cell: GridCell,
    },
}

// ─── Bridge resource (Bevy side) ────────────────────────────────────

#[derive(bevy::prelude::Resource)]
pub struct LightyearBridge {
    /// tokio → Bevy: bridge events (spawn, despawn, position updates, action responses)
    rx: std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<BridgeEvent>>,
    /// Bevy → tokio: action requests pushed by Bevy systems when lightyear messages arrive
    action_tx: tokio::sync::mpsc::UnboundedSender<ActionRequest>,
}

impl LightyearBridge {
    /// Non-blocking drain of all pending bridge events.
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

    /// Push an action request to the tokio handler.
    /// Called by Bevy systems when they receive a lightyear Message.
    pub fn send_action(&self, request: ActionRequest) {
        if self.action_tx.send(request).is_err() {
            tracing::warn!("Action request channel closed — request lost");
        }
    }
}

// ─── Bridge sender (tokio side) ─────────────────────────────────────

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

/// Receiver for action requests — lives in the tokio action handler task.
pub struct ActionRequestReceiver {
    pub rx: tokio::sync::mpsc::UnboundedReceiver<ActionRequest>,
}

// ─── Bridge creation ────────────────────────────────────────────────

/// Create the full bridge. Returns:
/// - `LightyearBridge`: Bevy resource
/// - `BridgeSender`: cloned into tungstenite handlers and ActionProcessor
/// - `ActionRequestReceiver`: consumed by the tokio action handler task
pub fn create_bridge() -> (LightyearBridge, BridgeSender, ActionRequestReceiver) {
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();

    (
        LightyearBridge {
            rx: std::sync::Mutex::new(event_rx),
            action_tx,
        },
        BridgeSender { tx: event_tx },
        ActionRequestReceiver { rx: action_rx },
    )
}