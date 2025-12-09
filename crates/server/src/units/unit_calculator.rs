use shared::{
    EquipmentSlotEnum, FullUnitData, ItemData, ItemsLookup, ProfessionEnum,
    ProfessionSkillBonusesLookup, SkillEnum, UnitBaseStats, UnitDerivedStats, UnitSkill,
};
use std::collections::HashMap;

/// Système de calcul côté serveur pour les stats des unités
/// IMPORTANT: Ces calculs font foi. Le client ne peut PAS les modifier.
pub struct UnitCalculator;

impl UnitCalculator {
    /// Calcule la vitesse de déplacement d'une unité
    /// Formule: base_speed + (agility * 2) - weight_penalty
    pub fn calculate_movement_speed(
        base_stats: &UnitBaseStats,
        derived_stats: &UnitDerivedStats,
        total_inventory_capacity: i32,
    ) -> f32 {
        let base_speed = 100.0;
        let agility_bonus = base_stats.agility as f32 * 2.0;

        // Pénalité de poids si l'unité est surchargée
        let weight_ratio = derived_stats.current_weight_kg / total_inventory_capacity as f32;
        let weight_penalty = if weight_ratio > 0.8 {
            (weight_ratio - 0.8) * 200.0 // Grosse pénalité si > 80% de capacité
        } else {
            0.0
        };

        (base_speed + agility_bonus - weight_penalty).max(10.0) // Minimum 10
    }

    /// Calcule la capacité d'inventaire totale
    pub fn calculate_total_inventory_capacity(
        derived_stats: &UnitDerivedStats,
        profession: ProfessionEnum,
        equipment_bonus: i32,
    ) -> i32 {
        derived_stats.base_inventory_capacity_kg
            + profession.inventory_capacity_bonus()
            + equipment_bonus
    }

    /// Calcule le bonus de stat donné par l'équipement
    pub fn calculate_equipment_stat_bonus(
        equipment: &[(EquipmentSlotEnum, i32)],
        items_lookup: &ItemsLookup,
        stat_name: &str,
    ) -> i32 {
        equipment
            .iter()
            .filter_map(|(_, item_id)| items_lookup.get(*item_id))
            .map(|item| item.get_stat_modifier(stat_name))
            .sum()
    }

    /// Calcule tous les bonus d'équipement sous forme de HashMap
    pub fn calculate_all_equipment_bonuses(
        equipment: &[(EquipmentSlotEnum, i32)],
        items_lookup: &ItemsLookup,
    ) -> HashMap<String, i32> {
        let mut bonuses = HashMap::new();

        for (_, item_id) in equipment {
            if let Some(item) = items_lookup.get(*item_id) {
                for (stat_name, value) in &item.stat_modifiers {
                    *bonuses.entry(stat_name.clone()).or_insert(0) += value;
                }
            }
        }

        bonuses
    }

    /// Calcule le skill effectif d'une unité
    pub fn calculate_effective_skill(
        skill: SkillEnum,
        unit_skill: Option<&UnitSkill>,
        base_stats: &UnitBaseStats,
        profession: ProfessionEnum,
        profession_bonuses: &ProfessionSkillBonusesLookup,
        equipment_bonuses: &HashMap<String, i32>,
    ) -> i32 {
        // Niveau de base du skill (0 si pas appris)
        let skill_level = unit_skill.map(|s| s.level).unwrap_or(0);

        // Bonus de la statistique principale
        let primary_stat = skill.primary_stat();
        let stat_bonus = base_stats.stat_bonus(primary_stat.to_name_lowercase());

        // Bonus de profession (en %)
        let profession_bonus_percent = profession_bonuses.get_bonus(profession, skill);

        // Bonus d'équipement spécifique au skill
        let skill_name = skill.to_name_lowercase();
        let skill_bonus_name = format!("{}_bonus", skill_name);
        let equipment_skill_bonus = equipment_bonuses
            .get(&skill_bonus_name)
            .copied()
            .unwrap_or(0);

        // Calcul final
        let base = skill_level + stat_bonus + equipment_skill_bonus;
        let with_profession = base + (base * profession_bonus_percent / 100);

        with_profession
    }

    /// Calcule tous les skills effectifs d'une unité
    pub fn calculate_all_effective_skills(
        unit: &FullUnitData,
        profession_bonuses: &ProfessionSkillBonusesLookup,
        items_lookup: &ItemsLookup,
    ) -> HashMap<SkillEnum, i32> {
        let mut effective_skills = HashMap::new();

        // Calculer les bonus d'équipement une seule fois
        let equipment: Vec<_> = unit
            .equipment
            .iter()
            .map(|e| (e.slot, e.item_id))
            .collect();
        let equipment_bonuses = Self::calculate_all_equipment_bonuses(&equipment, items_lookup);

        // Calculer chaque skill
        for skill in SkillEnum::iter() {
            let unit_skill = unit.skills.get(&skill);
            let effective_skill = Self::calculate_effective_skill(
                skill,
                unit_skill,
                &unit.base_stats,
                unit.unit.profession,
                profession_bonuses,
                &equipment_bonuses,
            );
            effective_skills.insert(skill, effective_skill);
        }

        effective_skills
    }

