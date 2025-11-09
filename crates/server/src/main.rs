use std::sync::Arc;

use bevy::prelude::*;
use shared::GameState;

mod database;
mod networking;
mod utils;
mod world;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();

    let (db_tables, game_state) = database::client::initialize_database().await;

    let mut map_name = "test_island";

    if args.contains(&"--regenerate-world".to_string())
        || args.contains(&"--generate-world".to_string())
        || args.contains(&"--clear".to_string())
        || args.contains(&"--save-png".to_string())
    {
        let result = args
            .iter()
            .find(|arg| arg.starts_with("--map="))
            .map(|arg| {
                map_name = arg.trim_start_matches("--map=");
                tracing::info!("Using map: {}", map_name);
                // Here you would set the map to be used in generation
                map_name
            });
        if result.is_none() {
            tracing::warn!(
                "--map flag provided but no map name found, using default: {}",
                map_name
            );
        }
        map_name = result.unwrap_or(map_name);
    }

    if args.contains(&"--clear".to_string()) {
        tracing::info!("=== Starting World Cleaning ===");
        world::systems::clear_world(map_name, &db_tables.terrains).await;
        tracing::info!("=== Cleaning Complete - Exiting ===");
        return;
    }
    else if args.contains(&"--save-png".to_string()) {
        tracing::info!("=== Starting Map png saving ===");
        world::systems::save_world_to_png(map_name).await;
        tracing::info!("=== Saving Complete - Exiting ===");
        return;
    }
    else if args.contains(&"--generate-world".to_string()) {
        tracing::info!("=== Starting World Generation ===");        
        // World generation
        world::systems::generate_world(map_name, &db_tables, &game_state).await;
        tracing::info!("=== Generation Complete - Exiting ===");
        return;
    }

    let sessions = networking::Sessions::default();

    networking::server::initialize_server(
        sessions.clone(),
        Arc::new(db_tables)
    );

    tokio::task::spawn_blocking(|| {
        App::new()
            .add_plugins(MinimalPlugins)
            .insert_resource(sessions)
            .run()
    })
    .await
    .expect("Failed to start server");
}
