use bevy::prelude::*;

use crate::ui::resources::ChatState;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionModeEnum {
    RoadActionMode,
    BuildingActionMode,
    ProductionActionMode,
    TrainingActionMode,
    DiplomacyActionMode,
}

/// UI state resource for action modes and chat.
/// Panel navigation is now handled entirely by the GameView state machine.
#[derive(Resource)]
pub struct UIState {
    pub action_mode: Option<ActionModeEnum>,
    pub hovered_action_mode: Option<ActionModeEnum>,
    pub chat_state: ChatState,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            action_mode: None,
            hovered_action_mode: None,
            chat_state: ChatState::default(),
        }
    }
}

impl UIState {
    pub fn set_action_mode(&mut self, action_mode: ActionModeEnum) {
        self.action_mode = Some(action_mode);
    }

    pub fn set_action_mode_hovered(&mut self, action_mode: ActionModeEnum, state: bool) {
        self.hovered_action_mode = if state { Some(action_mode) } else { None }
    }

    pub fn reset_action_mode(&mut self) {
        self.action_mode = None;
    }
}
