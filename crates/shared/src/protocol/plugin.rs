use bevy::prelude::*;
use lightyear::prelude::*;

use crate::protocol::{
    channels::{
        ExplorationChannel, LakeDataChannel, OceanDataChannel, ReliableGameChannel,
        TerrainChunkChannel, TerrainGlobalChannel,
    },
    components::{LordPosition, OwnedByPlayer},
    lightyear_messages::{
        ActionBuildBuildingMsg, ActionBuildRoadMsg, ActionCompletedMsg, ActionCraftResourceMsg,
        ActionErrorMsg, ActionExploreMsg, ActionHarvestResourceMsg, ActionMoveUnitMsg,
        ActionStatusMsg, ActionTrainUnitMsg, ExplorationMapMsg, ExplorationPatchMsg, GameDataMsg,
        LakeDataMsg, LoginSuccessMsg, LordDataMsg, OceanDataMsg, PlayerOrganizationDataMsg,
        RequestExplorationMapMsg, RequestLakeDataMsg, RequestOceanDataMsg,
        RequestTerrainChunksMsg, RequestTerrainGlobalDataMsg, TerrainChunkDataMsg,
        TerrainGlobalDataMsg, UnitPositionUpdatedMsg,
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

        // Bulk data channels: server → client
        app.add_channel::<TerrainChunkChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);

        app.add_channel::<OceanDataChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);

        app.add_channel::<LakeDataChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);

        app.add_channel::<TerrainGlobalChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);

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

        // Bulk data requests: client → server
        app.register_message::<RequestTerrainChunksMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<RequestOceanDataMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<RequestLakeDataMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<RequestTerrainGlobalDataMsg>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<RequestExplorationMapMsg>()
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

        // Bulk data responses: server → client
        app.register_message::<TerrainChunkDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<OceanDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<LakeDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<TerrainGlobalDataMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<ExplorationMapMsg>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<ExplorationPatchMsg>()
            .add_direction(NetworkDirection::ServerToClient);
    }
}
