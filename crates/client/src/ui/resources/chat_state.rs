use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ChatState {
    pub is_expanded: bool,
}

impl ChatState {
    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
    }
}
