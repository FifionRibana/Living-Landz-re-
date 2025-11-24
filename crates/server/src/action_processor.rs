use bevy::prelude::*;
use shared::{ActionStatusEnum, ActionTypeEnum, TerrainChunkId, grid::GridCell, protocol::ServerMessage};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::database::client::DatabaseTables;
use crate::networking::Sessions;

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
