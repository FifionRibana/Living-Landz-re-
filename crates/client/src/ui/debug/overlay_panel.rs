use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

use super::overlay_state::{DebugOverlayState, ShaderBaseMode};

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct DebugPanel;

#[derive(Component)]
pub(crate) struct ShaderBaseButton(ShaderBaseMode);

#[derive(Component)]
pub(crate) struct LevelLinesToggle;

#[derive(Component)]
pub(crate) struct ShoreTypesToggle;

#[derive(Component)]
pub(crate) struct ChunkBoundsToggle;

#[derive(Component)]
pub(crate) struct DomainHexesToggle;

#[derive(Component)]
pub(crate) struct VoronoiDomainsToggle;

#[derive(Component)]
pub(crate) struct VoronoiMistToggle;

#[derive(Component)]
pub(crate) struct YmirCliffsToggle;

// ---------------------------------------------------------------------------
// Colors
// ---------------------------------------------------------------------------

const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.88);
const SECTION_LABEL: Color = Color::srgba(0.6, 0.65, 0.7, 1.0);
const ITEM_TEXT: Color = Color::srgba(0.85, 0.9, 0.85, 1.0);
const BTN_OFF: Color = Color::srgba(0.18, 0.18, 0.22, 1.0);
const BTN_ON: Color = Color::srgba(0.22, 0.45, 0.65, 1.0);
const BTN_HOVER: Color = Color::srgba(0.28, 0.28, 0.35, 1.0);

// ---------------------------------------------------------------------------
// Setup — spawn once at startup, hidden by default
// ---------------------------------------------------------------------------

pub fn setup_debug_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
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
            DebugPanel,
        ))
        .with_children(|panel| {
            // Title
            spawn_label(panel, "Debug Overlays", 13.0, SECTION_LABEL);

            // -- Shader Base Mode --
            spawn_label(panel, "Shader", 11.0, SECTION_LABEL);
            for &mode in ShaderBaseMode::ALL {
                spawn_toggle_btn(panel, mode.label(), ShaderBaseButton(mode));
            }

            // -- Shader Overlays --
            spawn_label(panel, "Overlays", 11.0, SECTION_LABEL);
            spawn_toggle_btn(panel, "Level Lines", LevelLinesToggle);

            // -- Gizmos --
            spawn_label(panel, "Gizmos", 11.0, SECTION_LABEL);
            spawn_toggle_btn(panel, "Shore Types", ShoreTypesToggle);
            spawn_toggle_btn(panel, "Chunks", ChunkBoundsToggle);
            spawn_toggle_btn(panel, "Domain Hexes", DomainHexesToggle);
            spawn_toggle_btn(panel, "Voronoi Domains", VoronoiDomainsToggle);
            spawn_toggle_btn(panel, "Voronoi Mist", VoronoiMistToggle);
            spawn_toggle_btn(panel, "Ymir cliffs (ground truth)", YmirCliffsToggle);
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
        TextFont {
            font_size: size,
            ..default()
        },
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
            BackgroundColor(BTN_OFF),
            BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.6)),
            marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(ITEM_TEXT),
            ));
        });
}

// ---------------------------------------------------------------------------
// F4 — toggle panel visibility
// ---------------------------------------------------------------------------

