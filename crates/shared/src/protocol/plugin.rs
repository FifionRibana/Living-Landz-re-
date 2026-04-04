use bevy::prelude::*;
use lightyear::prelude::*;

use crate::protocol::{
    channels::{
        ExplorationChannel, ReliableGameChannel,
    },
    components::{LordPosition, OwnedByPlayer},
    messages::{
        ActionBuildBuildingMsg, ActionBuildRoadMsg, ActionCompletedMsg, ActionCraftResourceMsg,
        ActionErrorMsg, ActionExploreMsg, ActionHarvestResourceMsg, ActionMoveUnitMsg,
        ActionStatusMsg, ActionTrainUnitMsg, DebugErrorMsg, DebugOrganizationCreatedMsg,
        DebugOrganizationDeletedMsg, DebugUnitSpawnedMsg, ExplorationPatchMsg,
        GameDataMsg, HamletFoundErrorMsg, HamletFoundedMsg, InventoryDataMsg, InventoryUpdateMsg,
        LoginSuccessMsg, LordDataMsg,
        OrganizationAtCellMsg, PlayerOrganizationDataMsg, PopulationChangedMsg,
        RequestInventoryMsg,
        RequestOrganizationAtCellMsg,
        RoadChunkSdfUpdateMsg,
        TerritoryBorderCellsMsg, TerritoryBorderSdfUpdateMsg, TerritoryContourUpdateMsg,
        UnitPositionUpdatedMsg, UnitProfessionChangedMsg, UnitSlotUpdatedMsg,
        UnitWorkStatusUpdateMsg,
        CreateLordMsg, FoundHamletMsg, MoveUnitToSlotMsg, AssignUnitToSlotMsg,
        SendMessageMsg, DebugCreateOrganizationMsg, DebugDeleteOrganizationMsg,
        DebugSpawnUnitMsg, LordCreatedMsg, LordCreateErrorMsg,
    },
};

// === Protocol Plugin ===
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<LordPosition>()
            .add_prediction();
        // PAS d'add_interpolation_with pour l'instant

        app.register_component::<OwnedByPlayer>().add_prediction();

        app.add_channel::<ReliableGameChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        // Exploration patches remain on lightyear (incremental updates)
        app.add_channel::<ExplorationChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);

    // Messages: client → server
        app.register_message::<ActionMoveUnitMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ActionBuildBuildingMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ActionBuildRoadMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ActionHarvestResourceMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ActionCraftResourceMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ActionTrainUnitMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ActionExploreMsg>()
            .add_direction(NetworkDirection::ClientToServer);

        // Messages: server → client
        app.register_message::<ActionStatusMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<ActionErrorMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<UnitPositionUpdatedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<ActionCompletedMsg>()
            .add_direction(NetworkDirection::ServerToClient);

        // Post-login data messages: server → client
        app.register_message::<LoginSuccessMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<LordDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<PlayerOrganizationDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<GameDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);

        // Exploration patches remain on lightyear (incremental updates)
        app.register_message::<ExplorationPatchMsg>()
            .add_direction(NetworkDirection::ServerToClient);

        // Client → Server requests (#137 Step 2)
        app.register_message::<RequestInventoryMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<RequestOrganizationAtCellMsg>()
            .add_direction(NetworkDirection::ClientToServer);

        // Server → Client events (#137 Step 2)
        app.register_message::<InventoryDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<InventoryUpdateMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<UnitProfessionChangedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<UnitWorkStatusUpdateMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<RoadChunkSdfUpdateMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<TerritoryContourUpdateMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<TerritoryBorderSdfUpdateMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<TerritoryBorderCellsMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<PopulationChangedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<HamletFoundedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<HamletFoundErrorMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<OrganizationAtCellMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<UnitSlotUpdatedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<DebugOrganizationCreatedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<DebugOrganizationDeletedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<DebugUnitSpawnedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<DebugErrorMsg>()
            .add_direction(NetworkDirection::ServerToClient);

        // Remaining client → server commands (migrated from tungstenite #138)
        app.register_message::<CreateLordMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<FoundHamletMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<MoveUnitToSlotMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<AssignUnitToSlotMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<SendMessageMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<DebugCreateOrganizationMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<DebugDeleteOrganizationMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<DebugSpawnUnitMsg>()
            .add_direction(NetworkDirection::ClientToServer);

        // Lord lifecycle responses: server → client
        app.register_message::<LordCreatedMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<LordCreateErrorMsg>()
            .add_direction(NetworkDirection::ServerToClient);
    }
}
