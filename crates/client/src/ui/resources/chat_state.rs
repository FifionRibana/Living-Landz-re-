use bevy::prelude::*;

#[derive(Resource)]
pub struct ChatState {
    pub is_expanded: bool,
    pub unread_messages: u32,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            is_expanded: false,
            unread_messages: 0,
        }
    }
}

impl ChatState {
    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
        // Reset unread count when opening chat
        if self.is_expanded {
            self.unread_messages = 0;
        }
    }

    pub fn add_message(&mut self) {
        if !self.is_expanded {
            self.unread_messages += 1;
        }
    }
}
