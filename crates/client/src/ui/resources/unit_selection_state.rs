use bevy::prelude::*;

/// État de sélection d'unités, partagé entre vue Map et vue Cell.
///
/// Remplace l'ancien `CellViewState.selected_unit: Option<u64>` par un système
/// de multi-sélection avec support Ctrl+clic et Shift+clic.
#[derive(Resource, Default, Debug)]
pub struct UnitSelectionState {
    /// Unités actuellement sélectionnées (par ID)
    selected_units: Vec<u64>,
}

impl UnitSelectionState {
    /// Sélection simple (remplace toute la sélection)
    pub fn select(&mut self, unit_id: u64) {
        info!("Unit selection: {}, {:?}", unit_id, self.selected_units);
        self.selected_units = vec![unit_id];
        info!("After: {:?}", self.selected_units);
    }

    /// Toggle sélection (Ctrl+clic) — ajoute ou retire
    pub fn toggle(&mut self, unit_id: u64) {
        info!("Toggle unit selection: {}", unit_id);
        if let Some(pos) = self.selected_units.iter().position(|&id| id == unit_id) {
            self.selected_units.remove(pos);
        } else {
            self.selected_units.push(unit_id);
        }
    }

    /// Ajouter à la sélection (Shift+clic) — ajoute sans retirer
    pub fn add(&mut self, unit_id: u64) {
        if !self.selected_units.contains(&unit_id) {
            self.selected_units.push(unit_id);
        }
    }

    /// Tout désélectionner
    pub fn clear(&mut self) {
        self.selected_units.clear();
    }

    /// Sélectionner plusieurs unités d'un coup
    pub fn select_multiple(&mut self, unit_ids: &[u64]) {
        self.selected_units = unit_ids.to_vec();
    }

    /// L'unité est-elle sélectionnée ?
    pub fn is_selected(&self, unit_id: u64) -> bool {
        self.selected_units.contains(&unit_id)
    }

    /// Y a-t-il au moins une unité sélectionnée ?
    pub fn has_selection(&self) -> bool {
        !self.selected_units.is_empty()
    }

    /// Nombre d'unités sélectionnées
    pub fn count(&self) -> usize {
        self.selected_units.len()
    }

    /// Unité primaire (première sélectionnée) — utilisée pour le panel de détails
    pub fn primary(&self) -> Option<u64> {
        self.selected_units.first().copied()
    }

    /// Accès en lecture aux IDs sélectionnés
    pub fn selected_ids(&self) -> &[u64] {
        &self.selected_units
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_single() {
        let mut state = UnitSelectionState::default();
        state.select(42);
        assert_eq!(state.count(), 1);
        assert!(state.is_selected(42));
        assert_eq!(state.primary(), Some(42));
    }

    #[test]
    fn test_toggle() {
        let mut state = UnitSelectionState::default();
        state.select(1);
        state.toggle(2);
        assert_eq!(state.count(), 2);

        state.toggle(1);
        assert_eq!(state.count(), 1);
        assert!(!state.is_selected(1));
        assert!(state.is_selected(2));
    }

    #[test]
    fn test_add() {
        let mut state = UnitSelectionState::default();
        state.select(1);
        state.add(2);
        state.add(2); // duplicate — should not add
        assert_eq!(state.count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut state = UnitSelectionState::default();
        state.select(1);
        state.add(2);
        state.clear();
        assert!(!state.has_selection());
        assert_eq!(state.primary(), None);
    }

    #[test]
    fn test_select_replaces() {
        let mut state = UnitSelectionState::default();
        state.add(1);
        state.add(2);
        state.add(3);
        state.select(99);
        assert_eq!(state.count(), 1);
        assert!(state.is_selected(99));
    }
}
