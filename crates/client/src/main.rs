use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
mod camera;
mod grid;
// mod input;
mod networking;
mod rendering;
mod state;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(0, 15, 30)))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Living landz [re]".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "assets".to_string(),
                    ..default()
                }),
            MeshPickingPlugin,
        ))
        .add_plugins((
            camera::CameraPlugin,
            state::ClientStatePlugin,
            grid::GridInputPlugin,
            grid::GridPlugin,
            networking::NetworkingPlugin,
            rendering::terrain::TerrainPlugin,
            ui::debug::DebugUiPlugin,
        ))
        .add_plugins((
            // LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
        .run();
}
