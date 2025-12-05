use bevy::prelude::*;
use shared::{ActionStatusEnum, ActionTypeEnum, TerrainChunkId, grid::GridCell, protocol::ServerMessage};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::database::client::DatabaseTables;
use crate::networking::Sessions;

/// Convertit une cellule hexagonale en position monde (en pixels)
fn cell_to_world_pos(cell: &GridCell) -> Vec2 {
    use shared::constants::{HEX_SIZE, HEX_RATIO};
    use hexx::{Hex, HexLayout};

    // Utiliser le même HexLayout que le terrain pour garantir la cohérence
    let layout = HexLayout::flat()
        .with_hex_size(HEX_SIZE)
        .with_scale(Vec2::new(HEX_RATIO.x * HEX_SIZE, HEX_RATIO.y * HEX_SIZE));

    let hex = Hex::new(cell.q, cell.r);
    layout.hex_to_world_pos(hex)
}

#[derive(Debug, Clone)]
pub struct ActionInfo {
    pub action_id: u64,
    pub player_id: u64,
    pub chunk_id: TerrainChunkId,
    pub cell: GridCell,
    pub action_type: ActionTypeEnum,
    pub status: ActionStatusEnum,
    pub start_time: u64,
    pub duration_ms: u64,
    pub completion_time: u64,
}

pub struct ActionProcessor {
    db_tables: Arc<DatabaseTables>,
    sessions: Sessions,
    // Cache des actions actives en mémoire pour éviter les requêtes DB constantes
    active_actions: Arc<RwLock<HashMap<u64, ActionInfo>>>,
}

