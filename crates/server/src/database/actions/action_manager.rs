pub struct ActionManager {
    pool: PgPool,
    
    // Cache en mémoire
    actions: HashMap<u64, ScheduledAction>,
    player_actions: HashMap<u64, Vec<u64>>,
    cell_actions: HashMap<GridCell, Vec<u64>>,
    
    // Priority Queue
    completion_queue: BinaryHeap<Reverse<QueuedActionCompletion>>,
    
    next_action_id: u64,
}

impl ActionManager {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            actions: HashMap::new(),
            player_actions: HashMap::new(),
            cell_actions: HashMap::new(),
            completion_queue: BinaryHeap::new(),
            next_action_id: 1,
        }
    }

    /// Convertir ClientMessage en Action
    pub fn client_message_to_action(
        &self,
        message: &crate::messages::ClientMessage,
    ) -> Result<ScheduledActionEnum, String> {
        use crate::messages::ClientMessage;

        match message {
            ClientMessage::ActionBuildBuilding { building_type } => {
                Ok(ScheduledActionEnum::BuildBuilding(BuildBuildingAction {
                    building_type: building_type.clone(),
                }))
            }
            ClientMessage::ActionMoveUnit { unit_id, target_pos } => {
                Ok(ScheduledActionEnum::MoveUnit(MoveUnitAction {
                    unit_id: *unit_id,
                    target_pos: *target_pos,
                }))
            }
            ClientMessage::ActionSendMessage { receiver_id, content } => {
                Ok(ScheduledActionEnum::SendMessage(SendMessageAction {
                    receiver_id: *receiver_id,
                    content: content.clone(),
                }))
            }
            ClientMessage::ActionHarvestResource { resource_type } => {
                Ok(ScheduledActionEnum::HarvestResource(HarvestResourceAction {
                    resource_type: resource_type.clone(),
                }))
            }
            ClientMessage::ActionCraftResource { recipe_id, quantity } => {
                Ok(ScheduledActionEnum::CraftResource(CraftResourceAction {
                    recipe_id: recipe_id.clone(),
                    quantity: *quantity,
                }))
            }
            _ => Err("Not an action message".to_string()),
        }
    }

    /// Créer une action
    pub async fn create_action(
        &mut self,
        player_id: u64,
        grid_cell: GridCell,
        action: ScheduledActionEnum,
        current_time: u64,
    ) -> Result<u64, String> {
        // Validation
        let validation_context = ValidationContext {
            player_id,
            grid_cell,
        };
        action.as_action().validate(&validation_context)?;

        // Vérifier contrainte cell unique
        if let Some(existing_ids) = self.cell_actions.get(&grid_cell) {
            for action_id in existing_ids {
                if let Some(act) = self.actions.get(action_id) {
                    if act.status == ActionStatus::InProgress || act.status == ActionStatus::Pending {
                        return Err(format!("Cell {:?} already has an active action", grid_cell));
                    }
                }
            }
        }

        // Compute duration
        let action_context = ActionContext {
            player_id,
            grid_cell,
        };
        let duration_ms = action.as_action().duration_ms(&action_context);

        let action_id = self.next_action_id;
        self.next_action_id += 1;

        let completion_time = current_time + (duration_ms / 1000);

        // Insérer en DB
        let db_action_id = self.insert_action_to_db(
            player_id,
            grid_cell,
            &action,
            current_time,
            duration_ms,
            completion_time,
        ).await?;

        // Ajouter en cache
        let scheduled_action = ScheduledAction {
            id: action_id,
            player_id,
            grid_cell,
            action,
            start_time: current_time,
            duration_ms,
            completion_time,
            status: ActionStatus::InProgress,
        };

        self.actions.insert(action_id, scheduled_action);
        self.player_actions.entry(player_id).or_insert_with(Vec::new).push(action_id);
        self.cell_actions.entry(grid_cell).or_insert_with(Vec::new).push(action_id);

        self.completion_queue.push(Reverse(QueuedActionCompletion {
            completion_time,
            action_id,
        }));

        Ok(action_id)
    }

    /// Insérer action en DB (à adapter avec ton schéma)
    async fn insert_action_to_db(
        &self,
        player_id: u64,
        grid_cell: GridCell,
        action: &ScheduledActionEnum,
        start_time: u64,
        duration_ms: u64,
        completion_time: u64,
    ) -> Result<u64, String> {
        let action_type = action.action_type();

        let db_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scheduled_actions 
             (player_id, grid_cell_q, grid_cell_r, action_type, start_time, duration_ms, completion_time, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'InProgress')
             RETURNING id"
        )
        .bind(player_id as i64)
        .bind(grid_cell.q)
        .bind(grid_cell.r)
        .bind(action_type)
        .bind(start_time as i64)
        .bind(duration_ms as i64)
        .bind(completion_time as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        // Insérer dans la table spécifique
        self.insert_action_data(db_id, action).await?;

        Ok(db_id as u64)
    }

    /// Insérer les données spécifiques par type
    async fn insert_action_data(
        &self,
        action_id: i64,
        action: &ScheduledActionEnum,
    ) -> Result<(), String> {
        match action {
            ScheduledActionEnum::BuildBuilding(a) => {
                sqlx::query(
                    "INSERT INTO build_building_actions (action_id, building_type) VALUES ($1, $2)"
                )
                .bind(action_id)
                .bind(&a.building_type)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            ScheduledActionEnum::MoveUnit(a) => {
                sqlx::query(
                    "INSERT INTO move_unit_actions (action_id, unit_id, target_q, target_r) VALUES ($1, $2, $3, $4)"
                )
                .bind(action_id)
                .bind(a.unit_id as i64)
                .bind(a.target_pos.q)
                .bind(a.target_pos.r)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            ScheduledActionEnum::SendMessage(a) => {
                sqlx::query(
                    "INSERT INTO send_message_actions (action_id, receiver_id, content) VALUES ($1, $2, $3)"
                )
                .bind(action_id)
                .bind(a.receiver_id as i64)
                .bind(&a.content)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            ScheduledActionEnum::HarvestResource(a) => {
                sqlx::query(
                    "INSERT INTO harvest_resource_actions (action_id, resource_type) VALUES ($1, $2)"
                )
                .bind(action_id)
                .bind(&a.resource_type)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
            ScheduledActionEnum::CraftResource(a) => {
                sqlx::query(
                    "INSERT INTO craft_resource_actions (action_id, recipe_id, quantity) VALUES ($1, $2, $3)"
                )
                .bind(action_id)
                .bind(&a.recipe_id)
                .bind(a.quantity as i32)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            }
        }
        Ok(())
    }

    /// Charger les actions d'un chunk (lazy loading)
    pub async fn load_chunk_actions(
        &mut self,
        grid_cells: &[GridCell],
    ) -> Result<Vec<ScheduledAction>, sqlx::Error> {
        let mut loaded = Vec::new();

        for cell in grid_cells {
            if self.cell_actions.contains_key(cell) {
                continue; // Déjà en cache
            }

            // Query DB
            let rows = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, i64)>(
                "SELECT id, player_id, grid_cell_q, grid_cell_r, action_type, start_time, duration_ms, completion_time
                 FROM scheduled_actions 
                 WHERE grid_cell_q = $1 AND grid_cell_r = $2 
                 AND status IN ('InProgress', 'Pending')"
            )
            .bind(cell.q)
            .bind(cell.r)
            .fetch_all(&self.pool)
            .await?;

            for row in rows {
                // TODO: Charger l'action spécifique depuis la bonne table
                // Pour l'instant c'est un placeholder
            }
        }

        Ok(loaded)
    }

    /// Traiter les complétions (appelé par le timer)
    pub async fn process_completions(&mut self, current_time: u64) -> Vec<u64> {
        let mut completed = Vec::new();

        while let Some(Reverse(QueuedActionCompletion { completion_time, action_id })) = 
            self.completion_queue.peek() 
        {
            if *completion_time > current_time {
                break;
            }

            if let Some(action) = self.actions.get_mut(&action_id) {
                action.status = ActionStatus::Completed;

                // Mettre à jour la DB
                let _ = sqlx::query("UPDATE scheduled_actions SET status = 'Completed' WHERE id = $1")
                    .bind(*action_id as i64)
                    .execute(&self.pool)
                    .await;

                completed.push(*action_id);
            }

            self.completion_queue.pop();
        }

        completed
    }

    /// Récupérer les actions d'un chunk
    pub fn get_chunk_actions(&self, grid_cells: &[GridCell]) -> Vec<ScheduledAction> {
        grid_cells
            .iter()
            .flat_map(|cell| {
                self.cell_actions
                    .get(cell)
                    .map(|ids| {
                        ids.iter()
                            .filter_map(|id| self.actions.get(id).cloned())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            })
            .collect()
    }
}