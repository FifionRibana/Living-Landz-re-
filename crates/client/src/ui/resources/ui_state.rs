use bevy::prelude::*;

use crate::ui::resources::ChatState;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelEnum {
    MapView,
    CellView,
    ManagementPanel,
    RecordsPanel,
    MessagesPanel,
    RankingPanel,
    CalendarPanel,
    SearchView,
    SettingsView,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionModeEnum {
    RoadActionMode,
    BuildingActionMode,
    ProductionActionMode,
    TrainingActionMode,
    DiplomacyActionMode,
}

/// State resource for the cell detail view mode
#[derive(Resource)]
pub struct UIState {
    pub panel_state: PanelEnum,
    pub action_mode: Option<ActionModeEnum>,
    pub hovered_action_mode: Option<ActionModeEnum>,
    pub chat_state: ChatState,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            panel_state: PanelEnum::MapView,
            action_mode: None,
            hovered_action_mode: None,
            chat_state: ChatState::default(),
        }
    }
}

impl UIState {
    pub fn switch_to(&mut self, panel: PanelEnum) {
        self.panel_state = panel;
    }

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