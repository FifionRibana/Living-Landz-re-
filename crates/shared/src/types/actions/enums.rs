use bincode::{Decode, Encode};

use super::super::unit::ProfessionEnum;

// ============ ACTION MODE (catégories d'actions UI) ============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum ActionModeEnum {
    RoadActionMode = 1,
    BuildingActionMode = 2,
    ProductionActionMode = 3,
    TrainingActionMode = 4,
    DiplomacyActionMode = 5,
}

impl ActionModeEnum {
    /// All action modes.
    pub const ALL: [ActionModeEnum; 5] = [
        Self::RoadActionMode,
        Self::BuildingActionMode,
        Self::ProductionActionMode,
        Self::TrainingActionMode,
        Self::DiplomacyActionMode,
    ];

    /// Which professions can perform this action mode.
    /// Roads are basic labor — any unit can do it.
    pub fn required_professions(&self) -> &'static [ProfessionEnum] {
        use ProfessionEnum::*;
        match self {
            Self::RoadActionMode => &[
                Unknown, Settler, Baker, Farmer, Warrior, Blacksmith, Carpenter, Miner,
                Merchant, Hunter, Healer, Scholar, Cook, Fisherman, Lumberjack,
                Mason, Brewer,
            ],
            Self::BuildingActionMode => &[Carpenter, Mason, Lumberjack, Blacksmith],
            Self::ProductionActionMode => &[
                Farmer, Baker, Cook, Brewer, Blacksmith, Carpenter, Lumberjack,
                Mason, Fisherman, Miner,
            ],
            Self::TrainingActionMode => &[Settler, Warrior, Scholar, Hunter],
            Self::DiplomacyActionMode => &[Merchant, Scholar],
        }
    }

    /// Returns true if the given profession can perform this action mode.
    pub fn is_available_for(&self, profession: &ProfessionEnum) -> bool {
        self.required_professions().contains(profession)
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::RoadActionMode => "Routes",
            Self::BuildingActionMode => "Construction",
            Self::ProductionActionMode => "Production",
            Self::TrainingActionMode => "Formation",
            Self::DiplomacyActionMode => "Diplomatie",
        }
    }
}

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
    TrainUnit = 7,
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
            7 => Some(Self::TrainUnit),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Action",
            Self::BuildBuilding => "Construction",
            Self::BuildRoad => "Route",
            Self::MoveUnit => "Déplacement",
            Self::SendMessage => "Message",
            Self::HarvestResource => "Récolte",
            Self::CraftResource => "Fabrication",
            Self::TrainUnit => "Formation",
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
    TrainUnit = 7,
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
            7 => Some(Self::TrainUnit),
            _ => None,
        }
    }
}
