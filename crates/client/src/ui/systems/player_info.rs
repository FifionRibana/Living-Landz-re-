use bevy::prelude::*;

use crate::{
    state::resources::PlayerInfo,
    ui::components::{CharacterNameText, PlayerNameText},
};

pub fn update_player_info(
    player_info: Res<PlayerInfo>,
    mut player_name_query: Query<&mut Text, (With<PlayerNameText>, Without<CharacterNameText>)>,
    mut character_name_query: Query<&mut Text, (With<CharacterNameText>, Without<PlayerNameText>)>,
) {
    // Update player name
    for mut text in player_name_query.iter_mut() {
        if let Some(player) = &player_info.player {
            **text = format!("House {}", player.family_name);
        } else if let Some(temp_name) = &player_info.temp_player_name {
            **text = format!("House {}", temp_name);
        } else {
            **text = "--".to_string();
        }
    }

    // Update character name
    for mut text in character_name_query.iter_mut() {
        if let Some(character) = &player_info.active_character {
            **text = format!("{} {}", character.first_name, character.family_name);
        } else if let Some(temp_name) = &player_info.temp_character_name {
            **text = temp_name.clone();
        } else {
            **text = "--".to_string();
        }
    }
}
