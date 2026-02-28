use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub logged_in: bool,
    pub player_id: Option<u64>,
}

impl ConnectionStatus {
    pub fn is_ready(&self) -> bool {
        self.connected && self.logged_in
    }

    /// Reset auth state (on disconnect or logout).
    pub fn reset_auth(&mut self) {
        self.logged_in = false;
        self.player_id = None;
    }
}