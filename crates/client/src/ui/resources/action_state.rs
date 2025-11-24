use bevy::prelude::*;
use crate::ui::components::ActionCategory;
use shared::BuildingCategoryEnum;

#[derive(Resource)]
pub struct ActionState {
    pub selected_category: Option<ActionCategory>,
    pub selected_tab: Option<String>,
    pub selected_building_category: Option<BuildingCategoryEnum>,
    pub selected_building_id: Option<String>,
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            selected_category: None,
            selected_tab: None,
            selected_building_category: None,
            selected_building_id: None,
        }
    }
}

impl ActionState {
    pub fn select_category(&mut self, category: ActionCategory) {
        // Toggle if same category
        if self.selected_category == Some(category) {
            self.clear_category();
        } else {
            self.selected_category = Some(category);
            self.selected_tab = None;
            self.selected_building_category = None;
            self.selected_building_id = None;
        }
    }

    pub fn clear_category(&mut self) {
        self.selected_category = None;
        self.selected_tab = None;
        self.selected_building_category = None;
        self.selected_building_id = None;
    }

    pub fn select_tab(&mut self, tab_id: String) {
        self.selected_tab = Some(tab_id);
        self.selected_building_id = None;
    }

    pub fn select_building_category(&mut self, category: BuildingCategoryEnum) {
        self.selected_building_category = Some(category);
        self.selected_building_id = None;
    }

    pub fn select_building(&mut self, building_id: String) {
        self.selected_building_id = Some(building_id);
    }

    pub fn is_category_checked(&self, category: ActionCategory) -> bool {
        self.selected_category == Some(category)
    }
}
