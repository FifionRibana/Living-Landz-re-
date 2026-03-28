use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::camera::resources::SceneRenderTarget;
use crate::networking::client::NetworkClient;
use crate::state::resources::{InventoryCache, PlayerInfo};
use crate::states::GameView;
use crate::ui::frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial};
use crate::ui::systems::panels::components::{InventoryItemRow, InventoryPanel};

pub fn setup_inventory_panel(
    mut commands: Commands,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    render_target: Res<SceneRenderTarget>,
    player_info: Res<PlayerInfo>,
    inventory_cache: Res<InventoryCache>,
    mut network_client: Option<ResMut<NetworkClient>>,
) {
    let config = FrostedGlassConfig::dialog()
        .with_border_radius(8.0)
        .with_colors(Color::srgb_u8(220, 202, 169), Color::srgb_u8(235, 225, 209));

    let mut material = FrostedGlassMaterial::from(config);
    material.scene_texture = Some(render_target.0.clone());

    // Get lord unit_id to fetch inventory
    let lord_unit_id = player_info.lord.as_ref().map(|l| l.id);

    // Re-request fresh inventory data
    if let (Some(uid), Some(client)) = (lord_unit_id, &mut network_client) {
        client.send_message(shared::protocol::ClientMessage::RequestInventory { unit_id: uid });
    }

    let lord_name = player_info
        .lord
        .as_ref()
        .map(|l| l.full_name())
        .unwrap_or_else(|| "Unknown".to_string());

    let items = lord_unit_id
        .and_then(|uid| inventory_cache.get_inventory(uid))
        .cloned()
        .unwrap_or_default();

    let total_weight = lord_unit_id
        .map(|uid| inventory_cache.total_weight(uid))
        .unwrap_or(0.0);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
            DespawnOnExit(GameView::Inventory),
            InventoryPanel,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(500.0),
                        max_height: Val::Percent(80.0),
                        padding: UiRect::all(Val::Px(30.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    MaterialNode(materials.add(material)),
                    BorderColor::all(Color::srgba_u8(235, 225, 209, 196)),
                ))
                .with_children(|panel| {
                    // Header: Title
                    panel.spawn((
                        Text::new("INVENTAIRE"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));

                    // Sub-header: Lord name + weight
                    panel.spawn((
                        Text::new(format!(
                            "{} — Poids: {:.1} kg",
                            lord_name, total_weight
                        )),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
                    ));

                    // Separator
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
                    ));

                    if items.is_empty() {
                        panel.spawn((
                            Text::new("Aucun objet dans l'inventaire."),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                        ));
                    } else {
                        // Item list
                        for item in &items {
                            panel
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::all(Val::Px(8.0)),
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        border: UiRect::bottom(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.1)),
                                    InventoryItemRow {
                                        item_id: item.item_id,
                                    },
                                ))
                                .with_children(|row| {
                                    // Left: Name + type
                                    row.spawn((
                                        Node {
                                            flex_direction: FlexDirection::Column,
                                            row_gap: Val::Px(2.0),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|left| {
                                        left.spawn((
                                            Text::new(item.name.clone()),
                                            TextFont {
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(Color::BLACK),
                                        ));

                                        left.spawn((
                                            Text::new(format!(
                                                "{:?} — Q: {:.0}%",
                                                item.item_type,
                                                item.quality * 100.0
                                            )),
                                            TextFont {
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                                        ));
                                    });

                                    // Right: Quantity + weight
                                    row.spawn((
                                        Node {
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::End,
                                            row_gap: Val::Px(2.0),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|right| {
                                        right.spawn((
                                            Text::new(format!("x{}", item.quantity)),
                                            TextFont {
                                                font_size: 18.0,
                                                ..default()
                                            },
                                            TextColor(Color::BLACK),
                                        ));

                                        right.spawn((
                                            Text::new(format!(
                                                "{:.1} kg",
                                                item.weight_kg * item.quantity as f32
                                            )),
                                            TextFont {
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                                        ));
                                    });
                                });
                        }
                    }
                });
        });
}