    /// Calcule les points de vie max basés sur la constitution
    /// Formule: 100 + (constitution * 5)
    pub fn calculate_max_hp(constitution: i32) -> i32 {
        100 + (constitution * 5)
    }

    /// Calcule la défense physique totale
    pub fn calculate_physical_defense(equipment_bonuses: &HashMap<String, i32>) -> i32 {
        equipment_bonuses
            .get("defense_physical")
            .copied()
            .unwrap_or(0)
    }

    /// Calcule la défense contre les attaques de mêlée
    pub fn calculate_melee_defense(
        equipment_bonuses: &HashMap<String, i32>,
        agility_bonus: i32,
    ) -> i32 {
        let base_defense = equipment_bonuses
            .get("defense_melee")
            .copied()
            .unwrap_or(0);
        base_defense + agility_bonus
    }

    /// Calcule la défense contre les attaques à distance
    pub fn calculate_ranged_defense(
        equipment_bonuses: &HashMap<String, i32>,
        agility_bonus: i32,
    ) -> i32 {
        let base_defense = equipment_bonuses
            .get("defense_ranged")
            .copied()
            .unwrap_or(0);
        base_defense + (agility_bonus * 2) // L'agilité compte double pour esquiver les projectiles
    }

    /// Calcule le poids total de l'inventaire
    pub fn calculate_total_weight(
        inventory: &[(i32, i32)], // (item_id, quantity)
        items_lookup: &ItemsLookup,
    ) -> f32 {
        inventory
            .iter()
            .filter_map(|(item_id, quantity)| {
                items_lookup
                    .get(*item_id)
                    .map(|item| item.weight_kg * (*quantity as f32))
            })
            .sum()
    }

    /// Met à jour les stats dérivées d'une unité après un changement
    /// Retourne les nouvelles stats dérivées à sauvegarder
    pub fn recalculate_derived_stats(
        unit: &FullUnitData,
        items_lookup: &ItemsLookup,
    ) -> UnitDerivedStats {
        let mut derived = unit.derived_stats;

        // Recalculer max_hp
        derived.max_hp = Self::calculate_max_hp(unit.base_stats.constitution);

        // S'assurer que current_hp ne dépasse pas max_hp
        if derived.current_hp > derived.max_hp {
            derived.current_hp = derived.max_hp;
        }

        // Recalculer le poids actuel
        let inventory: Vec<_> = unit
            .inventory
            .iter()
            .map(|i| (i.item_id, i.quantity))
            .collect();
        derived.current_weight_kg = Self::calculate_total_weight(&inventory, items_lookup);

        derived
    }

    /// Vérifie si une unité peut équiper un item
    pub fn can_equip_item(
        unit: &FullUnitData,
        item: &ItemData,
        slot: EquipmentSlotEnum,
    ) -> Result<(), String> {
        // Vérifier que l'item est équipable
        if !item.is_equipable {
            return Err("Item is not equipable".to_string());
        }

        // Vérifier que l'item peut aller dans le slot demandé
        if !item.can_equip_in_slot(slot) {
            return Err(format!(
                "Item cannot be equipped in {} slot",
                slot.to_name()
            ));
        }

        // Vérifier que l'unité possède l'item dans son inventaire
        let has_item = unit.inventory.iter().any(|i| i.item_id == item.id && i.quantity > 0);
        if !has_item {
            return Err("Item not in inventory".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_speed_calculation() {
        let base_stats = UnitBaseStats {
            agility: 15,
            ..Default::default()
        };

        let derived_stats = UnitDerivedStats {
            current_weight_kg: 20.0,
            ..Default::default()
        };

        let speed = UnitCalculator::calculate_movement_speed(&base_stats, &derived_stats, 100);
        assert!(speed > 100.0); // Should be faster due to high agility
    }

    #[test]
    fn test_max_hp_calculation() {
        let hp = UnitCalculator::calculate_max_hp(10);
        assert_eq!(hp, 150); // 100 + (10 * 5)

        let hp = UnitCalculator::calculate_max_hp(20);
        assert_eq!(hp, 200); // 100 + (20 * 5)
    }

    #[test]
    fn test_stat_bonus() {
        let stats = UnitBaseStats {
            strength: 14,
            ..Default::default()
        };

        let bonus = stats.stat_bonus("strength");
        assert_eq!(bonus, 2); // (14 - 10) / 2 = 2
    }
}
