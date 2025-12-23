use bevy::prelude::*;
use shared::{ActionStatusEnum, ActionTypeEnum, TerrainChunkId, grid::GridCell, protocol::ServerMessage};
use std::{collections::{HashMap, HashSet}, sync::Arc};
use tokio::sync::RwLock;

use crate::database::client::DatabaseTables;
use crate::networking::Sessions;
use crate::road::RoadSegment;

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
    async fn process_pending_actions(&self, _current_time: u64) {
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
                    chunk_id: action_info.chunk_id,
                    cell: action_info.cell,
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

                // Si c'est une action BuildRoad, régénérer et envoyer la SDF de route
                if action_info.action_type == ActionTypeEnum::BuildRoad {
                    tracing::info!("Road segment {} completed at chunk ({}, {}) cell ({}, {})",
                        action_id,
                        action_info.chunk_id.x,
                        action_info.chunk_id.y,
                        action_info.cell.q,
                        action_info.cell.r
                    );

                    // Note: La SDF a déjà été envoyée lors de la création du segment,
                    // mais on la régénère à nouveau pour garantir la cohérence
                    // On régénère pour le chunk et ses voisins pour gérer les routes diagonales
                    self.regenerate_road_sdf_for_chunk_and_neighbors(&action_info.chunk_id).await;
                }

                // Envoyer notification au joueur qui a lancé l'action
                let status_message = ServerMessage::ActionStatusUpdate {
                    action_id,
                    player_id: action_info.player_id,
                    chunk_id: action_info.chunk_id,
                    cell: action_info.cell,
                    status: ActionStatusEnum::Completed,
                    action_type: action_info.action_type,
                    completion_time: action_info.completion_time,
                };

                self.send_message_to_player(action_info.player_id, status_message).await;

                // Au prochain tick, on enverra le résultat aux joueurs du chunk
                // Pour l'instant on envoie immédiatement
                let completion_message = ServerMessage::ActionCompleted {
                    action_id,
                    chunk_id: action_info.chunk_id,
                    cell: action_info.cell,
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

        tracing::info!("Building type id: {} for action {}", building_type_id, action_id);
        // Convertir building_type_id en BuildingTypeEnum
        let building_type = BuildingTypeEnum::from_id(building_type_id)
            .ok_or_else(|| format!("Invalid building type ID: {}", building_type_id))?;

        tracing::info!("Building type: {:?} for action {}", building_type, action_id);

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
                chunk: action_info.chunk_id,
                cell: action_info.cell,
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
        use shared::grid::pathfinding::{find_path, PathfindingOptions, NeighborType};

        // Charger les cellules start et end depuis la DB
        let (start_cell, end_cell) = self.db_tables.actions.get_build_road_cells(action_id).await?;

        let chunk_id = action_info.chunk_id;

        tracing::info!(
            "Building road from ({},{}) to ({},{}) for action {}",
            start_cell.q, start_cell.r, end_cell.q, end_cell.r, action_id
        );

        // Calculer le chemin entre start_cell et end_cell
        let cell_path = if start_cell == end_cell {
            // Cas spécial: un seul point
            vec![start_cell]
        } else {
            // Vérifier si les cellules sont voisines (direct ou indirect)
            let is_direct_neighbor = start_cell.neighbors().contains(&end_cell);
            let is_indirect_neighbor = start_cell.indirect_neighbors().contains(&end_cell);

            if is_direct_neighbor || is_indirect_neighbor {
                // Voisins: créer une route directe
                vec![start_cell, end_cell]
            } else {
                // Pas voisins: utiliser le pathfinding
                tracing::info!(
                    "Cells are not neighbors, using pathfinding from ({},{}) to ({},{})",
                    start_cell.q, start_cell.r, end_cell.q, end_cell.r
                );

                match find_path(start_cell, end_cell, PathfindingOptions {
                    neighbor_type: NeighborType::Both,
                    ..Default::default()
                }) {
                    Some(path) => {
                        tracing::info!("Found path with {} cells", path.len());
                        path
                    }
                    None => {
                        return Err(format!(
                            "No path found from ({},{}) to ({},{})",
                            start_cell.q, start_cell.r, end_cell.q, end_cell.r
                        ));
                    }
                }
            }
        };

        // Convertir le cell_path en positions monde
        let cell_positions: Vec<Vec2> = cell_path.iter().map(cell_to_world_pos).collect();

        // Générer une spline lisse passant par ces positions
        use crate::road::{RoadConfig, generate_path_spline};
        let config = RoadConfig::default();
        let samples_per_segment = config.samples_per_segment;

        let points = generate_path_spline(&cell_positions, samples_per_segment);

        tracing::info!(
            "Generated spline with {} points from {} cells (samples_per_segment={})",
            points.len(),
            cell_path.len(),
            samples_per_segment
        );

        // Créer le nouveau segment de route
        let segment = RoadSegment {
            id: 0,
            start_cell: *cell_path.first().unwrap(),
            end_cell: *cell_path.last().unwrap(),
            cell_path: cell_path.clone(),
            points,
            importance: 1,
            road_type: shared::RoadType::default(),
        };

        // Sauvegarder le segment
        let segment_id = self.db_tables.road_segments
            .save_road_segment_with_chunk(&segment, chunk_id.x, chunk_id.y)
            .await
            .map_err(|e| format!("Failed to save road segment: {}", e))?;

        tracing::info!(
            "Created road segment {} with {} cells from ({},{}) to ({},{}) for action {}",
            segment_id,
            cell_path.len(),
            segment.start_cell.q, segment.start_cell.r,
            segment.end_cell.q, segment.end_cell.r,
            action_id
        );

        // Tenter de fusionner avec les segments connectés
        let final_segment_id = self.merge_connected_segments_if_needed(segment_id).await?;

        // Régénérer et envoyer immédiatement la SDF de route pour TOUS les chunks où ce segment est visible
        self.regenerate_road_sdf_for_segment(final_segment_id).await;

        Ok(())
    }

    /// Fusionne les segments de route connectés en une seule route avec spline globale
    /// Retourne l'ID du segment final (fusionné ou original)
    async fn merge_connected_segments_if_needed(&self, segment_id: i64) -> Result<i64, String> {
        use crate::road::RoadSegment;
        use std::collections::HashSet;

        // Charger le segment nouvellement créé
        let segment = self.db_tables.road_segments
            .load_road_segment(segment_id)
            .await
            .map_err(|e| format!("Failed to load segment: {}", e))?
            .ok_or_else(|| format!("Segment {} not found", segment_id))?;

        // Charger tous les segments connectés aux extrémités (sauf le segment lui-même)
        let start_connected = self.db_tables.road_segments
            .load_connected_segments(&segment.start_cell)
            .await
            .unwrap_or_default();

        let end_connected = self.db_tables.road_segments
            .load_connected_segments(&segment.end_cell)
            .await
            .unwrap_or_default();

        // Filtrer pour exclure le segment actuel
        let start_connected: Vec<_> = start_connected.into_iter()
            .filter(|s| s.id != segment_id)
            .collect();
        let end_connected: Vec<_> = end_connected.into_iter()
            .filter(|s| s.id != segment_id)
            .collect();

        tracing::info!(
            "Segment {}: {} segments connected to start ({},{}), {} to end ({},{})",
            segment_id,
            start_connected.len(), segment.start_cell.q, segment.start_cell.r,
            end_connected.len(), segment.end_cell.q, segment.end_cell.r
        );

        // Si aucune connexion, pas de fusion nécessaire
        if start_connected.is_empty() && end_connected.is_empty() {
            return Ok(segment_id);
        }

        // Construire la chaîne de segments à fusionner
        let mut segments_to_merge = Vec::new();
        let mut visited = HashSet::new();
        visited.insert(segment_id);

        // Suivre la chaîne depuis le start_cell (en remontant)
        if let Some(prev) = self.find_linear_chain_segment(&segment, &segment.start_cell, &start_connected, &visited) {
            let chain = self.collect_chain_backwards(prev, &segment.start_cell, &mut visited).await;
            segments_to_merge.extend(chain);
        }

        // Ajouter le segment actuel
        segments_to_merge.push(segment.clone());

        // Suivre la chaîne depuis le end_cell (en avançant)
        if let Some(next) = self.find_linear_chain_segment(&segment, &segment.end_cell, &end_connected, &visited) {
            let chain = self.collect_chain_forwards(next, &segment.end_cell, &mut visited).await;
            segments_to_merge.extend(chain);
        }

        // Si on n'a que le segment actuel, pas de fusion nécessaire
        if segments_to_merge.len() == 1 {
            tracing::info!("No segments to merge with segment {}", segment_id);
            return Ok(segment_id);
        }

        tracing::info!("Merging {} segments into one: {:?}", segments_to_merge.len(),
            segments_to_merge.iter().map(|s| s.id).collect::<Vec<_>>());

        // Fusionner les cell_path de tous les segments
        let merged_path = self.merge_segment_paths(&segments_to_merge);

        tracing::info!("Merged path has {} cells", merged_path.len());

        // Générer une nouvelle spline sur le chemin fusionné
        let cell_positions: Vec<Vec2> = merged_path.iter().map(cell_to_world_pos).collect();

        use crate::road::{RoadConfig, generate_path_spline};
        let config = RoadConfig::default();
        let points = generate_path_spline(&cell_positions, config.samples_per_segment);

        // Créer le segment fusionné
        let merged_segment = RoadSegment {
            id: 0, // Nouveau segment
            start_cell: *merged_path.first().unwrap(),
            end_cell: *merged_path.last().unwrap(),
            cell_path: merged_path,
            points,
            importance: segments_to_merge.iter().map(|s| s.importance).max().unwrap_or(1),
            road_type: segment.road_type.clone(),
        };

        // Calculer le chunk principal (basé sur le premier segment)
        use crate::database::tables::RoadSegmentsTable;
        let chunk_id = RoadSegmentsTable::cell_to_chunk_id(&merged_segment.start_cell);
        let (chunk_x, chunk_y) = (chunk_id.x, chunk_id.y);

        // Sauvegarder le segment fusionné
        let merged_id = self.db_tables.road_segments
            .save_road_segment_with_chunk(&merged_segment, chunk_x, chunk_y)
            .await
            .map_err(|e| format!("Failed to save merged segment: {}", e))?;

        tracing::info!("Created merged segment {} from {} segments", merged_id, segments_to_merge.len());

        // Supprimer tous les anciens segments
        for old_segment in &segments_to_merge {
            if let Err(e) = self.db_tables.road_segments.delete_road_segment(old_segment.id).await {
                tracing::warn!("Failed to delete old segment {}: {}", old_segment.id, e);
            }
        }

        // Régénérer les SDF pour tous les chunks affectés par les anciens segments
        let mut affected_chunks = HashSet::new();
        for old_segment in &segments_to_merge {
            if let Ok(chunks) = self.db_tables.road_segments.get_chunks_for_segment(old_segment.id).await {
                affected_chunks.extend(chunks);
            }
        }

        for (chunk_x, chunk_y) in affected_chunks {
            let chunk_id = shared::TerrainChunkId { x: chunk_x, y: chunk_y };
            self.regenerate_and_send_road_sdf(&chunk_id).await;
        }

        Ok(merged_id)
    }

    /// Trouve un segment qui peut être fusionné dans une chaîne linéaire
    /// Retourne Some(segment) si exactement un segment est connecté et forme une chaîne linéaire
    fn find_linear_chain_segment(
        &self,
        _current: &RoadSegment,
        _connection_cell: &shared::grid::GridCell,
        connected: &[RoadSegment],
        visited: &HashSet<i64>
    ) -> Option<RoadSegment> {
        // Filtrer les segments déjà visités
        let available: Vec<_> = connected.iter()
            .filter(|s| !visited.contains(&s.id))
            .collect();

        // Si exactement un segment connecté, on peut fusionner
        if available.len() == 1 {
            let seg = available[0];

            // Vérifier que le segment connecté n'a pas d'autres connexions (pas une jonction)
            // Pour l'instant, on accepte la fusion simple
            Some(seg.clone())
        } else {
            // Plusieurs segments connectés = jonction, ou aucun = extrémité
            None
        }
    }

    /// Collecte tous les segments formant une chaîne en remontant (vers le début)
    async fn collect_chain_backwards(
        &self,
        mut current: RoadSegment,
        connection_cell: &shared::grid::GridCell,
        visited: &mut HashSet<i64>
    ) -> Vec<RoadSegment> {
        let mut chain = Vec::new();
        let mut current_cell = *connection_cell;

        loop {
            visited.insert(current.id);

            // Déterminer l'autre extrémité du segment actuel
            let other_end = if current.start_cell == current_cell {
                &current.clone().end_cell
            } else {
                &current.clone().start_cell
            };

            // Inverser le segment si nécessaire pour maintenir l'ordre
            let ordered_segment = if current.end_cell == current_cell {
                // Inverser le segment
                RoadSegment {
                    id: current.id,
                    start_cell: current.end_cell,
                    end_cell: current.start_cell,
                    cell_path: current.cell_path.iter().rev().cloned().collect(),
                    points: current.points.iter().rev().cloned().collect(),
                    importance: current.importance,
                    road_type: current.road_type.clone(),
                }
            } else {
                current.clone()
            };

            chain.push(ordered_segment);

            // Charger les segments connectés à l'autre extrémité
            let next_connected = self.db_tables.road_segments
                .load_connected_segments(other_end)
                .await
                .unwrap_or_default();

            // Trouver le prochain segment dans la chaîne
            match self.find_linear_chain_segment(&current, other_end, &next_connected, visited) {
                Some(next) => {
                    current = next;
                    current_cell = *other_end;
                }
                None => break,
            }
        }

        // Inverser pour avoir l'ordre correct (du début vers la connexion)
        chain.reverse();
        chain
    }

    /// Collecte tous les segments formant une chaîne en avançant (vers la fin)
    async fn collect_chain_forwards(
        &self,
        mut current: RoadSegment,
        connection_cell: &shared::grid::GridCell,
        visited: &mut HashSet<i64>
    ) -> Vec<RoadSegment> {
        let mut chain = Vec::new();
        let mut current_cell = *connection_cell;

        loop {
            visited.insert(current.id);

            // Déterminer l'autre extrémité du segment actuel
            let other_end = if current.start_cell == current_cell {
                &current.clone().end_cell
            } else {
                &current.clone().start_cell
            };

            // Inverser le segment si nécessaire pour maintenir l'ordre
            let ordered_segment = if current.start_cell == current_cell {
                current.clone()
            } else {
                // Inverser le segment
                RoadSegment {
                    id: current.id,
                    start_cell: current.end_cell,
                    end_cell: current.start_cell,
                    cell_path: current.cell_path.iter().rev().cloned().collect(),
                    points: current.points.iter().rev().cloned().collect(),
                    importance: current.importance,
                    road_type: current.road_type.clone(),
                }
            };

            chain.push(ordered_segment);

            // Charger les segments connectés à l'autre extrémité
            let next_connected = self.db_tables.road_segments
                .load_connected_segments(other_end)
                .await
                .unwrap_or_default();

            // Trouver le prochain segment dans la chaîne
            match self.find_linear_chain_segment(&current, other_end, &next_connected, visited) {
                Some(next) => {
                    current = next;
                    current_cell = *other_end;
                }
                None => break,
            }
        }

        chain
    }

    /// Fusionne les cell_path de plusieurs segments en un seul chemin continu
    fn merge_segment_paths(&self, segments: &[RoadSegment]) -> Vec<shared::grid::GridCell> {
        if segments.is_empty() {
            return Vec::new();
        }

        if segments.len() == 1 {
            return segments[0].cell_path.clone();
        }

        let mut merged = Vec::new();

        for (i, segment) in segments.iter().enumerate() {
            if i == 0 {
                // Premier segment : ajouter tous les points
                merged.extend_from_slice(&segment.cell_path);
            } else {
                // Segments suivants : sauter le premier point (déjà présent comme dernier du segment précédent)
                if !segment.cell_path.is_empty() {
                    merged.extend_from_slice(&segment.cell_path[1..]);
                }
            }
        }

        merged
    }

    /// OBSOLETE - Ancienne logique de création de route point par point
    /// Conservée pour référence, pourrait être supprimée
    async fn _create_road_for_action_old(&self, action_id: u64, action_info: &ActionInfo) -> Result<(), String> {
        use crate::road::RoadSegment;

        let cell = action_info.cell;
        let chunk_id = action_info.chunk_id;

        // Charger les routes existantes dans le chunk ET les chunks voisins pour détecter les connexions
        // Ceci permet de connecter les routes qui traversent les frontières de chunks
        let existing_segments = self.db_tables.road_segments
            .load_road_segments_with_neighbors(chunk_id.x, chunk_id.y)
            .await
            .unwrap_or_default();

        // Vérifier si la cellule est adjacente à une EXTRÉMITÉ de route
        // Utilise les voisins directs ET indirects pour permettre des routes plus flexibles
        let neighbors = cell.all_extended_neighbors();

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
                new_cell_path.push(existing.start_cell);
                if existing.start_cell != existing.end_cell {
                    new_cell_path.push(existing.end_cell);
                }
            }

            // Étendre la spline en conservant les points existants et en régénérant N cellules pour le lissage
            use crate::road::{extend_spline, RoadConfig};

            let config = RoadConfig::default();
            let samples_per_segment = config.samples_per_segment;
            let smoothing_influence = config.smoothing_influence;

            // Convertir les cellules existantes en positions monde
            let existing_cell_positions: Vec<bevy::prelude::Vec2> = existing.cell_path.iter()
                .map(cell_to_world_pos)
                .collect();

            let (new_start, new_end, new_points) = if is_start_connection {
                // Ajouter au début du chemin
                new_cell_path.insert(0, cell);

                // Étendre la spline au début avec lissage
                let extended_points = extend_spline(
                    &existing_cell_positions,
                    &existing.points,
                    cell_pos,
                    true, // at_start = true
                    samples_per_segment,
                    smoothing_influence
                );

                (cell, *new_cell_path.last().unwrap(), extended_points)
            } else {
                // Ajouter à la fin du chemin
                new_cell_path.push(cell);

                // Étendre la spline à la fin avec lissage
                let extended_points = extend_spline(
                    &existing_cell_positions,
                    &existing.points,
                    cell_pos,
                    false, // at_start = false
                    samples_per_segment,
                    smoothing_influence
                );

                (*new_cell_path.first().unwrap(), cell, extended_points)
            };

            tracing::info!(
                "Extended spline from {} to {} points for {} cells (added at {}, smoothing_influence={})",
                existing.points.len(),
                new_points.len(),
                new_cell_path.len(),
                if is_start_connection { "start" } else { "end" },
                smoothing_influence
            );

            // Créer le segment mis à jour
            let updated_segment = RoadSegment {
                id: 0, // Nouveau ID sera assigné
                start_cell: new_start,
                end_cell: new_end,
                cell_path: new_cell_path.clone(),
                points: new_points,
                importance: 1,
                road_type: shared::RoadType::default(), // Chemin de terre par défaut
            };

            // Supprimer l'ancien segment et sauvegarder le nouveau
            // (On pourrait aussi faire une mise à jour, mais c'est plus simple de recréer)
            tracing::info!("Deleting old road segment with id {}", existing.id);
            self.db_tables.road_segments
                .delete_road_segment(existing.id)
                .await
                .map_err(|e| format!("Failed to delete old road segment: {}", e))?;

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

            // Régénérer et envoyer immédiatement la SDF de route pour TOUS les chunks où ce segment est visible
            // Cela garantit que les routes qui traversent plusieurs chunks sont visibles partout
            self.regenerate_road_sdf_for_segment(segment_id).await;
        } else {
            // Créer une nouvelle route d'un seul point (comme dans Godot)
            // La route sera juste un point sur cette cellule
            tracing::info!(
                "Creating new single-point road on cell ({},{})",
                cell.q, cell.r
            );

            let segment = RoadSegment {
                id: 0,
                start_cell: cell,
                end_cell: cell,  // Même cellule pour indiquer un point unique
                cell_path: vec![cell],  // Une seule cellule dans le chemin
                points: vec![cell_pos],  // Un seul point
                importance: 1,
                road_type: shared::RoadType::default(), // Chemin de terre par défaut
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

            // Régénérer et envoyer immédiatement la SDF de route pour tous les chunks où ce segment est visible
            // Même pour un point unique, il peut être proche d'une frontière et visible dans plusieurs chunks
            self.regenerate_road_sdf_for_segment(segment_id).await;
        }

        Ok(())
    }

    /// Régénère la SDF de route pour un chunk et ses 8 voisins
    /// Utile pour garantir la cohérence visuelle des routes diagonales
    async fn regenerate_road_sdf_for_chunk_and_neighbors(&self, chunk_id: &shared::TerrainChunkId) {
        tracing::info!("Regenerating road SDF for chunk ({},{}) and its neighbors", chunk_id.x, chunk_id.y);

        // Régénérer pour le chunk central et ses 8 voisins
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor_chunk = shared::TerrainChunkId {
                    x: chunk_id.x + dx,
                    y: chunk_id.y + dy,
                };
                self.regenerate_and_send_road_sdf(&neighbor_chunk).await;
            }
        }
    }

    /// Régénère la SDF de route pour tous les chunks où un segment est visible
    /// Utilisé quand on modifie un segment qui peut traverser plusieurs chunks
    async fn regenerate_road_sdf_for_segment(&self, segment_id: i64) {
        // Récupérer tous les chunks où ce segment est visible
        let chunks = match self.db_tables.road_segments
            .get_chunks_for_segment(segment_id)
            .await
        {
            Ok(chunks) => chunks,
            Err(e) => {
                tracing::error!("Failed to get chunks for segment {}: {}", segment_id, e);
                return;
            }
        };

        tracing::info!(
            "Regenerating road SDF for segment {} across {} chunks: {:?}",
            segment_id,
            chunks.len(),
            chunks
        );

        // Régénérer le SDF pour chaque chunk affecté
        for (chunk_x, chunk_y) in chunks {
            let chunk_id = shared::TerrainChunkId { x: chunk_x, y: chunk_y };
            self.regenerate_and_send_road_sdf(&chunk_id).await;
        }
    }

    /// Régénère la SDF de route pour un chunk et l'envoie à tous les joueurs
    async fn regenerate_and_send_road_sdf(&self, chunk_id: &shared::TerrainChunkId) {
        // Charger les segments visibles dans ce chunk via la table de visibilité
        match self.db_tables.road_segments
            .load_road_segments_by_chunk_new(chunk_id.x, chunk_id.y)
            .await
        {
            Ok(road_segments) if !road_segments.is_empty() => {
                tracing::info!(
                    "Regenerating road SDF for chunk ({},{}) with {} segments",
                    chunk_id.x,
                    chunk_id.y,
                    road_segments.len()
                );

                // Log détaillé de tous les segments
                for (i, seg) in road_segments.iter().enumerate() {
                    tracing::info!(
                        "  Segment {}: id={}, cells={}, points={}, start=({},{}), end=({},{})",
                        i, seg.id, seg.cell_path.len(), seg.points.len(),
                        seg.start_cell.q, seg.start_cell.r,
                        seg.end_cell.q, seg.end_cell.r
                    );
                }

                use crate::road::{RoadConfig, compute_intersections, generate_road_sdf};

                let config = RoadConfig::default();
                let intersections = compute_intersections(&road_segments, &config);
                let road_sdf = generate_road_sdf(
                    &road_segments,
                    &intersections,
                    &config,
                    chunk_id.x,
                    chunk_id.y
                );

                tracing::info!(
                    "✓ Road SDF regenerated: {}x{} with {} intersections",
                    config.sdf_resolution.x,
                    config.sdf_resolution.y,
                    intersections.len()
                );

                // Envoyer la mise à jour de la SDF à tous les joueurs du chunk
                let road_update = shared::protocol::ServerMessage::RoadChunkSdfUpdate {
                    terrain_name: "Gaulyia".to_string(),
                    chunk_id: *chunk_id,
                    road_sdf_data: road_sdf,
                };

                self.broadcast_to_chunk(chunk_id, road_update).await;
            }
            Ok(_) => {
                tracing::debug!("No road segments found for chunk ({},{})", chunk_id.x, chunk_id.y);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load road segments for chunk ({},{}): {}",
                    chunk_id.x,
                    chunk_id.y,
                    e
                );
            }
        }
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
        let message_type = match &message {
            ServerMessage::LoginSuccess { .. } => "LoginSuccess",
            ServerMessage::LoginError { .. } => "LoginError",
            ServerMessage::TerrainChunkData { .. } => "TerrainChunkData",
            ServerMessage::OceanData { .. } => "OceanData",
            ServerMessage::RoadChunkSdfUpdate { chunk_id, .. } => {
                tracing::info!("Sending RoadChunkSdfUpdate to player {} for chunk ({},{})", player_id, chunk_id.x, chunk_id.y);
                "RoadChunkSdfUpdate"
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

        if !matches!(message, ServerMessage::RoadChunkSdfUpdate { .. }) {
            tracing::debug!("Sending {} to player {}", message_type, player_id);
        }

        if let Err(e) = self.sessions.send_to_player(player_id, message).await {
            tracing::warn!("Failed to send {} to player {}: {}", message_type, player_id, e);
        }
    }

    /// Broadcast un message à tous les joueurs qui ont chargé un chunk
    async fn broadcast_to_chunk(&self, _chunk_id: &TerrainChunkId, message: ServerMessage) {
        // TODO: Implémenter le broadcast aux joueurs d'un chunk spécifique
        // Pour l'instant on broadcast à tous les joueurs
        tracing::debug!("Broadcasting message to all players (chunk-specific broadcast not yet implemented)");
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
