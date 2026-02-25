use std::sync::Arc;

use bevy::prelude::*;
// use shared::GameState;

mod auth;
mod action_processor;
mod database;
mod networking;
mod road;
mod units;
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
        || args.contains(&"--regen-territory".to_string())
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
    else if args.contains(&"--regen-territory".to_string()) {
        tracing::info!("=== Starting Territory Contours Regeneration ===");
        world::systems::regenerate_territory_contours(&db_tables).await;
        tracing::info!("=== Regeneration Complete - Exiting ===");
        return;
    }

    let sessions = networking::Sessions::default();
    let db_tables_arc = Arc::new(db_tables);

    // Charger le générateur de noms
    let name_generator = match units::NameGenerator::load_from_files() {
        Ok(generator) => Arc::new(generator),
        Err(e) => {
            tracing::error!("Failed to load name generator: {}", e);
            tracing::warn!("Using fallback name generation");
            return;
        }
    };

    // Créer le processeur d'actions AVANT d'initialiser le serveur
    let action_processor = Arc::new(action_processor::ActionProcessor::new(
        db_tables_arc.clone(),
        sessions.clone()
    ));

    // Charger les actions actives au démarrage
    if let Err(e) = action_processor.load_active_actions().await {
        tracing::error!("Failed to load active actions: {}", e);
    }

    // Initialiser le serveur réseau avec l'action_processor et name_generator
    networking::server::initialize_server(
        sessions.clone(),
        db_tables_arc.clone(),
        action_processor.clone(),
        name_generator.clone()
    );

    // Démarrer le processeur d'actions en arrière-plan
    action_processor::start_action_processor(action_processor.clone());

    tokio::task::spawn_blocking(move || {
        App::new()
            .add_plugins(MinimalPlugins)
            .insert_resource(sessions)
            .run()
    })
    .await
    .expect("Failed to start server");
}
