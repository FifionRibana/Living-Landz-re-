use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum ActionStatusEnum {
    InProgress = 1,
    Pending = 2,
    Completed = 3,
    Failed = 4,
}

impl ActionStatusEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::InProgress),
            2 => Some(Self::Pending),
            3 => Some(Self::Completed),
            4 => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum ActionTypeEnum {
    Unknown = 0,
    BuildBuilding = 1,
    BuildRoad = 2,
    MoveUnit = 3,
    SendMessage = 4,
    HarvestResource = 5,
    CraftResource = 6,
}

impl ActionTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::BuildBuilding),
            2 => Some(Self::BuildRoad),
            3 => Some(Self::MoveUnit),
            4 => Some(Self::SendMessage),
            5 => Some(Self::HarvestResource),
            6 => Some(Self::CraftResource),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum ActionSpecificTypeEnum {
    BuildBuilding = 1,
    BuildRoad = 2,
    MoveUnit = 3,
    SendMessage = 4,
    HarvestResource = 5,
    CraftResource = 6,
}

impl ActionSpecificTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::BuildBuilding),
            2 => Some(Self::BuildRoad),
            3 => Some(Self::MoveUnit),
            4 => Some(Self::SendMessage),
            5 => Some(Self::HarvestResource),
            6 => Some(Self::CraftResource),
            _ => None,
        }
    }
}
