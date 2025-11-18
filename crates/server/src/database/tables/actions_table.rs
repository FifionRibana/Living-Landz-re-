use std::collections::HashMap;

use bevy::prelude::*;

use shared::{
    ActionBaseData, ActionData, ActionStatus, ActionType, BuildBuildingAction, BuildRoadAction, BuildingCategory, BuildingSpecificType, BuildingType, BuildingTypeData, CraftResourceAction, GameState, HarvestResourceAction, MoveUnitAction, ResourceType, SendMessageAction, SpecificAction, SpecificActionData, SpecificActionType, TerrainChunkId, TreeType, grid::GridCell
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

    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TYPE specific_action_type AS ENUM (
                'BuildBuilding',
                'BuildRoad',
                'MoveUnit', 
                'SendMessage',
                'HarvestResource',
                'CraftResource'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TYPE command_status_enum AS ENUM (
                'InProgress',
                'Pending',
                'Completed',
                'Failed'
            )"#,
        )
        .execute(&self.pool)
        .await
        .ok();

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS scheduled_actions (
                id BIGSERIAL PRIMARY KEY,
                player_id BIGINT NOT NULL,
                cell_q INT NOT NULL,
                cell_r INT NOT NULL,
                chunk_x INT NOT NULL,
                chunk_y INT NOT NULL,
                start_time BIGINT NOT NULL,
                duration_ms BIGINT NOT NULL,
                completion_time BIGINT NOT NULL,
                status command_status_enum NOT NULL DEFAULT 'InProgress',
                created_at TIMESTAMP DEFAULT NOW()
            )"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_cell_action 
                ON scheduled_actions(cell_q, cell_r) 
                WHERE status IN ('InProgress', 'Pending')"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_cell_chunk_action 
                ON scheduled_actions(cell_q, cell_r, chunk_x, chunk_y) 
                WHERE status IN ('InProgress', 'Pending')"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS build_building_commands (
                action_id BIGINT PRIMARY KEY REFERENCES scheduled_actions(id) ON DELETE CASCADE,
                building_type_id SERIAL REFERENCES building_types(id)
            )"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_building_type_id ON build_building_commands(building_type_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS build_road_commands (
                action_id BIGINT PRIMARY KEY REFERENCES scheduled_actions(id) ON DELETE CASCADE
            )"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS move_unit_commands (
                action_id BIGINT PRIMARY KEY REFERENCES scheduled_actions(id) ON DELETE CASCADE,
                unit_id BIGINT NOT NULL,
                cell_q INT NOT NULL,
                cell_r INT NOT NULL
            )"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS send_message_commands (
                action_id BIGINT PRIMARY KEY REFERENCES scheduled_actions(id) ON DELETE CASCADE,
                receiver_id BIGINT NOT NULL,
                message_content TEXT NOT NULL
            )"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS harvest_resource_commands (
                action_id BIGINT PRIMARY KEY REFERENCES scheduled_actions(id) ON DELETE CASCADE,
                resource_type_id SERIAL REFERENCES resource_types(id)
            )"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS craft_resource_commands (
                action_id BIGINT PRIMARY KEY REFERENCES scheduled_actions(id) ON DELETE CASCADE,
                recipe_id VARCHAR NOT NULL,
                quantity INT NOT NULL
            )"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_resource_type_id ON craft_resource_commands(resource_type_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_completion_time ON scheduled_actions(completion_time)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_chunk_commands ON scheduled_actions(hex_pos_x, hex_pos_y)
                WHERE status IN ('InProgress', 'Pending')"#)
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_player_commands ON scheduled_actions(player_id) 
                WHERE status IN ('InProgress', 'Pending')"#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("✓ Building types Database schema ready");
        Ok(())
    }

    pub async fn add_scheduled_action(&self, action: &ActionData) -> Result<u64, String> {
        let base_action = &action.base_data;

        // let action_type = base_action.action_type();

        let db_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scheduled_actions 
             (player_id, cell_q, cell_r, chunk_x, chunk_y, start_time, duration_ms, completion_time, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'InProgress')
             RETURNING id"
        )
        .bind(base_action.player_id as i64)
        .bind(base_action.cell.q)
        .bind(base_action.cell.r)
        .bind(base_action.chunk.x)
        .bind(base_action.chunk.y)
        // .bind(base_action.action_type)
        .bind(base_action.start_time as i64)
        .bind(base_action.duration_ms as i64)
        .bind(base_action.completion_time as i64)
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
                    "INSERT INTO build_building_actions (action_id, building_type_id) VALUES ($1, $2)",
                )
                .bind(action_id as i64)
                .bind(a.building_type.id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::BuildRoad(a) => {
                // sqlx::query(

                // )
            }
            SpecificAction::MoveUnit(a) => {
                sqlx::query(
                    "INSERT INTO move_unit_actions (action_id, unit_id, target_q, target_r) VALUES ($1, $2, $3, $4)"
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
                    "INSERT INTO send_message_actions (action_id, receiver_id, content) VALUES ($1, $2, $3)"
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
                    "INSERT INTO harvest_resource_actions (action_id, resource_type) VALUES ($1, $2)"
                )
                .bind(action_id as i64)
                .bind(&a.resource_type)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            SpecificAction::CraftResource(a) => {
                sqlx::query(
                    "INSERT INTO craft_resource_actions (action_id, recipe_id, quantity) VALUES ($1, $2, $3)"
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
                action_type, start_time, duration_ms, completion_time, status
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
            let action_type = r.get::<SpecificActionType, &str>("action_type");
            let cell = GridCell {
                q: r.get("cell_q"),
                r: r.get("cell_r"),
            };

            let base_data = ActionBaseData {
                player_id: r.get::<i64, &str>("player_id") as u64,
                cell: cell.clone(),
                chunk: chunk_id.clone(),
                // action_type,
                start_time: r.get::<i64, &str>("start_time") as u64,
                duration_ms: r.get::<i64, &str>("duration_ms") as u64,
                completion_time: r.get::<i64, &str>("completion_time") as u64,

                status: r.get::<ActionStatus, &str>("status"),
            };

            let specific_data = match action_type {
                SpecificActionType::BuildBuilding => {
                    let build_building = sqlx::query(
                        r#"
                            SELECT building_type_id
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id as i64)
                    .fetch_one(&self.pool)
                    .await?;

                    SpecificAction::BuildBuilding(BuildBuildingAction {
                        player_id,
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                        building_type: BuildingType {
                            id: build_building.get("building_type_id"),
                            variant: String::new(),
                            category: r.get::<BuildingCategory, &str>("building_category"),
                        },
                    })
                }
                SpecificActionType::BuildRoad => {
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
                SpecificActionType::CraftResource => {
                    let craft_resource = sqlx::query(
                        r#"
                            SELECT recipe_id, quantity
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
                SpecificActionType::HarvestResource => {
                    let harvest_resource = sqlx::query(
                        r#"
                            SELECT resource_type_id
                            FROM buildings_base b
                            JOIN building_types bt ON b.building_type_id = bt.id
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id as i64)
                    .fetch_one(&self.pool)
                    .await?;

                    SpecificAction::HarvestResource(HarvestResourceAction {
                        player_id,
                        resource_type: r.get("resource_type_id"),
                        chunk_id: chunk_id.clone(),
                        cell: cell.clone(),
                    })
                }
                SpecificActionType::MoveUnit => {
                    let move_unit = sqlx::query(
                        r#"
                            SELECT unit_id, cell_q, cell_r
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
                SpecificActionType::SendMessage => {
                    // TODO: Change to fetch all? or store vector of receiver_id
                    let send_message = sqlx::query(
                        r#"
                            SELECT receiver_id, message_content
                            WHERE action_id = $1
                        "#,
                    )
                    .bind(id as i64)
                    .fetch_one(&self.pool)
                    .await?;

                    SpecificAction::SendMessage(SendMessageAction {
                        player_id,
                        receivers: vec![r.get::<i64, &str>("receiver_id") as u64],
                        content: r.get("content"),
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
