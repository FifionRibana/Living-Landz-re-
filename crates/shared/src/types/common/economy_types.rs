use bincode::{Decode, Encode};

/// Construction cost entry (building_type_id -> item costs)
#[derive(Debug, Clone, Encode, Decode)]
pub struct ConstructionCost {
    pub building_type_id: i32,
    pub item_id: i32,
    pub quantity: i32,
}

/// Harvest yield definition (what a resource type produces)
#[derive(Debug, Clone, Encode, Decode)]
pub struct HarvestYield {
    pub id: i32,
    pub resource_specific_type_id: i16,
    pub result_item_id: i32,
    pub base_quantity: i32,
    pub quality_min: f32,
    pub quality_max: f32,
    pub required_profession_id: Option<i16>,
    pub required_tool_item_id: Option<i32>,
    pub tool_bonus_quantity: i32,
    pub duration_seconds: i32,
}

/// Translation key for lookup
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TranslationKey {
    pub entity_type: String,
    pub entity_id: i32,
    pub language_id: i16,
    pub field: String,
}
