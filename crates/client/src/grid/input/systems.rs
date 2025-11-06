use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use hexx::Hex;

use crate::camera::MainCamera;
use crate::grid::components::{HexHoverIndicator, HexSelectIndicator};
use crate::grid::materials::{HexHighlightMaterial, HexPulseMaterial};
use crate::grid::resources::{HexMesh, SelectedHexes};

use shared::grid::GridConfig;

pub fn spawn_hex_indicators(
    mut commands: Commands,
    mut highlight_materials: ResMut<Assets<HexHighlightMaterial>>,
    mut pulse_materials: ResMut<Assets<HexPulseMaterial>>,
    hex_mesh: Res<HexMesh>,
) {
    let hex_mesh = hex_mesh.mesh.clone(); // Taille unitaire
    // let hover_material = materials.add(ColorMaterial::from_color(Color::srgba(1.0, 1.0, 1.0, 0.3)));

    let time = 0.0;

    commands.spawn((
        Name::new("Hexagon Hover"),
        Mesh2d(hex_mesh.clone()),
        MeshMaterial2d(pulse_materials.add(HexPulseMaterial {
            color: LinearRgba::rgb(1.0, 1.0, 1.0).with_alpha(0.3),
            time,
        })),
        Transform::default(),
        Visibility::Hidden, // Caché au démarrage
        HexHoverIndicator,
    ));

    let highlight_material_handle = highlight_materials.add(HexHighlightMaterial {
        color: LinearRgba::rgb(0.0, 1.0, 0.0).with_alpha(0.5),
        time,
    });

    for i in 0..20 {
        commands.spawn((
            Name::new(format!("Hexagon Select {}", i)),
            Mesh2d(hex_mesh.clone()),
            MeshMaterial2d(highlight_material_handle.clone()),
            Transform::default(),
            Visibility::Hidden,
            HexSelectIndicator {
                hex: Hex::new(i32::MAX, i32::MAX),
            }, // Invalide au démarrage
        ));
    }

    commands.insert_resource(SelectedHexes::default());
}

pub fn update_hover_hexagon(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query: Query<(&mut Transform, &mut Visibility), With<HexHoverIndicator>>,
    grid_config: Res<GridConfig>,
) -> Result {
    let window = windows.single()?;
    let (camera, camera_transform) = cameras.single()?;

    if let Some(position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
    {
        if let Ok((mut transform, mut visibility)) = query.single_mut() {
            let hex_position = grid_config.layout.world_pos_to_hex(position);
            let world_pos = grid_config.layout.hex_to_world_pos(hex_position);
            transform.translation = world_pos.extend(0.1); // Z légèrement au-dessus
            *visibility = Visibility::Visible;
        } else {
            // *visibility = Visibility::Hidden;
        }
    }
    Ok(())
}

pub fn update_selected_hexagons(
    mut query: Query<(&mut Transform, &mut Visibility), With<HexSelectIndicator>>,
    selected_hexes: Res<SelectedHexes>,
    grid_config: Res<GridConfig>,
) -> Result {
    let selected_list: Vec<Hex> = selected_hexes.ids.iter().copied().collect();

    for (i, (mut transform, mut visibility)) in query.iter_mut().enumerate() {
        if i < selected_list.len() {
            let hex = selected_list[i];

            // ✨ Positionner l'indicateur sur l'hexagone sélectionné
            let hex_center = grid_config.layout.hex_to_world_pos(hex);
            transform.translation = hex_center.extend(0.2);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    Ok(())
}

pub fn animate_hexagons(
    mut highlight_query: Query<&mut MeshMaterial2d<HexHighlightMaterial>>,
    mut pulse_query: Query<&mut MeshMaterial2d<HexPulseMaterial>>,
    mut highlight_materials: ResMut<Assets<HexHighlightMaterial>>,
    mut pulse_materials: ResMut<Assets<HexPulseMaterial>>,
    time: Res<Time>,
) {
    for highlight_material_handle in highlight_query.iter_mut() {
        if let Some(material) = highlight_materials.get_mut(highlight_material_handle.clone()) {
            material.time = time.elapsed_secs();
        }
    }

    for pulse_material_handle in pulse_query.iter_mut() {
        if let Some(material) = pulse_materials.get_mut(pulse_material_handle.clone()) {
            material.time = time.elapsed_secs();
        }
    }
}
