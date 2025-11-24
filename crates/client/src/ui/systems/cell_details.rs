use bevy::prelude::*;
use shared::atlas::{GaugeAtlas, MoonAtlas};

use crate::{
    grid::resources::SelectedHexes,
    ui::components::CellDetailsPanelMarker,
};

use super::{
    action_bar::setup_action_bar, action_panel_setup::setup_action_panel,
    cell_details_panel::setup_cell_details_panel, chat_panel_setup::setup_chat_panel,
    top_bar::setup_top_bar,
};

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gauge_atlas: Res<GaugeAtlas>,
    moon_atlas: Res<MoonAtlas>,
) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            // Top bar
            setup_top_bar(parent, &asset_server, &moon_atlas);

            // Cell details panel (right side)
            setup_cell_details_panel(parent, &asset_server, &gauge_atlas);

            // Action bar (left sidebar)
            setup_action_bar(parent, &asset_server);

            // Action panel (bottom, with tabs)
            setup_action_panel(parent, &asset_server);

            // Chat panel and icon
            setup_chat_panel(parent, &asset_server);
        });
}

pub fn update_cell_details_visibility(
    mut query: Query<&mut Visibility, With<CellDetailsPanelMarker>>,
    selected: Res<SelectedHexes>,
) {
    if selected.is_changed() {
        let is_selected = !selected.ids.is_empty();

        for mut visibility in query.iter_mut() {
            *visibility = if is_selected {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
