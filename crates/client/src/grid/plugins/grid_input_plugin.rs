use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::grid::input;
use crate::grid::materials::{HexHighlightMaterial, HexPulseMaterial};

pub struct GridInputPlugin;

impl Plugin for GridInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<HexHighlightMaterial>::default())
            .add_plugins(Material2dPlugin::<HexPulseMaterial>::default())
            .add_systems(Startup, input::systems::spawn_hex_indicators)
            .add_systems(
                Update,
                (
                    input::handlers::handle_hexagon_selection,
                    input::systems::update_hover_hexagon,
                    input::systems::update_selected_hexagons,
                    input::systems::animate_hexagons,
                )
                    .chain(),
            );
    }
}
