use futures::{SinkExt, StreamExt};
use shared::{
    ActionBaseData, ActionContext, ActionData, ActionSpecificTypeEnum, ActionStatusEnum,
    ActionTypeEnum, BuildBuildingAction, BuildRoadAction, CraftResourceAction,
    HarvestResourceAction, MoveUnitAction, SendMessageAction, SpecificAction, SpecificActionData,
    TerrainChunkData,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::action_processor::{ActionInfo, ActionProcessor};
use crate::database::client::DatabaseTables;
use crate::auth::password;
use crate::units::NameGenerator;
use crate::{utils, world};
use shared::protocol::{ClientMessage, ColorData, ServerMessage};

use super::super::Sessions;

/// Helper function to add action to both DB and cache
async fn add_action_and_cache(
    action_table: &crate::database::tables::ScheduledActionsTable,
    action_processor: &ActionProcessor,
    action_data: &ActionData,
    action_type: ActionTypeEnum,
) -> Result<u64, String> {
    // Add to database
    let action_id = action_table.add_scheduled_action(action_data).await?;

    // Add to cache
    let completion_time =
        action_data.base_data.start_time + (action_data.base_data.duration_ms / 1000);
    action_processor
        .add_action(ActionInfo {
            action_id,
            player_id: action_data.base_data.player_id,
            chunk_id: action_data.base_data.chunk.clone(),
            cell: action_data.base_data.cell.clone(),
            action_type,
            status: ActionStatusEnum::Pending,
            start_time: action_data.base_data.start_time,
            duration_ms: action_data.base_data.duration_ms,
            completion_time,
        })
        .await;

    Ok(action_id)
}

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    sessions: Sessions,
    db_tables: Arc<DatabaseTables>,
    action_processor: Arc<ActionProcessor>,
    name_generator: Arc<NameGenerator>,
) {
    tracing::info!("New connection from {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            tracing::error!("WebSocket handshake error: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let session_id = rand::random::<u64>();

    // Créer un channel pour les messages asynchrones (depuis action_processor)
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    sessions.insert(session_id, addr, tx).await;

    // Traiter les messages entrants et sortants
    loop {
        tokio::select! {
            // Messages entrants du client
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        tracing::info!("Received message from {}: {} bytes", addr, data.len());
                        match bincode::decode_from_slice(&data[..], bincode::config::standard()) {
                            Ok((client_msg, _)) => {
                                tracing::debug!("Received: {:?}", client_msg);

                                let responses =
                                    handle_client_message(client_msg, session_id, &sessions, &db_tables, &action_processor, &name_generator).await;

                                // Envoyer les réponses DIRECTEMENT (comme avant)
                                for response in responses {
                                    let response_data =
                                        bincode::encode_to_vec(&response, bincode::config::standard())
                                            .unwrap();
                                    if let Err(e) = write.send(Message::Binary(response_data.into())).await {
                                        tracing::error!("Failed to send direct response: {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to deserialize message from {}\n{}", addr, e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            // Messages asynchrones depuis action_processor
            Some(async_message) = rx.recv() => {
                // Log le type de message sans afficher les données complètes (évite de logger des MB de SDF)
                let message_type = match &async_message {
                    ServerMessage::LoginSuccess { .. } => "LoginSuccess",
                    ServerMessage::LoginError { .. } => "LoginError",
                    ServerMessage::RegisterSuccess{ .. } => "RegisterSuccess",
                    ServerMessage::RegisterError{ .. } => "RegisterError",
                    ServerMessage::TerrainChunkData { .. } => "TerrainChunkData",
                    ServerMessage::OceanData { .. } => "OceanData",
                    ServerMessage::RoadChunkSdfUpdate { chunk_id, .. } => {
                        tracing::info!("Sending RoadChunkSdfUpdate to session {} for chunk ({},{})", session_id, chunk_id.x, chunk_id.y);
                        "RoadChunkSdfUpdate"
                    },
                    ServerMessage::TerritoryBorderSdfUpdate { chunk_id, .. } => {
                        tracing::info!("Sending TerritoryBorderSdfUpdate to session {} for chunk ({},{})", session_id, chunk_id.x, chunk_id.y);
                        "TerritoryBorderSdfUpdate"
                    },
                    ServerMessage::TerritoryContourUpdate { chunk_id, contours } => {
                        tracing::info!("Sending TerritoryContourUpdate to session {} for chunk ({},{}) with {} contours", session_id, chunk_id.x, chunk_id.y, contours.len());
                        "TerritoryContourUpdate"
                    },
                    ServerMessage::TerritoryBorderCells { organization_id, border_cells } => {
                        tracing::info!("Sending TerritoryBorderCells to session {} for org {} ({} cells)", session_id, organization_id, border_cells.len());
                        "TerritoryBorderCells"
                    },
                    ServerMessage::ActionStatusUpdate { .. } => "ActionStatusUpdate",
                    ServerMessage::ActionCompleted { .. } => "ActionCompleted",
                    ServerMessage::ActionSuccess { .. } => "ActionSuccess",
                    ServerMessage::ActionError { .. } => "ActionError",
                    ServerMessage::DebugOrganizationCreated { .. } => "DebugOrganizationCreated",
                    ServerMessage::DebugOrganizationDeleted { .. } => "DebugOrganizationDeleted",
                    ServerMessage::DebugUnitSpawned { .. } => "DebugUnitSpawned",
                    ServerMessage::OrganizationAtCell { .. } => "OrganizationAtCell",
                    ServerMessage::DebugError { .. } => "DebugError",
                    ServerMessage::UnitSlotUpdated { .. } => "UnitSlotUpdated",
                    ServerMessage::Pong => "Pong",
                };

                if !matches!(async_message, ServerMessage::RoadChunkSdfUpdate { .. }) {
                    tracing::debug!("Sending async {} to session {}", message_type, session_id);
                }

                let data = bincode::encode_to_vec(&async_message, bincode::config::standard()).unwrap();
                if let Err(e) = write.send(Message::Binary(data.into())).await {
                    tracing::error!("Failed to send async message: {}", e);
                    break;
                }
            }
        }
    }

    sessions.remove(&session_id).await;
    tracing::info!("Connection closed: {}", addr);
}

async fn handle_client_message(
    msg: ClientMessage,
    session_id: u64,
    sessions: &Sessions,
    db_tables: &DatabaseTables,
    action_processor: &ActionProcessor,
    name_generator: &NameGenerator,
) -> Vec<ServerMessage> {
    match msg {
        ClientMessage::Login { username } => {
            tracing::info!(
                "Session {} attempting to log in as {}",
                session_id,
                username
            );

            // Try to get or create the player in the database
            match shared::types::game::methods::get_or_create_player(
                &db_tables.pool,
                &username,
                1,                  // Default language_id (could be configurable)
                "default_location", // Default origin location
                None,               // No motto by default
            )
            .await
            {
                Ok(player) => {
                    tracing::info!(
                        "Player {} logged in successfully with DB ID {}",
                        username,
                        player.id
                    );

                    // Associate session with player_id
                    sessions
                        .authenticate_session(session_id, player.id as u64)
                        .await;

                    // Get or create a default character
                    match shared::types::game::methods::get_or_create_default_character(
                        &db_tables.pool,
                        player.id,
                        &player.family_name,
                    )
                    .await
                    {
                        Ok(character) => {
                            tracing::info!(
                                "Character {} {} loaded/created",
                                character.first_name,
                                character.family_name
                            );

                            // Convert to protocol types (without timestamps)
                            let player_data = shared::protocol::PlayerData {
                                id: player.id,
                                family_name: player.family_name,
                                language_id: player.language_id,
                                coat_of_arms_id: player.coat_of_arms_id,
                                motto: player.motto,
                                origin_location: player.origin_location,
                            };

                            let character_data = shared::protocol::CharacterData {
                                id: character.id,
                                player_id: character.player_id,
                                first_name: character.first_name,
                                family_name: character.family_name,
                                second_name: character.second_name,
                                nickname: character.nickname,
                                coat_of_arms_id: character.coat_of_arms_id,
                                image_id: character.image_id,
                                motto: character.motto,
                            };

                            vec![ServerMessage::LoginSuccess {
                                player: player_data,
                                character: Some(character_data),
                            }]
                        }
                        Err(e) => {
                            tracing::error!("Failed to get/create character: {}", e);
                            vec![ServerMessage::LoginError {
                                reason: format!("Character creation error: {}", e),
                            }]
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get/create player {}: {}", username, e);
                    vec![ServerMessage::LoginError {
                        reason: format!("Database error: {}", e),
                    }]
                }
            }
        }

        ClientMessage::RegisterAccount {
            family_name,
            password,
        } => {
            tracing::info!(
                "Session {} attempting to register account with family name: {}",
                session_id,
                family_name
            );

            // 1. Validate family name
            if let Err(e) = shared::auth::validate_family_name(&family_name) {
                tracing::warn!("Invalid family name during registration: {}", e);
                return vec![ServerMessage::RegisterError { reason: e }];
            }

            // 2. Validate password
            let requirements = shared::auth::PasswordRequirements::default();
            if let Err(e) = shared::auth::validate_password(&password, &requirements) {
                tracing::warn!("Invalid password during registration: {}", e);
                return vec![ServerMessage::RegisterError { reason: e }];
            }

            // 3. Check if family name already exists
            match shared::types::game::methods::get_player_by_family_name(
                &db_tables.pool,
                &family_name,
            )
            .await
            {
                Ok(Some(_)) => {
                    tracing::warn!(
                        "Registration failed: family name already exists: {}",
                        family_name
                    );
                    vec![ServerMessage::RegisterError {
                        reason: "Ce nom de famille est déjà utilisé".to_string(),
                    }]
                }
                Ok(None) => {
                    // 4. Hash the password
                    let password_hash: String = match password::hash_password(&password) {
                        Ok(hash) => hash,
                        Err(e) => {
                            tracing::error!("Failed to hash password: {}", e);
                            return vec![ServerMessage::RegisterError {
                                reason: "Erreur lors du traitement du mot de passe".to_string(),
                            }];
                        }
                    };

                    // 5. Create player with password
                    match shared::types::game::methods::create_player_with_password(
                        &db_tables.pool,
                        &family_name,
                        1, // Default language_id
                        "default_location",
                        None, // No motto
                        &password_hash,
                    )
                    .await
                    {
                        Ok(player) => {
                            tracing::info!(
                                "Account registered successfully: {} (ID: {})",
                                family_name,
                                player.id
                            );
                            vec![ServerMessage::RegisterSuccess {
                                message: "Compte créé avec succès. Vous pouvez maintenant vous connecter.".to_string(),
                            }]
                        }
                        Err(e) => {
                            tracing::error!("Failed to create player: {}", e);
                            vec![ServerMessage::RegisterError {
                                reason: format!("Erreur lors de la création du compte: {}", e),
                            }]
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Database error during registration: {}", e);
                    vec![ServerMessage::RegisterError {
                        reason: "Erreur de base de données".to_string(),
                    }]
                }
            }
        }

        ClientMessage::LoginWithPassword {
            family_name,
            password,
        } => {
            tracing::info!(
                "Session {} attempting to log in with password as {}",
                session_id,
                family_name
            );

            // 1. Fetch player by family name
            match shared::types::game::methods::get_player_by_family_name(
                &db_tables.pool,
                &family_name,
            )
            .await
            {
                Ok(Some(player)) => {
                    // 2. Check if password_hash exists
                    let password_hash = match &player.password_hash {
                        Some(hash) => hash,
                        None => {
                            tracing::warn!(
                                "Player {} has no password hash (account migration required)",
                                family_name
                            );
                            return vec![ServerMessage::LoginError {
                                reason: "Ce compte nécessite une migration de mot de passe. Veuillez contacter un administrateur.".to_string(),
                            }];
                        }
                    };

                    // 3. Verify password
                    match password::verify_password(&password, password_hash) {
                        Ok(true) => {
                            // Password correct, proceed with login
                            tracing::info!(
                                "Player {} logged in successfully with password authentication",
                                family_name
                            );

                            // Associate session with player_id
                            sessions
                                .authenticate_session(session_id, player.id as u64)
                                .await;

                            // Update last_login_at
                            if let Err(e) = shared::types::game::methods::update_last_login(
                                &db_tables.pool,
                                player.id,
                            )
                            .await
                            {
                                tracing::warn!(
                                    "Failed to update last_login_at for player {}: {}",
                                    player.id,
                                    e
                                );
                            }

                            // Get or create default character
                            match shared::types::game::methods::get_or_create_default_character(
                                &db_tables.pool,
                                player.id,
                                &player.family_name,
                            )
                            .await
                            {
                                Ok(character) => {
                                    tracing::info!(
                                        "Character {} {} loaded/created",
                                        character.first_name,
                                        character.family_name
                                    );

                                    // Convert to protocol types
                                    let player_data = shared::protocol::PlayerData {
                                        id: player.id,
                                        family_name: player.family_name,
                                        language_id: player.language_id,
                                        coat_of_arms_id: player.coat_of_arms_id,
                                        motto: player.motto,
                                        origin_location: player.origin_location,
                                    };

                                    let character_data = shared::protocol::CharacterData {
                                        id: character.id,
                                        player_id: character.player_id,
                                        first_name: character.first_name,
                                        family_name: character.family_name,
                                        second_name: character.second_name,
                                        nickname: character.nickname,
                                        coat_of_arms_id: character.coat_of_arms_id,
                                        image_id: character.image_id,
                                        motto: character.motto,
                                    };

                                    vec![ServerMessage::LoginSuccess {
                                        player: player_data,
                                        character: Some(character_data),
                                    }]
                                }
                                Err(e) => {
                                    tracing::error!("Failed to get/create character: {}", e);
                                    vec![ServerMessage::LoginError {
                                        reason: format!(
                                            "Erreur lors du chargement du personnage: {}",
                                            e
                                        ),
                                    }]
                                }
                            }
                        }
                        Ok(false) => {
                            tracing::warn!(
                                "Failed login attempt for {}: invalid password",
                                family_name
                            );
                            vec![ServerMessage::LoginError {
                                reason: "Identifiants invalides".to_string(),
                            }]
                        }
                        Err(e) => {
                            tracing::error!(
                                "Password verification error for {}: {}",
                                family_name,
                                e
                            );
                            vec![ServerMessage::LoginError {
                                reason: "Erreur lors de l'authentification".to_string(),
                            }]
                        }
                    }
                }
                Ok(None) => {
                    tracing::warn!(
                        "Login attempt with non-existent family name: {}",
                        family_name
                    );
                    // Don't reveal that the account doesn't exist (security)
                    vec![ServerMessage::LoginError {
                        reason: "Identifiants invalides".to_string(),
                    }]
                }
                Err(e) => {
                    tracing::error!("Database error during login for {}: {}", family_name, e);
                    vec![ServerMessage::LoginError {
                        reason: "Erreur de base de données".to_string(),
                    }]
                }
            }
        }

        ClientMessage::RequestTerrainChunks {
            terrain_name,
            terrain_chunk_ids,
        } => {
            let mut responses = Vec::new();
            let terrain_name_ref = &terrain_name;
            for terrain_chunk_id in terrain_chunk_ids.iter() {
                let cell_data = match db_tables.cells.load_chunk_cells(terrain_chunk_id).await {
                    Ok(cells_data) => cells_data,
                    _ => {
                        vec![]
                    }
                };
                let building_data = match db_tables
                    .buildings
                    .load_chunk_buildings(terrain_chunk_id)
                    .await
                {
                    Ok(building_data) => building_data,
                    _ => {
                        vec![]
                    }
                };
                let unit_data = match db_tables.units.load_chunk_units(*terrain_chunk_id).await {
                    Ok(units) => {
                        tracing::info!(
                            "Loaded {} units for chunk ({},{})",
                            units.len(),
                            terrain_chunk_id.x,
                            terrain_chunk_id.y
                        );
                        units
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load units for chunk ({},{}): {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            e
                        );
                        vec![]
                    }
                };
                let (terrain_chunk_data, biome_chunk_data) = match db_tables
                    .terrains
                    .load_terrain(terrain_name_ref, terrain_chunk_id)
                    .await
                {
                    Ok((Some(terrain_chunk_data), Some(biome_chunk_data))) => {
                        (terrain_chunk_data, biome_chunk_data)
                    }
                    Ok((Some(terrain_chunk_data), None)) => (terrain_chunk_data, vec![]),
                    Ok((None, Some(biome_chunk_data))) => (
                        TerrainChunkData {
                            name: terrain_name.clone(),
                            id: *terrain_chunk_id,
                            ..TerrainChunkData::default()
                        },
                        biome_chunk_data,
                    ),
                    Ok((None, None)) => {
                        tracing::error!(
                            "DB error for chunk ({},{}) in terrain {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            terrain_name_ref
                        );

                        (
                            TerrainChunkData {
                                name: terrain_name.clone(),
                                id: *terrain_chunk_id,
                                ..TerrainChunkData::default()
                            },
                            vec![],
                        )
                    }
                    Err(e) => {
                        tracing::error!(
                            "DB error for chunk ({},{}) in terrain {}: {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            terrain_name_ref,
                            e
                        );

                        (
                            TerrainChunkData {
                                name: terrain_name.clone(),
                                id: *terrain_chunk_id,
                                ..TerrainChunkData::default()
                            },
                            vec![],
                        )
                    }
                };

                // Send terrain data first (without roads to keep message size down)
                responses.push(ServerMessage::TerrainChunkData {
                    terrain_chunk_data: terrain_chunk_data.clone(),
                    biome_chunk_data,
                    cell_data,
                    building_data,
                    unit_data,
                });

                // Charger et générer les routes pour ce chunk (envoyé séparément)
                // Utilise la table de visibilité pour charger tous les segments qui traversent ce chunk
                tracing::info!(
                    "Attempting to load road segments for chunk ({},{})",
                    terrain_chunk_id.x,
                    terrain_chunk_id.y
                );
                match db_tables
                    .road_segments
                    .load_road_segments_by_chunk_new(terrain_chunk_id.x, terrain_chunk_id.y)
                    .await
                {
                    Ok(road_segments) if !road_segments.is_empty() => {
                        tracing::info!(
                            "Generating road SDF for chunk ({},{}) with {} segments",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            road_segments.len()
                        );

                        use crate::road::{RoadConfig, compute_intersections, generate_road_sdf};

                        let config = RoadConfig::default();
                        let intersections = compute_intersections(&road_segments, &config);
                        let road_sdf = generate_road_sdf(
                            &road_segments,
                            &intersections,
                            &config,
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                        );

                        tracing::info!(
                            "✓ Road SDF generated: {}x{} with {} intersections",
                            config.sdf_resolution.x,
                            config.sdf_resolution.y,
                            intersections.len()
                        );

                        // Send road data separately to avoid message size limits
                        responses.push(ServerMessage::RoadChunkSdfUpdate {
                            terrain_name: terrain_name.clone(),
                            chunk_id: *terrain_chunk_id,
                            road_sdf_data: road_sdf,
                        });
                    }
                    Ok(road_segments) => {
                        tracing::info!(
                            "No road segments found for chunk ({},{}) (loaded {} segments)",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            road_segments.len()
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load road segments for chunk ({},{}): {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            e
                        );
                    }
                }

                // Load and send territory contours for this chunk (sent separately)
                tracing::info!(
                    "Loading territory contours for chunk ({},{})",
                    terrain_chunk_id.x,
                    terrain_chunk_id.y
                );

                match db_tables
                    .territory_contours
                    .load_chunk_contours(&terrain_chunk_id)
                    .await
                {
                    Ok(territories_chunk_data) if !territories_chunk_data.is_empty() => {
                        tracing::info!(
                            "✓ Found {} organization contours in chunk ({},{})",
                            territories_chunk_data.len(),
                            terrain_chunk_id.x,
                            terrain_chunk_id.y
                        );

                        let mut contour_data = Vec::new();
                        for territory_chunk_data in territories_chunk_data {
                            // Generate colors for this organization
                            let (border_color, fill_color) = world::territory::generate_org_colors(
                                territory_chunk_data.organization_id,
                            );

                            let segment_count = territory_chunk_data.segments.len();

                            contour_data.push(shared::protocol::TerritoryContourChunkData {
                                organization_id: territory_chunk_data.organization_id,
                                chunk_id: *terrain_chunk_id,
                                segments: territory_chunk_data.segments,
                                border_color: ColorData::from_array(border_color),
                                fill_color: ColorData::from_array(fill_color),
                            });

                            tracing::debug!(
                                "Added contour for org {} ({} segments)",
                                territory_chunk_data.organization_id,
                                segment_count
                            );
                        }

                        // Send contours update
                        responses.push(ServerMessage::TerritoryContourUpdate {
                            chunk_id: *terrain_chunk_id,
                            contours: contour_data,
                        });
                    }
                    Ok(_) => {
                        tracing::debug!(
                            "No territory contours in chunk ({},{})",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load territory contours for chunk ({},{}): {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            e
                        );
                    }
                }
            }

            responses
        }

        ClientMessage::ActionBuildBuilding {
            player_id,
            chunk_id,
            cell,
            building_type,
        } => {
            let building_specific_type = building_type.to_specific_type();
            tracing::info!(
                "Player {} requested to build {:?} ({:?}) at chunk ({},{}) cell ({},{})",
                player_id,
                building_type,
                building_specific_type,
                chunk_id.x,
                chunk_id.y,
                cell.q,
                cell.r
            );
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::BuildBuilding(BuildBuildingAction {
                player_id,
                chunk_id,
                cell,
                building_type,
                building_specific_type,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell,
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id,
                    cell,
                    action_type: ActionTypeEnum::BuildBuilding,
                    action_specific_type: ActionSpecificTypeEnum::BuildBuilding,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(
                action_table,
                action_processor,
                &action_data,
                ActionTypeEnum::BuildBuilding,
            )
            .await
            {
                Ok(action_id) => {
                    tracing::info!("Scheduled build building action with ID {}", action_id);

                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id,
                        cell,
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::BuildBuilding,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionBuildRoad {
            player_id,
            start_cell,
            end_cell,
        } => {
            tracing::info!(
                "Player {} requested to build road from ({},{}) to ({},{})",
                player_id,
                start_cell.q,
                start_cell.r,
                end_cell.q,
                end_cell.r
            );
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::BuildRoad(BuildRoadAction {
                player_id,
                start_cell,
                end_cell,
            });

            // Calculer le chunk à partir de la cellule de départ
            use crate::database::tables::RoadSegmentsTable;
            let chunk_id = RoadSegmentsTable::cell_to_chunk_id(&start_cell);

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: start_cell,
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id,
                    cell: start_cell,
                    action_type: ActionTypeEnum::BuildRoad,
                    action_specific_type: ActionSpecificTypeEnum::BuildRoad,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(
                action_table,
                action_processor,
                &action_data,
                ActionTypeEnum::BuildRoad,
            )
            .await
            {
                Ok(action_id) => {
                    tracing::info!("Scheduled build road action {}", action_id);

                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id,
                        cell: start_cell,
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::BuildRoad,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionCraftResource {
            player_id,
            chunk_id,
            cell,
            recipe_id,
            quantity,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::CraftResource(CraftResourceAction {
                player_id,
                recipe_id,
                chunk_id,
                cell,
                quantity,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell,
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id,
                    cell,
                    action_type: ActionTypeEnum::CraftResource,
                    action_specific_type: ActionSpecificTypeEnum::CraftResource,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(
                action_table,
                action_processor,
                &action_data,
                ActionTypeEnum::CraftResource,
            )
            .await
            {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id,
                        cell,
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::CraftResource,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionHarvestResource {
            player_id,
            chunk_id,
            cell,
            resource_specific_type,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::HarvestResource(HarvestResourceAction {
                player_id,
                chunk_id,
                cell,
                resource_specific_type,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell,
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id,
                    cell,
                    action_type: ActionTypeEnum::HarvestResource,
                    action_specific_type: ActionSpecificTypeEnum::HarvestResource,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(
                action_table,
                action_processor,
                &action_data,
                ActionTypeEnum::HarvestResource,
            )
            .await
            {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id,
                        cell,
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::HarvestResource,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionMoveUnit {
            player_id,
            unit_id,
            chunk_id,
            cell,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::MoveUnit(MoveUnitAction {
                player_id,
                unit_id,
                chunk_id,
                cell,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell,
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id,
                    cell,
                    action_type: ActionTypeEnum::MoveUnit,
                    action_specific_type: ActionSpecificTypeEnum::MoveUnit,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(
                action_table,
                action_processor,
                &action_data,
                ActionTypeEnum::MoveUnit,
            )
            .await
            {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id,
                        cell,
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::MoveUnit,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::MoveUnitToSlot {
            unit_id,
            cell,
            from_slot,
            to_slot,
        } => {
            let mut responses = Vec::new();

            tracing::info!(
                "Moving unit {} from slot {:?} to {:?} at cell {:?}",
                unit_id,
                from_slot,
                to_slot,
                cell
            );

            // Convert SlotPosition to DB format
            let slot_type_str = match to_slot.slot_type {
                shared::SlotType::Interior => "interior",
                shared::SlotType::Exterior => "exterior",
            };
            let slot_index = to_slot.index as i32;

            // Update database
            match db_tables
                .units
                .update_slot_position(unit_id, Some(slot_type_str.to_string()), Some(slot_index))
                .await
            {
                Ok(_) => {
                    tracing::info!("Unit {} slot updated in database", unit_id);

                    // Broadcast success to all clients
                    responses.push(ServerMessage::UnitSlotUpdated {
                        unit_id,
                        cell,
                        slot_position: Some(to_slot),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to update unit slot in database: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to update slot position: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::AssignUnitToSlot {
            unit_id,
            cell,
            slot,
        } => {
            let mut responses = Vec::new();

            tracing::info!(
                "Assigning unit {} to slot {:?}:{} at cell {:?}",
                unit_id,
                slot.slot_type,
                slot.index,
                cell
            );

            // Convert SlotPosition to DB format
            let slot_type_str = match slot.slot_type {
                shared::SlotType::Interior => "interior",
                shared::SlotType::Exterior => "exterior",
            };
            let slot_index = slot.index as i32;

            // Update database
            match db_tables
                .units
                .update_slot_position(unit_id, Some(slot_type_str.to_string()), Some(slot_index))
                .await
            {
                Ok(_) => {
                    tracing::info!("Unit {} slot assigned in database", unit_id);

                    // Broadcast success to all clients
                    responses.push(ServerMessage::UnitSlotUpdated {
                        unit_id,
                        cell,
                        slot_position: Some(slot),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to assign unit slot in database: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to assign slot position: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionSendMessage {
            player_id,
            chunk_id,
            cell,
            receivers,
            content,
        } => {
            let mut responses = Vec::new();
            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::SendMessage(SendMessageAction {
                player_id,
                receivers,
                content,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell,
            });

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id,
                    cell,
                    action_type: ActionTypeEnum::SendMessage,
                    action_specific_type: ActionSpecificTypeEnum::SendMessage,
                    start_time,
                    duration_ms,
                    completion_time: start_time + (duration_ms / 1000),
                    status: ActionStatusEnum::Pending,
                },
                specific_data,
            };

            match add_action_and_cache(
                action_table,
                action_processor,
                &action_data,
                ActionTypeEnum::SendMessage,
            )
            .await
            {
                Ok(action_id) => {
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id,
                        cell,
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::SendMessage,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule action: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::RequestOceanData { world_name } => {
            tracing::info!("=== CLIENT REQUESTED OCEAN DATA ===");
            tracing::info!("World name: {}", world_name);

            match db_tables.ocean_data.load_ocean_data(&world_name).await {
                Ok(Some(ocean_data)) => {
                    tracing::info!("Ocean data loaded from database:");
                    tracing::info!("  - width: {}", ocean_data.width);
                    tracing::info!("  - height: {}", ocean_data.height);
                    tracing::info!("  - sdf_values: {} bytes", ocean_data.sdf_values.len());
                    tracing::info!(
                        "  - heightmap_values: {} bytes",
                        ocean_data.heightmap_values.len()
                    );

                    tracing::info!("Encoding ocean data with bincode for network transmission...");
                    let encoded_size =
                        bincode::encode_to_vec(&ocean_data, bincode::config::standard())
                            .map(|v| v.len())
                            .unwrap_or(0);
                    tracing::info!(
                        "Encoded size: {} bytes ({:.2} MB)",
                        encoded_size,
                        encoded_size as f64 / 1_000_000.0
                    );

                    tracing::info!("✓ Sending ocean data to client");
                    vec![ServerMessage::OceanData { ocean_data }]
                }
                Ok(None) => {
                    tracing::warn!("No ocean data found for world: {}", world_name);
                    vec![]
                }
                Err(e) => {
                    tracing::error!("Failed to load ocean data: {}", e);
                    vec![]
                }
            }
        }

        // ====================================================================
        // DEBUG COMMANDS
        // ====================================================================
        ClientMessage::DebugCreateOrganization {
            name,
            organization_type,
            cell,
            parent_organization_id,
        } => {
            tracing::info!(
                "DEBUG: Creating organization '{}' of type {:?} at {:?}",
                name,
                organization_type,
                cell
            );

            // First, create a leader unit for the organization
            let (first_name, last_name, gender, portrait_variant_id, avatar_url) = {
                use crate::units::PortraitGenerator;
                use rand::Rng;
                let mut rng = rand::rng();

                // Generate random gender
                let is_male = rng.random_bool(0.5);
                let gender_str = if is_male { "male" } else { "female" };

                // Use NameGenerator to get realistic names based on gender
                let (first_name, last_name) = name_generator.generate_random_name(Some(is_male));

                // Generate portrait for Merchant profession
                let (variant_id, avatar_url) = PortraitGenerator::generate_variant_and_url(
                    gender_str,
                    shared::ProfessionEnum::Merchant,
                );

                (
                    first_name,
                    last_name,
                    gender_str.to_string(),
                    variant_id,
                    avatar_url,
                )
            };

            // Create the leader unit
            let founder_unit_result = db_tables
                .units
                .create_unit(
                    None,
                    first_name.clone(),
                    last_name.clone(),
                    gender.clone(),
                    portrait_variant_id,
                    avatar_url,
                    cell,
                    shared::TerrainChunkId { x: 0, y: 0 },
                    shared::ProfessionEnum::Merchant,
                )
                .await;

            match founder_unit_result {
                Ok(founder_unit_id) => {
                    let request = shared::CreateOrganizationRequest {
                        name: name.clone(),
                        organization_type,
                        headquarters_cell: Some(cell),
                        parent_organization_id,
                        founder_unit_id,
                    };

                    match db_tables.organizations.create_organization(request).await {
                        Ok(org_id) => {
                            tracing::info!("✓ Organization created with ID {}", org_id);

                            // Try to claim entire Voronoi zone
                            let mut claimed_count = 0;

                            match db_tables.voronoi_zones.get_zone_at_cell(cell).await {
                                Ok(Some(zone_id)) => {
                                    tracing::info!(
                                        "Found Voronoi zone {} at cell {:?}",
                                        zone_id,
                                        cell
                                    );

                                    // Check if zone is available
                                    match db_tables.voronoi_zones.is_zone_available(zone_id).await {
                                        Ok(true) => {
                                            // Zone is available, claim all cells
                                            match db_tables
                                                .voronoi_zones
                                                .get_zone_cells(zone_id)
                                                .await
                                            {
                                                Ok(zone_cells) => {
                                                    tracing::info!(
                                                        "Claiming {} cells from Voronoi zone {}",
                                                        zone_cells.len(),
                                                        zone_id
                                                    );

                                                    // Add all cells to territory
                                                    for zone_cell in &zone_cells {
                                                        if let Err(e) = db_tables
                                                            .organizations
                                                            .add_territory_cell(org_id, zone_cell)
                                                            .await
                                                        {
                                                            tracing::warn!(
                                                                "Failed to add cell {:?}: {}",
                                                                zone_cell,
                                                                e
                                                            );
                                                        } else {
                                                            claimed_count += 1;
                                                        }
                                                    }

                                                    // Link organization to zone
                                                    let link_result = sqlx::query(
                                                        "UPDATE organizations.organizations SET voronoi_zone_id = $1 WHERE id = $2"
                                                    )
                                                    .bind(zone_id)
                                                    .bind(org_id as i64)
                                                    .execute(&db_tables.pool)
                                                    .await;

                                                    if let Err(e) = link_result {
                                                        tracing::warn!(
                                                            "Failed to link zone to organization: {}",
                                                            e
                                                        );
                                                    }

                                                    tracing::info!(
                                                        "✓ Organization {} claimed {} cells from Voronoi zone {}",
                                                        org_id,
                                                        claimed_count,
                                                        zone_id
                                                    );
                                                }
                                                Err(e) => {
                                                    tracing::error!(
                                                        "Failed to get zone cells: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        Ok(false) => {
                                            tracing::warn!(
                                                "Voronoi zone {} already claimed, using fallback",
                                                zone_id
                                            );
                                            // Fallback: just claim HQ and neighbors
                                            for neighbor in cell.neighbors() {
                                                let _ = db_tables
                                                    .organizations
                                                    .add_territory_cell(org_id, &neighbor)
                                                    .await;
                                                claimed_count += 1;
                                            }
                                            let _ = db_tables
                                                .organizations
                                                .add_territory_cell(org_id, &cell)
                                                .await;
                                            claimed_count += 1;
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "Failed to check zone availability: {}",
                                                e
                                            );
                                            // Fallback
                                            for neighbor in cell.neighbors() {
                                                let _ = db_tables
                                                    .organizations
                                                    .add_territory_cell(org_id, &neighbor)
                                                    .await;
                                                claimed_count += 1;
                                            }
                                            let _ = db_tables
                                                .organizations
                                                .add_territory_cell(org_id, &cell)
                                                .await;
                                            claimed_count += 1;
                                        }
                                    }
                                }
                                Ok(None) => {
                                    tracing::warn!(
                                        "No Voronoi zone found at cell {:?}, using fallback",
                                        cell
                                    );
                                    // Fallback: just claim HQ and neighbors
                                    for neighbor in cell.neighbors() {
                                        let _ = db_tables
                                            .organizations
                                            .add_territory_cell(org_id, &neighbor)
                                            .await;
                                        claimed_count += 1;
                                    }
                                    let _ = db_tables
                                        .organizations
                                        .add_territory_cell(org_id, &cell)
                                        .await;
                                    claimed_count += 1;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to query Voronoi zone: {}", e);
                                    // Fallback
                                    for neighbor in cell.neighbors() {
                                        let _ = db_tables
                                            .organizations
                                            .add_territory_cell(org_id, &neighbor)
                                            .await;
                                        claimed_count += 1;
                                    }
                                    let _ = db_tables
                                        .organizations
                                        .add_territory_cell(org_id, &cell)
                                        .await;
                                    claimed_count += 1;
                                }
                            }

                            tracing::info!(
                                "✓ Organization {} claimed {} total cells",
                                org_id,
                                claimed_count
                            );

                            // Generate territory contours and split by chunks
                            tracing::info!(
                                "Generating territory contours for organization {}...",
                                org_id
                            );
                            match db_tables.organizations.load_territory_cells(org_id).await {
                                Ok(territory_cells) if !territory_cells.is_empty() => {
                                    tracing::info!(
                                        "Loaded {} territory cells for organization {}",
                                        territory_cells.len(),
                                        org_id
                                    );

                                    // Convert GridCell to Hex
                                    use hexx::Hex;
                                    let territory_hex: std::collections::HashSet<Hex> =
                                        territory_cells.iter().map(|cell| cell.to_hex()).collect();

                                    // Use grid config for layout
                                    use shared::grid::GridConfig;
                                    let grid_config = GridConfig::new(
                                        shared::constants::HEX_SIZE,
                                        hexx::HexOrientation::Flat,
                                        bevy::math::Vec2::new(
                                            shared::constants::HEX_RATIO.x,
                                            shared::constants::HEX_RATIO.y,
                                        ),
                                        3,
                                    );

                                    let contour_points = &world::territory::build_contour(
                                        &grid_config.layout,
                                        &territory_hex,
                                        0.0,
                                        12345,
                                    );

                                    // Generate and split contours
                                    let contour_chunks =
                                        utils::chunks::split_contour_into_chunks(contour_points);
                                    // let contour_chunks = crate::world::territory::generate_and_split_contour(
                                    //     &territory_hex,
                                    //     &grid_config.layout,
                                    //     2.0,    // jitter amplitude
                                    //     org_id, // jitter seed (ensures consistency)
                                    // );

                                    tracing::info!(
                                        "Generated {} contour chunks for organization {}",
                                        contour_chunks.len(),
                                        org_id
                                    );

                                    // Store contours in database
                                    let mut stored_count = 0;
                                    for (chunk_id, contour_segments) in contour_chunks {
                                        match db_tables
                                            .territory_contours
                                            .store_contour(
                                                org_id,
                                                chunk_id.x,
                                                chunk_id.y,
                                                &contour_segments,
                                            )
                                            .await
                                        {
                                            Ok(_) => {
                                                stored_count += 1;
                                                tracing::debug!(
                                                    "Stored contour for org {} in chunk ({},{})",
                                                    org_id,
                                                    chunk_id.x,
                                                    chunk_id.y
                                                );
                                            }
                                            Err(e) => {
                                                tracing::warn!(
                                                    "Failed to store contour for chunk ({},{}): {}",
                                                    chunk_id.x,
                                                    chunk_id.y,
                                                    e
                                                );
                                            }
                                        }
                                    }

                                    tracing::info!(
                                        "✓ Stored {} territory contour chunks for organization {}",
                                        stored_count,
                                        org_id
                                    );
                                }
                                Ok(_) => {
                                    tracing::warn!(
                                        "No territory cells found for organization {}",
                                        org_id
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to load territory cells for organization {}: {}",
                                        org_id,
                                        e
                                    );
                                }
                            }

                            // Get territory border cells for debugging
                            let mut response_messages =
                                vec![ServerMessage::DebugOrganizationCreated {
                                    organization_id: org_id,
                                    name: name.clone(),
                                }];

                            // Get border cells (cells at the frontier of the territory)
                            tracing::info!(
                                "Getting territory border cells for organization {}...",
                                org_id
                            );
                            match crate::world::territory::get_territory_border_cells(
                                &db_tables.pool,
                                org_id,
                            )
                            .await
                            {
                                Ok(border_cells) => {
                                    tracing::info!(
                                        "Organization {}: {} border cells identified",
                                        org_id,
                                        border_cells.len()
                                    );

                                    // Send border cells to client for debug visualization
                                    response_messages.push(ServerMessage::TerritoryBorderCells {
                                        organization_id: org_id,
                                        border_cells,
                                    });
                                }
                                Err(e) => {
                                    tracing::error!("Failed to get territory border cells: {}", e);
                                }
                            }

                            response_messages
                        }
                        Err(e) => {
                            tracing::error!("✗ Failed to create organization: {}", e);
                            vec![ServerMessage::DebugError {
                                reason: format!("Failed to create organization: {}", e),
                            }]
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("✗ Failed to create leader unit: {}", e);
                    vec![ServerMessage::DebugError {
                        reason: format!("Failed to create leader unit: {}", e),
                    }]
                }
            }
        }

        ClientMessage::DebugDeleteOrganization { organization_id } => {
            tracing::info!("DEBUG: Deleting organization ID {}", organization_id);

            match sqlx::query("DELETE FROM organizations.organizations WHERE id = $1")
                .bind(organization_id as i64)
                .execute(&db_tables.pool)
                .await
            {
                Ok(_) => {
                    tracing::info!("✓ Organization {} deleted", organization_id);
                    vec![ServerMessage::DebugOrganizationDeleted { organization_id }]
                }
                Err(e) => {
                    tracing::error!("✗ Failed to delete organization: {}", e);
                    vec![ServerMessage::DebugError {
                        reason: format!("Failed to delete organization: {}", e),
                    }]
                }
            }
        }

        ClientMessage::DebugSpawnUnit { cell } => {
            tracing::info!("DEBUG: Spawning random unit at {:?}", cell);

            // Check slot availability before spawning
            let default_chunk = shared::TerrainChunkId { x: 0, y: 0 };

            // Get occupied slots on the cell
            let occupied_slots = match db_tables
                .units
                .get_occupied_slots_on_cell(&cell, &default_chunk)
                .await
            {
                Ok(slots) => slots,
                Err(e) => {
                    tracing::error!("✗ Failed to get occupied slots: {}", e);
                    return vec![ServerMessage::DebugError {
                        reason: format!("Failed to get occupied slots: {}", e),
                    }];
                }
            };

            // Determine slot configuration based on building type or terrain biome
            let slot_config = {
                use shared::SlotConfiguration;

                // Try to get building type first
                match db_tables.buildings.get_building_type_at_cell(&cell).await {
                    Ok(Some(building_type)) => SlotConfiguration::for_building_type(building_type),
                    Ok(None) => {
                        // No building, check terrain biome
                        match db_tables.cells.get_biome_at_cell(&cell).await {
                            Ok(Some(biome)) => SlotConfiguration::for_terrain_type(biome),
                            Ok(None) => {
                                tracing::warn!(
                                    "No biome found for cell {:?}, using default config",
                                    cell
                                );
                                SlotConfiguration::default()
                            }
                            Err(e) => {
                                tracing::error!("✗ Failed to get biome: {}", e);
                                return vec![ServerMessage::DebugError {
                                    reason: format!("Failed to get biome: {}", e),
                                }];
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("✗ Failed to get building type: {}", e);
                        return vec![ServerMessage::DebugError {
                            reason: format!("Failed to get building type: {}", e),
                        }];
                    }
                }
            };

            let total_slots = slot_config.total_slots();

            // Generate list of all available slots
            let mut all_slots = Vec::new();

            // Add all interior slots
            for i in 0..slot_config.interior_slots() {
                all_slots.push(shared::SlotPosition {
                    slot_type: shared::SlotType::Interior,
                    index: i,
                });
            }

            // Add all exterior slots
            for i in 0..slot_config.exterior_slots() {
                all_slots.push(shared::SlotPosition {
                    slot_type: shared::SlotType::Exterior,
                    index: i,
                });
            }

            // Filter out occupied slots to get available slots
            let available_slots: Vec<_> = all_slots
                .into_iter()
                .filter(|slot| !occupied_slots.contains(slot))
                .collect();

            // Check if there's space available
            if available_slots.is_empty() {
                tracing::warn!(
                    "✗ Cannot spawn unit: cell {:?} is full ({}/{} slots occupied)",
                    cell,
                    occupied_slots.len(),
                    total_slots
                );
                return vec![ServerMessage::DebugError {
                    reason: format!(
                        "Cell is full ({}/{} slots occupied)",
                        occupied_slots.len(),
                        total_slots
                    ),
                }];
            }

            tracing::info!(
                "Cell has available slots: {}/{} occupied",
                occupied_slots.len(),
                total_slots
            );

            // Generate random unit data using NameGenerator and PortraitGenerator
            let (first_name, last_name, gender, profession, portrait_variant_id, avatar_url) = {
                use crate::units::PortraitGenerator;
                use rand::Rng;
                let mut rng = rand::rng();

                // Generate random gender (true = male, false = female)
                let is_male = rng.random_bool(0.5);
                let gender_str = if is_male { "male" } else { "female" };

                // Use NameGenerator to get realistic names based on gender
                let (first_name, last_name) = name_generator.generate_random_name(Some(is_male));

                // Generate random profession
                let profession_id: i16 = rng.random_range(1..=16);
                let profession = shared::ProfessionEnum::from_id(profession_id)
                    .unwrap_or(shared::ProfessionEnum::Farmer);

                // Generate portrait variant and avatar URL
                let (variant_id, avatar_url) =
                    PortraitGenerator::generate_variant_and_url(gender_str, profession);

                (
                    first_name,
                    last_name,
                    gender_str.to_string(),
                    profession,
                    variant_id,
                    avatar_url,
                )
            };

            // Choose a random available slot
            let chosen_slot = {
                use rand::Rng;
                let mut rng = rand::rng();
                let index = rng.random_range(0..available_slots.len());
                available_slots[index]
            };

            match db_tables
                .units
                .create_unit(
                    None, // No player
                    first_name.clone(),
                    last_name.clone(),
                    gender.clone(),
                    portrait_variant_id.clone(),
                    avatar_url.clone(),
                    cell,
                    default_chunk,
                    profession,
                )
                .await
            {
                Ok(unit_id) => {
                    // Assign the chosen slot to the unit
                    let slot_type_str = match chosen_slot.slot_type {
                        shared::SlotType::Interior => "interior",
                        shared::SlotType::Exterior => "exterior",
                    };

                    match db_tables
                        .units
                        .update_slot_position(
                            unit_id,
                            Some(slot_type_str.to_string()),
                            Some(chosen_slot.index as i32),
                        )
                        .await
                    {
                        Ok(_) => {
                            tracing::info!(
                                "✓ Unit spawned: {} {} ({}, {}, ID: {}) - Assigned to slot {:?} {} - {}/{} slots occupied",
                                first_name,
                                last_name,
                                gender,
                                avatar_url,
                                unit_id,
                                slot_type_str,
                                chosen_slot.index,
                                occupied_slots.len() + 1,
                                total_slots
                            );

                            // Load the full unit data to send to the client
                            match db_tables.units.load_unit(unit_id).await {
                                Ok(unit_data) => {
                                    vec![ServerMessage::DebugUnitSpawned { unit_data }]
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "✗ Failed to load unit data after spawn: {}",
                                        e
                                    );
                                    vec![ServerMessage::DebugError {
                                        reason: format!(
                                            "Unit created but failed to load data: {}",
                                            e
                                        ),
                                    }]
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("✗ Failed to assign slot to unit: {}", e);
                            // Unit was created but slot assignment failed
                            // We could delete the unit here, or just warn
                            vec![ServerMessage::DebugError {
                                reason: format!("Unit created but slot assignment failed: {}", e),
                            }]
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("✗ Failed to spawn unit: {}", e);
                    vec![ServerMessage::DebugError {
                        reason: format!("Failed to spawn unit: {}", e),
                    }]
                }
            }
        }

        ClientMessage::RequestOrganizationAtCell { cell } => {
            tracing::debug!("Checking organization at cell {:?}", cell);

            // Query to find which organization owns this cell
            match sqlx::query_as::<_, (i64,)>(
                "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2"
            )
            .bind(cell.q)
            .bind(cell.r)
            .fetch_optional(&db_tables.pool)
            .await
            {
                Ok(Some((org_id,))) => {
                    // Load organization summary
                    match db_tables.organizations.load_organization(org_id as u64).await {
                        Ok(org_data) => {
                            let summary = shared::OrganizationSummary {
                                id: org_data.id,
                                name: org_data.name,
                                organization_type: org_data.organization_type,
                                leader_unit_id: org_data.leader_unit_id,
                                population: org_data.population,
                                emblem_url: org_data.emblem_url,
                            };
                            vec![ServerMessage::OrganizationAtCell {
                                cell,
                                organization: Some(summary),
                            }]
                        }
                        Err(e) => {
                            tracing::error!("Failed to load organization: {}", e);
                            vec![ServerMessage::OrganizationAtCell {
                                cell,
                                organization: None,
                            }]
                        }
                    }
                }
                Ok(None) => {
                    // No organization at this cell
                    vec![ServerMessage::OrganizationAtCell {
                        cell,
                        organization: None,
                    }]
                }
                Err(e) => {
                    tracing::error!("Database error checking organization: {}", e);
                    vec![ServerMessage::OrganizationAtCell {
                        cell,
                        organization: None,
                    }]
                }
            }
        }

        ClientMessage::Ping => vec![ServerMessage::Pong],
        _ => vec![ServerMessage::Pong],
    }
}

pub async fn broadcast_message(sessions: Sessions, msg: ServerMessage) {
    let count = sessions.count().await;
    tracing::debug!("Broadcasting message to {} sessions: {:?}", count, msg);
    // TODO: implement proper broadcasting
}