impl ActionProcessor {
    pub fn new(db_tables: Arc<DatabaseTables>, sessions: Sessions) -> Self {
        Self {
            db_tables,
            sessions,
            active_actions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Charge les actions actives depuis la base de données au démarrage
    pub async fn load_active_actions(&self) -> Result<(), String> {
        let actions = self.db_tables.actions.load_active_actions().await?;

        let mut active_actions = self.active_actions.write().await;

        for (action_id, player_id, chunk_id, cell, action_type, status, start_time, duration_ms, completion_time) in actions {
            let action_info = ActionInfo {
                action_id,
                player_id,
                chunk_id,
                cell,
                action_type,
                status,
                start_time,
                duration_ms,
                completion_time,
            };

            active_actions.insert(action_id, action_info);
        }

        tracing::info!("Loaded {} active actions", active_actions.len());
        Ok(())
    }

    /// Ajoute une nouvelle action au cache
    pub async fn add_action(&self, action_info: ActionInfo) {
        tracing::info!("Adding new action {} to active actions", action_info.action_id);
        let mut active_actions = self.active_actions.write().await;
        active_actions.insert(action_info.action_id, action_info);
    }

    /// Système de tick principal - à appeler régulièrement
    pub async fn tick(&self) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Traiter les transitions Pending -> InProgress
        self.process_pending_actions(current_time).await;

        // Traiter les actions terminées InProgress -> Completed
        self.process_completed_actions(current_time).await;
    }

    /// Traite les actions Pending qui doivent passer à InProgress
    async fn process_pending_actions(&self, current_time: u64) {
        let mut active_actions = self.active_actions.write().await;
        let mut to_start = Vec::new();

        // Trouver les actions Pending qui doivent démarrer
        for (action_id, action_info) in active_actions.iter() {
            if action_info.status == ActionStatusEnum::Pending {
                tracing::info!("Action {} is Pending, ready to start", action_id);
                // Les actions démarrent immédiatement pour l'instant
                // On pourrait ajouter une logique de file d'attente ici
                to_start.push(*action_id);
            }
        }

        // Mettre à jour les actions qui démarrent
        for action_id in to_start {
            if let Some(action_info) = active_actions.get_mut(&action_id) {
                action_info.status = ActionStatusEnum::InProgress;

                // Mettre à jour la DB via la table
                if let Err(e) = self.db_tables.actions.update_action_status(action_id, ActionStatusEnum::InProgress).await {
                    tracing::error!("Failed to update action {} to InProgress: {}", action_id, e);
                    continue;
                }

                // Si c'est une action BuildBuilding, créer le bâtiment en construction
                if action_info.action_type == ActionTypeEnum::BuildBuilding {
                    if let Err(e) = self.create_building_for_action(action_id, action_info).await {
                        tracing::error!("Failed to create building for action {}: {}", action_id, e);
                        // Continue quand même, l'action peut se terminer mais sans bâtiment
                    }
                }

                // Si c'est une action BuildRoad, créer le segment de route
                if action_info.action_type == ActionTypeEnum::BuildRoad {
                    if let Err(e) = self.create_road_for_action(action_id, action_info).await {
                        tracing::error!("Failed to create road for action {}: {}", action_id, e);
                        // Continue quand même
                    }
                }

                // Envoyer notification au joueur
                let message = ServerMessage::ActionStatusUpdate {
                    action_id,
                    player_id: action_info.player_id,
                    chunk_id: action_info.chunk_id.clone(),
                    cell: action_info.cell.clone(),
                    status: ActionStatusEnum::InProgress,
                    action_type: action_info.action_type,
                    completion_time: action_info.completion_time,
                };

                self.send_message_to_player(action_info.player_id, message).await;

                tracing::info!(
                    "Action {} started (InProgress) for player {}",
                    action_id,
                    action_info.player_id
                );
            }
        }
    }

    /// Traite les actions InProgress qui sont arrivées à échéance
    async fn process_completed_actions(&self, current_time: u64) {
        let mut active_actions = self.active_actions.write().await;
        let mut to_complete = Vec::new();

        // Trouver les actions InProgress qui sont terminées
        for (action_id, action_info) in active_actions.iter() {
            tracing::info!(
                "Checking action {}: status={:?}, completion_time={}, current_time={}",
                action_id,
                action_info.status,
                action_info.completion_time,
                current_time
            );
            if action_info.status == ActionStatusEnum::InProgress
                && current_time >= action_info.completion_time
            {
                to_complete.push(*action_id);
            }
        }

        // Mettre à jour les actions terminées
        for action_id in to_complete {
            if let Some(action_info) = active_actions.get_mut(&action_id) {
                action_info.status = ActionStatusEnum::Completed;

                // Mettre à jour la DB via la table
                if let Err(e) = self.db_tables.actions.update_action_status(action_id, ActionStatusEnum::Completed).await {
                    tracing::error!("Failed to update action {} to Completed: {}", action_id, e);
                    continue;
                }

                // Si c'est une action BuildBuilding, marquer le bâtiment comme construit
                if action_info.action_type == ActionTypeEnum::BuildBuilding {
                    if let Err(e) = self.db_tables.buildings.mark_building_as_built(action_id).await {
                        tracing::error!("Failed to mark building {} as built: {}", action_id, e);
                    } else {
                        tracing::info!("Building {} marked as built", action_id);
                    }
                }

                // Si c'est une action BuildRoad, la route est déjà créée, rien à faire de plus
                if action_info.action_type == ActionTypeEnum::BuildRoad {
                    tracing::info!("Road segment {} completed at chunk ({}, {}) cell ({}, {})",
                        action_id,
                        action_info.chunk_id.x,
                        action_info.chunk_id.y,
                        action_info.cell.q,
                        action_info.cell.r
                    );
                }

                // Envoyer notification au joueur qui a lancé l'action
                let status_message = ServerMessage::ActionStatusUpdate {
                    action_id,
                    player_id: action_info.player_id,
                    chunk_id: action_info.chunk_id.clone(),
                    cell: action_info.cell.clone(),
                    status: ActionStatusEnum::Completed,
                    action_type: action_info.action_type,
                    completion_time: action_info.completion_time,
                };

                self.send_message_to_player(action_info.player_id, status_message).await;

                // Au prochain tick, on enverra le résultat aux joueurs du chunk
                // Pour l'instant on envoie immédiatement
                let completion_message = ServerMessage::ActionCompleted {
                    action_id,
                    chunk_id: action_info.chunk_id.clone(),
                    cell: action_info.cell.clone(),
                    action_type: action_info.action_type,
                };

                self.broadcast_to_chunk(&action_info.chunk_id, completion_message).await;

                tracing::info!(
                    "Action {} completed for player {} at chunk ({}, {}) cell ({}, {})",
                    action_id,
                    action_info.player_id,
                    action_info.chunk_id.x,
                    action_info.chunk_id.y,
                    action_info.cell.q,
                    action_info.cell.r
                );
            }
        }

        // Nettoyer les actions complétées du cache (optionnel)
        active_actions.retain(|_, action| action.status != ActionStatusEnum::Completed);
    }

    /// Crée un bâtiment en construction pour une action BuildBuilding
    async fn create_building_for_action(&self, action_id: u64, action_info: &ActionInfo) -> Result<(), String> {
        use shared::{
            AgricultureData, AgricultureTypeEnum, AnimalBreedingData, AnimalBreedingTypeEnum,
            BuildingBaseData, BuildingData, BuildingSpecific, BuildingTypeEnum,
            CommerceData, CommerceTypeEnum, CultData, CultTypeEnum,
            EntertainmentData, EntertainmentTypeEnum, ManufacturingWorkshopData,
            ManufacturingWorkshopTypeEnum,
        };

        // Récupérer le type de bâtiment depuis la base de données
        let building_type_id = self.db_tables.actions.get_build_building_type(action_id).await?
            .ok_or_else(|| format!("No building type found for action {}", action_id))?;

        // Convertir building_type_id en BuildingTypeEnum
        let building_type = BuildingTypeEnum::from_id(building_type_id)
            .ok_or_else(|| format!("Invalid building type ID: {}", building_type_id))?;

        // Déterminer la catégorie et le type spécifique
        let building_specific_type = building_type.to_specific_type();
        let category = building_type.category();

        // Créer les données spécifiques selon le type de bâtiment
        let specific_data = match building_specific_type {
            shared::BuildingSpecificTypeEnum::ManufacturingWorkshop => {
                let workshop_type = match building_type {
                    BuildingTypeEnum::Blacksmith => ManufacturingWorkshopTypeEnum::Blacksmith,
                    BuildingTypeEnum::BlastFurnace => ManufacturingWorkshopTypeEnum::BlastFurnace,
                    BuildingTypeEnum::Bloomery => ManufacturingWorkshopTypeEnum::Bloomery,
                    BuildingTypeEnum::CarpenterShop => ManufacturingWorkshopTypeEnum::CarpenterShop,
                    BuildingTypeEnum::GlassFactory => ManufacturingWorkshopTypeEnum::GlassFactory,
                    _ => ManufacturingWorkshopTypeEnum::Blacksmith,
                };
                BuildingSpecific::ManufacturingWorkshop(ManufacturingWorkshopData {
                    workshop_type,
                    variant: 0,
                })
            }
            shared::BuildingSpecificTypeEnum::Agriculture => {
                let agriculture_type = match building_type {
                    BuildingTypeEnum::Farm => AgricultureTypeEnum::Farm,
                    _ => AgricultureTypeEnum::Farm,
                };
                BuildingSpecific::Agriculture(AgricultureData {
                    agriculture_type,
                    variant: 0,
                })
            }
            shared::BuildingSpecificTypeEnum::AnimalBreeding => {
                let animal_type = match building_type {
                    BuildingTypeEnum::Cowshed => AnimalBreedingTypeEnum::Cowshed,
                    BuildingTypeEnum::Piggery => AnimalBreedingTypeEnum::Piggery,
                    BuildingTypeEnum::Sheepfold => AnimalBreedingTypeEnum::Sheepfold,
                    BuildingTypeEnum::Stable => AnimalBreedingTypeEnum::Stable,
                    _ => AnimalBreedingTypeEnum::Cowshed,
                };
                BuildingSpecific::AnimalBreeding(AnimalBreedingData {
                    animal_type,
                    variant: 0,
                })
            }
            shared::BuildingSpecificTypeEnum::Entertainment => {
                let entertainment_type = match building_type {
                    BuildingTypeEnum::Theater => EntertainmentTypeEnum::Theater,
                    _ => EntertainmentTypeEnum::Theater,
                };
                BuildingSpecific::Entertainment(EntertainmentData {
                    entertainment_type,
                    variant: 0,
                })
            }
            shared::BuildingSpecificTypeEnum::Cult => {
                let cult_type = match building_type {
                    BuildingTypeEnum::Temple => CultTypeEnum::Temple,
                    _ => CultTypeEnum::Temple,
                };
                BuildingSpecific::Cult(CultData {
                    cult_type,
                    variant: 0,
                })
            }
            shared::BuildingSpecificTypeEnum::Commerce => {
                let commerce_type = match building_type {
                    BuildingTypeEnum::Bakehouse => CommerceTypeEnum::Bakehouse,
                    BuildingTypeEnum::Brewery => CommerceTypeEnum::Brewery,
                    BuildingTypeEnum::Distillery => CommerceTypeEnum::Distillery,
                    BuildingTypeEnum::Slaughterhouse => CommerceTypeEnum::Slaughterhouse,
                    BuildingTypeEnum::IceHouse => CommerceTypeEnum::IceHouse,
                    BuildingTypeEnum::Market => CommerceTypeEnum::Market,
                    _ => CommerceTypeEnum::Bakehouse,
                };
                BuildingSpecific::Commerce(CommerceData {
                    commerce_type,
                    variant: 0,
                })
            }
            _ => BuildingSpecific::Unknown(),
        };

        // Créer les données de base du bâtiment
        let building_data = BuildingData {
            base_data: BuildingBaseData {
                id: action_id,
                category,
                specific_type: building_specific_type,
                chunk: action_info.chunk_id.clone(),
                cell: action_info.cell.clone(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                quality: 1.0,
                durability: 1.0,
                damage: 0.0,
            },
            specific_data,
        };

        // Créer le bâtiment en construction (is_built = false)
        self.db_tables.buildings.create_building(&building_data).await?;

        tracing::info!(
            "Created building {:?} (in construction) for action {} at chunk ({}, {}) cell ({}, {})",
            building_type,
            action_id,
            action_info.chunk_id.x,
            action_info.chunk_id.y,
            action_info.cell.q,
            action_info.cell.r
        );

        Ok(())
    }

    /// Crée un segment de route pour une action BuildRoad
    async fn create_road_for_action(&self, action_id: u64, action_info: &ActionInfo) -> Result<(), String> {
        use crate::road::RoadSegment;

        let cell = action_info.cell.clone();
        let chunk_id = action_info.chunk_id;

        // Charger les routes existantes dans le chunk pour détecter les connexions
        let existing_segments = self.db_tables.road_segments
            .load_road_segments_by_chunk(chunk_id.x, chunk_id.y)
            .await
            .unwrap_or_default();

        // Vérifier si la cellule est adjacente à une EXTRÉMITÉ de route
        let neighbors = cell.neighbors();

        // Chercher toutes les routes adjacentes dont la cellule voisine est une extrémité
        let mut adjacent_endpoints: Vec<(&RoadSegment, bool)> = Vec::new(); // (segment, is_start)
        let mut adjacent_to_middle = false;

        for neighbor in &neighbors {
            for segment in &existing_segments {
                // Vérifier que cell_path n'est pas vide
                if segment.cell_path.is_empty() {
                    continue;
                }

                // La cellule voisine est-elle le premier élément du chemin ?
                if segment.cell_path.first() == Some(neighbor) {
                    tracing::debug!(
                        "Found road endpoint at start: segment {} has first cell ({},{}) adjacent to new cell ({},{})",
                        segment.id, neighbor.q, neighbor.r, cell.q, cell.r
                    );
                    adjacent_endpoints.push((segment, true));
                }
                // Ou le dernier élément ?
                else if segment.cell_path.last() == Some(neighbor) {
                    tracing::debug!(
                        "Found road endpoint at end: segment {} has last cell ({},{}) adjacent to new cell ({},{})",
                        segment.id, neighbor.q, neighbor.r, cell.q, cell.r
                    );
                    adjacent_endpoints.push((segment, false));
                }
                // Si le voisin est au milieu du chemin, on ne fait rien (intersection manuelle requise)
                else if segment.cell_path.contains(neighbor) {
                    tracing::info!(
                        "Cell ({},{}) is adjacent to middle of road segment {} - intersection requires manual action",
                        cell.q, cell.r, segment.id
                    );
                    adjacent_to_middle = true;
                }
            }
        }

        // Si adjacent au milieu d'une route, retourner une erreur
        if adjacent_to_middle && adjacent_endpoints.is_empty() {
            return Err(format!(
                "Cannot build road here: cell ({},{}) is adjacent to middle of existing road. Use intersection action instead.",
                cell.q, cell.r
            ));
        }

        // Sélectionner la route la plus longue si plusieurs extrémités adjacentes
        let selected_segment = if adjacent_endpoints.is_empty() {
            None
        } else if adjacent_endpoints.len() == 1 {
            Some(adjacent_endpoints[0])
        } else {
            // Plusieurs extrémités adjacentes : choisir la route la plus longue
            tracing::info!(
                "Cell ({},{}) is adjacent to {} road endpoints - selecting longest",
                cell.q, cell.r, adjacent_endpoints.len()
            );
            adjacent_endpoints.iter()
                .max_by_key(|(seg, _)| seg.cell_path.len())
                .copied()
        };

        let (adjacent_segment, is_start_connection) = match selected_segment {
            Some((seg, is_start)) => {
                tracing::info!(
                    "Selected road segment {} with {} cells (connecting to {})",
                    seg.id, seg.cell_path.len(),
                    if is_start { "start" } else { "end" }
                );
                (Some(seg), is_start)
            }
            None => (None, false),
        };

        let is_end_connection = !is_start_connection;

        // Position monde de la cellule
        let cell_pos = cell_to_world_pos(&cell);

        if let Some(existing) = adjacent_segment {
            // Étendre le chemin de la route existante
            tracing::info!(
                "Extending road path {} by adding cell ({},{}) - connecting to {} (start={}, end={})",
                existing.id,
                cell.q, cell.r,
                if is_start_connection { "start" } else { "end" },
                is_start_connection,
                is_end_connection
            );

            // Créer un nouveau chemin étendu en ajoutant la cellule
            let mut new_cell_path = existing.cell_path.clone();

            if new_cell_path.is_empty() {
                // Reconstruire le path à partir de start_cell et end_cell si vide
                new_cell_path.push(existing.start_cell.clone());
                if existing.start_cell != existing.end_cell {
                    new_cell_path.push(existing.end_cell.clone());
                }
            }

            let (new_start, new_end) = if is_start_connection {
                // Ajouter au début du chemin
                new_cell_path.insert(0, cell.clone());
                (cell.clone(), new_cell_path.last().unwrap().clone())
            } else {
                // Ajouter à la fin du chemin
                new_cell_path.push(cell.clone());
                (new_cell_path.first().unwrap().clone(), cell.clone())
            };

            // Convertir les cellules en positions monde
            let cell_positions: Vec<bevy::prelude::Vec2> = new_cell_path.iter()
                .map(|c| cell_to_world_pos(c))
                .collect();

            // Générer une spline continue sur tout le chemin
            use crate::road::generate_path_spline;

            let samples_per_segment = 8; // 8 points entre chaque paire de cellules
            let new_points = generate_path_spline(&cell_positions, samples_per_segment);

            tracing::info!(
                "Generated continuous spline with {} points for {} cells",
                new_points.len(),
                new_cell_path.len()
            );

            // Créer le segment mis à jour
            let updated_segment = RoadSegment {
                id: 0, // Nouveau ID sera assigné
                start_cell: new_start.clone(),
                end_cell: new_end.clone(),
                cell_path: new_cell_path.clone(),
                points: new_points,
                importance: 1,
            };

            // Supprimer l'ancien segment et sauvegarder le nouveau
            // (On pourrait aussi faire une mise à jour, mais c'est plus simple de recréer)
            let segment_id = self.db_tables.road_segments
                .save_road_segment_with_chunk(&updated_segment, chunk_id.x, chunk_id.y)
                .await
                .map_err(|e| format!("Failed to save updated road segment: {}", e))?;

            tracing::info!(
                "Updated road segment {} with {} cells from ({},{}) to ({},{}) in chunk ({},{}) for action {}",
                segment_id,
                new_cell_path.len(),
                new_start.q, new_start.r,
                new_end.q, new_end.r,
                chunk_id.x, chunk_id.y,
                action_id
            );
        } else {
            // Créer une nouvelle route d'un seul point (comme dans Godot)
            // La route sera juste un point sur cette cellule
            tracing::info!(
                "Creating new single-point road on cell ({},{})",
                cell.q, cell.r
            );

            let segment = RoadSegment {
                id: 0,
                start_cell: cell.clone(),
                end_cell: cell.clone(),  // Même cellule pour indiquer un point unique
                cell_path: vec![cell.clone()],  // Une seule cellule dans le chemin
                points: vec![cell_pos],  // Un seul point
                importance: 1,
            };

            let segment_id = self.db_tables.road_segments
                .save_road_segment_with_chunk(&segment, chunk_id.x, chunk_id.y)
                .await
                .map_err(|e| format!("Failed to save road segment: {}", e))?;

            tracing::info!(
                "Created single-point road segment {} at cell ({},{}) in chunk ({},{}) for action {}",
                segment_id,
                cell.q, cell.r,
                chunk_id.x, chunk_id.y,
                action_id
            );
        }

        Ok(())
    }

    /// Gère l'échec d'une action (supprime le bâtiment si nécessaire)
    pub async fn fail_action(&self, action_id: u64) -> Result<(), String> {
        let mut active_actions = self.active_actions.write().await;

        if let Some(action_info) = active_actions.get_mut(&action_id) {
            action_info.status = ActionStatusEnum::Failed;

            // Mettre à jour la DB
            self.db_tables.actions.update_action_status(action_id, ActionStatusEnum::Failed).await?;

            // Si c'est une action BuildBuilding qui était InProgress, supprimer le bâtiment
            if action_info.action_type == ActionTypeEnum::BuildBuilding
                && action_info.status != ActionStatusEnum::Pending {
                if let Err(e) = self.db_tables.buildings.delete_building(action_id).await {
                    tracing::error!("Failed to delete building {} after action failure: {}", action_id, e);
                } else {
                    tracing::info!("Building {} deleted after action failure", action_id);
                }
            }

            // Notifier le joueur
            let message = ServerMessage::ActionStatusUpdate {
                action_id,
                player_id: action_info.player_id,
                chunk_id: action_info.chunk_id.clone(),
                cell: action_info.cell.clone(),
                status: ActionStatusEnum::Failed,
                action_type: action_info.action_type,
                completion_time: action_info.completion_time,
            };

            self.send_message_to_player(action_info.player_id, message).await;

            tracing::info!("Action {} failed for player {}", action_id, action_info.player_id);
        }

        Ok(())
    }

    /// Envoie un message à un joueur spécifique
    async fn send_message_to_player(&self, player_id: u64, message: ServerMessage) {
        tracing::info!("Sending message to player {}: {:?}", player_id, message);
        if let Err(e) = self.sessions.send_to_player(player_id, message).await {
            tracing::warn!("Failed to send message to player {}: {}", player_id, e);
        }
    }

    /// Broadcast un message à tous les joueurs qui ont chargé un chunk
    async fn broadcast_to_chunk(&self, _chunk_id: &TerrainChunkId, message: ServerMessage) {
        // TODO: Implémenter le broadcast aux joueurs d'un chunk spécifique
        // Pour l'instant on broadcast à tous les joueurs
        self.sessions.broadcast(message).await;
    }
}

/// Démarre le processeur d'actions en arrière-plan
pub fn start_action_processor(processor: Arc<ActionProcessor>) {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;
            processor.tick().await;
        }
    });
}
