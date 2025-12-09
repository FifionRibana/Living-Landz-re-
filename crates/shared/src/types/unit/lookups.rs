use bincode::{Decode, Encode};
use std::collections::HashMap;

use super::{EquipmentSlotEnum, ItemTypeEnum, ProfessionEnum, SkillEnum};

// ============ PROFESSION DATA (Données de profession depuis DB) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct ProfessionData {
    pub id: i16,
    pub profession_enum: ProfessionEnum,
    pub name: String,
    pub description: String,
    pub base_inventory_capacity_bonus: i32,
}

// ============ SKILL DATA (Données de skill depuis DB) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct SkillData {
    pub id: i16,
    pub skill_enum: SkillEnum,
    pub name: String,
    pub description: String,
    pub primary_stat: String,
}

// ============ PROFESSION SKILL BONUS (Bonus de profession pour un skill) ============
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct ProfessionSkillBonus {
    pub profession: ProfessionEnum,
    pub skill: SkillEnum,
    pub bonus_percentage: i32,
}

// ============ ITEM DATA (Données d'item depuis DB) ============
#[derive(Debug, Clone, Encode, Decode)]
pub struct ItemData {
    pub id: i32,
    pub name: String,
    pub item_type: ItemTypeEnum,
    pub description: String,
    pub weight_kg: f32,
    pub is_equipable: bool,
    pub equipment_slot: Option<EquipmentSlotEnum>,
    pub stat_modifiers: HashMap<String, i32>, // Ex: {"strength_bonus": 2, "defense_physical": 10}
}

impl ItemData {
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
}

// ============ LOOKUP TABLES (Caches côté client/serveur) ============

/// Cache de toutes les professions
#[derive(Debug, Clone, Default)]
pub struct ProfessionsLookup {
    pub professions: HashMap<ProfessionEnum, ProfessionData>,
}

impl ProfessionsLookup {
    pub fn new() -> Self {
        Self {
            professions: HashMap::new(),
        }
    }

    pub fn add(&mut self, profession: ProfessionData) {
        self.professions.insert(profession.profession_enum, profession);
    }

    pub fn get(&self, profession: ProfessionEnum) -> Option<&ProfessionData> {
        self.professions.get(&profession)
    }
}

/// Cache de tous les skills
#[derive(Debug, Clone, Default)]
pub struct SkillsLookup {
    pub skills: HashMap<SkillEnum, SkillData>,
}

impl SkillsLookup {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn add(&mut self, skill: SkillData) {
        self.skills.insert(skill.skill_enum, skill);
    }

    pub fn get(&self, skill: SkillEnum) -> Option<&SkillData> {
        self.skills.get(&skill)
    }
}

/// Cache de tous les bonus de profession pour les skills
#[derive(Debug, Clone, Default)]
pub struct ProfessionSkillBonusesLookup {
    // Map: Profession -> Skill -> Bonus%
    pub bonuses: HashMap<ProfessionEnum, HashMap<SkillEnum, i32>>,
}

impl ProfessionSkillBonusesLookup {
    pub fn new() -> Self {
        Self {
            bonuses: HashMap::new(),
        }
    }

    pub fn add(&mut self, bonus: ProfessionSkillBonus) {
        self.bonuses
            .entry(bonus.profession)
            .or_insert_with(HashMap::new)
            .insert(bonus.skill, bonus.bonus_percentage);
    }

    pub fn get_bonus(&self, profession: ProfessionEnum, skill: SkillEnum) -> i32 {
        self.bonuses
            .get(&profession)
            .and_then(|skills| skills.get(&skill))
            .copied()
            .unwrap_or(0)
    }

    /// Retourne tous les skills avec bonus pour une profession donnée
    pub fn get_profession_bonuses(&self, profession: ProfessionEnum) -> Vec<(SkillEnum, i32)> {
        self.bonuses
            .get(&profession)
            .map(|skills| skills.iter().map(|(skill, bonus)| (*skill, *bonus)).collect())
            .unwrap_or_default()
    }
}

/// Cache de tous les items
#[derive(Debug, Clone, Default)]
pub struct ItemsLookup {
    pub items: HashMap<i32, ItemData>,
}

impl ItemsLookup {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn add(&mut self, item: ItemData) {
        self.items.insert(item.id, item);
    }

    pub fn get(&self, item_id: i32) -> Option<&ItemData> {
        self.items.get(&item_id)
    }

    /// Retourne tous les items équipables pour un slot donné
    pub fn get_equipable_for_slot(&self, slot: EquipmentSlotEnum) -> Vec<&ItemData> {
        self.items
            .values()
            .filter(|item| item.can_equip_in_slot(slot))
            .collect()
    }
}
