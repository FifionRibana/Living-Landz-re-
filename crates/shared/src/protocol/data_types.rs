use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use crate::{
    ContourSegmentData, EquipmentSlotEnum, ItemTypeEnum, OrganizationSummary,
    TerrainChunkId, UnitData, grid::GridCell,
};

/// Simplified Player data for network protocol (without timestamps)
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct PlayerData {
    pub id: i64,
    pub family_name: String,
    pub language_id: i16,
    pub coat_of_arms_id: Option<i64>,
    pub motto: Option<String>,
    pub origin_location: String,
}

/// Simplified Character data for network protocol (without timestamps)
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct CharacterData {
    pub id: i64,
    pub player_id: i64,
    pub first_name: String,
    pub family_name: String,
    pub second_name: Option<String>,
    pub nickname: Option<String>,
    pub coat_of_arms_id: Option<i64>,
    pub image_id: Option<i64>,
    pub motto: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ColorData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorData {
    pub fn from_array(color: [f32; 4]) -> Self {
        Self {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Territory contour data for a specific organization in a specific chunk
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct TerritoryContourChunkData {
    pub organization_id: u64,
    pub chunk_id: TerrainChunkId,
    pub segments: Vec<ContourSegmentData>,
    pub border_color: ColorData,
    pub fill_color: ColorData,
}

// =============================================================================
// GAME DATA PAYLOAD
// =============================================================================

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct GameDataPayload {
    pub items: Vec<ItemDefinitionNet>,
    pub recipes: Vec<RecipeNet>,
    pub construction_costs: Vec<ConstructionCostNet>,
    pub harvest_yields: Vec<HarvestYieldNet>,
    pub translations: Vec<TranslationEntry>,
    pub dev_mode: bool,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ItemDefinitionNet {
    pub id: i32,
    pub name: String,
    pub item_type_id: i16,
    pub category_id: Option<i16>,
    pub weight_kg: f32,
    pub base_price: i32,
    pub is_perishable: bool,
    pub is_equipable: bool,
    pub equipment_slot_id: Option<i16>,
    pub is_craftable: bool,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct RecipeNet {
    pub id: i32,
    pub name: String,
    pub result_item_id: i32,
    pub result_quantity: i32,
    pub required_skill_id: Option<i16>,
    pub required_skill_level: i32,
    pub craft_duration_seconds: i32,
    pub required_building_type_id: Option<i16>,
    pub ingredients: Vec<RecipeIngredientNet>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct RecipeIngredientNet {
    pub item_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ConstructionCostNet {
    pub building_type_id: i32,
    pub item_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct HarvestYieldNet {
    pub resource_specific_type_id: i16,
    pub result_item_id: i32,
    pub base_quantity: i32,
    pub required_profession_id: Option<i16>,
    pub duration_seconds: i32,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub entity_type: String,
    pub entity_id: i32,
    pub language_id: i16,
    pub field: String,
    pub value: String,
}

// =============================================================================
// INVENTORY
// =============================================================================

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct InventoryItemData {
    pub instance_id: u64,
    pub item_id: i32,
    pub name: String,
    pub item_type: ItemTypeEnum,
    pub quality: f32,
    pub weight_kg: f32,
    pub quantity: i32,
    pub is_equipped: bool,
    pub equipment_slot: Option<EquipmentSlotEnum>,
}
