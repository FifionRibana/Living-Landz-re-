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

// Action bar components (left sidebar)
#[derive(Component)]
pub struct ActionBarMarker;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    Roads,
    Buildings,
    Production,
    Management,
    Entertainment,
}

#[derive(Component)]
pub struct ActionCategoryButton {
    pub category: ActionCategory,
}

// Action panel components (tabbed panel)
#[derive(Component)]
pub struct ActionsPanelMarker;

#[derive(Component)]
pub struct ActionTabButton {
    pub tab_id: String,
}

#[derive(Component)]
pub struct ActionButtonMarker {
    pub action_type: String,
}

#[derive(Component)]
pub struct ActionTitleText;

#[derive(Component)]
pub struct ActionDescriptionText;

#[derive(Component)]
pub struct ActionContentContainer;

#[derive(Component)]
pub struct ActionTabsContainer;

#[derive(Component)]
pub struct BuildingGridContainer;

#[derive(Component)]
pub struct RecipeContainer;

#[derive(Component)]
pub struct BuildingButton {
    pub building_id: String,
    pub building_name: String,
}

#[derive(Component)]
pub struct ActionRunButton;

// Cell details components
#[derive(Component)]
pub struct CellDetailsPanelMarker;

// Top bar components
#[derive(Component)]
pub struct TopBarMarker;

// Chat components
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
pub struct ChatIconButton;

#[derive(Component)]
pub struct ChatNotificationBadge;
