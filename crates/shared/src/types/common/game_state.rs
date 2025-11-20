use bevy::prelude::*;
use sqlx::PgPool;

use crate::{
    BuildingCategory, BuildingSpecificType, BuildingType, ResourceCategory, ResourceSpecificType,
    ResourceType, atlas::TreeAtlas,
};

#[derive(Resource)]
pub struct GameState {
    pub pool: PgPool,

    // Lookup table cache
    pub building_categories: Vec<BuildingCategory>,
    pub building_specific_types: Vec<BuildingSpecificType>,
    pub resource_categories: Vec<ResourceCategory>,
    pub resource_specific_types: Vec<ResourceSpecificType>,

    // Principal cache
    pub building_types: Vec<BuildingType>,
    pub resource_types: Vec<ResourceType>,

    pub tree_atlas: TreeAtlas,
}

impl GameState {
    pub fn new(pool: PgPool) -> Self {
        let mut tree_atlas = TreeAtlas::default();
        tree_atlas.load();

        Self {
            pool,
            building_categories: Vec::new(),
            building_specific_types: Vec::new(),
            resource_categories: Vec::new(),
            resource_specific_types: Vec::new(),
            building_types: Vec::new(),
            resource_types: Vec::new(),
            tree_atlas,
        }
    }

    // ============ INITIALIZATION ============

    pub async fn initialize_caches(&mut self) -> Result<(), sqlx::Error> {
        self.building_categories = sqlx::query_as::<_, BuildingCategory>(
            "SELECT * FROM buildings.building_categories ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        self.building_specific_types = sqlx::query_as::<_, BuildingSpecificType>(
            "SELECT * FROM buildings.building_specific_types WHERE archived = FALSE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        self.resource_categories = sqlx::query_as::<_, ResourceCategory>(
            "SELECT * FROM resources.resource_categories ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        self.resource_specific_types = sqlx::query_as::<_, ResourceSpecificType>(
            "SELECT * FROM resources.resource_specific_types WHERE archived = FALSE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        // Charger les caches principaux
        self.building_types = sqlx::query_as::<_, BuildingType>(
            r#"
            SELECT 
                id,
                name,
                category_id,
                specific_type_id,
                description,
                archived
            FROM buildings.building_types
            WHERE archived = FALSE
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        self.resource_types = sqlx::query_as::<_, ResourceType>(
            r#"
            SELECT 
                id,
                name,
                category_id,
                specific_type_id,
                description,
                archived
            FROM resources.resource_types
            WHERE archived = FALSE
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(())
    }

    // ============ BUILDING CATEGORIES ============

    pub fn get_building_category(&self, id: i16) -> Option<&BuildingCategory> {
        self.building_categories.iter().find(|bc| bc.id == id)
    }

    pub fn get_building_category_id(&self, name: &str) -> Option<i16> {
        self.building_categories
            .iter()
            .find(|bc| bc.name.eq_ignore_ascii_case(name))
            .map(|bc| bc.id)
    }

    // ============ BUILDING SPECIFIC TYPES ============

    pub fn get_building_specific_type(&self, id: i16) -> Option<&BuildingSpecificType> {
        self.building_specific_types.iter().find(|bst| bst.id == id)
    }

    pub fn get_building_specific_type_id(&self, name: &str) -> Option<i16> {
        self.building_specific_types
            .iter()
            .find(|bst| bst.name.eq_ignore_ascii_case(name))
            .map(|bst| bst.id)
    }

    // ============ BUILDING TYPES ============

    pub fn get_building_type(&self, id: i32) -> Option<&BuildingType> {
        self.building_types.iter().find(|bt| bt.id == id)
    }

    pub fn get_building_type_by_name(&self, name: &str) -> Option<&BuildingType> {
        self.building_types
            .iter()
            .find(|bt| bt.name.eq_ignore_ascii_case(name))
    }

    pub fn get_building_type_id(&self, name: &str) -> Option<i32> {
        self.get_building_type_by_name(name).map(|bt| bt.id)
    }

    pub fn get_buildings_by_category(&self, category_id: i16) -> Vec<&BuildingType> {
        self.building_types
            .iter()
            .filter(|bt| bt.category_id == category_id)
            .collect()
    }

    pub fn get_buildings_by_specific_type(&self, specific_type_id: i16) -> Vec<&BuildingType> {
        self.building_types
            .iter()
            .filter(|bt| bt.specific_type_id == specific_type_id)
            .collect()
    }

    // ============ RESOURCE CATEGORIES ============

    pub fn get_resource_category(&self, id: i16) -> Option<&ResourceCategory> {
        self.resource_categories.iter().find(|rc| rc.id == id)
    }

    pub fn get_resource_category_id(&self, name: &str) -> Option<i16> {
        self.resource_categories
            .iter()
            .find(|rc| rc.name.eq_ignore_ascii_case(name))
            .map(|rc| rc.id)
    }

    // ============ RESOURCE SPECIFIC TYPES ============

    pub fn get_resource_specific_type(&self, id: i16) -> Option<&ResourceSpecificType> {
        self.resource_specific_types.iter().find(|bst| bst.id == id)
    }

    pub fn get_resource_specific_type_id(&self, name: &str) -> Option<i16> {
        self.resource_specific_types
            .iter()
            .find(|bst| bst.name.eq_ignore_ascii_case(name))
            .map(|bst| bst.id)
    }

    // ============ RESOURCE TYPES ============

    pub fn get_resource_type(&self, id: i32) -> Option<&ResourceType> {
        self.resource_types.iter().find(|rt| rt.id == id)
    }

    pub fn get_resource_type_by_name(&self, name: &str) -> Option<&ResourceType> {
        self.resource_types
            .iter()
            .find(|rt| rt.name.eq_ignore_ascii_case(name))
    }

    pub fn get_resource_type_id(&self, name: &str) -> Option<i32> {
        self.get_resource_type_by_name(name).map(|rt| rt.id)
    }

    pub fn get_resources_by_category(&self, category_id: i16) -> Vec<&ResourceType> {
        self.resource_types
            .iter()
            .filter(|rt| rt.category_id == category_id)
            .collect()
    }

    pub fn get_resources_by_specific_type(&self, specific_type_id: i16) -> Vec<&ResourceType> {
        self.resource_types
            .iter()
            .filter(|rt| rt.specific_type_id == specific_type_id)
            .collect()
    }
}
