use bevy::prelude::*;

use shared::{
    ActionBaseData, ActionData, ActionSpecificTypeEnum, ActionStatusEnum, ActionTypeEnum, BuildBuildingAction, BuildRoadAction, BuildingSpecificTypeEnum, CraftResourceAction, HarvestResourceAction, MoveUnitAction, ResourceSpecificTypeEnum, SendMessageAction, SpecificAction, SpecificActionData, TerrainChunkId, grid::GridCell
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

        let db_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scheduled_actions 
             (player_id, cell_q, cell_r, chunk_x, chunk_y, action_type_id, action_specific_type_id, start_time, duration_ms, completion_time, status)
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

        // Insérer dans la table spécifique
        self.add_action_data(db_id as u64, &action.specific_data)
            .await?;

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
                .bind(a.building_specific_type.to_id())
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::BuildRoad(a) => {
                sqlx::query("INSERT INTO actions.build_road_actions (action_id) VALUES ($1, $2)")
                    .bind(action_id as i64)
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
                    "INSERT INTO actions.harvest_resource_actions (action_id, resource_type) VALUES ($1, $2)"
                )
                .bind(action_id as i64)
                .bind(&a.resource_specific_type.to_id())
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
            _ => {}
        }

        Ok(())
    }

    pub async fn load_chunk_actions(
        &self,
        chunk_id: TerrainChunkId,
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
                cell: cell.clone(),
                chunk: chunk_id.clone(),
                action_type: action_type.clone(),
                action_specific_type: action_specific_type.clone(),
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
                    .bind(id as i64)
                    .fetch_one(&self.pool)
                    .await?;

                    let Some(building_specific_type) =
                        BuildingSpecificTypeEnum::from_id(build_building.get("building_type_id"))
                    else {
                        continue;
                    };

                    SpecificAction::BuildBuilding(BuildBuildingAction {
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        building_specific_type,
                    })
                }
                ActionSpecificTypeEnum::BuildRoad => {
                    // let build_road = sqlx::query(
                    //     r#"
                    //         SELECT *
                    //         WHERE action_id = $1
                    //     "#,
                    // )
                    // .bind(id as i64)
                    // .fetch_one(&self.pool)
                    // .await?;

                    SpecificAction::BuildRoad(BuildRoadAction {
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                    })
                }
                ActionSpecificTypeEnum::CraftResource => {
                    let craft_resource = sqlx::query(
                        r#"
                            SELECT recipe_id, quantity
                            FROM actions.craft_resource_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id as i64)
                    .fetch_one(&self.pool)
                    .await?;

                    SpecificAction::CraftResource(CraftResourceAction {
                        player_id,
                        recipe_id: r.get("recipe_id"),
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        quantity: r.get::<i32, &str>("quantity") as u32,
                    })
                }
                ActionSpecificTypeEnum::HarvestResource => {
                    let harvest_resource = sqlx::query(
                        r#"
                            SELECT resource_type_id
                            FROM actions.harvest_resource_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id as i64)
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
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                    })
                }
                ActionSpecificTypeEnum::MoveUnit => {
                    let move_unit = sqlx::query(
                        r#"
                            SELECT unit_id, cell_q, cell_r
                            FROM actions.move_unit_actions
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id as i64)
                    .fetch_one(&self.pool)
                    .await?;

                    SpecificAction::MoveUnit(MoveUnitAction {
                        player_id,
                        unit_id: r.get::<i64, &str>("unit_id") as u64,
                        chunk_id: chunk_id.clone(),
                        cell: GridCell {
                            q: r.get("cell_q"),
                            r: r.get("cell_r"),
                        },
                    })
                }
                ActionSpecificTypeEnum::SendMessage => {
                    let send_message = sqlx::query(
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
                    .bind(id as i64)
                    .fetch_one(&self.pool)
                    .await?;

                    let receivers_array: Vec<i64> = r.get("receivers");
                    SpecificAction::SendMessage(SendMessageAction {
                        player_id,
                        receivers: receivers_array.into_iter().map(|uid| uid as u64).collect(),
                        content: r.get("message_content"),
                    })
                }
                _ => SpecificAction::Unknown(),
            };

            actions.push(ActionData {
                base_data,
                specific_data,
            });
        }

        Ok(actions)
    }
}
