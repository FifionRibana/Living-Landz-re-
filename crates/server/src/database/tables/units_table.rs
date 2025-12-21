use bevy::prelude::*;
use shared::{
    EquipmentSlotEnum, FullUnitData, InventoryItem, ItemData, ItemTypeEnum, ProfessionEnum,
    ProfessionSkillBonus, SkillEnum, TerrainChunkId, UnitBaseStats, UnitData, UnitDerivedStats,
    UnitSkill, grid::GridCell, AutomatedAction, ConsumptionDemand, EquippedItem,
};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

#[derive(Resource, Clone)]
pub struct UnitsTable {
    pool: PgPool,
}

impl UnitsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============ UNIT CRUD ============

    /// Crée une nouvelle unité avec ses stats par défaut
    pub async fn create_unit(
        &self,
        player_id: Option<u64>,
        first_name: String,
        last_name: String,
        cell: GridCell,
        chunk: TerrainChunkId,
        profession: ProfessionEnum,
    ) -> Result<u64, String> {
        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        // Insérer l'unité
        let unit_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO units.units
            (player_id, first_name, last_name, level, current_cell_q, current_cell_r,
             current_chunk_x, current_chunk_y, profession_id, money)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
        )
        .bind(player_id.map(|id| id as i64))
        .bind(&first_name)
        .bind(&last_name)
        .bind(1) // level = 1
        .bind(cell.q)
        .bind(cell.r)
        .bind(chunk.x)
        .bind(chunk.y)
        .bind(profession.to_id())
        .bind(0i64) // money = 0
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create unit: {}", e))?;

        // Insérer les stats de base
        let base_stats = UnitBaseStats::default();
        sqlx::query(
            r#"
            INSERT INTO units.unit_base_stats
            (unit_id, strength, agility, constitution, intelligence, wisdom, charisma)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(unit_id)
        .bind(base_stats.strength)
        .bind(base_stats.agility)
        .bind(base_stats.constitution)
        .bind(base_stats.intelligence)
        .bind(base_stats.wisdom)
        .bind(base_stats.charisma)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create base stats: {}", e))?;

        // Insérer les stats dérivées
        let derived_stats = UnitDerivedStats::default();
        sqlx::query(
            r#"
            INSERT INTO units.unit_derived_stats
            (unit_id, max_hp, current_hp, happiness, mental_health,
             base_inventory_capacity_kg, current_weight_kg)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(unit_id)
        .bind(derived_stats.max_hp)
        .bind(derived_stats.current_hp)
        .bind(derived_stats.happiness)
        .bind(derived_stats.mental_health)
        .bind(derived_stats.base_inventory_capacity_kg)
        .bind(derived_stats.current_weight_kg)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create derived stats: {}", e))?;

        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(unit_id as u64)
    }

    /// Charge une unité par son ID
    pub async fn load_unit(&self, unit_id: u64) -> Result<UnitData, String> {
        let row = sqlx::query(
            r#"
            SELECT id, player_id, first_name, last_name, level, avatar_url,
                   current_cell_q, current_cell_r, current_chunk_x, current_chunk_y,
                   profession_id, money, slot_type, slot_index
            FROM units.units
            WHERE id = $1
            "#,
        )
        .bind(unit_id as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load unit: {}", e))?;

        Ok(UnitData {
            id: row.get::<i64, _>("id") as u64,
            player_id: row.get::<Option<i64>, _>("player_id").map(|id| id as u64),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            level: row.get("level"),
            avatar_url: row.get("avatar_url"),
            current_cell: GridCell {
                q: row.get("current_cell_q"),
                r: row.get("current_cell_r"),
            },
            current_chunk: TerrainChunkId {
                x: row.get("current_chunk_x"),
                y: row.get("current_chunk_y"),
            },
            slot_type: row.get("slot_type"),
            slot_index: row.get("slot_index"),
            profession: ProfessionEnum::from_id(row.get("profession_id"))
                .unwrap_or(ProfessionEnum::Unknown),
            money: row.get("money"),
        })
    }

    /// Charge les unités d'un joueur
    pub async fn load_player_units(&self, player_id: u64) -> Result<Vec<UnitData>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, player_id, first_name, last_name, level, avatar_url,
                   current_cell_q, current_cell_r, current_chunk_x, current_chunk_y,
                   profession_id, money, slot_type, slot_index
            FROM units.units
            WHERE player_id = $1
            "#,
        )
        .bind(player_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load player units: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| UnitData {
                id: row.get::<i64, _>("id") as u64,
                player_id: row.get::<Option<i64>, _>("player_id").map(|id| id as u64),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                level: row.get("level"),
                avatar_url: row.get("avatar_url"),
                current_cell: GridCell {
                    q: row.get("current_cell_q"),
                    r: row.get("current_cell_r"),
                },
                current_chunk: TerrainChunkId {
                    x: row.get("current_chunk_x"),
                    y: row.get("current_chunk_y"),
                },
                slot_type: row.get("slot_type"),
                slot_index: row.get("slot_index"),
                profession: ProfessionEnum::from_id(row.get("profession_id"))
                    .unwrap_or(ProfessionEnum::Unknown),
                money: row.get("money"),
            })
            .collect())
    }

    /// Charge les unités dans un chunk donné
    pub async fn load_chunk_units(&self, chunk: TerrainChunkId) -> Result<Vec<UnitData>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, player_id, first_name, last_name, level, avatar_url,
                   current_cell_q, current_cell_r, current_chunk_x, current_chunk_y,
                   profession_id, money, slot_type, slot_index
            FROM units.units
            WHERE current_chunk_x = $1 AND current_chunk_y = $2
            "#,
        )
        .bind(chunk.x)
        .bind(chunk.y)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load chunk units: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| UnitData {
                id: row.get::<i64, _>("id") as u64,
                player_id: row.get::<Option<i64>, _>("player_id").map(|id| id as u64),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                level: row.get("level"),
                avatar_url: row.get("avatar_url"),
                current_cell: GridCell {
                    q: row.get("current_cell_q"),
                    r: row.get("current_cell_r"),
                },
                current_chunk: TerrainChunkId {
                    x: row.get("current_chunk_x"),
                    y: row.get("current_chunk_y"),
                },
                slot_type: row.get("slot_type"),
                slot_index: row.get("slot_index"),
                profession: ProfessionEnum::from_id(row.get("profession_id"))
                    .unwrap_or(ProfessionEnum::Unknown),
                money: row.get("money"),
            })
            .collect())
    }

    // ============ STATS ============

    /// Charge les stats de base d'une unité
    pub async fn load_base_stats(&self, unit_id: u64) -> Result<UnitBaseStats, String> {
        let row = sqlx::query(
            r#"
            SELECT strength, agility, constitution, intelligence, wisdom, charisma
            FROM units.unit_base_stats
            WHERE unit_id = $1
            "#,
        )
        .bind(unit_id as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load base stats: {}", e))?;

        Ok(UnitBaseStats {
            strength: row.get("strength"),
            agility: row.get("agility"),
            constitution: row.get("constitution"),
            intelligence: row.get("intelligence"),
            wisdom: row.get("wisdom"),
            charisma: row.get("charisma"),
        })
    }

    /// Charge les stats dérivées d'une unité
    pub async fn load_derived_stats(&self, unit_id: u64) -> Result<UnitDerivedStats, String> {
        let row = sqlx::query(
            r#"
            SELECT max_hp, current_hp, happiness, mental_health,
                   base_inventory_capacity_kg, current_weight_kg
            FROM units.unit_derived_stats
            WHERE unit_id = $1
            "#,
        )
        .bind(unit_id as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load derived stats: {}", e))?;

        Ok(UnitDerivedStats {
            max_hp: row.get("max_hp"),
            current_hp: row.get("current_hp"),
            happiness: row.get("happiness"),
            mental_health: row.get("mental_health"),
            base_inventory_capacity_kg: row.get("base_inventory_capacity_kg"),
            current_weight_kg: row.get("current_weight_kg"),
        })
    }

    /// Met à jour les stats de base d'une unité
    pub async fn update_base_stats(
        &self,
        unit_id: u64,
        stats: &UnitBaseStats,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE units.unit_base_stats
            SET strength = $2, agility = $3, constitution = $4,
                intelligence = $5, wisdom = $6, charisma = $7
            WHERE unit_id = $1
            "#,
        )
        .bind(unit_id as i64)
        .bind(stats.strength)
        .bind(stats.agility)
        .bind(stats.constitution)
        .bind(stats.intelligence)
        .bind(stats.wisdom)
        .bind(stats.charisma)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update base stats: {}", e))?;

        Ok(())
    }

    /// Met à jour les stats dérivées d'une unité
    pub async fn update_derived_stats(
        &self,
        unit_id: u64,
        stats: &UnitDerivedStats,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE units.unit_derived_stats
            SET max_hp = $2, current_hp = $3, happiness = $4, mental_health = $5,
                base_inventory_capacity_kg = $6, current_weight_kg = $7
            WHERE unit_id = $1
            "#,
        )
        .bind(unit_id as i64)
        .bind(stats.max_hp)
        .bind(stats.current_hp)
        .bind(stats.happiness)
        .bind(stats.mental_health)
        .bind(stats.base_inventory_capacity_kg)
        .bind(stats.current_weight_kg)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update derived stats: {}", e))?;

        Ok(())
    }

    // ============ SKILLS ============

    /// Charge tous les skills d'une unité
    pub async fn load_unit_skills(&self, unit_id: u64) -> Result<HashMap<SkillEnum, UnitSkill>, String> {
        let rows = sqlx::query(
            r#"
            SELECT skill_id, xp, level
            FROM units.unit_skills
            WHERE unit_id = $1
            "#,
        )
        .bind(unit_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load skills: {}", e))?;

        let mut skills = HashMap::new();
        for row in rows {
            let skill_id: i16 = row.get("skill_id");
            if let Some(skill_enum) = SkillEnum::from_id(skill_id) {
                skills.insert(
                    skill_enum,
                    UnitSkill {
                        skill: skill_enum,
                        xp: row.get("xp"),
                        level: row.get("level"),
                    },
                );
            }
        }

        Ok(skills)
    }

    /// Ajoute ou met à jour un skill pour une unité
    pub async fn upsert_unit_skill(
        &self,
        unit_id: u64,
        skill: &UnitSkill,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO units.unit_skills (unit_id, skill_id, xp, level)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (unit_id, skill_id)
            DO UPDATE SET xp = $3, level = $4
            "#,
        )
        .bind(unit_id as i64)
        .bind(skill.skill.to_id())
        .bind(skill.xp)
        .bind(skill.level)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to upsert skill: {}", e))?;

        Ok(())
    }

    // ============ INVENTORY ============

    /// Charge l'inventaire d'une unité
    pub async fn load_inventory(&self, unit_id: u64) -> Result<Vec<InventoryItem>, String> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, quantity
            FROM units.unit_inventory
            WHERE unit_id = $1
            "#,
        )
        .bind(unit_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load inventory: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| InventoryItem {
                item_id: row.get("item_id"),
                quantity: row.get("quantity"),
            })
            .collect())
    }

    /// Ajoute un item à l'inventaire (ou augmente la quantité)
    pub async fn add_to_inventory(
        &self,
        unit_id: u64,
        item_id: i32,
        quantity: i32,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO units.unit_inventory (unit_id, item_id, quantity)
            VALUES ($1, $2, $3)
            ON CONFLICT (unit_id, item_id)
            DO UPDATE SET quantity = units.unit_inventory.quantity + $3
            "#,
        )
        .bind(unit_id as i64)
        .bind(item_id)
        .bind(quantity)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add to inventory: {}", e))?;

        Ok(())
    }

    /// Retire un item de l'inventaire
    pub async fn remove_from_inventory(
        &self,
        unit_id: u64,
        item_id: i32,
        quantity: i32,
    ) -> Result<(), String> {
        // Vérifier la quantité disponible
        let current_quantity: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT quantity FROM units.unit_inventory
            WHERE unit_id = $1 AND item_id = $2
            "#,
        )
        .bind(unit_id as i64)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to check inventory: {}", e))?;

        match current_quantity {
            Some(qty) if qty > quantity => {
                // Réduire la quantité
                sqlx::query(
                    r#"
                    UPDATE units.unit_inventory
                    SET quantity = quantity - $3
                    WHERE unit_id = $1 AND item_id = $2
                    "#,
                )
                .bind(unit_id as i64)
                .bind(item_id)
                .bind(quantity)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to remove from inventory: {}", e))?;
            }
            Some(qty) if qty == quantity => {
                // Supprimer l'entrée
                sqlx::query(
                    r#"
                    DELETE FROM units.unit_inventory
                    WHERE unit_id = $1 AND item_id = $2
                    "#,
                )
                .bind(unit_id as i64)
                .bind(item_id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to delete from inventory: {}", e))?;
            }
            _ => {
                return Err("Not enough items in inventory".to_string());
            }
        }

        Ok(())
    }

    // ============ EQUIPMENT ============

    /// Charge l'équipement d'une unité
    pub async fn load_equipment(&self, unit_id: u64) -> Result<Vec<EquippedItem>, String> {
        let rows = sqlx::query(
            r#"
            SELECT equipment_slot_id, item_id
            FROM units.unit_equipment
            WHERE unit_id = $1
            "#,
        )
        .bind(unit_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load equipment: {}", e))?;

        Ok(rows
            .iter()
            .filter_map(|row| {
                let slot_id: i16 = row.get("equipment_slot_id");
                EquipmentSlotEnum::from_id(slot_id).map(|slot| EquippedItem {
                    slot,
                    item_id: row.get("item_id"),
                })
            })
            .collect())
    }

    /// Équipe un item
    pub async fn equip_item(
        &self,
        unit_id: u64,
        slot: EquipmentSlotEnum,
        item_id: i32,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO units.unit_equipment (unit_id, equipment_slot_id, item_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (unit_id, equipment_slot_id)
            DO UPDATE SET item_id = $3
            "#,
        )
        .bind(unit_id as i64)
        .bind(slot.to_id())
        .bind(item_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to equip item: {}", e))?;

        Ok(())
    }

    /// Déséquipe un item
    pub async fn unequip_item(&self, unit_id: u64, slot: EquipmentSlotEnum) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM units.unit_equipment
            WHERE unit_id = $1 AND equipment_slot_id = $2
            "#,
        )
        .bind(unit_id as i64)
        .bind(slot.to_id())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to unequip item: {}", e))?;

        Ok(())
    }

    // ============ FULL UNIT DATA ============

    /// Charge toutes les données d'une unité
    pub async fn load_full_unit(&self, unit_id: u64) -> Result<FullUnitData, String> {
        let unit = self.load_unit(unit_id).await?;
        let base_stats = self.load_base_stats(unit_id).await?;
        let derived_stats = self.load_derived_stats(unit_id).await?;
        let skills = self.load_unit_skills(unit_id).await?;
        let inventory = self.load_inventory(unit_id).await?;
        let equipment = self.load_equipment(unit_id).await?;

        // TODO: Load automated actions and consumption demands

        Ok(FullUnitData {
            unit,
            base_stats,
            derived_stats,
            skills,
            inventory,
            equipment,
            automated_actions: vec![],
            consumption_demands: vec![],
        })
    }

    /// Update unit slot position
    pub async fn update_slot_position(
        &self,
        unit_id: u64,
        slot_type: Option<String>,
        slot_index: Option<i32>,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE units.units
            SET slot_type = $2, slot_index = $3
            WHERE id = $1
            "#,
        )
        .bind(unit_id as i64)
        .bind(slot_type)
        .bind(slot_index)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update slot position: {}", e))?;

        Ok(())
    }
}
