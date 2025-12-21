use bevy::prelude::*;
use shared::types::UnitData;
use std::collections::HashMap;

/// Cache pour stocker les données complètes des unités
#[derive(Resource, Default)]
pub struct UnitsDataCache {
    /// Map: unit_id -> UnitData
    units_data: HashMap<u64, UnitData>,
}

impl UnitsDataCache {
    /// Add or update unit data
    pub fn insert_unit(&mut self, unit: UnitData) {
        self.units_data.insert(unit.id, unit);
    }

    /// Add or update multiple units
    pub fn insert_units(&mut self, units: Vec<UnitData>) {
        for unit in units {
            self.insert_unit(unit);
        }
    }

    /// Get unit data by ID
    pub fn get_unit(&self, unit_id: u64) -> Option<&UnitData> {
        self.units_data.get(&unit_id)
    }

    /// Remove unit data
    pub fn remove_unit(&mut self, unit_id: u64) {
        self.units_data.remove(&unit_id);
    }

    /// Check if unit data exists
    pub fn has_unit(&self, unit_id: u64) -> bool {
        self.units_data.contains_key(&unit_id)
    }

    /// Get all unit IDs
    pub fn get_all_unit_ids(&self) -> Vec<u64> {
        self.units_data.keys().copied().collect()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.units_data.clear();
    }
}
