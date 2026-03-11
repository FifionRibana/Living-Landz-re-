use bevy::prelude::*;
use shared::protocol::InventoryItemData;
use std::collections::HashMap;

/// Cache local de l'inventaire du joueur
#[derive(Resource, Default)]
pub struct InventoryCache {
    /// Inventaire par unit_id -> liste d'items
    pub inventories: HashMap<u64, Vec<InventoryItemData>>,
    /// Flag pour savoir si on a déjà demandé l'inventaire
    pub requested: bool,
}

impl InventoryCache {
    /// Met à jour l'inventaire complet d'une unité
    pub fn set_inventory(&mut self, unit_id: u64, items: Vec<InventoryItemData>) {
        self.inventories.insert(unit_id, items);
    }

    /// Applique un update incrémental
    pub fn apply_update(&mut self, unit_id: u64, item_id: i32, new_total: i32) {
        if let Some(items) = self.inventories.get_mut(&unit_id) {
            if let Some(item) = items.iter_mut().find(|i| i.item_id == item_id) {
                item.quantity = new_total;
            } else if new_total > 0 {
                items.push(InventoryItemData {
                    instance_id: 0,
                    item_id,
                    name: format!("Item #{}", item_id),
                    item_type: shared::ItemTypeEnum::Unknown,
                    quality: 1.0,
                    weight_kg: 0.0,
                    quantity: new_total,
                    is_equipped: false,
                    equipment_slot: None,
                });
            }

            // Supprimer les items à quantité 0
            items.retain(|i| i.quantity > 0);
        } else if new_total > 0 {
            // Pas encore d'inventaire chargé, créer un placeholder
            self.inventories.insert(
                unit_id,
                vec![InventoryItemData {
                    instance_id: 0,
                    item_id,
                    name: format!("Item #{}", item_id),
                    item_type: shared::ItemTypeEnum::Unknown,
                    quality: 1.0,
                    weight_kg: 0.0,
                    quantity: new_total,
                    is_equipped: false,
                    equipment_slot: None,
                }],
            );
        }
    }

    pub fn get_inventory(&self, unit_id: u64) -> Option<&Vec<InventoryItemData>> {
        self.inventories.get(&unit_id)
    }

    /// Poids total de l'inventaire d'une unité
    pub fn total_weight(&self, unit_id: u64) -> f32 {
        self.inventories
            .get(&unit_id)
            .map(|items| {
                items
                    .iter()
                    .map(|i| i.weight_kg * i.quantity as f32)
                    .sum()
            })
            .unwrap_or(0.0)
    }
}
