use bevy::prelude::*;
use shared::{grid::GridCell, SlotPosition, SlotType};

/// State resource for the cell detail view mode
#[derive(Resource, Default)]
pub struct CellViewState {
    /// Whether the cell view mode is currently active
    pub is_active: bool,
    /// The cell currently being viewed
    pub viewed_cell: Option<GridCell>,
    /// The currently selected slot (if any)
    pub selected_slot: Option<SlotPosition>,
    /// The currently selected unit (for details panel)
    pub selected_unit: Option<u64>,
    /// Information about a unit being dragged (if any)
    pub dragging_unit: Option<DraggingUnit>,
}

/// Information about a unit currently being dragged
#[derive(Debug, Clone)]
pub struct DraggingUnit {
    pub unit_id: u64,
    pub from_slot: SlotPosition,
}

impl CellViewState {
    /// Enter cell view mode for a specific cell
    pub fn enter_view(&mut self, cell: GridCell) {
        self.is_active = true;
        self.viewed_cell = Some(cell);
        self.selected_slot = None;
        self.dragging_unit = None;
    }

    /// Exit cell view mode
    pub fn exit_view(&mut self) {
        self.is_active = false;
        self.viewed_cell = None;
        self.selected_slot = None;
        self.dragging_unit = None;
    }

    /// Start dragging a unit from a slot
    pub fn start_dragging(&mut self, unit_id: u64, from_slot: SlotPosition) {
        self.dragging_unit = Some(DraggingUnit { unit_id, from_slot });
    }

    /// Stop dragging (cancel drag)
    pub fn stop_dragging(&mut self) {
        self.dragging_unit = None;
    }

    /// Check if currently dragging a unit
    pub fn is_dragging(&self) -> bool {
        self.dragging_unit.is_some()
    }

    /// Select a slot
    pub fn select_slot(&mut self, slot: SlotPosition) {
        self.selected_slot = Some(slot);
    }

    /// Deselect current slot
    pub fn deselect_slot(&mut self) {
        self.selected_slot = None;
    }

    /// Toggle slot selection
    pub fn toggle_slot(&mut self, slot: SlotPosition) {
        if self.selected_slot == Some(slot) {
            self.deselect_slot();
        } else {
            self.select_slot(slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_view_state() {
        let mut state = CellViewState::default();
        assert!(!state.is_active);

        let cell = GridCell { q: 0, r: 0 };
        state.enter_view(cell);
        assert!(state.is_active);
        assert_eq!(state.viewed_cell, Some(cell));

        state.exit_view();
        assert!(!state.is_active);
        assert_eq!(state.viewed_cell, None);
    }

    #[test]
    fn test_dragging() {
        let mut state = CellViewState::default();
        let slot = SlotPosition::interior(0);

        state.start_dragging(123, slot);
        assert!(state.is_dragging());
        assert_eq!(state.dragging_unit.as_ref().unwrap().unit_id, 123);

        state.stop_dragging();
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_slot_selection() {
        let mut state = CellViewState::default();
        let slot = SlotPosition::exterior(5);

        state.select_slot(slot);
        assert_eq!(state.selected_slot, Some(slot));

        state.toggle_slot(slot);
        assert_eq!(state.selected_slot, None);

        state.toggle_slot(slot);
        assert_eq!(state.selected_slot, Some(slot));
    }
}
