use bevy::prelude::*;
use std::collections::HashMap;

/// Tracks which units are currently working on an action.
#[derive(Resource, Default)]
pub struct UnitWorkState {
    /// unit_id → action_id
    working: HashMap<u64, u64>,
}

impl UnitWorkState {
    pub fn set_working(&mut self, unit_id: u64, action_id: u64) {
        self.working.insert(unit_id, action_id);
    }

    pub fn clear_working(&mut self, unit_id: u64) {
        self.working.remove(&unit_id);
    }

    pub fn is_working(&self, unit_id: u64) -> bool {
        self.working.contains_key(&unit_id)
    }

    pub fn working_on(&self, unit_id: u64) -> Option<u64> {
        self.working.get(&unit_id).copied()
    }

    /// Get all unit_ids working on a specific action
    pub fn units_for_action(&self, action_id: u64) -> Vec<u64> {
        self.working
            .iter()
            .filter(|(_, aid)| **aid == action_id)
            .map(|(&uid, _)| uid)
            .collect()
    }
}
