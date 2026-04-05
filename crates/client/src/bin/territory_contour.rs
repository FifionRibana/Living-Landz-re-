use std::collections::HashSet;

use bevy::{prelude::*, window::PresentMode};
use hexx::*;

mod camera;
mod materials;
mod territory;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(0, 15, 30)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Territory tests".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "../../assets".to_string(),
                    ..default()
                }),
        )
        .add_plugins((
            camera::CameraPlugin,
            territory::TerritoryPlugin,
            ui::UiPlugin,
        ))
        .run();
}

fn debug_corners(layout: &HexLayout) {
    let hex = Hex::ZERO;
    let center = layout.hex_to_world_pos(hex);
    let corners = layout.hex_corners(hex);

    println!("Center: {:?}", center);
    for (i, corner) in corners.iter().enumerate() {
        let delta = *corner - center;
        let angle = delta.y.atan2(delta.x).to_degrees();
        println!("Corner {}: {:?} (angle: {:.1}°)", i, corner, angle);
    }
}
