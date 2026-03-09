use shared::{BuildingTypeEnum, ProfessionEnum, TerrainChunkId, grid::GridCell};
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::database::client::DatabaseTables;
use crate::networking::Sessions;
use crate::units::{NameGenerator, PortraitGenerator};

pub struct PopulationSystem {
    db_tables: Arc<DatabaseTables>,
    sessions: Sessions,
    name_generator: Arc<NameGenerator>,
}

impl PopulationSystem {
    pub fn new(
        db_tables: Arc<DatabaseTables>,
        sessions: Sessions,
        name_generator: Arc<NameGenerator>,
    ) -> Self {
        Self {
            db_tables,
            sessions,
            name_generator,
        }
    }

    /// Tick principal — appelé toutes les ~60 secondes
    pub async fn tick(&self) {
        // Charger toutes les organisations actives
        let orgs = match self.load_active_organizations().await {
            Ok(orgs) => orgs,
            Err(e) => {
                tracing::error!("Population tick: failed to load orgs: {}", e);
                return;
            }
        };

        for (org_id, leader_unit_id) in &orgs {
            if let Err(e) = self.tick_organization(*org_id, *leader_unit_id).await {
                tracing::warn!("Population tick failed for org {}: {}", org_id, e);
            }
        }
    }

    /// Charge les organisations qui ont un leader (donc fondées par un joueur)
    async fn load_active_organizations(&self) -> Result<Vec<(u64, u64)>, String> {
        let rows = sqlx::query(
            "SELECT id, leader_unit_id FROM organizations.organizations WHERE leader_unit_id IS NOT NULL"
        )
        .fetch_all(&self.db_tables.pool)
        .await
        .map_err(|e| format!("Failed to load organizations: {}", e))?;

        Ok(rows
            .iter()
            .map(|r| {
                (
                    r.get::<i64, _>("id") as u64,
                    r.get::<i64, _>("leader_unit_id") as u64,
                )
            })
            .collect())
    }

    /// Tick une organisation individuelle
    async fn tick_organization(&self, org_id: u64, leader_unit_id: u64) -> Result<(), String> {
        // 1. Calculer la capacité de logement (somme des housing_capacity des bâtiments)
        let housing_capacity = self.calculate_housing_capacity(org_id).await?;

        // 2. Compter la population actuelle (unités NPC sur les cellules du territoire)
        let current_population = self.count_population(org_id).await?;

        tracing::debug!(
            "Org {}: housing={}, population={}",
            org_id,
            housing_capacity,
            current_population
        );

        // 3. Si de la place → spawn un immigrant
        if housing_capacity > current_population as u32 && housing_capacity > 0 {
            match self.spawn_immigrant(org_id, leader_unit_id).await {
                Ok(unit_data) => {
                    let new_pop = current_population + 1;

                    // Mettre à jour le compteur de population
                    let _ = sqlx::query(
                        "UPDATE organizations.organizations SET population = $1 WHERE id = $2",
                    )
                    .bind(new_pop as i32)
                    .bind(org_id as i64)
                    .execute(&self.db_tables.pool)
                    .await;

                    tracing::info!(
                        "✓ Immigrant {} {} joined org {} (pop: {} → {})",
                        unit_data.first_name,
                        unit_data.last_name,
                        org_id,
                        current_population,
                        new_pop
                    );

                    // Notifier le leader (le joueur qui possède cette org)
                    // Trouver le player_id du leader
                    if let Ok(leader) = self.db_tables.units.load_unit(leader_unit_id).await {
                        if let Some(player_id) = leader.player_id {
                            let msg = shared::protocol::ServerMessage::PopulationChanged {
                                organization_id: org_id,
                                new_population: new_pop as i32,
                                immigrant: Some(unit_data.clone()),
                            };
                            let _ = self.sessions.send_to_player(player_id, msg).await;

                            // Envoyer aussi DebugUnitSpawned pour que le client
                            // ajoute l'unité à ses caches
                            let spawn_msg =
                                shared::protocol::ServerMessage::DebugUnitSpawned { unit_data };
                            let _ = self.sessions.send_to_player(player_id, spawn_msg).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to spawn immigrant for org {}: {}", org_id, e);
                }
            }
        }

        Ok(())
    }

    /// Calcule la capacité totale de logement pour une organisation
    async fn calculate_housing_capacity(&self, org_id: u64) -> Result<u32, String> {
        // Joindre les bâtiments avec les cellules du territoire
        let rows = sqlx::query(
            r#"
            SELECT b.building_type_id
            FROM buildings.buildings_base b
            INNER JOIN organizations.territory_cells tc
                ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r
            WHERE tc.organization_id = $1
              AND b.is_built = true
            "#,
        )
        .bind(org_id as i64)
        .fetch_all(&self.db_tables.pool)
        .await
        .map_err(|e| format!("Failed to count buildings: {}", e))?;

        let total: u32 = rows
            .iter()
            .filter_map(|r| {
                let type_id: i32 = r.get("building_type_id");
                BuildingTypeEnum::from_id(type_id as i16).map(|bt| bt.housing_capacity())
            })
            .sum();

        Ok(total)
    }

    /// Compte les unités NPC dans le territoire d'une organisation
    async fn count_population(&self, org_id: u64) -> Result<usize, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as cnt
            FROM units.units u
            INNER JOIN organizations.territory_cells tc
                ON u.current_cell_q = tc.cell_q AND u.current_cell_r = tc.cell_r
            WHERE tc.organization_id = $1
              AND u.is_lord = false
            "#,
        )
        .bind(org_id as i64)
        .fetch_one(&self.db_tables.pool)
        .await
        .map_err(|e| format!("Failed to count population: {}", e))?;

        Ok(row.get::<i64, _>("cnt") as usize)
    }

