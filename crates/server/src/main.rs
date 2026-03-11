use std::sync::Arc;

use bevy::prelude::*;
// use shared::GameState;

mod action_processor;
mod auth;
mod database;
mod dev;
mod networking;
mod population;
mod road;
mod units;
mod utils;
mod world;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let dev_config = dev::DevConfig::from_env();
    if dev_config.dev_mode {
        tracing::warn!(
            "⚡ DEV MODE ACTIVE — speed: {}x, bypass resources: {}",
            dev_config.speed_factor,
            dev_config.bypass_resources
        );
    }
    let dev_config_arc = Arc::new(dev_config);

    let args: Vec<String> = std::env::args().collect();

    // Résoudre le chemin du seed tool relativement à la racine du projet
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    // CARGO_MANIFEST_DIR pointe sur crates/server/, donc on remonte de 2 niveaux
    let seed_dir = manifest_dir.join("../../tools/game_seed");

    if !seed_dir.exists() {
        tracing::warn!(
            "Seed directory not found at {}, skipping game-seed",
            seed_dir.display()
        );
    } else {
        let status = std::process::Command::new("uv")
            .args(["run", "game-seed"])
            .current_dir(&seed_dir)
            .status()
            .expect("Failed to run game-seed");

        if !status.success() {
            tracing::error!("Game seed failed with status {}", status);
            return;
        }
    }

    let (db_tables, game_state) = database::client::initialize_database().await;

    // Fix chunk assignments using hex layout
    let grid_config = world::systems::setup_grid_config();
    utils::chunks::fix_chunk_assignments(&db_tables.pool, &grid_config.layout).await;
    utils::portraits::fix_avatar_urls(&db_tables.pool).await;

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
    } else if args.contains(&"--save-png".to_string()) {
        tracing::info!("=== Starting Map png saving ===");
        world::systems::save_world_to_png(map_name).await;
        tracing::info!("=== Saving Complete - Exiting ===");
        return;
    } else if args.contains(&"--generate-world".to_string()) {
        tracing::info!("=== Starting World Generation ===");
        // World generation
        world::systems::generate_world(map_name, &db_tables, &game_state).await;
        tracing::info!("=== Generation Complete - Exiting ===");
        return;
    } else if args.contains(&"--regen-territory".to_string()) {
        tracing::info!("=== Starting Territory Contours Regeneration ===");
        world::systems::regenerate_territory_contours(&db_tables).await;
        tracing::info!("=== Regeneration Complete - Exiting ===");
        return;
    }

    let sessions = networking::Sessions::default();
    let db_tables_arc = Arc::new(db_tables);
    let game_state_arc = Arc::new(game_state);
    let grid_config_arc = Arc::new(grid_config);

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
        sessions.clone(),
        game_state_arc.clone(),
        grid_config_arc.clone(),
        dev_config_arc.clone(),
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
        name_generator.clone(),
        game_state_arc.clone(),
        grid_config_arc.clone(),
        dev_config_arc.clone(),
    );

    // Démarrer le processeur d'actions en arrière-plan
    action_processor::start_action_processor(action_processor.clone());

    // Démarrer le système de population en arrière-plan
    let population_system = Arc::new(population::PopulationSystem::new(
        db_tables_arc.clone(),
        sessions.clone(),
        name_generator.clone(),
    ));
    population::start_population_tick(population_system);

    tokio::task::spawn_blocking(move || {
        App::new()
            .add_plugins(MinimalPlugins)
            .insert_resource(sessions)
            .run()
    })
    .await
    .expect("Failed to start server");
}
