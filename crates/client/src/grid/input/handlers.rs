use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::grid::resources::SelectedHexes;
use crate::state::resources::{PlayerInfo, WorldCache};
use crate::states::GameView;
use crate::ui::resources::{CellState, UnitSelectionState};
use crate::{camera::MainCamera, ui::resources::ContextMenuState};

use hexx::Hex;
use shared::grid::{GridCell, GridConfig};

pub fn handle_hexagon_selection(
    mut selected_hexes: ResMut<SelectedHexes>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    grid_config: Res<GridConfig>,
    game_view: Option<Res<State<GameView>>>,
    // Check if cursor is over UI - now all UI elements (buttons and panels) have Interaction
    ui_interaction_query: Query<(&Interaction, &Pickable), With<Node>>,
) -> Result {
    let Some(gv) = game_view else { return Ok(()) };
    if *gv.get() != GameView::Map {
        return Ok(());
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single()?;

        // Check if cursor is over any UI element that should block lower layers
        let is_over_ui = ui_interaction_query.iter().any(|(interaction, pickable)| {
            pickable.should_block_lower
                && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
        });

        // Don't select hexagons if cursor is over UI
        if is_over_ui {
            return Ok(());
        }
        let (camera, camera_transform) = cameras.single()?;

        if let Some(position) = window
            .cursor_position()
            .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
        {
            let hovered_hex = grid_config.layout.world_pos_to_hex(position);
            // ✨ Ctrl + Click = Multi-select
            if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
                selected_hexes.toggle(hovered_hex);
            } else {
                // Single select
                if selected_hexes.is_selected(hovered_hex) && selected_hexes.selection_count() == 1
                {
                    selected_hexes.remove(hovered_hex);
                } else {
                    selected_hexes.clear();
                    selected_hexes.add(hovered_hex);
                }
            }
        }
    }

    // Delete = Deselect all
    if keyboard.just_pressed(KeyCode::Delete) {
        selected_hexes.clear();
    }
    Ok(())
}

/// Detect double-click on hexagons to enter cell view mode
pub fn handle_cell_view_entry(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cell_state: ResMut<CellState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid_config: Res<GridConfig>,
    time: Res<Time>,
    // Check if cursor is over UI
    ui_interaction_query: Query<(&Interaction, &Pickable), With<Node>>,
    mut last_click: Local<Option<(Hex, f32)>>,
    mut next_view: ResMut<NextState<GameView>>,
    game_view: Option<Res<State<GameView>>>,
    world_cache: Res<WorldCache>,
) {
    let Some(gv) = game_view else { return };
    // Only process clicks when NOT in cell view mode
    if *gv.get() == GameView::Cell {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else {
            return;
        };

        // Check if cursor is over any UI element that should block lower layers
        let is_over_ui = ui_interaction_query.iter().any(|(interaction, pickable)| {
            pickable.should_block_lower
                && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
        });

        // Don't process clicks on UI
        if is_over_ui {
            return;
        }

        let Ok((camera, camera_transform)) = cameras.single() else {
            return;
        };

        if let Some(position) = window
            .cursor_position()
            .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
        {
            let clicked_hex = grid_config.layout.world_pos_to_hex(position);
            let current_time = time.elapsed_secs();

            // Check for double-click
            if let Some((last_hex, last_time)) = *last_click
                && last_hex == clicked_hex
                && (current_time - last_time) < 0.3
            {
                // Double-click detected!
                let cell = GridCell::from_hex(&clicked_hex);
                info!("Double-click detected on cell: q={}, r={}", cell.q, cell.r);

                cell_state.enter_view(
                    world_cache.get_cell(&cell).cloned(),
                    world_cache.get_building(&cell).cloned(),
                );
                next_view.set(GameView::Cell);
                *last_click = None;
                return;
            }

            // Store this click for potential double-click
            *last_click = Some((clicked_hex, current_time));
        }
    }
}

// TODO : Lord movement should be done only if the lord is selected.
// TODO : Furthermore, every selected units that can actually move should move toward the target cell
// TODO : Unit that can move: No pending action (move, train, production, building, attack, etc..), not stuck for any reason, alive, ...
/// Clic droit sur la carte = ouvrir le menu contextuel (si unités sélectionnées)
pub fn handle_map_right_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid_config: Res<GridConfig>,
    unit_selection: Res<UnitSelectionState>,
    player_info: Res<PlayerInfo>,
    mut context_menu: ResMut<ContextMenuState>,
    game_view: Option<Res<State<GameView>>>,
    ui_interaction_query: Query<(&Interaction, &Pickable), With<Node>>,
) {
    let Some(gv) = game_view else { return };
    if *gv.get() != GameView::Map {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    // Fermer le menu s'il est déjà ouvert (re-clic droit)
    if context_menu.open {
        context_menu.close();
        return;
    }

    // Pas de sélection → pas de menu
    if !unit_selection.has_selection() {
        return;
    }

    // Ne pas ouvrir si clic sur l'UI
    let is_over_ui = ui_interaction_query.iter().any(|(interaction, pickable)| {
        pickable.should_block_lower
            && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    });
    if is_over_ui {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    // Position écran du curseur
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Position monde → hex
    let Some(world_pos) = camera
        .viewport_to_world_2d(camera_transform, cursor_pos)
        .ok()
    else {
        return;
    };

    let target_hex = grid_config.layout.world_pos_to_hex(world_pos);
    let target_cell = shared::grid::GridCell::from_hex(&target_hex);

    // TODO: calculer le chunk cible à partir de la position monde
    // Pour le MVP on utilise le chunk (0,0) — à améliorer
    let target_chunk = shared::TerrainChunkId { x: 0, y: 0 };

    // Construire la liste d'actions disponibles
    let mut actions = Vec::new();
    actions.push(crate::ui::resources::ContextMenuAction::Move);

    // Fonder — disponible si le lord est sélectionné ET la cellule est la sienne
    // (On fonde à la position actuelle du lord, pas à la cellule cliquée)
    if let Some(lord) = &player_info.lord {
        let lord_selected = unit_selection.selected_ids().contains(&lord.id);
        if lord_selected {
            // On propose "Fonder" — le serveur vérifiera les conditions
            actions.push(crate::ui::resources::ContextMenuAction::Found);
        }
    }

    // Construire — disponible si le lord est sélectionné
    // et qu'on est dans le territoire du joueur
    if let Some(lord) = &player_info.lord {
        let lord_selected = unit_selection.selected_ids().contains(&lord.id);
        if lord_selected {
            // Proposer les bâtiments constructibles
            // Pour le MVP : liste fixe de bâtiments de base
            let buildable = [
                shared::BuildingTypeEnum::Farm,
                shared::BuildingTypeEnum::Blacksmith,
                shared::BuildingTypeEnum::CarpenterShop,
                shared::BuildingTypeEnum::Bakehouse,
                shared::BuildingTypeEnum::Brewery,
                shared::BuildingTypeEnum::Market,
            ];

            for bt in &buildable {
                actions.push(crate::ui::resources::ContextMenuAction::Build(*bt));
            }
        }
    }

    // Futures: vérifier le contexte pour proposer Build, Harvest, etc.

    info!(
        "Opening context menu at ({},{}) for {} selected units",
        target_cell.q,
        target_cell.r,
        unit_selection.count()
    );

    context_menu.open_at(cursor_pos, target_cell, target_chunk, actions);
}
