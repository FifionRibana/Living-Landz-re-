use bevy::prelude::*;
use shared::grid::GridCell;
use shared::SlotPosition;
use std::collections::HashMap;

/// Cache pour stocker les unités par cellule
#[derive(Resource, Default)]
pub struct UnitsCache {
    /// Map: GridCell -> Vec<unit_id>
    units_by_cell: HashMap<GridCell, Vec<u64>>,
    /// Map: (GridCell, SlotPosition) -> unit_id
    /// Tracks which unit occupies which slot in each cell
    slot_occupancy: HashMap<(GridCell, SlotPosition), u64>,
}

impl UnitsCache {
    pub fn add_unit(&mut self, cell: GridCell, unit_id: u64) {
        self.units_by_cell
            .entry(cell)
            .or_insert_with(Vec::new)
            .push(unit_id);
    }

    pub fn remove_unit(&mut self, unit_id: u64) {
        for (_, units) in self.units_by_cell.iter_mut() {
            units.retain(|&id| id != unit_id);
        }
        // Nettoyer les cellules vides
        self.units_by_cell.retain(|_, units| !units.is_empty());
    }

    pub fn get_units_at_cell(&self, cell: &GridCell) -> Option<&Vec<u64>> {
        self.units_by_cell.get(cell)
    }

    pub fn get_unit_count_at_cell(&self, cell: &GridCell) -> usize {
        self.units_by_cell
            .get(cell)
            .map(|units| units.len())
            .unwrap_or(0)
    }

    pub fn get_all_cells_with_units(&self) -> impl Iterator<Item = (&GridCell, &Vec<u64>)> {
        self.units_by_cell.iter()
    }

    // ========== Slot Management Methods ==========

    /// Set a unit's slot position within a cell
    pub fn set_unit_slot(&mut self, cell: GridCell, slot: SlotPosition, unit_id: u64) {
        self.slot_occupancy.insert((cell, slot), unit_id);
    }

    /// Remove a unit from its slot
    pub fn remove_unit_from_slot(&mut self, cell: GridCell, slot: SlotPosition) {
        self.slot_occupancy.remove(&(cell, slot));
    }

    /// Get the unit ID occupying a specific slot, if any
    pub fn get_unit_at_slot(&self, cell: &GridCell, slot: &SlotPosition) -> Option<u64> {
        self.slot_occupancy.get(&(*cell, *slot)).copied()
    }

    /// Check if a specific slot is occupied
    pub fn is_slot_occupied(&self, cell: &GridCell, slot: &SlotPosition) -> bool {
        self.slot_occupancy.contains_key(&(*cell, *slot))
    }

    /// Get the slot position of a unit in a specific cell, if it has one
    pub fn get_unit_slot(&self, cell: &GridCell, unit_id: u64) -> Option<SlotPosition> {
        self.slot_occupancy
            .iter()
            .find(|((c, _), uid)| c == cell && **uid == unit_id)
            .map(|((_, slot), _)| *slot)
    }

    /// Get all slots occupied in a specific cell
    pub fn get_occupied_slots(&self, cell: &GridCell) -> Vec<(SlotPosition, u64)> {
        self.slot_occupancy
            .iter()
            .filter(|((c, _), _)| c == cell)
            .map(|((_, slot), &unit_id)| (*slot, unit_id))
            .collect()
    }

    /// Clear all slot occupancy for a specific cell
    pub fn clear_cell_slots(&mut self, cell: &GridCell) {
        self.slot_occupancy.retain(|(c, _), _| c != cell);
    }
}
