use bincode::{Decode, Encode};
use std::collections::HashMap;

use crate::{EquipmentSlotEnum, ItemTypeEnum, SkillEnum};

use super::{ResourceCategoryEnum};

// ============ ITEM (Définition d'item) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct ItemDefinition {
    pub id: i32,
    pub name: String,
    pub item_type: ItemTypeEnum,
    pub category: Option<ResourceCategoryEnum>,

    // Propriétés physiques
    pub weight_kg: f32,
    pub volume_liters: f32,

    // Économie
    pub base_price: i32,

    // Périssabilité
    pub is_perishable: bool,
    pub base_decay_rate_per_day: f32, // 0.0 - 1.0

    // Équipement
    pub is_equipable: bool,
    pub equipment_slot: Option<EquipmentSlotEnum>,

    // Craft
    pub is_craftable: bool,

    // Description
    pub description: String,
    pub image_url: Option<String>,

    // Modificateurs de stats
    pub stat_modifiers: HashMap<String, i32>,
}

impl ItemDefinition {
    /// Obtient un modificateur de stat donné
    pub fn get_stat_modifier(&self, stat_name: &str) -> i32 {
        self.stat_modifiers.get(stat_name).copied().unwrap_or(0)
    }

    /// Vérifie si l'item peut être équipé dans un slot donné
    pub fn can_equip_in_slot(&self, slot: EquipmentSlotEnum) -> bool {
        if !self.is_equipable {
            return false;
        }
        if let Some(item_slot) = self.equipment_slot {
            item_slot == slot
        } else {
            false
        }
    }

    /// Calcule le prix en fonction de la qualité
    pub fn price_for_quality(&self, quality: f32) -> i32 {
        (self.base_price as f32 * quality).round() as i32
    }

    /// Vérifie si l'item est pourri (decay >= 1.0)
    pub fn is_rotten(&self, decay: f32) -> bool {
        self.is_perishable && decay >= 1.0
    }

    /// Calcule le prix avec qualité et decay
    pub fn effective_price(&self, quality: f32, decay: f32) -> i32 {
        if self.is_rotten(decay) {
            0 // Item pourri = sans valeur
        } else {
            let quality_price = self.price_for_quality(quality);
            if self.is_perishable {
                // Le decay réduit le prix
                let decay_factor = 1.0 - decay;
                (quality_price as f32 * decay_factor).round() as i32
            } else {
                quality_price
            }
        }
    }
}

// ============ ITEM INSTANCE (Instance d'item avec qualité et decay) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct ItemInstance {
    pub id: u64,
    pub item_id: i32,

    // Qualité (0.0 - 1.0)
    pub quality: f32,

    // Périssabilité
    pub current_decay: f32, // 0.0 - 1.0
    pub last_decay_update: u64, // Timestamp en secondes

    // Propriétaire
    pub owner_unit_id: Option<u64>,

    // Position dans le monde (si pas possédé)
    pub world_position: Option<WorldPosition>,

    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct WorldPosition {
    pub cell_q: i32,
    pub cell_r: i32,
    pub chunk_x: i32,
    pub chunk_y: i32,
}

impl ItemInstance {
    pub fn new(item_id: i32, quality: f32) -> Self {
        Self {
            id: 0, // Will be set by DB
            item_id,
            quality: quality.clamp(0.0, 1.0),
            current_decay: 0.0,
            last_decay_update: 0,
            owner_unit_id: None,
            world_position: None,
            created_at: 0,
        }
    }

    /// Met à jour le decay en fonction du temps écoulé
    pub fn update_decay(&mut self, current_time: u64, decay_rate_per_day: f32) {
        if self.last_decay_update == 0 {
            self.last_decay_update = current_time;
            return;
        }

        let time_diff_seconds = current_time.saturating_sub(self.last_decay_update);
        let time_diff_days = time_diff_seconds as f32 / 86400.0;

        let decay_amount = decay_rate_per_day * time_diff_days;
        self.current_decay = (self.current_decay + decay_amount).min(1.0);
        self.last_decay_update = current_time;
    }

    /// Vérifie si l'item est pourri
    pub fn is_rotten(&self) -> bool {
        self.current_decay >= 1.0
    }

    /// Calcule la qualité effective (quality - decay effect)
    pub fn effective_quality(&self) -> f32 {
        if self.is_rotten() {
            0.0
        } else {
            self.quality * (1.0 - self.current_decay * 0.5) // Le decay réduit la qualité de 50% max
        }
    }
}

// ============ RECIPE (Recette de craft) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct Recipe {
    pub id: i32,
    pub name: String,
    pub description: String,

    // Résultat
    pub result_item_id: i32,
    pub result_quantity: i32,

    // Skill requis
    pub required_skill: Option<SkillEnum>,
    pub required_skill_level: i32,

    // Temps de craft
    pub craft_duration_seconds: i32,

    // Station requise (optionnel)
    pub required_building_type_id: Option<i16>,

    // Ingrédients
    pub ingredients: Vec<RecipeIngredient>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RecipeIngredient {
    pub item_id: i32,
    pub quantity: i32,
}

impl Recipe {
    /// Vérifie si le joueur a tous les ingrédients dans son inventaire
    pub fn has_ingredients(&self, inventory: &HashMap<i32, i32>) -> bool {
        self.ingredients.iter().all(|ingredient| {
            inventory
                .get(&ingredient.item_id)
                .map(|qty| *qty >= ingredient.quantity)
                .unwrap_or(false)
        })
    }

    /// Vérifie si le joueur a le skill requis
    pub fn has_required_skill(&self, skill_level: i32) -> bool {
        skill_level >= self.required_skill_level
    }

    /// Calcule la chance de succès basée sur le skill level
    /// Plus le skill est élevé, plus la chance de succès est grande
    pub fn success_chance(&self, skill_level: i32) -> f32 {
        if skill_level < self.required_skill_level {
            0.0 // Impossible de crafter sans le skill requis
        } else {
            let skill_diff = skill_level - self.required_skill_level;
            let base_chance = 0.7; // 70% de base
            let bonus = skill_diff as f32 * 0.05; // +5% par niveau au-dessus
            (base_chance + bonus).min(0.95) // Max 95% de réussite
        }
    }

    /// Calcule la qualité du produit crafté en fonction du skill
    pub fn craft_quality(&self, skill_level: i32) -> f32 {
        if skill_level < self.required_skill_level {
            0.5 // Qualité minimale si sous-qualifié
        } else {
            let skill_diff = skill_level - self.required_skill_level;
            let base_quality = 0.7; // 70% de base
            let bonus = skill_diff as f32 * 0.03; // +3% par niveau
            (base_quality + bonus).min(1.0) // Max 100% de qualité
        }
    }
}

// ============ FULL ITEM DATA (Item complet avec définition) ============
/// Item instance avec sa définition complète
#[derive(Debug, Clone, Encode, Decode)]
pub struct FullItemData {
    pub definition: ItemDefinition,
    pub instance: ItemInstance,
}

impl FullItemData {
    /// Calcule le prix effectif de cet item instance
    pub fn effective_price(&self) -> i32 {
        self.definition
            .effective_price(self.instance.quality, self.instance.current_decay)
    }

    /// Met à jour le decay de l'instance
    pub fn update_decay(&mut self, current_time: u64) {
        if self.definition.is_perishable {
            self.instance
                .update_decay(current_time, self.definition.base_decay_rate_per_day);
        }
    }
}
