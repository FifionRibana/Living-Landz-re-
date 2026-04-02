use bevy::prelude::*;
use lightyear::prelude::*;

use crate::protocol::{
    channels::ReliableGameChannel,
    components::{LordPosition, OwnedByPlayer},
    lightyear_messages::{
        ActionBuildBuildingMsg, ActionBuildRoadMsg, ActionCompletedMsg, ActionCraftResourceMsg,
        ActionErrorMsg, ActionExploreMsg, ActionHarvestResourceMsg, ActionMoveUnitMsg,
        ActionStatusMsg, ActionTrainUnitMsg, UnitPositionUpdatedMsg,
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
    }
}
