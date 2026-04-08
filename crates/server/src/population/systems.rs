use shared::{BuildingTypeEnum, ProfessionEnum, SlotConfiguration, SlotPosition, SlotType, TerrainChunkId, grid::GridCell};
use sqlx::Row;
use std::sync::Arc;

use crate::database::client::DatabaseTables;
use crate::networking::server::bridge::{BridgeEvent, BridgeSender};
use crate::units::{NameGenerator, PortraitGenerator};

pub struct PopulationSystem {
    db_tables: Arc<DatabaseTables>,
    bridge_sender: Arc<BridgeSender>,
    name_generator: Arc<NameGenerator>,
}

impl PopulationSystem {
    pub fn new(
        db_tables: Arc<DatabaseTables>,
        bridge_sender: Arc<BridgeSender>,
        name_generator: Arc<NameGenerator>,
    ) -> Self {
        Self {
            db_tables,
            bridge_sender,
            name_generator,
        }
    }

    /// Main tick — called every ~60 seconds
    pub async fn tick(&self) {
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

    /// Load organizations that have a leader (founded by a player)
    async fn load_active_organizations(&self) -> Result<Vec<(u64, u64)>, String> {
        let rows = sqlx::query(
            "SELECT id, leader_unit_id FROM organizations.organizations WHERE leader_unit_id IS NOT NULL",
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

    /// Tick an individual organization — logistic growth + named unit emergence
    async fn tick_organization(&self, org_id: u64, leader_unit_id: u64) -> Result<(), String> {
        // 1. Housing capacity = sum of housing_capacity() of all built buildings
        let capacity = self.calculate_housing_capacity(org_id).await?;

        // 2. Current aggregated population from DB
        let current_pop = self.load_population(org_id).await?;

        // 3. Named unit count (non-lord units in territory)
        let named_count = self.count_named_units(org_id).await?;

        // 4. Logistic growth (only if capacity > 0 and room available)
        let mut new_pop = current_pop;
        if capacity > 0 && current_pop < capacity as i32 {
            let cap = capacity as f64;
            let pop = current_pop as f64;
            let r = 0.05; // 5% growth rate per tick
            let growth = (r * pop * (1.0 - pop / cap)).max(0.0);
            // Bootstrap: minimum +1 if below capacity (so pop=1 can grow)
            let growth = if pop < cap && growth < 1.0 {
                1.0
            } else {
                growth
            };
            new_pop = ((pop + growth) as i32).min(capacity as i32);

            if new_pop > current_pop {
                self.update_population(org_id, new_pop).await?;
                self.notify_population_changed(
                    org_id,
                    leader_unit_id,
                    new_pop,
                    named_count as i32,
                    capacity as i32,
                    None,
                )
                .await;

                tracing::info!(
                    "📈 Org {} population: {} → {} / {} ({} named)",
                    org_id,
                    current_pop,
                    new_pop,
                    capacity,
                    named_count
                );
            }
        }

        // 5. Named unit emergence check
        let emergence_threshold = 25 * (named_count as i32 + 1);

        if new_pop >= emergence_threshold {
            match self.find_cell_with_free_slot(org_id).await {
                Ok((cell, chunk, building_type)) => {
                    match self
                        .spawn_named_unit(org_id, leader_unit_id, cell, chunk, building_type)
                        .await
                    {
                        Ok(unit_data) => {
                            let new_named = named_count + 1;
                            tracing::info!(
                                "🌟 Notable {} {} emerged in org {} (pop {}, {} named)",
                                unit_data.first_name,
                                unit_data.last_name,
                                org_id,
                                new_pop,
                                new_named
                            );
                            self.notify_population_changed(
                                org_id,
                                leader_unit_id,
                                new_pop,
                                new_named as i32,
                                capacity as i32,
                                Some(unit_data.clone()),
                            )
                            .await;
                            if let Ok(player_id) = self.get_player_id(leader_unit_id).await {
                                self.bridge_sender.send(BridgeEvent::SendDebugUnitSpawned {
                                    player_id,
                                    unit_data,
                                });
                            }
                        }
                        Err(e) => {
                            tracing::debug!("No emergence for org {}: {}", org_id, e);
                        }
                    }
                }
                Err(_) => {} // No free slot — no emergence
            }
        }

        Ok(())
    }

    // ─── Helpers ────────────────────────────────────────────────────────

    /// Sum housing_capacity() of all built buildings in the territory
    async fn calculate_housing_capacity(&self, org_id: u64) -> Result<u32, String> {
        let rows = sqlx::query(
            r#"SELECT b.building_type_id
            FROM buildings.buildings_base b
            INNER JOIN organizations.territory_cells tc
                ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r
            WHERE tc.organization_id = $1
              AND b.is_built = true"#,
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

    async fn load_population(&self, org_id: u64) -> Result<i32, String> {
        sqlx::query_scalar::<_, i32>(
            "SELECT population FROM organizations.organizations WHERE id = $1",
        )
        .bind(org_id as i64)
        .fetch_one(&self.db_tables.pool)
        .await
        .map_err(|e| format!("Failed to load population: {}", e))
    }

    async fn update_population(&self, org_id: u64, new_pop: i32) -> Result<(), String> {
        sqlx::query("UPDATE organizations.organizations SET population = $1 WHERE id = $2")
            .bind(new_pop)
            .bind(org_id as i64)
            .execute(&self.db_tables.pool)
            .await
            .map_err(|e| format!("Failed to update population: {}", e))?;
        Ok(())
    }

    async fn count_named_units(&self, org_id: u64) -> Result<usize, String> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM units.units u
               INNER JOIN organizations.territory_cells tc
                   ON u.current_cell_q = tc.cell_q AND u.current_cell_r = tc.cell_r
               WHERE tc.organization_id = $1 AND u.is_lord = false"#,
        )
        .bind(org_id as i64)
        .fetch_one(&self.db_tables.pool)
        .await
        .map_err(|e| format!("Failed to count named units: {}", e))?;
        Ok(count as usize)
    }

    async fn get_player_id(&self, leader_unit_id: u64) -> Result<u64, String> {
        let leader = self.db_tables.units.load_unit(leader_unit_id).await?;
        leader
            .player_id
            .ok_or_else(|| "Leader has no player_id".to_string())
    }

    async fn notify_population_changed(
        &self,
        org_id: u64,
        leader_unit_id: u64,
        new_pop: i32,
        named_count: i32,
        capacity: i32,
        immigrant: Option<shared::UnitData>,
    ) {
        if let Ok(player_id) = self.get_player_id(leader_unit_id).await {
            self.bridge_sender.send(BridgeEvent::SendPopulationChanged {
                player_id,
                organization_id: org_id,
                new_population: new_pop,
                named_unit_count: named_count,
                population_capacity: capacity,
                immigrant,
            });
        }
    }

    /// Find a cell in the territory with a building that has a free slot
    async fn find_cell_with_free_slot(
        &self,
        org_id: u64,
    ) -> Result<(GridCell, TerrainChunkId, BuildingTypeEnum), String> {
        let rows = sqlx::query(
            r#"SELECT b.cell_q, b.cell_r, b.chunk_x, b.chunk_y, b.building_type_id,
                      (SELECT COUNT(*) FROM units.units u
                       WHERE u.current_cell_q = b.cell_q AND u.current_cell_r = b.cell_r
                         AND u.is_lord = false) as unit_count
               FROM buildings.buildings_base b
               INNER JOIN organizations.territory_cells tc
                   ON b.cell_q = tc.cell_q AND b.cell_r = tc.cell_r
               WHERE tc.organization_id = $1 AND b.is_built = true
               ORDER BY unit_count ASC"#,
        )
        .bind(org_id as i64)
        .fetch_all(&self.db_tables.pool)
        .await
        .map_err(|e| format!("Failed to find cell with free slot: {}", e))?;

        for row in &rows {
            let type_id: i32 = row.get("building_type_id");
            let unit_count: i64 = row.get("unit_count");

            if let Some(building_type) = BuildingTypeEnum::from_id(type_id as i16) {
                let slot_config = SlotConfiguration::for_building_type(building_type);
                let total_slots =
                    (slot_config.interior_slots() + slot_config.exterior_slots()) as i64;
                if unit_count < total_slots {
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

        Err("No building with free slot in territory".to_string())
    }

    /// Spawn a named unit (settler) in the organization
    async fn spawn_named_unit(
        &self,
        org_id: u64,
        _leader_unit_id: u64,
        cell: GridCell,
        chunk: TerrainChunkId,
        building_type: BuildingTypeEnum,
    ) -> Result<shared::UnitData, String> {
        let (is_male, gender_str, profession) = {
            let profession = ProfessionEnum::Settler;
            use rand::Rng;
            let mut rng = rand::rng();
            let is_male: bool = rng.random_bool(0.5);
            let gender_str = if is_male { "male" } else { "female" };
            (is_male, gender_str, profession)
        };

        let (first_name, last_name) = self.name_generator.generate_random_name(Some(is_male));
        let (variant_id, avatar_url) =
            PortraitGenerator::generate_variant_and_url(gender_str, profession);

        let unit_id = self
            .db_tables
            .units
            .create_unit(
                None,
                first_name.clone(),
                last_name.clone(),
                gender_str.to_string(),
                variant_id,
                avatar_url,
                cell,
                chunk,
                profession,
                false,
                None,
            )
            .await?;

        self.assign_random_slot(unit_id, &cell, &chunk, building_type)
            .await;

        let _ = self
            .db_tables
            .organizations
            .add_member(org_id, unit_id, None)
            .await;

        self.db_tables.units.load_unit(unit_id).await
    }

    /// Assign a random free slot to a unit on a cell
    async fn assign_random_slot(
        &self,
        unit_id: u64,
        cell: &GridCell,
        chunk: &TerrainChunkId,
        building_type: BuildingTypeEnum,
    ) {
        let slot_config = SlotConfiguration::for_building_type(building_type);

        let occupied = self
            .db_tables
            .units
            .get_occupied_slots_on_cell(cell, chunk)
            .await
            .unwrap_or_default();

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
        // First tick after 30 seconds (let the server stabilize)
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            interval.tick().await;
            system.tick().await;
        }
    });
}
