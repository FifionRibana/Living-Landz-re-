use bevy::prelude::*;

/// Tracks which action card is currently expanded in the detail panel.
#[derive(Resource, Default)]
pub struct ActionSelectionState {
    pub expanded_action: Option<String>,
}

impl ActionSelectionState {
    pub fn toggle(&mut self, action_id: &str) {
        if self.expanded_action.as_deref() == Some(action_id) {
            self.expanded_action = None;
        } else {
            self.expanded_action = Some(action_id.to_string());
        }
    }

    pub fn close(&mut self) {
        self.expanded_action = None;
    }

    pub fn is_expanded(&self, action_id: &str) -> bool {
        self.expanded_action.as_deref() == Some(action_id)
    }
}
