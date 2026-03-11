use bevy::prelude::*;

use shared::{
    ActionBaseData, ActionData, ActionSpecificTypeEnum, ActionStatusEnum, ActionTypeEnum,
    BuildBuildingAction, BuildRoadAction, BuildingTypeEnum, CraftResourceAction,
    HarvestResourceAction, MoveUnitAction, ProfessionEnum, ResourceSpecificTypeEnum,
    SendMessageAction, SpecificAction, TerrainChunkId, TrainUnitAction, grid::GridCell,
};
use sqlx::{PgPool, Row};

#[derive(Resource, Clone)]
pub struct ScheduledActionsTable {
    pool: PgPool,
}

impl ScheduledActionsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_scheduled_action(&self, action: &ActionData) -> Result<u64, String> {
        let base_action = &action.base_data;

        // let action_type = base_action.action_type();
        tracing::info!(
            "Adding scheduled action of type {:?} (id: {}), for player {}",
            base_action.action_type,
            base_action.player_id,
            base_action.action_type.to_id()
        );

        let db_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO actions.scheduled_actions 
             (player_id, cell_q, cell_r, chunk_x, chunk_y, action_type_id, action_specific_type_id, start_time, duration_ms, completion_time, status_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING id"
        )
        .bind(base_action.player_id as i64)
        .bind(base_action.cell.q)
        .bind(base_action.cell.r)
        .bind(base_action.chunk.x)
        .bind(base_action.chunk.y)
        .bind(base_action.action_type.to_id())
        .bind(base_action.action_specific_type.to_id())
        .bind(base_action.start_time as i64)
        .bind(base_action.duration_ms as i64)
        .bind(base_action.completion_time as i64)
        .bind(base_action.status.to_id())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        tracing::info!("Scheduled action inserted with ID {}", db_id);

        // Insérer dans la table spécifique
        self.add_action_data(db_id as u64, &action.specific_data)
            .await?;

        tracing::info!(
            "Added scheduled action ID {} for player {}",
            db_id,
            base_action.player_id
        );

