use bincode::{Decode, Encode};
use std::collections::HashMap;

use crate::grid::GridCell;
use crate::TerrainChunkId;

use super::{EquipmentSlotEnum, ProfessionEnum, SkillEnum};

// ============ UNIT (Unité/Personnage) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct UnitData {
    pub id: u64,
    pub player_id: Option<u64>, // None si NPC
    pub first_name: String,
    pub last_name: String,
    pub level: i32,
    pub avatar_url: Option<String>,

    // Position
    pub current_cell: GridCell,
    pub current_chunk: TerrainChunkId,

    // Profession
    pub profession: ProfessionEnum,

    // Argent
    pub money: i64,
}

impl UnitData {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn is_player_character(&self) -> bool {
        self.player_id.is_some()
    }

    pub fn is_npc(&self) -> bool {
        self.player_id.is_none()
    }
}

// ============ BASE STATS (Statistiques de base) ============
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct UnitBaseStats {
    pub strength: i32,
    pub agility: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl Default for UnitBaseStats {
    fn default() -> Self {
        Self {
            strength: 10,
            agility: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

impl UnitBaseStats {
    pub fn get_stat(&self, stat_name: &str) -> i32 {
        match stat_name {
            "strength" => self.strength,
            "agility" => self.agility,
            "constitution" => self.constitution,
            "intelligence" => self.intelligence,
            "wisdom" => self.wisdom,
            "charisma" => self.charisma,
            _ => 0,
        }
    }

    /// Calcule le bonus de stat (pour skills, etc.)
    /// Formule: (stat - 10) / 2 pour un bonus de +1 tous les 2 points
    pub fn stat_bonus(&self, stat_name: &str) -> i32 {
        let stat_value = self.get_stat(stat_name);
        (stat_value - 10) / 2
    }
}

// ============ DERIVED STATS (Statistiques dérivées) ============
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct UnitDerivedStats {
    pub max_hp: i32,
    pub current_hp: i32,
    pub happiness: i32,      // 0-100
    pub mental_health: i32,  // 0-100
    pub base_inventory_capacity_kg: i32,
    pub current_weight_kg: f32,
}

impl Default for UnitDerivedStats {
    fn default() -> Self {
        Self {
            max_hp: 100,
            current_hp: 100,
            happiness: 50,
            mental_health: 100,
            base_inventory_capacity_kg: 50,
            current_weight_kg: 0.0,
        }
    }
}

impl UnitDerivedStats {
    /// Calcule la capacité d'inventaire totale
    /// (base + équipement + profession) sera calculée par le serveur
    pub fn total_inventory_capacity(&self, equipment_bonus: i32, profession_bonus: i32) -> i32 {
        self.base_inventory_capacity_kg + equipment_bonus + profession_bonus
    }

    pub fn is_overencumbered(&self, total_capacity: i32) -> bool {
        self.current_weight_kg > total_capacity as f32
    }

    pub fn hp_percentage(&self) -> f32 {
        if self.max_hp == 0 {
            0.0
        } else {
            (self.current_hp as f32 / self.max_hp as f32) * 100.0
        }
    }
}

// ============ UNIT SKILL (Compétence d'une unité) ============
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct UnitSkill {
    pub skill: SkillEnum,
    pub xp: i64,
    pub level: i32,
}

impl UnitSkill {
    pub fn new(skill: SkillEnum) -> Self {
        Self {
            skill,
            xp: 0,
            level: 1,
        }
    }

    /// Calcule le skill effectif avec les bonus
    /// base_skill = level du skill
    /// stat_bonus = bonus de la statistique principale
    /// profession_bonus = bonus de profession (en %)
    pub fn effective_skill(&self, stat_bonus: i32, profession_bonus_percent: i32) -> i32 {
        let base = self.level + stat_bonus;
        let with_profession = base + (base * profession_bonus_percent / 100);
        with_profession
    }
}

// ============ INVENTORY ITEM (Item dans l'inventaire) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct InventoryItem {
    pub item_id: i32,
    pub quantity: i32,
}

// ============ EQUIPPED ITEM (Item équipé) ============
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct EquippedItem {
    pub slot: EquipmentSlotEnum,
    pub item_id: i32,
}

// ============ AUTOMATED ACTION (Action automatisée) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct AutomatedAction {
    pub id: u64,
    pub action_type: String, // Ex: "auto_craft_bread"
    pub is_enabled: bool,
    pub parameters: HashMap<String, String>, // Paramètres JSON-like
}

// ============ CONSUMPTION DEMAND (Demande de consommation) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct ConsumptionDemand {
    pub item_id: i32,
    pub quantity_per_day: f32,
    pub priority: i32, // 1-10
}

// ============ FULL UNIT (Unité complète avec toutes ses données) ============
/// Structure complète d'une unité avec toutes ses informations
/// Utilisée pour la synchronisation client-serveur
#[derive(Debug, Clone, Encode, Decode)]
pub struct FullUnitData {
    pub unit: UnitData,
    pub base_stats: UnitBaseStats,
    pub derived_stats: UnitDerivedStats,
    pub skills: HashMap<SkillEnum, UnitSkill>,
    pub inventory: Vec<InventoryItem>,
    pub equipment: Vec<EquippedItem>,
    pub automated_actions: Vec<AutomatedAction>,
    pub consumption_demands: Vec<ConsumptionDemand>,
}

impl FullUnitData {
    /// Calcule la vitesse de déplacement
    /// Formule: base_speed + (agility_bonus) - (weight_penalty)
    pub fn movement_speed(&self) -> f32 {
        let base_speed = 100.0;
        let agility_bonus = self.base_stats.agility as f32 * 2.0;

        // Pénalité de poids si l'unité est surchargée
        let total_capacity = self.total_inventory_capacity();
        let weight_ratio = self.derived_stats.current_weight_kg / total_capacity as f32;
        let weight_penalty = if weight_ratio > 0.8 {
            (weight_ratio - 0.8) * 200.0 // Grosse pénalité si > 80% de capacité
        } else {
            0.0
        };

        (base_speed + agility_bonus - weight_penalty).max(10.0) // Minimum 10
    }

    /// Calcule la capacité d'inventaire totale
    pub fn total_inventory_capacity(&self) -> i32 {
        let equipment_bonus = self.calculate_equipment_stat_bonus("inventory_capacity_kg");
        let profession_bonus = self.unit.profession.inventory_capacity_bonus();
        self.derived_stats.total_inventory_capacity(equipment_bonus, profession_bonus)
    }

    /// Calcule un bonus de stat donné par l'équipement
    /// (Cette fonction devrait être appelée côté serveur avec les données des items)
    pub fn calculate_equipment_stat_bonus(&self, stat_name: &str) -> i32 {
        // À implémenter avec les données réelles des items depuis la DB
        // Pour l'instant on retourne 0
        0
    }

    /// Obtient le skill effectif d'une unité
    pub fn get_effective_skill(&self, skill: SkillEnum, profession_bonus: i32) -> i32 {
        if let Some(unit_skill) = self.skills.get(&skill) {
            let primary_stat = skill.primary_stat();
            let stat_bonus = self.base_stats.stat_bonus(primary_stat.to_name_lowercase());
            unit_skill.effective_skill(stat_bonus, profession_bonus)
        } else {
            // Skill non appris = niveau 0 + bonus de stat
            let primary_stat = skill.primary_stat();
            let stat_bonus = self.base_stats.stat_bonus(primary_stat.to_name_lowercase());
            stat_bonus
        }
    }
}
