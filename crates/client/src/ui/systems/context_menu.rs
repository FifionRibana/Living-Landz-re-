use bevy::prelude::*;

use crate::networking::client::NetworkClient;
use crate::state::resources::{ConnectionStatus, UnitsDataCache};
use crate::ui::components::{ContextMenuEntry, ContextMenuRoot};
use crate::ui::resources::{ContextMenuAction, ContextMenuState, UnitSelectionState};

// ─── Palette ────────────────────────────────────────────────────────

const MENU_BG: Color = Color::srgba(0.12, 0.09, 0.06, 0.95);
const MENU_BORDER: Color = Color::srgb(0.55, 0.45, 0.30);
const ENTRY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const ENTRY_HOVER: Color = Color::srgba(0.79, 0.66, 0.30, 0.20);
const GOLD: Color = Color::srgb(0.79, 0.66, 0.30);
const TEXT_LIGHT: Color = Color::srgb(0.92, 0.88, 0.80);
const TEXT_DIM: Color = Color::srgb(0.60, 0.52, 0.40);

// ─── Spawn / Despawn ────────────────────────────────────────────────

/// Surveille les changements de `ContextMenuState` pour spawn/despawn le menu
pub fn update_context_menu(
    mut commands: Commands,
    context_menu: Res<ContextMenuState>,
    existing_menu: Query<Entity, With<ContextMenuRoot>>,
    asset_server: Res<AssetServer>,
) {
    if !context_menu.is_changed() {
        return;
    }

    // Despawn l'ancien menu
    for entity in existing_menu.iter() {
        commands.entity(entity).despawn();
    }

    // Si le menu doit être ouvert, spawn un nouveau
    if !context_menu.open {
        return;
    }
    if context_menu.available_actions.is_empty() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Regular.ttf");
    let font_bold = asset_server.load("fonts/FiraSans-Bold.ttf");

    let pos = context_menu.screen_position;

    // Root du menu — positionné en absolu à la position du curseur
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x),
                top: Val::Px(pos.y),
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(160.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                padding: UiRect::axes(Val::Px(0.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(MENU_BG),
            BorderColor::all(MENU_BORDER),
            GlobalZIndex(1000),
            ContextMenuRoot,
            // Bloque les clics en dessous
            Pickable {
                should_block_lower: true,
                ..default()
            },
        ))
        .with_children(|menu| {
            // Titre (optionnel)
            if let Some(cell) = &context_menu.target_cell {
                menu.spawn((
                    Text::new(format!("({}, {})", cell.q, cell.r)),
                    TextFont {
                        font: font.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(TEXT_DIM),
                    Node {
                        padding: UiRect::new(
                            Val::Px(10.0),
                            Val::Px(10.0),
                            Val::Px(4.0),
                            Val::Px(4.0),
                        ),
                        ..default()
                    },
                ));
            }

            // Entrées d'action
            for action in &context_menu.available_actions {
                menu.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        padding: UiRect::new(
                            Val::Px(10.0),
                            Val::Px(10.0),
                            Val::Px(6.0),
                            Val::Px(6.0),
                        ),
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    Button,
                    BackgroundColor(ENTRY_BG),
                    ContextMenuEntry { action: *action },
                ))
                .with_children(|entry| {
                    // Icône
                    entry.spawn((
                        Text::new(action.icon()),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(GOLD),
                    ));
                    // Label
                    entry.spawn((
                        Text::new(action.label()),
                        TextFont {
                            font: font_bold.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(TEXT_LIGHT),
                    ));
                });
            }
        });
}

// ─── Handle entry click ─────────────────────────────────────────────

pub fn handle_context_menu_click(
    entry_query: Query<(&Interaction, &ContextMenuEntry), Changed<Interaction>>,
    mut context_menu: ResMut<ContextMenuState>,
    unit_selection: Res<UnitSelectionState>,
    units_data_cache: Option<Res<UnitsDataCache>>,
    connection: Res<ConnectionStatus>,
    mut network_client: Option<ResMut<NetworkClient>>,
) {
    for (interaction, entry) in entry_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(target_cell) = context_menu.target_cell else {
            continue;
        };
        let Some(target_chunk) = context_menu.target_chunk else {
            continue;
        };
        let Some(player_id) = connection.player_id else {
            continue;
        };

        match entry.action {
            ContextMenuAction::Move => {
                let selected_ids = unit_selection.selected_ids().to_vec();
                info!(
                    "Move {} units to ({},{})",
                    selected_ids.len(),
                    target_cell.q,
                    target_cell.r
                );

                if let Some(ref mut client) = network_client {
                    for unit_id in &selected_ids {
                        // Vérifier que l'unité n'est pas déjà sur la cellule cible
                        let already_there = units_data_cache
                            .as_ref()
                            .and_then(|cache| cache.get_unit(*unit_id))
                            .map(|u| u.current_cell == target_cell)
                            .unwrap_or(false);

                        if already_there {
                            info!("Unit {} already at target, skipping", unit_id);
                            continue;
                        }

                        client.send_message(shared::protocol::ClientMessage::ActionMoveUnit {
                            player_id,
                            unit_id: *unit_id,
                            chunk_id: target_chunk,
                            cell: target_cell,
                        });
                        info!("Sent move command for unit {}", unit_id);
                    }
                }
            }
            ContextMenuAction::Found => {
                info!("Founding hamlet!");

                if let Some(ref mut client) = network_client {
                    client.send_message(shared::protocol::ClientMessage::FoundHamlet);
                    info!("Sent FoundHamlet to server");
                }
            }
            ContextMenuAction::Build(building_type) => {
                info!(
                    "Building {:?} at ({},{})",
                    building_type, target_cell.q, target_cell.r
                );

                if let Some(ref mut client) = network_client {
                    client.send_message(shared::protocol::ClientMessage::ActionBuildBuilding {
                        player_id,
                        chunk_id: target_chunk,
                        cell: target_cell,
                        building_type,
                    });
                    info!("✓ Build {:?} request sent", building_type);
                }
            }
        }

        // Fermer le menu après l'action
        context_menu.close();
    }
}

// ─── Hover feedback ─────────────────────────────────────────────────

pub fn update_context_menu_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ContextMenuEntry>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(ENTRY_HOVER),
            Interaction::None => BackgroundColor(ENTRY_BG),
        };
    }
}

// ─── Dismiss (clic ailleurs ou ESC) ─────────────────────────────────

pub fn dismiss_context_menu(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut context_menu: ResMut<ContextMenuState>,
    menu_query: Query<&Interaction, With<ContextMenuRoot>>,
    entry_query: Query<&Interaction, With<ContextMenuEntry>>,
) {
    if !context_menu.open {
        return;
    }

    // ESC → fermer
    if keyboard.just_pressed(KeyCode::Escape) {
        context_menu.close();
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        // Vérifier si le clic est sur le root OU sur une entrée
        let clicking_menu = menu_query
            .iter()
            .any(|i| matches!(i, Interaction::Hovered | Interaction::Pressed))
            || entry_query
                .iter()
                .any(|i| matches!(i, Interaction::Hovered | Interaction::Pressed));

        if !clicking_menu {
            context_menu.close();
        }
    }
}
