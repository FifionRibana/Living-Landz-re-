use bincode::{Decode, Encode};

use crate::{BuildingCategory, TreeType};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct BuildingType {
    pub id: i32,
    pub category: BuildingCategory,
    pub variant: String,
}

impl BuildingType {
    pub fn tree(tree_type: TreeType, variant: String, id: i32) -> Self {
        Self {
            id,
            category: BuildingCategory::Natural,
            variant,
        }
    }
}
