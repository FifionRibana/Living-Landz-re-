use bevy::prelude::*;
use shared::types::game::{Character, Player};

#[derive(Resource, Debug, Clone)]
pub struct PlayerInfo {
    pub player: Option<Player>,
    pub characters: Vec<Character>,
    pub active_character: Option<Character>,
    // Temporary storage for player/character names when full data is not available
    pub temp_player_name: Option<String>,
    pub temp_character_name: Option<String>,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            player: None,
            characters: Vec::new(),
            active_character: None,
            temp_player_name: None,
            temp_character_name: None,
        }
    }
}

impl PlayerInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_player(&mut self, player: Player) {
        self.player = Some(player);
    }

    pub fn set_characters(&mut self, characters: Vec<Character>) {
        self.characters = characters;
    }

    pub fn set_active_character(&mut self, character: Character) {
        self.active_character = Some(character);
    }

    pub fn get_player_id(&self) -> Option<i64> {
        self.player.as_ref().map(|p| p.id)
    }

    pub fn get_active_character_id(&self) -> Option<i64> {
        self.active_character.as_ref().map(|c| c.id)
    }

    pub fn is_initialized(&self) -> bool {
        self.player.is_some()
    }
}
