use bevy::prelude::*;
use shared::SlotPosition;

use crate::ui::resources::{ActionModeEnum, PanelEnum};

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
pub struct PanelContainer {
    pub panel: PanelEnum,
}

#[derive(Component)]
pub struct MenuButton {
    pub button_id: usize,
    pub panel: PanelEnum,
}

#[derive(Component)]
pub struct ActionModeMenuButton {
    pub action_mode: ActionModeEnum,
}

#[derive(Component)]
pub struct ActionModeMenuIcon {
    pub action_mode: ActionModeEnum,
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

#[derive(Component)]
pub struct ActionMenuMarker;

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
pub struct InteriorSlotContainer;

#[derive(Component)]
pub struct ExteriorSlotContainer;

#[derive(Component)]
pub struct SlotGridContainer {
    pub slot_type: shared::SlotType,
}

/// Visual state of a slot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlotState {
    #[default]
    Normal,
    Selected,
    Disabled,
    Blocked,
    Full,
    Flooded,
    Burning,
    Invalid,
}

impl SlotState {
    /// Get the sprite path for this state
    pub fn get_sprite_path(&self, is_occupied: bool) -> String {
        let base = match self {
            SlotState::Normal => "ui/ui_hex_normal",
            SlotState::Selected => "ui/ui_hex_selected",
            SlotState::Disabled => "ui/ui_hex_disabled",
            SlotState::Blocked => "ui/ui_hex_disabled", // Reuse disabled for now
            SlotState::Full => "ui/ui_hex_full",
            SlotState::Flooded => "ui/ui_hex_invalid_crippled", // Reuse for now
            SlotState::Burning => "ui/ui_hex_invalid_crippled", // Reuse for now
            SlotState::Invalid => "ui/ui_hex_invalid_crippled",
        };

        // Use _empty suffix when slot is occupied
        if is_occupied
            && matches!(
                self,
                SlotState::Normal | SlotState::Selected | SlotState::Disabled
            )
        {
            format!("{}_empty.png", base)
        } else {
            format!("{}.png", base)
        }
    }

    /// Get the opacity/alpha value for this state
    pub fn get_opacity(&self, is_occupied: bool) -> f32 {
        match self {
            SlotState::Normal => {
                if is_occupied {
                    0.6
                } else {
                    0.1
                }
            }
            SlotState::Selected => 1.0,
            SlotState::Disabled => 0.1,
            SlotState::Blocked => 0.1,
            SlotState::Full => 0.1,
            SlotState::Flooded => 0.1,
            SlotState::Burning => 0.1,
            SlotState::Invalid => 0.1,
        }
    }

    /// Get the opacity when hovering over this state
    pub fn get_hover_opacity(&self, is_occupied: bool) -> f32 {
        match self {
            SlotState::Normal => {
                if is_occupied {
                    1.0
                } else {
                    0.2
                }
            }
            SlotState::Selected => 1.0, // Selected stays at 100%
            SlotState::Disabled => 0.1, // Disabled doesn't change
            SlotState::Blocked => 0.1,  // Blocked doesn't change
            SlotState::Full => 0.2,
            SlotState::Flooded => 0.2,
            SlotState::Burning => 0.2,
            SlotState::Invalid => 0.1, // Invalid doesn't change
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SlotIndicator {
    pub position: SlotPosition,
    pub occupied_by: Option<u64>, // unit_id if occupied
    pub state: SlotState,
    pub hovered: bool,
    pub is_dragging: bool,
}

impl SlotIndicator {
    pub fn new(position: SlotPosition) -> Self {
        Self {
            position,
            occupied_by: None,
            state: SlotState::Normal,
            hovered: false,
            is_dragging: false,
        }
    }

    pub fn with_state(mut self, state: SlotState) -> Self {
        self.state = state;
        self
    }

    pub fn is_occupied(&self) -> bool {
        self.occupied_by.is_some()
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
}

#[derive(Component)]
pub struct Slot {
    pub slot_position: SlotPosition,
}

#[derive(Component)]
pub struct SlotUnitPortrait {
    pub unit_id: u64,
    pub slot_position: SlotPosition,
}

#[derive(Component)]
pub struct SlotUnitSprite {
    pub unit_id: u64,
    pub slot_position: SlotPosition,
}

/// Component to mark border overlays that should be displayed on top of portraits
#[derive(Component)]
pub struct SlotBorderOverlay {
    pub slot_position: shared::SlotPosition,
}

/// Component to mark portraits that need hex masking
#[derive(Component)]
pub struct PendingHexMask {
    pub portrait_handle: Handle<Image>,
    pub mask_handle: Handle<Image>,
}


#[derive(Component)]
pub struct CellViewBackButton;

// Unit details panel components
#[derive(Component)]
pub struct UnitDetailsPanelMarker;

#[derive(Component)]
pub struct UnitDetailsAvatar;

#[derive(Component)]
pub struct UnitDetailsNameText;

#[derive(Component)]
pub struct UnitDetailsLevelText;

#[derive(Component)]
pub struct UnitDetailsProfessionText;

#[derive(Component)]
pub struct UnitDetailsCloseButton;
