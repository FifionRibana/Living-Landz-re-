use bevy::prelude::*;
use shared::SlotPosition;

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

#[derive(Component)]
pub struct CellDetailsTitleText;

#[derive(Component)]
pub struct CellDetailsBiomeText;

#[derive(Component)]
pub struct CellDetailsBuildingImage;

#[derive(Component)]
pub struct CellDetailsQualityGaugeContainer;

#[derive(Component)]
pub struct CellDetailsActionStatusText;

#[derive(Component)]
pub struct CellDetailsActionTypeText;

#[derive(Component)]
pub struct CellDetailsOrganizationText;

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

#[derive(Component)]
pub struct ChatNotificationBadgeText;

// Cell view components (detailed cell view mode)
#[derive(Component)]
pub struct CellViewContainer;

#[derive(Component)]
pub struct CellViewBackgroundImage;

#[derive(Component)]
pub struct SlotGridContainer {
    pub slot_type: shared::SlotType,
}

#[derive(Component)]
pub struct SlotIndicator {
    pub position: SlotPosition,
    pub occupied_by: Option<u64>, // unit_id if occupied
}

#[derive(Component)]
pub struct SlotUnitSprite {
    pub unit_id: u64,
    pub slot_position: SlotPosition,
}

#[derive(Component)]
pub struct CellViewBackButton;

// Unit details panel components
#[derive(Component)]
pub struct UnitDetailsPanelMarker;

#[derive(Component)]
pub struct UnitDetailsNameText;

#[derive(Component)]
pub struct UnitDetailsLevelText;

#[derive(Component)]
pub struct UnitDetailsProfessionText;

#[derive(Component)]
pub struct UnitDetailsCloseButton;
