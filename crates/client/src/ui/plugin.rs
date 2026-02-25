// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::prelude::*;

use super::{
    resources::{ActionState, CellViewState, ChatState, UIState},
    systems,
};
use crate::state::resources;
use crate::ui::resources::{CellState, DragState};

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
            .add_systems(
                Startup,
                (
                    resources::setup_gauge_atlas,
                    resources::setup_moon_atlas,
                    systems::setup_ui,
                    // systems::setup_cell_view_ui,
                    systems::setup_unit_details_panel,
                    // Panels
                    systems::panels::setup_login_panel,
                    systems::panels::setup_register_panel,
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
            .add_systems(
                Update,
                (
                    systems::panels::update_top_bar_visibility,
                    systems::panels::update_panel_visibility.run_if(resource_changed::<UIState>),
                    systems::panels::setup_cell_layout
                        .run_if(resource_changed::<UIState>.or(resource_changed::<CellState>)),
                    systems::panels::setup_cell_slots
                        .before(systems::panels::setup_cell_layout)
                        .run_if(resource_changed::<UIState>.or(resource_changed::<CellState>)),
                    systems::panels::update_unit_portraits,
                    //     .run_if(
                    // resource_changed::<UIState>
                    //     .or(resource_changed::<CellState>)
                    //     .or(resource_changed::<UnitsCache>)
                    //     .or(resource_changed::<UnitsDataCache>),
                    // ),
                    // .before(systems::panels::update_unit_portraits),
                    systems::panels::update_slot_occupancy,
                    systems::panels::apply_hex_mask_to_portraits
                        .before(systems::panels::update_unit_portraits),
                    systems::panels::sync_slot_hierarchy_on_relation_change,
                    systems::panels::auto_assign_unslotted_units
                        .before(systems::panels::update_unit_portraits)
                        .run_if(resource_changed::<UIState>.or(resource_changed::<CellState>)),
                    systems::update_action_menu_visual,
                    // systems::update_cell_details_visibility,
                    // systems::update_cell_details_content,
                    // .before(systems::update_chat_notification_badge)
                    // .before(systems::update_player_info),
                    systems::update_chat_visibility,
                    systems::update_chat_notification_badge,
                ),
            )
            .add_systems(Update, (systems::update_organization_info,))
            .add_systems(
                Update,
                (
                    // systems::enforce_single_selection,
                    // Dynamic updates
                    systems::update_clock,
                    systems::update_moon_phase_image,
                    systems::update_player_info,
                    systems::update_cell_action_display,
                    systems::hide_action_panel_during_action,
                ),
            )
            .add_systems(
                Update,
                (
                    // Button interactions
                    // systems::handle_menu_button_interactions,
                    // Auth panel interactions
                    systems::panels::handle_login_button_click,
                    systems::panels::handle_to_register_button_click,
                    // systems::panels::handle_login_response,
                    systems::panels::handle_login_button_hover,
                    systems::panels::handle_register_button_click,
                    systems::panels::handle_back_button_click,
                    // systems::panels::handle_register_response,
                    // systems::panels::handle_auto_switch_to_login,
                    systems::panels::handle_register_button_hover,
                    // Other button interactions
                    systems::handle_action_category_button_interactions,
                    systems::update_action_category_button_appearance,
                    systems::handle_action_tab_button_interactions,
                    systems::handle_action_button_interactions,
                    systems::handle_chat_send_button,
                    systems::handle_chat_toggle_button,
                    systems::handle_chat_icon_button,
                    // Action panel content updates
                    systems::update_action_panel_content,
                    systems::handle_building_button_interactions,
                    systems::handle_action_run_button,
                ),
                // )
                // .add_systems(
                //     Update,
                //     (
                // Unit selection and details panel
                // systems::handle_unit_deselect,
                // systems::handle_empty_slot_click,
                // systems::update_panel_visibility,
                // systems::update_panel_content,
                // systems::handle_close_button,
                // Cell view slot interactions
                // systems::handle_slot_click,
                // Cell view drag & drop
                // systems::handle_slot_drag_start,
                // systems::detect_drag_movement,
                // systems::update_drag_visual,
                // systems::handle_slot_drop,
                // systems::cancel_drag_on_escape,
                // ),
            );
    }
}