        Ok(db_id as u64)
    }

    pub async fn add_action_data(
        &self,
        action_id: u64,
        action: &SpecificAction,
    ) -> Result<(), String> {
        match action {
            SpecificAction::BuildBuilding(a) => {
                sqlx::query(
                    "INSERT INTO actions.build_building_actions (action_id, building_type_id) VALUES ($1, $2)",
                )
                .bind(action_id as i64)
                .bind(a.building_type.to_id())
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::BuildRoad(a) => {
                sqlx::query(
                    "INSERT INTO actions.build_road_actions (action_id, start_q, start_r, end_q, end_r) VALUES ($1, $2, $3, $4, $5)"
                )
                    .bind(action_id as i64)
                    .bind(a.start_cell.q)
                    .bind(a.start_cell.r)
                    .bind(a.end_cell.q)
                    .bind(a.end_cell.r)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::MoveUnit(a) => {
                sqlx::query(
                    "INSERT INTO actions.move_unit_actions (action_id, unit_id, target_q, target_r) VALUES ($1, $2, $3, $4)"
                )
                .bind(action_id as i64)
                .bind(a.unit_id as i64)
                .bind(a.cell.q)
                .bind(a.cell.r)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::SendMessage(a) => {
                sqlx::query(
                    "INSERT INTO actions.send_message_actions (action_id, receiver_id, content) VALUES ($1, $2, $3)"
                )
                .bind(action_id as i64)
                .bind(a.receivers[0] as i64)
                .bind(&a.content)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::HarvestResource(a) => {
                sqlx::query(
                    "INSERT INTO actions.harvest_resource_actions (action_id, resource_type_id) VALUES ($1, $2)"
                )
                .bind(action_id as i64)
                .bind(a.resource_specific_type.to_id())
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::CraftResource(a) => {
                sqlx::query(
                    "INSERT INTO actions.craft_resource_actions (action_id, recipe_id, quantity) VALUES ($1, $2, $3)"
                )
                .bind(action_id as i64)
                .bind(&a.recipe_id)
                .bind(a.quantity as i32)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::TrainUnit(a) => {
                sqlx::query(
                    "INSERT INTO actions.train_unit_actions (action_id, unit_id, target_profession_id) VALUES ($1, $2, $3)"
                )
                .bind(action_id as i64)
                .bind(a.unit_id as i64)
                .bind(a.target_profession.to_id())
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn load_chunk_actions(
        &self,
        chunk_id: &TerrainChunkId,
    ) -> Result<Vec<ActionData>, sqlx::Error> {
        let mut actions = Vec::new();

        let action_base_rows = sqlx::query(
            r#"
            SELECT
                id, player_id,
                cell_q, cell_r, 
                action_type_id, action_specific_type_id, start_time, duration_ms, completion_time, status
            FROM scheduled_actions 
            WHERE chunk_x = $1 AND chunk_y = $2 
            AND status IN ('InProgress', 'Pending')
        "#,
        )
        .bind(chunk_id.x)
        .bind(chunk_id.y)
        .fetch_all(&self.pool)
        .await?;

        for r in action_base_rows {
            let id = r.get::<i64, &str>("id");
            let player_id = r.get::<i64, &str>("player_id") as u64;
            let Some(action_type) = ActionTypeEnum::from_id(r.get("action_type_id")) else {
                continue;
            };
            let Some(action_specific_type) =
                ActionSpecificTypeEnum::from_id(r.get("action_specific_type_id"))
            else {
                continue;
            };
            let cell = GridCell {
                q: r.get("cell_q"),
                r: r.get("cell_r"),
            };
            let Some(status) = ActionStatusEnum::from_id(r.get("action_status")) else {
                continue;
            };

            let base_data = ActionBaseData {
                player_id: r.get::<i64, &str>("player_id") as u64,
                cell,
                chunk: *chunk_id,
                action_type,
                action_specific_type,
                start_time: r.get::<i64, &str>("start_time") as u64,
                duration_ms: r.get::<i64, &str>("duration_ms") as u64,
                completion_time: r.get::<i64, &str>("completion_time") as u64,

                status,
            };

            let specific_data = match action_specific_type {
                ActionSpecificTypeEnum::BuildBuilding => {
                    let build_building = sqlx::query(
                        r#"
                            SELECT building_type_id
                            FROM actions.build_building_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let Some(building_type) =
                        BuildingTypeEnum::from_id(build_building.get("building_type_id"))
                    else {
                        continue;
                    };

                    let building_specific_type = building_type.to_specific_type();

                    SpecificAction::BuildBuilding(BuildBuildingAction {
                        player_id,
                        chunk_id: *chunk_id,
                        cell,
                        building_type,
                        building_specific_type,
                    })
                }
                ActionSpecificTypeEnum::BuildRoad => {
                    let build_road = sqlx::query(
                        r#"
                            SELECT start_q, start_r, end_q, end_r
                            FROM actions.build_road_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let start_q: i32 = build_road.get("start_q");
                    let start_r: i32 = build_road.get("start_r");
                    let end_q: i32 = build_road.get("end_q");
                    let end_r: i32 = build_road.get("end_r");

                    SpecificAction::BuildRoad(BuildRoadAction {
                        player_id,
                        start_cell: GridCell {
                            q: start_q,
                            r: start_r,
                        },
                        end_cell: GridCell { q: end_q, r: end_r },
                    })
                }
                ActionSpecificTypeEnum::CraftResource => {
                    let _craft_resource = sqlx::query(
                        r#"
                            SELECT recipe_id, quantity
                            FROM actions.craft_resource_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    SpecificAction::CraftResource(CraftResourceAction {
                        player_id,
                        recipe_id: r.get("recipe_id"),
                        chunk_id: *chunk_id,
                        cell,
                        quantity: r.get::<i32, &str>("quantity") as u32,
                    })
                }
                ActionSpecificTypeEnum::HarvestResource => {
                    let _harvest_resource = sqlx::query(
                        r#"
                            SELECT resource_type_id
                            FROM actions.harvest_resource_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let Some(resource_specific_type) =
                        ResourceSpecificTypeEnum::from_id(r.get("resource_type_id"))
                    else {
                        continue;
                    };

                    SpecificAction::HarvestResource(HarvestResourceAction {
                        player_id,
                        resource_specific_type,
                        chunk_id: *chunk_id,
                        cell,
                    })
                }
                ActionSpecificTypeEnum::MoveUnit => {
                    let _move_unit = sqlx::query(
                        r#"
                            SELECT unit_id, cell_q, cell_r
                            FROM actions.move_unit_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    SpecificAction::MoveUnit(MoveUnitAction {
                        player_id,
                        unit_id: r.get::<i64, &str>("unit_id") as u64,
                        chunk_id: *chunk_id,
                        cell: GridCell {
                            q: r.get("cell_q"),
                            r: r.get("cell_r"),
                        },
                    })
                }
                ActionSpecificTypeEnum::SendMessage => {
                    let _send_message = sqlx::query(
                        r#"
                            SELECT 
                                sma.action_id,
                                sma.message_content,
                                COALESCE(ARRAY_AGG(smr.receiver_id), '{}') as receivers
                            FROM actions.send_message_actions sma
                            LEFT JOIN actions.send_message_receivers smr 
                                ON sma.action_id = smr.action_id
                            WHERE sma.action_id = $1
                            GROUP BY sma.action_id, sma.message_content
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let receivers_array: Vec<i64> = r.get("receivers");
                    SpecificAction::SendMessage(SendMessageAction {
                        player_id,
                        receivers: receivers_array.into_iter().map(|uid| uid as u64).collect(),
                        content: r.get("message_content"),
                    })
                }
                ActionSpecificTypeEnum::TrainUnit => {
                    let train_row = sqlx::query(
                        r#"
                            SELECT unit_id, target_profession_id
                            FROM actions.train_unit_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;

                    let unit_id = train_row.get::<i64, &str>("unit_id") as u64;
                    let profession_id: i16 = train_row.get("target_profession_id");
                    let target_profession =
                        ProfessionEnum::from_id(profession_id).unwrap_or(ProfessionEnum::Unknown);

                    SpecificAction::TrainUnit(TrainUnitAction {
                        player_id,
                        unit_id,
                        chunk_id: *chunk_id,
                        cell,
                        target_profession,
                    })
                }
            };

            actions.push(ActionData {
                base_data,
                specific_data,
            });
        }

        Ok(actions)
    }

    /// Charge toutes les actions actives (Pending ou InProgress)
    pub async fn load_active_actions(
        &self,
    ) -> Result<
        Vec<(
            u64,
            u64,
            TerrainChunkId,
            GridCell,
            ActionTypeEnum,
            ActionStatusEnum,
            u64,
            u64,
            u64,
        )>,
        String,
    > {
        let result = sqlx::query(
            r#"
            SELECT
                id, player_id, chunk_x, chunk_y, cell_q, cell_r,
                action_type_id, status_id, start_time, duration_ms, completion_time
            FROM actions.scheduled_actions
            WHERE status_id IN (1, 2)
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load active actions: {}", e))?;

        let mut actions = Vec::new();

        for row in result {
            let action_id = row.get::<i64, &str>("id") as u64;
            let player_id = row.get::<i64, &str>("player_id") as u64;
            let Some(action_type) = ActionTypeEnum::from_id(row.get("action_type_id")) else {
                continue;
            };
            let Some(status) = ActionStatusEnum::from_id(row.get("status_id")) else {
                continue;
            };

            let chunk_id = TerrainChunkId {
                x: row.get("chunk_x"),
                y: row.get("chunk_y"),
            };

            let cell = GridCell {
                q: row.get("cell_q"),
                r: row.get("cell_r"),
            };

            let start_time = row.get::<i64, &str>("start_time") as u64;
            let duration_ms = row.get::<i64, &str>("duration_ms") as u64;
            let completion_time = row.get::<i64, &str>("completion_time") as u64;

            actions.push((
                action_id,
                player_id,
                chunk_id,
                cell,
                action_type,
                status,
                start_time,
                duration_ms,
                completion_time,
            ));
        }

        Ok(actions)
    }

    /// Load train_unit specific data for a completed action.
    /// Returns (unit_id, target_profession) or None if not found.
    pub async fn load_train_unit_data(
        &self,
        action_id: u64,
    ) -> Result<Option<(u64, ProfessionEnum)>, String> {
        let row = sqlx::query(
            r#"
            SELECT unit_id, target_profession_id
            FROM actions.train_unit_actions
            WHERE action_id = $1
            "#,
        )
        .bind(action_id as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load train_unit data: {}", e))?;

        Ok(row.map(|r| {
            let unit_id = r.get::<i64, &str>("unit_id") as u64;
            let profession_id: i16 = r.get("target_profession_id");
            let profession =
                ProfessionEnum::from_id(profession_id).unwrap_or(ProfessionEnum::Unknown);
            (unit_id, profession)
        }))
    }

    /// Charge les données spécifiques d'une action MoveUnit
    /// Table: actions.move_unit_actions (action_id, unit_id, target_q, target_r)
    /// Le chunk cible est dans la table parent scheduled_actions (chunk_x, chunk_y)
    pub async fn load_move_unit_data(
        &self,
        action_id: u64,
    ) -> Result<Option<(u64, GridCell, TerrainChunkId)>, String> {
        let row = sqlx::query(
            r#"
            SELECT m.unit_id, m.target_q, m.target_r,
                   s.chunk_x, s.chunk_y
            FROM actions.move_unit_actions m
            JOIN actions.scheduled_actions s ON s.id = m.action_id
            WHERE m.action_id = $1
            "#,
        )
        .bind(action_id as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load move_unit data: {}", e))?;

        Ok(row.map(|r| {
            let unit_id = r.get::<i64, _>("unit_id") as u64;
            let cell = GridCell {
                q: r.get("target_q"),
                r: r.get("target_r"),
            };
            let chunk = TerrainChunkId {
                x: r.get("chunk_x"),
                y: r.get("chunk_y"),
            };
            (unit_id, cell, chunk)
        }))
    }

    /// Met à jour le statut d'une action
    pub async fn update_action_status(
        &self,
        action_id: u64,
        new_status: ActionStatusEnum,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE actions.scheduled_actions
            SET status_id = $1
            WHERE id = $2
            "#,
        )
        .bind(new_status.to_id())
        .bind(action_id as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update action status: {}", e))?;

        Ok(())
    }

    /// Charge les données d'une action HarvestResource
    pub async fn load_harvest_data(
        &self,
        action_id: u64,
    ) -> Result<Option<(u64, ResourceSpecificTypeEnum)>, String> {
        let row = sqlx::query(
            r#"
            SELECT sa.player_id, hra.resource_type_id
            FROM actions.scheduled_actions sa
            JOIN actions.harvest_resource_actions hra ON hra.action_id = sa.id
            WHERE sa.id = $1
            "#,
        )
        .bind(action_id as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load harvest data for action {}: {}", action_id, e))?;

        Ok(row.map(|r| {
            let player_id: i64 = r.get("player_id");
            let resource_type_id: i16 = r.get::<i32, _>("resource_type_id") as i16;
            let resource_type = ResourceSpecificTypeEnum::from_id(resource_type_id)
                .unwrap_or(ResourceSpecificTypeEnum::Unknown);
            (player_id as u64, resource_type)
        }))
    }

    /// Charge les données d'une action CraftResource
    pub async fn load_craft_data(
        &self,
        action_id: u64,
    ) -> Result<Option<(u64, String, u32)>, String> {
        let row = sqlx::query(
            r#"
            SELECT sa.player_id, cra.recipe_id, cra.quantity
            FROM actions.scheduled_actions sa
            JOIN actions.craft_resource_actions cra ON cra.action_id = sa.id
            WHERE sa.id = $1
            "#,
        )
        .bind(action_id as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load craft data for action {}: {}", action_id, e))?;

        Ok(row.map(|r| {
            let player_id: i64 = r.get("player_id");
            let recipe_id: String = r.get("recipe_id");
            let quantity: i32 = r.get("quantity");
            (player_id as u64, recipe_id, quantity as u32)
        }))
    }

    /// Récupère le building_type_id pour une action BuildBuilding
    pub async fn get_build_building_type(&self, action_id: u64) -> Result<Option<i16>, String> {
        let result = sqlx::query(
            r#"
            SELECT building_type_id
            FROM actions.build_building_actions
            WHERE action_id = $1
            "#,
        )
        .bind(action_id as i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get building type: {}", e))?;

        Ok(result.map(|row| row.get::<i32, &str>("building_type_id") as i16))
    }

    /// Récupère les cellules start et end pour une action BuildRoad
    pub async fn get_build_road_cells(
        &self,
        action_id: u64,
    ) -> Result<(GridCell, GridCell), String> {
        let result = sqlx::query(
            r#"
            SELECT start_q, start_r, end_q, end_r
            FROM actions.build_road_actions
            WHERE action_id = $1
            "#,
        )
        .bind(action_id as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get road cells: {}", e))?;

        let start_cell = GridCell {
            q: result.get("start_q"),
            r: result.get("start_r"),
        };
        let end_cell = GridCell {
            q: result.get("end_q"),
            r: result.get("end_r"),
        };

        Ok((start_cell, end_cell))
    }
}
