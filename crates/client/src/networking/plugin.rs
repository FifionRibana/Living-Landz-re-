use bevy::prelude::*;

use super::client;
use super::systems;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::setup_network_client)
            .add_systems(Update, client::handlers::handle_server_message);
    }
}
