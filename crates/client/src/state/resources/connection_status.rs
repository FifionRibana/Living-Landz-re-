use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ConnectionStatus {
    pub logged_in: bool,
    pub player_id: Option<u64>,
}

impl ConnectionStatus {
    pub fn is_ready(&self) -> bool {
        self.logged_in
    }
}