use futures::{SinkExt, StreamExt};
use shared::grid::{GridCell, GridConfig};
use shared::{
    ActionBaseData, ActionContext, ActionData, ActionSpecificTypeEnum, ActionStatusEnum,
    ActionTypeEnum, BuildBuildingAction, BuildRoadAction, ContourSegmentData, CraftResourceAction,
    HarvestResourceAction, MoveUnitAction, SendMessageAction, SpecificAction, SpecificActionData,
    TerrainChunkData, TrainUnitAction,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::action_processor::{ActionInfo, ActionProcessor};
use crate::auth::password;
use crate::database::client::DatabaseTables;
use crate::dev::DevConfig;
use crate::units::NameGenerator;
use crate::{utils, world};
use shared::GameState;
use shared::protocol::{
    ClientMessage, ColorData, ConstructionCostNet, GameDataPayload, HarvestYieldNet,
    ItemDefinitionNet, RecipeIngredientNet, RecipeNet, ServerMessage, TranslationEntry,
};

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

/// Claim une cellule et ses 6 voisins pour une organisation (fallback si pas de Voronoï)
async fn claim_cell_and_neighbors(
    db_tables: &DatabaseTables,
    org_id: u64,
    center: &shared::grid::GridCell,
    claimed_cells: &mut Vec<shared::grid::GridCell>,
) {
    // Cellule centrale
    if db_tables
        .organizations
        .add_territory_cell(org_id, center)
        .await
        .is_ok()
    {
        claimed_cells.push(*center);
    }

    // 6 voisins directs
    for neighbor in center.neighbors() {
        // Vérifier que le voisin n'est pas déjà pris
        let already_taken = sqlx::query_scalar::<_, i64>(
            "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2"
        )
        .bind(neighbor.q)
        .bind(neighbor.r)
        .fetch_optional(&db_tables.pool)
        .await
        .ok()
        .flatten()
        .is_some();

        if !already_taken {
            if db_tables
                .organizations
                .add_territory_cell(org_id, &neighbor)
                .await
                .is_ok()
            {
                claimed_cells.push(neighbor);
            }
        }
    }
}

/// Build the GameDataPayload from the cached GameState
fn build_game_data_payload(game_state: &GameState, dev_config: &DevConfig) -> GameDataPayload {
    let items = game_state
        .item_definitions
        .iter()
        .map(|i| ItemDefinitionNet {
            id: i.id,
            name: i.name.clone(),
            item_type_id: i.item_type.to_id(),
            category_id: i.category.map(|c| c.to_id()),
            weight_kg: i.weight_kg,
            base_price: i.base_price,
            is_perishable: i.is_perishable,
            is_equipable: i.is_equipable,
            equipment_slot_id: i.equipment_slot.map(|s| s.to_id()),
            is_craftable: i.is_craftable,
        })
        .collect();

    let recipes = game_state
        .recipes
        .iter()
        .map(|r| RecipeNet {
            id: r.id,
            name: r.name.clone(),
            result_item_id: r.result_item_id,
            result_quantity: r.result_quantity,
            required_skill_id: r.required_skill.map(|s| s.to_id()),
            required_skill_level: r.required_skill_level,
            craft_duration_seconds: r.craft_duration_seconds,
            required_building_type_id: r.required_building_type_id,
            ingredients: r
                .ingredients
                .iter()
                .map(|ing| RecipeIngredientNet {
                    item_id: ing.item_id,
                    quantity: ing.quantity,
                })
                .collect(),
        })
        .collect();

    let construction_costs = game_state
        .construction_costs
        .iter()
        .flat_map(|(_, costs)| {
            costs.iter().map(|c| ConstructionCostNet {
                building_type_id: c.building_type_id,
                item_id: c.item_id,
                quantity: c.quantity,
            })
        })
        .collect();

    let harvest_yields = game_state
        .harvest_yields
        .iter()
        .map(|h| HarvestYieldNet {
            resource_specific_type_id: h.resource_specific_type_id,
            result_item_id: h.result_item_id,
            base_quantity: h.base_quantity,
            required_profession_id: h.required_profession_id,
            duration_seconds: h.duration_seconds,
        })
        .collect();

    let translations = game_state
        .translations
        .iter()
        .map(|(k, v)| TranslationEntry {
            entity_type: k.entity_type.clone(),
            entity_id: k.entity_id,
            language_id: k.language_id,
            field: k.field.clone(),
            value: v.clone(),
        })
        .collect();

    GameDataPayload {
        items,
        recipes,
        construction_costs,
        harvest_yields,
        translations,
        dev_mode: dev_config.dev_mode,
    }
}

/// Check if a player controls a unit, either directly (lord) or via organization membership.
async fn player_controls_unit(db_tables: &DatabaseTables, player_id: u64, unit_id: u64) -> bool {
    // 1. Direct ownership (lord)
    let unit = match db_tables.units.load_unit(unit_id).await {
        Ok(u) => u,
        Err(_) => return false,
    };

    if unit.player_id == Some(player_id) {
        return true;
    }

    // 2. Organization membership — unit belongs to an org led by the player's lord
    let result = sqlx::query(
        r#"
        SELECT 1 FROM organizations.members om
        JOIN organizations.organizations o ON o.id = om.organization_id
        JOIN units.units lord ON lord.id = o.leader_unit_id
        WHERE om.unit_id = $1 AND lord.player_id = $2 AND lord.is_lord = true
        LIMIT 1
        "#,
    )
    .bind(unit_id as i64)
    .bind(player_id as i64)
    .fetch_optional(&db_tables.pool)
    .await;

    matches!(result, Ok(Some(_)))
}

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    sessions: Sessions,
    db_tables: Arc<DatabaseTables>,
    action_processor: Arc<ActionProcessor>,
    name_generator: Arc<NameGenerator>,
    game_state: Arc<GameState>,
    grid_config: Arc<GridConfig>,
    dev_config: Arc<DevConfig>,
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
                                    handle_client_message(client_msg, session_id, &sessions, &db_tables, &action_processor, &name_generator, &game_state, &grid_config, &dev_config).await;

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
                    ServerMessage::LordData { .. } => "LordData",
                    ServerMessage::LordCreated { .. } => "LordCreated",
                    ServerMessage::LordCreateError { .. } => "LordCreateError",
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
                    ServerMessage::UnitPositionUpdated { .. } => "UnitPositionUpdated",
                    ServerMessage::UnitSlotUpdated { .. } => "UnitSlotUpdated",
                    ServerMessage::UnitProfessionChanged { .. } => "UnitProfessionChanged",
                    ServerMessage::UnitWorkStatusUpdate { .. } => "UnitWorkStatusUpdate",
                    ServerMessage::HamletFounded { .. } => "HamletFounded",
                    ServerMessage::HamletFoundError { .. } => "HamletFoundError",
                    ServerMessage::PlayerOrganizationData { .. } => "PlayerOrganizationData",
                    ServerMessage::PopulationChanged { .. } => "PouplationChanged",
                    ServerMessage::InventoryData { .. } => "InventoryData",
                    ServerMessage::InventoryUpdate { .. } => "InventoryUpdate",
                    ServerMessage::GameData { .. } => "GameData",
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
    game_state: &GameState,
    grid_config: &GridConfig,
    dev_config: &DevConfig,
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

                    let characters = shared::types::game::methods::get_player_characters(
                        &db_tables.pool,
                        player.id,
                    )
                    .await
                    .unwrap_or_default();

                    let character = characters.into_iter().next(); // Premier personnage ou None

                    let player_data = shared::protocol::PlayerData {
                        id: player.id,
                        family_name: player.family_name.clone(),
                        language_id: player.language_id,
                        coat_of_arms_id: player.coat_of_arms_id,
                        motto: player.motto.clone(),
                        origin_location: player.origin_location.clone(),
                    };

                    let character_data = character.map(|c| shared::protocol::CharacterData {
                        id: c.id,
                        player_id: c.player_id,
                        first_name: c.first_name,
                        family_name: c.family_name,
                        second_name: c.second_name,
                        nickname: c.nickname,
                        coat_of_arms_id: c.coat_of_arms_id,
                        image_id: c.image_id,
                        motto: c.motto,
                    });

                    // Envoyer LoginSuccess
                    let login_response = vec![ServerMessage::LoginSuccess {
                        player: player_data,
                        character: character_data,
                    }];

                    // Puis charger et envoyer le lord
                    let player_id_u64 = player.id as u64;
                    let lord = db_tables
                        .units
                        .load_lord_for_player(player_id_u64)
                        .await
                        .unwrap_or(None);

                    tracing::info!(
                        "Lord for player {}: {}",
                        player_id_u64,
                        lord.as_ref().map_or("None".to_string(), |l| l.full_name())
                    );

                    let _ = sessions
                        .send_to_player(player_id_u64, ServerMessage::LordData { lord })
                        .await;

                    // Envoyer les données statiques du jeu
                    let payload = build_game_data_payload(game_state, dev_config);
                    let _ = sessions
                        .send_to_player(player_id_u64, ServerMessage::GameData { payload })
                        .await;

                    login_response
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

                            let characters = shared::types::game::methods::get_player_characters(
                                &db_tables.pool,
                                player.id,
                            )
                            .await
                            .unwrap_or_default();

                            let character = characters.into_iter().next(); // Premier personnage ou None

                            let player_data = shared::protocol::PlayerData {
                                id: player.id,
                                family_name: player.family_name.clone(),
                                language_id: player.language_id,
                                coat_of_arms_id: player.coat_of_arms_id,
                                motto: player.motto.clone(),
                                origin_location: player.origin_location.clone(),
                            };

                            let character_data =
                                character.map(|c| shared::protocol::CharacterData {
                                    id: c.id,
                                    player_id: c.player_id,
                                    first_name: c.first_name,
                                    family_name: c.family_name,
                                    second_name: c.second_name,
                                    nickname: c.nickname,
                                    coat_of_arms_id: c.coat_of_arms_id,
                                    image_id: c.image_id,
                                    motto: c.motto,
                                });

                            // Envoyer LoginSuccess
                            let login_response = vec![ServerMessage::LoginSuccess {
                                player: player_data,
                                character: character_data,
                            }];

                            // Puis charger et envoyer le lord
                            let player_id_u64 = player.id as u64;
                            let lord = db_tables
                                .units
                                .load_lord_for_player(player_id_u64)
                                .await
                                .unwrap_or(None);

                            tracing::info!(
                                "Lord for player {}: {}",
                                player_id_u64,
                                lord.as_ref().map_or("None".to_string(), |l| l.full_name())
                            );

                            let _ = sessions
                                .send_to_player(
                                    player_id_u64,
                                    ServerMessage::LordData { lord: lord.clone() },
                                )
                                .await;

                            // Chercher l'organisation du joueur (via le lord)
                            if let Some(ref lord) = lord {
                                let player_org = sqlx::query_as::<_, (i64, String, i16, Option<i64>, i32, Option<String>)>(
                                    r#"
                                    SELECT o.id, o.name, o.organization_type_id, o.leader_unit_id, o.population, o.emblem_url
                                    FROM organizations.organizations o
                                    WHERE o.leader_unit_id = $1
                                    LIMIT 1
                                    "#,
                                )
                                .bind(lord.id as i64)
                                .fetch_optional(&db_tables.pool)
                                .await;

                                let org_summary = match player_org {
                                    Ok(Some((id, name, type_id, leader_id, pop, emblem))) => {
                                        Some(shared::OrganizationSummary {
                                            id: id as u64,
                                            name,
                                            organization_type: shared::OrganizationType::from_id(
                                                type_id,
                                            ),
                                            leader_unit_id: leader_id.map(|l| l as u64),
                                            population: pop,
                                            emblem_url: emblem,
                                        })
                                    }
                                    _ => None,
                                };

                                let _ = sessions
                                    .send_to_player(
                                        player_id_u64,
                                        ServerMessage::PlayerOrganizationData {
                                            organization: org_summary,
                                        },
                                    )
                                    .await;
                            }

                            // Envoyer les données statiques du jeu
                            let payload = build_game_data_payload(game_state, dev_config);
                            let _ = sessions
                                .send_to_player(player_id_u64, ServerMessage::GameData { payload })
                                .await;

                            login_response
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
                    Ok(building_data) => {
                        tracing::debug!(
                            "Loaded {} buildings for chunk ({},{})",
                            building_data.len(),
                            terrain_chunk_id.x,
                            terrain_chunk_id.y
                        );
                        building_data
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to load buildings for chunk ({},{}): {}",
                            terrain_chunk_id.x,
                            terrain_chunk_id.y,
                            e
                        );
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
            chunk_id: _,
            cell,
            building_type,
        } => {
            let chunk_id = cell.to_chunk_id(&grid_config.layout);
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

            // ── Validation ──────────────────────────────────

            // 1. Find the lord
            let lord_unit_id = match db_tables.units.load_lord_for_player(player_id).await {
                Ok(Some(lord)) => lord.id,
                Ok(None) => {
                    return vec![ServerMessage::ActionError {
                        reason: "Aucun seigneur trouvé".to_string(),
                    }];
                }
                Err(e) => {
                    tracing::error!("Failed to load lord: {}", e);
                    return vec![ServerMessage::ActionError {
                        reason: "Erreur serveur".to_string(),
                    }];
                }
            };

            // 2. Check construction costs
            if !dev_config.skip_resource_check() {
                let bt_id = building_type.to_id() as i32;
                let costs = game_state.building_costs(bt_id);

                if !costs.is_empty() {
                    let inventory = match db_tables
                        .resources
                        .load_inventory_summary(lord_unit_id)
                        .await
                    {
                        Ok(inv) => inv,
                        Err(e) => {
                            tracing::error!("Failed to load inventory: {}", e);
                            return vec![ServerMessage::ActionError {
                                reason: "Erreur de chargement de l'inventaire".to_string(),
                            }];
                        }
                    };

                    let mut missing = Vec::new();
                    for cost in costs {
                        let have = inventory.get(&cost.item_id).copied().unwrap_or(0);
                        if have < cost.quantity {
                            let item_name = game_state.item_name(cost.item_id, 1);
                            missing.push(format!(
                                "{} (besoin: {}, possédé: {})",
                                item_name, cost.quantity, have
                            ));
                        }
                    }

                    if !missing.is_empty() {
                        return vec![ServerMessage::ActionError {
                            reason: format!(
                                "Matériaux de construction manquants : {}",
                                missing.join(", ")
                            ),
                        }];
                    }
                }
            }

            // ── Schedule ────────────────────────────────────

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
            // Duration from DB
            let bt_id = building_type.to_id() as i32;
            let duration_ms =
                dev_config.apply_speed((game_state.building_duration_seconds(bt_id) as u64) * 1000);

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
            let duration_ms = dev_config.apply_speed(specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: start_cell,
            }));

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
            unit_ids,
        } => {
            // ── Validation ──────────────────────────────────

            // 1. Find the recipe (by numeric ID or slug)
            let recipe = match game_state.find_recipe(&recipe_id) {
                Some(r) => r,
                None => {
                    tracing::warn!(
                        "Player {} requested unknown recipe '{}'",
                        player_id,
                        recipe_id
                    );
                    return vec![ServerMessage::ActionError {
                        reason: format!("Recette inconnue : {}", recipe_id),
                    }];
                }
            };

            // 2. Find the lord
            let lord_unit_id = match db_tables.units.load_lord_for_player(player_id).await {
                Ok(Some(lord)) => lord.id,
                Ok(None) => {
                    return vec![ServerMessage::ActionError {
                        reason: "Aucun seigneur trouvé".to_string(),
                    }];
                }
                Err(e) => {
                    tracing::error!("Failed to load lord for player {}: {}", player_id, e);
                    return vec![ServerMessage::ActionError {
                        reason: "Erreur serveur".to_string(),
                    }];
                }
            };

            // 3. Check ingredients
            if !dev_config.skip_resource_check() && !recipe.ingredients.is_empty() {
                let inventory = match db_tables
                    .resources
                    .load_inventory_summary(lord_unit_id)
                    .await
                {
                    Ok(inv) => inv,
                    Err(e) => {
                        tracing::error!("Failed to load inventory: {}", e);
                        return vec![ServerMessage::ActionError {
                            reason: "Erreur de chargement de l'inventaire".to_string(),
                        }];
                    }
                };

                let mut missing = Vec::new();
                for ingredient in &recipe.ingredients {
                    let needed = ingredient.quantity * quantity as i32;
                    let have = inventory.get(&ingredient.item_id).copied().unwrap_or(0);
                    if have < needed {
                        let item_name = game_state.item_name(ingredient.item_id, 1).clone();
                        missing.push(format!(
                            "{} (besoin: {}, possédé: {})",
                            item_name, needed, have
                        ));
                    }
                }

                if !missing.is_empty() {
                    return vec![ServerMessage::ActionError {
                        reason: format!("Ressources manquantes : {}", missing.join(", ")),
                    }];
                }
            }

            // 4. Check production line capacity
            let building_type = db_tables
                .buildings
                .get_building_type_at_cell(&cell)
                .await
                .unwrap_or(None);

            if let Some(bt) = building_type {
                let max_lines = bt.production_lines() as usize;
                let active_count = action_processor
                    .active_production_count_on_cell(&cell)
                    .await;

                if active_count >= max_lines {
                    return vec![ServerMessage::ActionError {
                        reason: format!(
                            "Toutes les lignes de production sont occupées ({}/{})",
                            active_count, max_lines
                        ),
                    }];
                }
            }

            // 5. Validate units aren't already busy
            if !unit_ids.is_empty() {
                let busy = db_tables
                    .units
                    .get_busy_units(&unit_ids)
                    .await
                    .unwrap_or_default();
                if !busy.is_empty() {
                    return vec![ServerMessage::ActionError {
                        reason: format!("Certaines unités sont déjà occupées : {:?}", busy),
                    }];
                }
            }

            // ── Schedule the action ─────────────────────────

            let mut responses = Vec::new();
            let action_table = &db_tables.actions;

            // Use the actual recipe_id (numeric) for storage
            let specific_data = SpecificAction::CraftResource(CraftResourceAction {
                player_id,
                recipe_id: recipe.id.to_string(),
                chunk_id,
                cell,
                quantity,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Duration from DB recipe instead of hardcoded
            let duration_ms = dev_config
                .apply_speed((recipe.craft_duration_seconds as u64) * 1000 * (quantity as u64));

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
                    // Assign units to this action
                    if !unit_ids.is_empty() {
                        if let Err(e) = db_tables
                            .units
                            .set_units_working_on(&unit_ids, action_id)
                            .await
                        {
                            tracing::error!(
                                "Failed to assign units to action {}: {}",
                                action_id,
                                e
                            );
                        } else {
                            for &uid in &unit_ids {
                                responses.push(ServerMessage::UnitWorkStatusUpdate {
                                    unit_id: uid,
                                    working_on_action_id: Some(action_id),
                                });
                            }
                        }
                    }

                    tracing::info!(
                        "Craft action {} scheduled: recipe '{}' x{} for player {} \
                         (duration: {}s)",
                        action_id,
                        recipe.name,
                        quantity,
                        player_id,
                        duration_ms / 1000
                    );
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
                    tracing::error!("Failed to schedule craft action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Erreur lors de la planification : {}", e),
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
            unit_ids,
        } => {
            // ── Validation ──────────────────────────────────

            // 1. Check harvest yields exist for this resource type
            let yields = game_state.harvest_yields_for(resource_specific_type.to_id());

            if yields.is_empty() {
                return vec![ServerMessage::ActionError {
                    reason: format!(
                        "Aucun rendement de récolte défini pour ce type de ressource ({:?})",
                        resource_specific_type
                    ),
                }];
            }

            // 2. Find the lord
            match db_tables.units.load_lord_for_player(player_id).await {
                Ok(Some(_)) => {} // Lord exists, good
                Ok(None) => {
                    return vec![ServerMessage::ActionError {
                        reason: "Aucun seigneur trouvé".to_string(),
                    }];
                }
                Err(e) => {
                    tracing::error!("Failed to load lord: {}", e);
                    return vec![ServerMessage::ActionError {
                        reason: "Erreur serveur".to_string(),
                    }];
                }
            }

            // 3. Check production line capacity
            let building_type = db_tables
                .buildings
                .get_building_type_at_cell(&cell)
                .await
                .unwrap_or(None);

            if let Some(bt) = building_type {
                let max_lines = bt.production_lines() as usize;
                let active_count = action_processor
                    .active_production_count_on_cell(&cell)
                    .await;

                if active_count >= max_lines {
                    return vec![ServerMessage::ActionError {
                        reason: format!(
                            "Toutes les lignes de production sont occupées ({}/{})",
                            active_count, max_lines
                        ),
                    }];
                }
            }

            // 4. Validate units aren't already busy
            if !unit_ids.is_empty() {
                let busy = db_tables
                    .units
                    .get_busy_units(&unit_ids)
                    .await
                    .unwrap_or_default();
                if !busy.is_empty() {
                    return vec![ServerMessage::ActionError {
                        reason: format!("Certaines unités sont déjà occupées : {:?}", busy),
                    }];
                }
            }

            // ── Schedule ────────────────────────────────────

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

            // Duration from DB harvest yield (use the first yield's duration)
            let duration_ms = dev_config.apply_speed((yields[0].duration_seconds as u64) * 1000);

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
                    // Assign units to this action
                    if !unit_ids.is_empty() {
                        if let Err(e) = db_tables
                            .units
                            .set_units_working_on(&unit_ids, action_id)
                            .await
                        {
                            tracing::error!(
                                "Failed to assign units to action {}: {}",
                                action_id,
                                e
                            );
                        } else {
                            for &uid in &unit_ids {
                                responses.push(ServerMessage::UnitWorkStatusUpdate {
                                    unit_id: uid,
                                    working_on_action_id: Some(action_id),
                                });
                            }
                        }
                    }

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
                    tracing::error!("Failed to schedule harvest action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Erreur : {}", e),
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

            // Charger la position actuelle de l'unité pour calculer la distance
            let unit_data = match db_tables.units.load_unit(unit_id).await {
                Ok(u) => u,
                Err(e) => {
                    return vec![ServerMessage::ActionError {
                        reason: format!("Unité introuvable: {}", e),
                    }];
                }
            };

            // Vérifier que l'unité appartient au joueur
            if !player_controls_unit(&db_tables, player_id, unit_id).await {
                return vec![ServerMessage::ActionError {
                    reason: "Cette unité ne vous appartient pas".to_string(),
                }];
            }

            // TODO : Compute properly the path using A* pathfinding algorithm
            // TODO : given crossed cells, distance, roads, etc...
            // Calculer la distance hex entre position actuelle et cible
            let from_hex = unit_data.current_cell.to_hex();
            let to_hex = cell.to_hex();
            let hex_distance = from_hex.unsigned_distance_to(to_hex) as u64;

            if hex_distance == 0 {
                return vec![ServerMessage::ActionError {
                    reason: "L'unité est déjà sur cette cellule".to_string(),
                }];
            }

            // Durée : 2 secondes par hex de distance
            let duration_ms = dev_config.apply_speed(hex_distance * 2000);

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
                    tracing::info!(
                        "Unit {} moving {} hexes from ({},{}) to ({},{}) — {}ms (action {})",
                        unit_id,
                        hex_distance,
                        unit_data.current_cell.q,
                        unit_data.current_cell.r,
                        cell.q,
                        cell.r,
                        duration_ms,
                        action_id
                    );
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
                    tracing::error!("Failed to schedule move action: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Échec de la planification: {}", e),
                    });
                }
            }

            responses
        }
        ClientMessage::ActionTrainUnit {
            player_id,
            unit_id,
            chunk_id,
            cell,
            target_profession,
        } => {
            let mut responses = Vec::new();

            // 1. Check production line capacity
            let building_type = db_tables
                .buildings
                .get_building_type_at_cell(&cell)
                .await
                .unwrap_or(None);

            if let Some(bt) = building_type {
                let max_lines = bt.production_lines() as usize;
                let active_count = action_processor
                    .active_production_count_on_cell(&cell)
                    .await;

                if active_count >= max_lines {
                    return vec![ServerMessage::ActionError {
                        reason: format!(
                            "Toutes les lignes de production sont occupées ({}/{})",
                            active_count, max_lines
                        ),
                    }];
                }
            }

            let action_table = &db_tables.actions;
            let specific_data = SpecificAction::TrainUnit(TrainUnitAction {
                player_id,
                unit_id,
                chunk_id,
                cell,
                target_profession,
            });

            let start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let duration_ms = dev_config.apply_speed(specific_data.duration_ms(&ActionContext {
                player_id,
                grid_cell: cell,
            }));

            let action_data = ActionData {
                base_data: ActionBaseData {
                    player_id,
                    chunk: chunk_id,
                    cell,
                    action_type: ActionTypeEnum::TrainUnit,
                    action_specific_type: ActionSpecificTypeEnum::TrainUnit,
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
                ActionTypeEnum::TrainUnit,
            )
            .await
            {
                Ok(action_id) => {
                    tracing::info!(
                        "Training unit {} to {:?} (action {})",
                        unit_id,
                        target_profession,
                        action_id
                    );
                    responses.push(ServerMessage::ActionStatusUpdate {
                        action_id,
                        player_id,
                        chunk_id,
                        cell,
                        status: ActionStatusEnum::Pending,
                        action_type: ActionTypeEnum::TrainUnit,
                        completion_time: start_time + (duration_ms / 1000),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to schedule training: {}", e);
                    responses.push(ServerMessage::ActionError {
                        reason: format!("Failed to schedule training: {}", e),
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
        // LORD COMMANDS
        // ====================================================================
        ClientMessage::CreateLord {
            first_name,
            gender,
            portrait_layers,
        } => {
            tracing::info!(
                "Session {} creating lord: {} ({}) layers={}",
                session_id,
                first_name,
                gender,
                portrait_layers
            );

            // 1. Récupérer le player_id depuis la session
            let player_id = match sessions.get_player_id(session_id).await {
                Some(id) => id,
                None => {
                    return vec![ServerMessage::LordCreateError {
                        reason: "Non authentifié".to_string(),
                    }];
                }
            };

            // 2. Vérifier que le joueur n'a pas déjà un lord
            match db_tables.units.load_lord_for_player(player_id).await {
                Ok(Some(existing_lord)) => {
                    tracing::warn!(
                        "Player {} already has a lord: {}",
                        player_id,
                        existing_lord.full_name()
                    );
                    return vec![ServerMessage::LordCreateError {
                        reason: "Vous avez déjà un Lord/Lady".to_string(),
                    }];
                }
                Ok(None) => { /* OK, pas de lord existant */ }
                Err(e) => {
                    return vec![ServerMessage::LordCreateError {
                        reason: format!("Erreur: {}", e),
                    }];
                }
            }

            // 3. Récupérer le family_name du joueur
            let family_name = match shared::types::game::methods::get_player_by_id(
                &db_tables.pool,
                player_id as i64,
            )
            .await
            {
                Ok(Some(player)) => player.family_name,
                Ok(None) => {
                    return vec![ServerMessage::LordCreateError {
                        reason: "Joueur introuvable".to_string(),
                    }];
                }
                Err(e) => {
                    return vec![ServerMessage::LordCreateError {
                        reason: format!("Erreur DB: {}", e),
                    }];
                }
            };

            // 4. Choisir une cellule de départ
            //    Pour le MVP : cellule fixe sur terre. À terme : choix du joueur.
            let starting_cell = GridCell { q: 0, r: 0 };
            let starting_chunk = shared::TerrainChunkId { x: 0, y: 0 };

            // 5. Construire l'avatar_url à partir du portrait_layers
            //    Le portrait du lord est le patchwork de couches, pas un avatar serveur.
            //    On stocke une URL placeholder — le client reconstruit le portrait depuis les layers.
            let portrait_variant_id = "lord".to_string();
            let avatar_url = format!("lord_{}_{}", gender, player_id);

            // 6. Créer l'unité lord
            let profession = shared::ProfessionEnum::Unknown; // Le lord n'a pas de profession artisane

            match db_tables
                .units
                .create_unit(
                    Some(player_id),
                    first_name.clone(),
                    family_name.clone(),
                    gender.clone(),
                    portrait_variant_id,
                    avatar_url,
                    starting_cell,
                    starting_chunk,
                    profession,
                    true,                  // is_lord = true
                    Some(portrait_layers), // portrait_layers
                )
                .await
            {
                Ok(unit_id) => {
                    tracing::info!(
                        "✓ Lord created: {} {} (ID: {}) for player {}",
                        first_name,
                        family_name,
                        unit_id,
                        player_id
                    );

                    // 7. Créer aussi le Character dans game.characters
                    let _ = shared::types::game::methods::create_character(
                        &db_tables.pool,
                        player_id as i64,
                        &first_name,
                        &family_name,
                        None, // second_name
                        None, // nickname
                        None, // motto
                    )
                    .await;

                    // 8. Charger et renvoyer les données complètes
                    match db_tables.units.load_unit(unit_id).await {
                        Ok(unit_data) => {
                            vec![ServerMessage::LordCreated { unit_data }]
                        }
                        Err(e) => {
                            tracing::error!("Lord created but failed to reload: {}", e);
                            vec![ServerMessage::LordCreateError {
                                reason: format!("Lord créé mais erreur au chargement: {}", e),
                            }]
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create lord: {}", e);
                    vec![ServerMessage::LordCreateError {
                        reason: format!("Erreur lors de la création: {}", e),
                    }]
                }
            }
        }

        // ====================================================================
        // ORGANIZATION ACTIONS
        // ====================================================================
        ClientMessage::FoundHamlet => {
            tracing::info!("Session {} requesting to found a hamlet", session_id);

            // 1. Récupérer le player_id
            let player_id = match sessions.get_player_id(session_id).await {
                Some(id) => id,
                None => {
                    return vec![ServerMessage::HamletFoundError {
                        reason: "Non authentifié".to_string(),
                    }];
                }
            };

            // 2. Charger le lord du joueur
            let lord = match db_tables.units.load_lord_for_player(player_id).await {
                Ok(Some(lord)) => lord,
                Ok(None) => {
                    return vec![ServerMessage::HamletFoundError {
                        reason: "Vous n'avez pas de Lord/Lady".to_string(),
                    }];
                }
                Err(e) => {
                    return vec![ServerMessage::HamletFoundError {
                        reason: format!("Erreur: {}", e),
                    }];
                }
            };

            let cell = lord.current_cell;

            // 3. Vérifier que la cellule n'appartient pas déjà à une organisation
            match sqlx::query_scalar::<_, i64>(
                "SELECT organization_id FROM organizations.territory_cells WHERE cell_q = $1 AND cell_r = $2"
            )
            .bind(cell.q)
            .bind(cell.r)
            .fetch_optional(&db_tables.pool)
            .await
            {
                Ok(Some(_)) => {
                    return vec![ServerMessage::HamletFoundError {
                        reason: "Cette cellule appartient déjà à un territoire".to_string(),
                    }];
                }
                Ok(None) => { /* libre, on continue */ }
                Err(e) => {
                    return vec![ServerMessage::HamletFoundError {
                        reason: format!("Erreur DB: {}", e),
                    }];
                }
            }

            // 4. Vérifier que le joueur n'a pas déjà une organisation
            //    (pour le MVP, un seul hameau par joueur)
            match sqlx::query_scalar::<_, i64>(
                "SELECT id FROM organizations.organizations WHERE leader_unit_id = $1",
            )
            .bind(lord.id as i64)
            .fetch_optional(&db_tables.pool)
            .await
            {
                Ok(Some(existing_org_id)) => {
                    return vec![ServerMessage::HamletFoundError {
                        reason: format!(
                            "Vous avez déjà une organisation (ID: {})",
                            existing_org_id
                        ),
                    }];
                }
                Ok(None) => { /* OK */ }
                Err(e) => {
                    return vec![ServerMessage::HamletFoundError {
                        reason: format!("Erreur DB: {}", e),
                    }];
                }
            }

            // 5. Récupérer le nom de famille du joueur
            let family_name = match shared::types::game::methods::get_player_by_id(
                &db_tables.pool,
                player_id as i64,
            )
            .await
            {
                Ok(Some(player)) => player.family_name,
                _ => "Inconnu".to_string(),
            };

            let hamlet_name = format!("Hameau de {}", family_name);

            // 6. Créer l'organisation
            let request = shared::CreateOrganizationRequest {
                name: hamlet_name.clone(),
                organization_type: shared::OrganizationType::Hamlet,
                headquarters_cell: Some(cell),
                parent_organization_id: None,
                founder_unit_id: lord.id,
            };

            let org_id = match db_tables.organizations.create_organization(request).await {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Failed to create hamlet: {}", e);
                    return vec![ServerMessage::HamletFoundError {
                        reason: format!("Échec de la création: {}", e),
                    }];
                }
            };

            tracing::info!(
                "✓ Hamlet '{}' created with ID {} for player {}",
                hamlet_name,
                org_id,
                player_id
            );

            // 7. Claim territoire — Voronoï si disponible, sinon voisins
            let mut claimed_cells = Vec::new();

            match db_tables.voronoi_zones.get_zone_at_cell(cell).await {
                Ok(Some(zone_id)) => {
                    match db_tables.voronoi_zones.is_zone_available(zone_id).await {
                        Ok(true) => {
                            // Claim toute la zone Voronoï
                            match db_tables.voronoi_zones.get_zone_cells(zone_id).await {
                                Ok(zone_cells) => {
                                    tracing::info!(
                                        "Claiming {} cells from Voronoi zone {}",
                                        zone_cells.len(),
                                        zone_id
                                    );
                                    for zone_cell in &zone_cells {
                                        if db_tables
                                            .organizations
                                            .add_territory_cell(org_id, zone_cell)
                                            .await
                                            .is_ok()
                                        {
                                            claimed_cells.push(*zone_cell);
                                        }
                                    }

                                    // Lier la zone à l'organisation
                                    let _ = sqlx::query(
                                        "UPDATE organizations.organizations SET voronoi_zone_id = $1 WHERE id = $2"
                                    )
                                    .bind(zone_id)
                                    .bind(org_id as i64)
                                    .execute(&db_tables.pool)
                                    .await;
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to get zone cells: {}, using fallback",
                                        e
                                    );
                                    claim_cell_and_neighbors(
                                        db_tables,
                                        org_id,
                                        &cell,
                                        &mut claimed_cells,
                                    )
                                    .await;
                                }
                            }
                        }
                        _ => {
                            tracing::info!(
                                "Voronoi zone {} not available, using fallback",
                                zone_id
                            );
                            claim_cell_and_neighbors(db_tables, org_id, &cell, &mut claimed_cells)
                                .await;
                        }
                    }
                }
                _ => {
                    tracing::info!("No Voronoi zone at cell, using fallback (cell + neighbors)");
                    claim_cell_and_neighbors(db_tables, org_id, &cell, &mut claimed_cells).await;
                }
            }

            tracing::info!(
                "✓ Hamlet {} claimed {} territory cells",
                org_id,
                claimed_cells.len()
            );

            // 8. Générer les contours territoriaux
            let mut contour_messages = Vec::new();

            match db_tables.organizations.load_territory_cells(org_id).await {
                Ok(territory_cells) if !territory_cells.is_empty() => {
                    use hexx::Hex;
                    let territory_hex: std::collections::HashSet<Hex> =
                        territory_cells.iter().map(|c| c.to_hex()).collect();

                    let contour_points = &world::territory::build_contour(
                        &grid_config.layout,
                        &territory_hex,
                        0.0,
                        org_id as u64,
                    );

                    let contour_chunks = utils::chunks::split_contour_into_chunks(contour_points);

                    tracing::info!(
                        "Generated {} contour chunks for hamlet {}",
                        contour_chunks.len(),
                        org_id
                    );

                    // Stocker les contours en DB
                    for (chunk_id, contour_segments) in &contour_chunks {
                        let _ = db_tables
                            .territory_contours
                            .store_contour(org_id, chunk_id.x, chunk_id.y, contour_segments)
                            .await;
                    }

                    // Préparer les messages TerritoryContourUpdate à envoyer au client
                    // Regrouper par chunk_id
                    for (chunk_id, contour_segments) in &contour_chunks {
                        let (border_color, fill_color) =
                            world::territory::generate_org_colors(org_id);

                        contour_messages.push(ServerMessage::TerritoryContourUpdate {
                            chunk_id: *chunk_id,
                            contours: vec![shared::protocol::TerritoryContourChunkData {
                                organization_id: org_id,
                                chunk_id: *chunk_id,
                                segments: contour_segments
                                    .iter()
                                    .map(ContourSegmentData::from_contour_segment)
                                    .collect(),
                                border_color: ColorData::from_array(border_color),
                                fill_color: ColorData::from_array(fill_color),
                            }],
                        });
                    }
                }
                _ => {
                    tracing::warn!("No territory cells after founding hamlet {}", org_id);
                }
            }

            // 9. Envoyer les réponses
            let mut responses = vec![ServerMessage::HamletFounded {
                organization_id: org_id,
                name: hamlet_name,
                headquarters: cell,
                territory_cells: claimed_cells,
            }];

            // Ajouter les contours pour que le client les affiche immédiatement
            responses.extend(contour_messages);

            responses
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
                    false,
                    None,
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
                    false,
                    None,
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

        ClientMessage::RequestInventory { unit_id } => {
            use shared::protocol::InventoryItemData;

            match db_tables.resources.load_items_for_unit(unit_id).await {
                Ok(full_items) => {
                    // Grouper par item_id pour l'affichage
                    let mut grouped: std::collections::HashMap<
                        i32,
                        (String, shared::ItemTypeEnum, f32, f32, i32),
                    > = std::collections::HashMap::new();

                    for item in &full_items {
                        let entry = grouped.entry(item.definition.id).or_insert((
                            item.definition.name.clone(),
                            item.definition.item_type,
                            item.definition.weight_kg,
                            item.instance.quality,
                            0,
                        ));
                        entry.4 += 1;
                    }

                    let items: Vec<InventoryItemData> = grouped
                        .into_iter()
                        .map(|(item_id, (name, item_type, weight, quality, qty))| {
                            InventoryItemData {
                                instance_id: 0,
                                item_id,
                                name,
                                item_type,
                                quality,
                                weight_kg: weight,
                                quantity: qty,
                                is_equipped: false,
                                equipment_slot: None,
                            }
                        })
                        .collect();

                    vec![ServerMessage::InventoryData { unit_id, items }]
                }
                Err(e) => {
                    tracing::error!("Failed to load inventory for unit {}: {}", unit_id, e);
                    vec![ServerMessage::ActionError {
                        reason: format!("Failed to load inventory: {}", e),
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
