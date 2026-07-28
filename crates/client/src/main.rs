use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use std::time::Duration;

use crate::networking::client::auth_task::AuthTask;

mod camera;
mod grid;
mod networking;
mod rendering;
mod state;
pub mod states;
mod ui;

fn main() {
    // Load .env (walks up parent dirs) so AUTH_HTTP_URL / AUTH_HTTP_HOST are picked up
    dotenv::dotenv().ok();

    // Truncate the log file on each launch
    let _ = std::fs::write("client.log", "");

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
                })
                .set(bevy::log::LogPlugin {
                    filter: "client=info,wgpu=error,bevy_render=error,naga=warn,lightyear=warn".to_string(),
                    level: bevy::log::Level::INFO,
                    custom_layer: |_app| {
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open("client.log")
                            .expect("Failed to open client.log");

                        Some(Box::new(
                            tracing_subscriber::fmt::layer()
                                .with_writer(std::sync::Mutex::new(file))
                                .with_ansi(false),
                        ))
                    },
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
        .init_resource::<AuthTask>()
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
            rendering::mist::MistPlugin,
            ui::frosted_glass::FrostedGlassPlugin,
            ui::debug::DebugUiPlugin,
            ui::UiPlugin,
        ))
        // 🔧 LIGHTYEAR: Client plugins + protocol
        .add_plugins(lightyear::prelude::client::ClientPlugins {
            tick_duration: Duration::from_millis(50), // 20Hz, must match server
        })
        .add_plugins(shared::protocol::plugin::ProtocolPlugin)
        .add_plugins(networking::client::game_client::GameClientPlugin)
        //
        .add_plugins((
            // LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
        .run();
}
