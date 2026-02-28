use bevy::prelude::*;

use crate::grid::systems;
use crate::grid::resources::RoadPreview;
use crate::states::AppState;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoadPreview>()
            .add_systems(
                PreStartup,
                (systems::setup_grid_config, systems::setup_meshes).chain(),
            )
            .add_systems(
                Update,
                (
                    systems::update_action_indicators,
                    systems::animate_in_progress_indicators,
                    systems::update_action_timer_text,
                    systems::cleanup_completed_indicators,
                    systems::update_road_preview,
                    systems::draw_road_preview,
                    systems::draw_unit_indicators,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
