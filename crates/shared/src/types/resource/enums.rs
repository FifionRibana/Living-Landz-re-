use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum ResourceCategoryEnum {
    Unknown = 0,
    Wood = 1,
    Metal = 2,
    CrudeMaterial = 3,
    Food = 4,
    Furniture = 5,
    Weaponry = 6,
    Jewelry = 7,
    Meat = 8,
    Fruits = 9,
    Vegetables = 10,
}

impl ResourceCategoryEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Wood),
            2 => Some(Self::Metal),
            3 => Some(Self::CrudeMaterial),
            4 => Some(Self::Food),
            5 => Some(Self::Furniture),
            6 => Some(Self::Weaponry),
            7 => Some(Self::Jewelry),
            8 => Some(Self::Meat),
            9 => Some(Self::Fruits),
            10 => Some(Self::Vegetables),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum ResourceSpecificTypeEnum {
    Unknown = 0,
    Wood = 1,
    Ore = 2,
    Metal = 3,
    Mineral = 4,
}

impl ResourceSpecificTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Wood),
            2 => Some(Self::Ore),
            3 => Some(Self::Metal),
            4 => Some(Self::Mineral),
            _ => None,
        }
    }
}
