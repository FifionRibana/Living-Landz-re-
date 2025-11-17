use crate::BiomeType;
use bincode::{Encode, Decode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Encode, Decode)]
#[sqlx(type_name = "dwellings_type")]
pub enum DwellingsType {
    House,
}

impl DwellingsType {

    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn iter() -> impl Iterator<Item = DwellingsType> {
        [
            DwellingsType::House,
        ]
        .into_iter()
    }
}
