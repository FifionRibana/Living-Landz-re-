// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;
use bevy::state::condition;

use super::{
    resources::{ActionState, CellViewState, ChatState, UIState},
    systems,
};
use crate::state::resources;
use crate::states::AppState;
use crate::ui::resources::{CellState, DragState};
use crate::ui::systems::panels::auth::AuthPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChatState::default())
            .insert_resource(ActionState::default())
            .insert_resource(CellViewState::default())
            .insert_resource(CellState::default())
            .insert_resource(DragState::default())
            .insert_resource(UIState::default())
            .add_plugins(bevy_ui_text_input::TextInputPlugin)
            // Auth screens are now managed by AuthPlugin via states
            .add_plugins(AuthPlugin)
            .add_systems(
                Startup,
                (
                    resources::setup_gauge_atlas,
                    resources::setup_moon_atlas,
                    systems::setup_ui,
                    // systems::setup_cell_view_ui,
                    systems::setup_unit_details_panel,
                    // Panels (in-game panels — still spawned at Startup for now)
                    systems::panels::setup_calendar_panel,
                    systems::panels::setup_cell_panel,
                    systems::panels::setup_management_panel,
                    systems::panels::setup_messages_panel,
                    systems::panels::setup_ranking_panel,
                    systems::panels::setup_records_panel,
                    systems::panels::setup_settings_panel,
                )
                    .chain(),
            )
            // Force panel visibility update when entering InGame for the first time
            .add_systems(
                OnEnter(AppState::InGame),
                force_initial_panel_visibility,
            )
            .add_systems(
                Update,
                (
                    systems::panels::update_top_bar_visibility,
                    systems::panels::update_panel_visibility
                        .run_if(in_state(AppState::InGame))
                        .run_if(resource_changed::<UIState>),
                    systems::panels::setup_cell_layout
                        .run_if(in_state(AppState::InGame))
                        .run_if(resource_changed::<UIState>.or(resource_changed::<CellState>)),
                    systems::panels::setup_cell_slots
                        .before(systems::panels::setup_cell_layout)
                        .run_if(in_state(AppState::InGame))
                        .run_if(resource_changed::<UIState>.or(resource_changed::<CellState>)),
                    systems::panels::update_unit_portraits,
                    systems::panels::update_slot_occupancy,
                    systems::panels::apply_hex_mask_to_portraits
                        .before(systems::panels::update_unit_portraits),
                    systems::panels::sync_slot_hierarchy_on_relation_change,
                    systems::panels::auto_assign_unslotted_units
                        .before(systems::panels::update_unit_portraits)
                        .run_if(resource_changed::<UIState>.or(resource_changed::<CellState>)),
                    systems::update_action_menu_visual,
                    systems::update_chat_visibility,
                    systems::update_chat_notification_badge,
                ),
            )
            .add_systems(
                Update,
                (systems::update_organization_info,)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    systems::update_clock,
                    systems::update_moon_phase_image,
                    systems::update_player_info,
                    systems::update_cell_action_display,
                    systems::hide_action_panel_during_action,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    // In-game button interactions (no auth buttons here anymore)
                    systems::handle_action_category_button_interactions,
                    systems::update_action_category_button_appearance,
                    systems::handle_action_tab_button_interactions,
                    systems::handle_action_button_interactions,
                    systems::handle_chat_send_button,
                    systems::handle_chat_toggle_button,
                    systems::handle_chat_icon_button,
                    systems::update_action_panel_content,
                    systems::handle_building_button_interactions,
                    systems::handle_action_run_button,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// Triggers `resource_changed::<UIState>` so that `update_panel_visibility`
/// runs on the first frame after entering InGame.
fn force_initial_panel_visibility(mut ui_state: ResMut<UIState>) {
    // Re-setting the same panel marks the resource as changed for this frame,
    // which triggers update_panel_visibility to set correct initial visibility.
    let panel = ui_state.panel_state;
    ui_state.switch_to(panel);
}