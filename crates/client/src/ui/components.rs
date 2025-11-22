use bevy::prelude::*;

#[derive(Component)]
pub struct ClockText;

#[derive(Component)]
pub struct DateText;

#[derive(Component)]
pub struct MoonText;

#[derive(Component)]
pub struct MoonPhaseImage;

#[derive(Component)]
pub struct PlayerNameText;

#[derive(Component)]
pub struct CharacterNameText;

#[derive(Component)]
pub struct MenuButton {
    pub button_id: usize,
}

#[derive(Component)]
pub struct ActionsPanelMarker;

#[derive(Component)]
pub struct ActionButtonMarker {
    pub action_type: String,
}

#[derive(Component)]
pub struct ActionTitleText;

#[derive(Component)]
pub struct ActionDescriptionText;

#[derive(Component)]
pub struct ChatPanelMarker;

#[derive(Component)]
pub struct ChatMessagesContainer;

#[derive(Component)]
pub struct ChatInputField;

#[derive(Component)]
pub struct ChatSendButton;

#[derive(Component)]
pub struct ChatToggleButton;

#[derive(Component)]
pub struct ChatInputContainer;

#[derive(Component)]
pub struct ChatInputText;

#[derive(Component, Default)]
pub struct ChatInputState {
    pub text: String,
    pub is_focused: bool,
}
