use crate::BiomeType;
use bincode::{Encode, Decode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Encode, Decode)]
#[sqlx(type_name = "dwellings_type")]
pub enum UrbanismType {
    Path,
    Road,
    PavedRoad,
    Avenue,
}

impl UrbanismType {

    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn iter() -> impl Iterator<Item = UrbanismType> {
        [
            UrbanismType::Path,
            UrbanismType::Road,
            UrbanismType::PavedRoad,
            UrbanismType::Avenue,
        ]
        .into_iter()
    }
}
