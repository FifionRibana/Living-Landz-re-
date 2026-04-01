use bevy::prelude::*;
use lightyear::prelude::*;

use crate::protocol::{
    channels::ReliableGameChannel,
    components::{LordPosition, OwnedByPlayer},
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
        });
    }
}
