use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
mod camera;
mod grid;
// mod input;
mod networking;
mod rendering;
mod state;
pub mod states;
mod ui;

fn main() {
    App::new()
        // .insert_resource(ClearColor(Color::srgb_u8(0, 15, 30)))
        .insert_resource(ClearColor(Color::srgb_u8(34, 58, 81)))
        // .insert_resource(ClearColor(Color::linear_rgba(0.012, 0.035, 0.07, 1.0)))
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
                    file_path: "../../assets".to_string(),
                    ..default()
                }),
            MeshPickingPlugin,
        ))
        // State machine
        .init_state::<states::AppState>()
        .add_sub_state::<states::AuthScreen>()
        .add_sub_state::<states::GameView>()
        .add_sub_state::<states::Overlay>()
        //
        .add_plugins((
            camera::CameraPlugin,
            state::ClientStatePlugin,
            grid::GridInputPlugin,
            grid::GridPlugin,
            networking::NetworkingPlugin,
            rendering::terrain::TerrainPlugin,
            rendering::ocean::OceanPlugin,
            rendering::lake::LakePlugin,
            rendering::territory::TerritoryBorderPlugin,
            ui::frosted_glass::FrostedGlassPlugin,
            ui::debug::DebugUiPlugin,
            ui::UiPlugin,
        ))
        .add_plugins((
            // LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
        .run();
}