pub fn toggle_debug_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug: ResMut<DebugOverlayState>,
    mut panel: Query<&mut Node, With<DebugPanel>>,
) {
    if keyboard.just_pressed(KeyCode::F4) {
        debug.panel_visible = !debug.panel_visible;
        for mut node in panel.iter_mut() {
            node.display = if debug.panel_visible {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Button interactions — update DebugOverlayState
// ---------------------------------------------------------------------------

pub fn handle_debug_panel_clicks(
    mut debug: ResMut<DebugOverlayState>,
    base_buttons: Query<(&Interaction, &ShaderBaseButton), Changed<Interaction>>,
    level_lines: Query<&Interaction, (Changed<Interaction>, With<LevelLinesToggle>)>,
    shore_types: Query<&Interaction, (Changed<Interaction>, With<ShoreTypesToggle>)>,
    chunk_bounds: Query<&Interaction, (Changed<Interaction>, With<ChunkBoundsToggle>)>,
    domain_hexes: Query<&Interaction, (Changed<Interaction>, With<DomainHexesToggle>)>,
    voronoi_domains: Query<&Interaction, (Changed<Interaction>, With<VoronoiDomainsToggle>)>,
    voronoi_mist: Query<&Interaction, (Changed<Interaction>, With<VoronoiMistToggle>)>,
    ymir_cliffs: Query<&Interaction, (Changed<Interaction>, With<YmirCliffsToggle>)>,
) {
    for (interaction, btn) in base_buttons.iter() {
        if *interaction == Interaction::Pressed {
            if debug.shader_base == btn.0 {
                debug.shader_base = ShaderBaseMode::Normal;
            } else {
                debug.shader_base = btn.0;
            }
        }
    }

    fn toggle_on_press(query: &Query<&Interaction, impl bevy::ecs::query::QueryFilter>, flag: &mut bool) {
        for interaction in query.iter() {
            if *interaction == Interaction::Pressed {
                *flag = !*flag;
            }
        }
    }

    toggle_on_press(&level_lines, &mut debug.level_lines);
    toggle_on_press(&shore_types, &mut debug.hex_shore_types);
    toggle_on_press(&chunk_bounds, &mut debug.chunk_boundaries);
    toggle_on_press(&domain_hexes, &mut debug.domain_hexes);
    toggle_on_press(&voronoi_domains, &mut debug.voronoi_domains);
    toggle_on_press(&voronoi_mist, &mut debug.voronoi_mist);
    toggle_on_press(&ymir_cliffs, &mut debug.ymir_cliffs);
}

// ---------------------------------------------------------------------------
// Visual feedback — highlight active buttons
// ---------------------------------------------------------------------------

pub fn update_debug_button_visuals(
    debug: Res<DebugOverlayState>,
    mut base_buttons: Query<(&Interaction, &ShaderBaseButton, &mut BackgroundColor)>,
    mut all_toggles: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        (With<Button>, Without<ShaderBaseButton>),
    >,
    q_ll: Query<Entity, With<LevelLinesToggle>>,
    q_st: Query<Entity, With<ShoreTypesToggle>>,
    q_cb: Query<Entity, With<ChunkBoundsToggle>>,
    q_dh: Query<Entity, With<DomainHexesToggle>>,
    q_vd: Query<Entity, With<VoronoiDomainsToggle>>,
    q_vm: Query<Entity, With<VoronoiMistToggle>>,
    q_yc: Query<Entity, With<YmirCliffsToggle>>,
) {
    // Shader base buttons (radio)
    for (interaction, btn, mut bg) in base_buttons.iter_mut() {
        let active = debug.shader_base == btn.0;
        *bg = btn_color(*interaction, active);
    }

    // Checkbox toggles — map entity to active state
    let active_map: Vec<(Entity, bool)> = [
        (q_ll.iter().next(), debug.level_lines),
        (q_st.iter().next(), debug.hex_shore_types),
        (q_cb.iter().next(), debug.chunk_boundaries),
        (q_dh.iter().next(), debug.domain_hexes),
        (q_vd.iter().next(), debug.voronoi_domains),
        (q_vm.iter().next(), debug.voronoi_mist),
        (q_yc.iter().next(), debug.ymir_cliffs),
    ]
    .iter()
    .filter_map(|(e, a)| e.map(|e| (e, *a)))
    .collect();

    for (entity, interaction, mut bg) in all_toggles.iter_mut() {
        if let Some((_, active)) = active_map.iter().find(|(e, _)| *e == entity) {
            *bg = btn_color(*interaction, *active);
        }
    }
}

fn btn_color(interaction: Interaction, active: bool) -> BackgroundColor {
    match (interaction, active) {
        (Interaction::Hovered, _) | (Interaction::Pressed, _) => BackgroundColor(BTN_HOVER),
        (_, true) => BackgroundColor(BTN_ON),
        _ => BackgroundColor(BTN_OFF),
    }
}
