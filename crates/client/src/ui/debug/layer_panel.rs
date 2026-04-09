use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

use super::layer_state::{DebugLayer, DebugLayerVisibility, DebugOverride};

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct LayerPanel;

#[derive(Component)]
pub(crate) struct LayerToggleBtn(DebugLayer);

#[derive(Component)]
pub(crate) struct GroupToggleBtn(&'static str);

#[derive(Component)]
pub(crate) struct OverrideToggleBtn(DebugOverride);

// ---------------------------------------------------------------------------
// Colors (same as F4 debug overlay panel)
// ---------------------------------------------------------------------------

const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.88);
const SECTION_LABEL: Color = Color::srgba(0.6, 0.65, 0.7, 1.0);
const ITEM_TEXT: Color = Color::srgba(0.85, 0.9, 0.85, 1.0);
const BTN_OFF: Color = Color::srgba(0.18, 0.18, 0.22, 1.0);
const BTN_ON: Color = Color::srgba(0.22, 0.45, 0.65, 1.0);
const BTN_HOVER: Color = Color::srgba(0.28, 0.28, 0.35, 1.0);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub fn setup_layer_panel(mut commands: Commands) {
    // F4 panel is top-right → put F5 panel top-left
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(95.0), // below the topbar (91px)
                width: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                display: Display::None,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            LayerPanel,
        ))
        .with_children(|panel| {
            spawn_label(panel, "Layer Visibility", 13.0, SECTION_LABEL);

            // -- Layers section --
            spawn_label(panel, "Layers", 11.0, SECTION_LABEL);

            // Collect unique group names in order
            let mut seen_groups: Vec<&str> = Vec::new();
            for &layer in DebugLayer::all() {
                if layer.group().is_none() {
                    // Ungrouped layer → simple button
                    spawn_toggle_btn(panel, layer.label(), LayerToggleBtn(layer));
                } else {
                    let group = layer.group().unwrap();
                    if !seen_groups.contains(&group) {
                        seen_groups.push(group);
                        // Group header button
                        spawn_toggle_btn(panel, group, GroupToggleBtn(group));
                    }
                    // Indented child button
                    spawn_indented_btn(panel, layer.label(), LayerToggleBtn(layer));
                }
            }

            // -- Overrides section --
            spawn_label(panel, "Overrides", 11.0, SECTION_LABEL);
            for &ov in DebugOverride::all() {
                spawn_toggle_btn(panel, ov.label(), OverrideToggleBtn(ov));
            }
        });
}

fn spawn_label(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: &str,
    size: f32,
    color: Color,
) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: size, ..default() },
        TextColor(color),
        Node::default(),
    ));
}

fn spawn_toggle_btn(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    marker: impl Component,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            Button,
            BackgroundColor(BTN_ON), // layers default ON
            BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.6)),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 11.0, ..default() },
                TextColor(ITEM_TEXT),
            ));
        });
}

fn spawn_indented_btn(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    marker: impl Component,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(22.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect { left: Val::Px(14.0), ..default() },
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            Button,
            BackgroundColor(BTN_ON),
            BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.6)),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 10.0, ..default() },
                TextColor(ITEM_TEXT),
            ));
        });
}

// ---------------------------------------------------------------------------
// F5 — toggle panel visibility
// ---------------------------------------------------------------------------

pub fn toggle_layer_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugLayerVisibility>,
    mut panel: Query<&mut Node, With<LayerPanel>>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        state.panel_visible = !state.panel_visible;
        for mut node in panel.iter_mut() {
            node.display = if state.panel_visible {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Button interactions
// ---------------------------------------------------------------------------

pub fn handle_layer_panel_clicks(
    mut state: ResMut<DebugLayerVisibility>,
    layer_btns: Query<(&Interaction, &LayerToggleBtn), Changed<Interaction>>,
    group_btns: Query<(&Interaction, &GroupToggleBtn), Changed<Interaction>>,
    override_btns: Query<(&Interaction, &OverrideToggleBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in layer_btns.iter() {
        if *interaction == Interaction::Pressed {
            state.toggle_layer(btn.0);
        }
    }
    for (interaction, btn) in group_btns.iter() {
        if *interaction == Interaction::Pressed {
            // Toggle group: if all on → all off, otherwise → all on
            let all_on = state.is_group_all_on(btn.0);
            state.set_group(btn.0, !all_on);
        }
    }
    for (interaction, btn) in override_btns.iter() {
        if *interaction == Interaction::Pressed {
            let was_active = state.is_override_active(btn.0);
            state.toggle_override(btn.0);
            if !was_active {
                match btn.0 {
                    DebugOverride::LoadUnexploredChunks => {
                        warn!("Debug override: loading all chunks regardless of exploration status");
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Visual feedback
// ---------------------------------------------------------------------------

pub fn update_layer_button_visuals(
    state: Res<DebugLayerVisibility>,
    mut layer_btns: Query<(&Interaction, &LayerToggleBtn, &mut BackgroundColor)>,
    mut group_btns: Query<
        (&Interaction, &GroupToggleBtn, &mut BackgroundColor),
        Without<LayerToggleBtn>,
    >,
    mut override_btns: Query<
        (&Interaction, &OverrideToggleBtn, &mut BackgroundColor),
        (Without<LayerToggleBtn>, Without<GroupToggleBtn>),
    >,
) {
    for (interaction, btn, mut bg) in layer_btns.iter_mut() {
        let active = state.is_visible(btn.0);
        *bg = btn_color(*interaction, active);
    }
    for (interaction, btn, mut bg) in group_btns.iter_mut() {
        let active = state.is_group_any_on(btn.0);
        *bg = btn_color(*interaction, active);
    }
    for (interaction, btn, mut bg) in override_btns.iter_mut() {
        let active = state.is_override_active(btn.0);
        *bg = btn_color(*interaction, active);
    }
}

fn btn_color(interaction: Interaction, active: bool) -> BackgroundColor {
    match (interaction, active) {
        (Interaction::Hovered, _) | (Interaction::Pressed, _) => BackgroundColor(BTN_HOVER),
        (_, true) => BackgroundColor(BTN_ON),
        _ => BackgroundColor(BTN_OFF),
    }
}