    /// Spawn un immigrant NPC dans l'organisation
    async fn spawn_immigrant(
        &self,
        org_id: u64,
        leader_unit_id: u64,
    ) -> Result<shared::UnitData, String> {
        // 1. Trouver une cellule avec un bâtiment qui a de la place
        let target = self.find_available_cell(org_id).await?;
        let (cell, chunk, building_type) = target;

        let (is_male, gender_str, profession) = {
            // 2. Choisir profession en fonction du bâtiment
            let professions = building_type.relevant_professions();
            let profession = if professions.is_empty() {
                ProfessionEnum::Farmer
            } else {
                let idx = rand::rng().random_range(0..professions.len());
                professions[idx]
            };

            // 3. Générer nom/genre/portrait
            use rand::Rng;
            let mut rng = rand::rng();
            let is_male: bool = rng.random_bool(0.5);
            let gender_str = if is_male { "male" } else { "female" };
            (is_male, gender_str, profession)
        };

        let (first_name, last_name) = self.name_generator.generate_random_name(Some(is_male));
        let (variant_id, avatar_url) =
            PortraitGenerator::generate_variant_and_url(gender_str, profession);

        // 4. Créer l'unité
        let unit_id = self
            .db_tables
            .units
            .create_unit(
                None, // NPC — pas de player_id
                first_name.clone(),
                last_name.clone(),
                gender_str.to_string(),
                variant_id,
                avatar_url,
                cell,
                chunk,
                profession,
                false, // is_lord = false
                None,  // portrait_layers = None
            )
            .await?;

        // 5. Assigner un slot (trouver un slot libre sur la cellule)
        self.assign_random_slot(unit_id, &cell, &chunk, building_type)
            .await;

        // 6. Ajouter comme membre de l'organisation
        let _ = self
            .db_tables
            .organizations
            .add_member(org_id, unit_id, None)
            .await;

        // 7. Charger et retourner les données complètes
        self.db_tables.units.load_unit(unit_id).await
    }

    /// Trouve une cellule du territoire avec un bâtiment qui a de la place
    async fn find_available_cell(
        &self,
        org_id: u64,
    ) -> Result<(GridCell, TerrainChunkId, BuildingTypeEnum), String> {
        // Charger tous les bâtiments du territoire avec leurs infos
        let rows = sqlx::query(
            r#"
            SELECT b.cell_q, b.cell_r, b.chunk_x, b.chunk_y, b.building_type_id,
                   (SELECT COUNT(*) FROM units.units u
                    WHERE u.current_cell_q = b.cell_q AND u.current_cell_r = b.cell_r
                      AND u.is_lord = false) as unit_count
            FROM buildings.buildings_base b
            INNER JOIN organizations.territory_cells tc
                ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r
            WHERE tc.organization_id = $1
              AND b.is_built = true
            ORDER BY unit_count ASC
            "#,
        )
        .bind(org_id as i64)
        .fetch_all(&self.db_tables.pool)
        .await
        .map_err(|e| format!("Failed to find available cell: {}", e))?;

        for row in &rows {
            let type_id: i32 = row.get("building_type_id");
            let unit_count: i64 = row.get("unit_count");

            if let Some(building_type) = BuildingTypeEnum::from_id(type_id as i16) {
                let capacity = building_type.housing_capacity();
                if unit_count < capacity as i64 {
                    return Ok((
                        GridCell {
                            q: row.get("cell_q"),
                            r: row.get("cell_r"),
                        },
                        TerrainChunkId {
                            x: row.get("chunk_x"),
                            y: row.get("chunk_y"),
                        },
                        building_type,
                    ));
                }
            }
        }

        Err("No available cell with housing capacity".to_string())
    }

    /// Assigne un slot aléatoire disponible à l'unité
    async fn assign_random_slot(
        &self,
        unit_id: u64,
        cell: &GridCell,
        chunk: &TerrainChunkId,
        building_type: BuildingTypeEnum,
    ) {
        use shared::{SlotConfiguration, SlotPosition, SlotType};

        let slot_config = SlotConfiguration::for_building_type(building_type);

        // Récupérer les slots déjà occupés
        let occupied = self
            .db_tables
            .units
            .get_occupied_slots_on_cell(cell, chunk)
            .await
            .unwrap_or_default();

        // Essayer les slots intérieurs d'abord, puis extérieurs
        let slot_candidates: Vec<SlotPosition> = (0..slot_config.interior_slots())
            .map(|i| SlotPosition {
                slot_type: SlotType::Interior,
                index: i,
            })
            .chain((0..slot_config.exterior_slots()).map(|i| SlotPosition {
                slot_type: SlotType::Exterior,
                index: i,
            }))
            .collect();

        for slot in &slot_candidates {
            if !occupied.contains(slot) {
                let slot_type_str = match slot.slot_type {
                    SlotType::Interior => "interior",
                    SlotType::Exterior => "exterior",
                };
                let _ = self
                    .db_tables
                    .units
                    .update_slot_position(
                        unit_id,
                        Some(slot_type_str.to_string()),
                        Some(slot.index as i32),
                    )
                    .await;
                return;
            }
        }

        tracing::debug!(
            "No free slot for unit {} on cell ({},{})",
            unit_id,
            cell.q,
            cell.r
        );
    }
}

pub fn start_population_tick(system: Arc<PopulationSystem>) {
    tokio::task::spawn(async move {
        // Premier tick après 30 secondes (laisser le serveur se stabiliser)
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            interval.tick().await;
            system.tick().await;
        }
    });
}
