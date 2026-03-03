use bevy::prelude::*;
use shared::{UIActionContext, ActionViewContext, BiomeTypeEnum, BuildingTypeEnum, ProfessionEnum};

/// Client-side resource that provides the current UIActionContext,
/// computed from game state every frame.
#[derive(Resource, Default)]
pub struct ActionContextState {
    pub context: Option<UIActionContext>,
}

impl ActionContextState {
    pub fn get(&self) -> Option<&UIActionContext> {
        self.context.as_ref()
    }

    pub fn update(
        &mut self,
        view: ActionViewContext,
        building: Option<BuildingTypeEnum>,
        terrain: BiomeTypeEnum,
        professions: Vec<ProfessionEnum>,
        has_adjacent_road: bool,
    ) {
        self.context = Some(UIActionContext {
            view,
            building,
            terrain,
            selected_professions: professions,
            has_adjacent_road,
        });
    }

    pub fn clear(&mut self) {
        self.context = None;
    }
}
