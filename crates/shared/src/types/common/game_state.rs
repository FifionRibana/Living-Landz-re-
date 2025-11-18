use std::collections::HashMap;

use bevy::prelude::*;

use crate::{BuildingCategory, BuildingSpecificType, ResourceCategory, ResourceSpecificType, atlas::TreeAtlas};

#[derive(Resource)]
pub struct GameState {
    pub building_categories: HashMap<i32, BuildingCategoryData>,
    pub building_types: HashMap<i32, BuildingTypeData>,
    pub resource_types: HashMap<i32, ResourceTypeData>,

    pub tree_atlas: TreeAtlas,
}

#[derive(Clone)]
pub struct BuildingCategoryData {
    pub id: i32,
    pub name: String,
}

#[derive(Clone)]
pub struct BuildingTypeData {
    pub id: i32,
    pub name: String,
    pub category: BuildingCategory,
    pub specific_type: BuildingSpecificType,
    pub description: String
}

#[derive(Clone)]
pub struct ResourceTypeData {
    pub id: i32,
    pub name: String,
    pub category: ResourceCategory,
    pub specific_type: ResourceSpecificType,
    pub description: String
}

impl Default for GameState {
    fn default() -> Self {
        let mut tree_atlas = TreeAtlas::default();
        tree_atlas.load();

        Self {
            building_categories: HashMap::new(),
            building_types: HashMap::new(),
            resource_types: HashMap::new(),
            tree_atlas
        }
    }
}

impl GameState {
    pub fn get_building_category_id(&self, name: &str) -> Option<i32> {
        self.building_categories
            .iter()
            .find(|(_, bt)| bt.name == name)
            .map(|(id, _)| *id)
    }

    pub fn get_building_type_id(&self, name: &str) -> Option<i32> {
        self.building_types
            .iter()
            .find(|(_, bt)| bt.name == name)
            .map(|(id, _)| *id)
    }

    pub fn get_resource_type_id(&self, name: &str) -> Option<i32> {
        self.resource_types
            .iter()
            .find(|(_, bt)| bt.name == name)
            .map(|(id, _)| *id)
    }


}
