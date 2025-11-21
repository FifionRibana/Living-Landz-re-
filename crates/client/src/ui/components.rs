use bevy::prelude::*;

#[derive(Component)]
pub struct ClockText;

#[derive(Component)]
pub struct DateText;

#[derive(Component)]
pub struct MoonText;

#[derive(Component)]
pub struct PlayerNameText;

#[derive(Component)]
pub struct CharacterNameText;

#[derive(Component)]
pub struct MenuButton {
    pub button_id: usize,
}
