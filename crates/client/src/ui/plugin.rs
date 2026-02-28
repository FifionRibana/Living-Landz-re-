// =============================================================================
// UI - Plugin
// =============================================================================

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use super::{
    resources::{ActionState, CellViewState, ChatState, UIState},
    systems,
};
use crate::state::resources;
use crate::states::{AppState, GameView, Overlay};
use crate::ui::resources::{CellState, DragState};
use crate::ui::systems::panels::auth::AuthPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy_ui_text_input::TextInputPlugin)
            // Auth screens managed by AuthPlugin via AuthScreen state
            .add_plugins(AuthPlugin)
            // ─── Startup: load atlases (resources, not entities) ─────────
            .add_systems(
                Startup,
                (resources::setup_gauge_atlas, resources::setup_moon_atlas).chain(),
            )
            // ─── View state resources — scoped to InGame ────────────────
            .add_systems(OnEnter(AppState::InGame), init_view_resources)
            .add_systems(OnExit(AppState::InGame), cleanup_view_resources)
            // ─── InGame lifecycle ────────────────────────────────────────
            // HUD (top bar, action bar, chat, cell details) spawns on InGame entry,
            // auto-despawned via DespawnOnExit(AppState::InGame)
            .add_systems(
                OnEnter(AppState::InGame),
                (systems::setup_ui, systems::setup_unit_details_panel),
            )
            // ─── GameView panel lifecycle ─────────────────────────────────
            // Each panel spawns on OnEnter and is auto-despawned via DespawnOnExit
            .add_systems(
                OnEnter(GameView::Calendar),
                systems::panels::setup_calendar_panel,
            )
            .add_systems(
                OnEnter(GameView::Cell),
                (
                    systems::panels::setup_cell_panel,
                    systems::panels::setup_cell_layout,
                    systems::panels::setup_cell_slots,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(GameView::CityManagement),
                systems::panels::setup_management_panel,
            )
            .add_systems(
                OnEnter(GameView::Messages),
                systems::panels::setup_messages_panel,
            )
            .add_systems(
                OnEnter(GameView::Rankings),
                systems::panels::setup_ranking_panel,
            )
            .add_systems(
                OnEnter(GameView::Records),
                systems::panels::setup_records_panel,
            )
            .add_systems(
                OnEnter(GameView::Settings),
                systems::panels::setup_settings_panel,
            )
            // ─── Pause menu overlay ────────────────────────────────────
            .add_systems(
                OnEnter(Overlay::PauseMenu),
                systems::panels::pause_menu::setup_pause_menu,
            )
            .add_systems(
                Update,
                (
                    systems::panels::pause_menu::handle_resume_click,
                    systems::panels::pause_menu::handle_disconnect_click,
                    systems::panels::pause_menu::update_pause_button_hover,
                )
                    .run_if(in_state(Overlay::PauseMenu)),
            )
            // Clean up when leaving Cell view
            .add_systems(OnExit(GameView::Cell), on_exit_cell_view)
            // ─── Update systems ──────────────────────────────────────────
            // Global ESC handler
            .add_systems(Update, handle_escape_key.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                systems::panels::update_action_menu_visibility.run_if(in_state(AppState::InGame)),
            )
            // Cell view systems — only run in GameView::Cell
            .add_systems(
                Update,
                (
                    systems::handle_cell_view_back_button,
                    systems::panels::setup_cell_layout.run_if(resource_exists::<CellState>.and(resource_changed::<CellState>)),
                    systems::panels::setup_cell_slots
                        .before(systems::panels::setup_cell_layout)
                        .run_if(resource_exists::<CellState>.and(resource_changed::<CellState>)),
                    systems::panels::update_unit_portraits,
                    systems::panels::update_slot_occupancy,
                    systems::panels::apply_hex_mask_to_portraits
                        .before(systems::panels::update_unit_portraits),
                    systems::panels::sync_slot_hierarchy_on_relation_change,
                    systems::panels::auto_assign_unslotted_units
                        .run_if(resource_exists::<CellState>.and(resource_changed::<CellState>)),
                )
                    .run_if(in_state(GameView::Cell)),
            )
            .add_systems(
                Update,
                (
                    systems::update_action_menu_visual,
                    systems::update_chat_visibility,
                    systems::update_chat_notification_badge,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (systems::update_organization_info,).run_if(in_state(AppState::InGame)),
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
                    // In-game button interactions
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

/// Clean up cell state when leaving cell view.
fn on_exit_cell_view(
    mut cell_state: Option<ResMut<CellState>>,
    mut input_focus: ResMut<InputFocus>,
) {
    info!("Exit cell view");
    if let Some(ref mut cell_state) = cell_state {
        cell_state.exit_view();
    }
    input_focus.0 = None;
}

fn init_view_resources(mut commands: Commands) {
    commands.insert_resource(ChatState::default());
    commands.insert_resource(ActionState::default());
    commands.insert_resource(CellViewState::default());
    commands.insert_resource(CellState::default());
    commands.insert_resource(DragState::default());
    commands.insert_resource(UIState::default());
}

fn cleanup_view_resources(mut commands: Commands) {
    commands.remove_resource::<ChatState>();
    commands.remove_resource::<ActionState>();
    commands.remove_resource::<CellViewState>();
    commands.remove_resource::<CellState>();
    commands.remove_resource::<DragState>();
    commands.remove_resource::<UIState>();
}

/// Global ESC handler for in-game.
/// - GameView::Cell → exit to Map
/// - Overlay::PauseMenu → close overlay
/// - Otherwise → open PauseMenu overlay
fn handle_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_view: Option<Res<State<GameView>>>,
    overlay: Option<Res<State<Overlay>>>,
    mut next_view: ResMut<NextState<GameView>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    // Priority 1: close pause menu if open
    if let Some(ref ov) = overlay && *ov.get() == Overlay::PauseMenu {
        info!("Closing pause menu");
        next_overlay.set(Overlay::None);
        return;
    }

    // Priority 2: exit cell view
    if let Some(ref gv) = game_view
        && *gv.get() == GameView::Cell
    {
        info!("Exiting cell view via ESC");
        next_view.set(GameView::Map);
        return;
    }

    // Priority 3: open pause menu
    info!("Opening pause menu");
    next_overlay.set(Overlay::PauseMenu);
}
