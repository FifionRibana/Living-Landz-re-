use bincode::{Decode, Encode};

// ============ PROFESSIONS ============
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum ProfessionEnum {
    Unknown = 0,
    Baker = 1,
    Farmer = 2,
    Warrior = 3,
    Blacksmith = 4,
    Carpenter = 5,
    Miner = 6,
    Merchant = 7,
    Hunter = 8,
    Healer = 9,
    Scholar = 10,
    Cook = 11,
    Fisherman = 12,
    Lumberjack = 13,
    Mason = 14,
    Brewer = 15,
}

impl ProfessionEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Baker),
            2 => Some(Self::Farmer),
            3 => Some(Self::Warrior),
            4 => Some(Self::Blacksmith),
            5 => Some(Self::Carpenter),
            6 => Some(Self::Miner),
            7 => Some(Self::Merchant),
            8 => Some(Self::Hunter),
            9 => Some(Self::Healer),
            10 => Some(Self::Scholar),
            11 => Some(Self::Cook),
            12 => Some(Self::Fisherman),
            13 => Some(Self::Lumberjack),
            14 => Some(Self::Mason),
            15 => Some(Self::Brewer),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Baker => "Baker",
            Self::Farmer => "Farmer",
            Self::Warrior => "Warrior",
            Self::Blacksmith => "Blacksmith",
            Self::Carpenter => "Carpenter",
            Self::Miner => "Miner",
            Self::Merchant => "Merchant",
            Self::Hunter => "Hunter",
            Self::Healer => "Healer",
            Self::Scholar => "Scholar",
            Self::Cook => "Cook",
            Self::Fisherman => "Fisherman",
            Self::Lumberjack => "Lumberjack",
            Self::Mason => "Mason",
            Self::Brewer => "Brewer",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Baker => "baker",
            Self::Farmer => "farmer",
            Self::Warrior => "warrior",
            Self::Blacksmith => "blacksmith",
            Self::Carpenter => "carpenter",
            Self::Miner => "miner",
            Self::Merchant => "merchant",
            Self::Hunter => "hunter",
            Self::Healer => "healer",
            Self::Scholar => "scholar",
            Self::Cook => "cook",
            Self::Fisherman => "fisherman",
            Self::Lumberjack => "lumberjack",
            Self::Mason => "mason",
            Self::Brewer => "brewer",
        }
    }

    /// Bonus de capacité d'inventaire en kg pour cette profession
    pub fn inventory_capacity_bonus(&self) -> i32 {
        match self {
            Self::Unknown => 0,
            Self::Baker => 5,
            Self::Farmer => 10,
            Self::Warrior => 15,
            Self::Blacksmith => 20,
            Self::Carpenter => 15,
            Self::Miner => 25,
            Self::Merchant => 30,
            Self::Hunter => 12,
            Self::Healer => 8,
            Self::Scholar => 5,
            Self::Cook => 10,
            Self::Fisherman => 15,
            Self::Lumberjack => 20,
            Self::Mason => 15,
            Self::Brewer => 12,
        }
    }

    pub fn iter() -> impl Iterator<Item = ProfessionEnum> {
        [
            ProfessionEnum::Unknown,
            ProfessionEnum::Baker,
            ProfessionEnum::Farmer,
            ProfessionEnum::Warrior,
            ProfessionEnum::Blacksmith,
            ProfessionEnum::Carpenter,
            ProfessionEnum::Miner,
            ProfessionEnum::Merchant,
            ProfessionEnum::Hunter,
            ProfessionEnum::Healer,
            ProfessionEnum::Scholar,
            ProfessionEnum::Cook,
            ProfessionEnum::Fisherman,
            ProfessionEnum::Lumberjack,
            ProfessionEnum::Mason,
            ProfessionEnum::Brewer,
        ]
        .into_iter()
    }
}

// ============ SKILLS ============
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum SkillEnum {
    // Force-based
    MeleeAttack = 1,
    Carrying = 2,
    Mining = 3,
    Lumberjacking = 4,
    Blacksmithing = 5,

    // Agility-based
    RangedAttack = 10,
    Dodging = 11,
    Stealth = 12,
    Fishing = 13,
    Hunting = 14,

    // Constitution-based
    Endurance = 20,
    DiseaseResistance = 21,
    PoisonResistance = 22,

    // Intelligence-based
    Crafting = 30,
    Engineering = 31,
    Alchemy = 32,
    Research = 33,

    // Wisdom-based
    Farming = 40,
    Cooking = 41,
    Baking = 42,
    Healing = 43,
    AnimalHandling = 44,
    Brewing = 45,

    // Charisma-based
    Trading = 50,
    Leadership = 51,
    Persuasion = 52,
    Negotiation = 53,
}

impl SkillEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::MeleeAttack),
            2 => Some(Self::Carrying),
            3 => Some(Self::Mining),
            4 => Some(Self::Lumberjacking),
            5 => Some(Self::Blacksmithing),
            10 => Some(Self::RangedAttack),
            11 => Some(Self::Dodging),
            12 => Some(Self::Stealth),
            13 => Some(Self::Fishing),
            14 => Some(Self::Hunting),
            20 => Some(Self::Endurance),
            21 => Some(Self::DiseaseResistance),
            22 => Some(Self::PoisonResistance),
            30 => Some(Self::Crafting),
            31 => Some(Self::Engineering),
            32 => Some(Self::Alchemy),
            33 => Some(Self::Research),
            40 => Some(Self::Farming),
            41 => Some(Self::Cooking),
            42 => Some(Self::Baking),
            43 => Some(Self::Healing),
            44 => Some(Self::AnimalHandling),
            45 => Some(Self::Brewing),
            50 => Some(Self::Trading),
            51 => Some(Self::Leadership),
            52 => Some(Self::Persuasion),
            53 => Some(Self::Negotiation),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::MeleeAttack => "Melee Attack",
            Self::Carrying => "Carrying",
            Self::Mining => "Mining",
            Self::Lumberjacking => "Lumberjacking",
            Self::Blacksmithing => "Blacksmithing",
            Self::RangedAttack => "Ranged Attack",
            Self::Dodging => "Dodging",
            Self::Stealth => "Stealth",
            Self::Fishing => "Fishing",
            Self::Hunting => "Hunting",
            Self::Endurance => "Endurance",
            Self::DiseaseResistance => "Disease Resistance",
            Self::PoisonResistance => "Poison Resistance",
            Self::Crafting => "Crafting",
            Self::Engineering => "Engineering",
            Self::Alchemy => "Alchemy",
            Self::Research => "Research",
            Self::Farming => "Farming",
            Self::Cooking => "Cooking",
            Self::Baking => "Baking",
            Self::Healing => "Healing",
            Self::AnimalHandling => "Animal Handling",
            Self::Brewing => "Brewing",
            Self::Trading => "Trading",
            Self::Leadership => "Leadership",
            Self::Persuasion => "Persuasion",
            Self::Negotiation => "Negotiation",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::MeleeAttack => "melee_attack",
            Self::Carrying => "carrying",
            Self::Mining => "mining",
            Self::Lumberjacking => "lumberjacking",
            Self::Blacksmithing => "blacksmithing",
            Self::RangedAttack => "ranged_attack",
            Self::Dodging => "dodging",
            Self::Stealth => "stealth",
            Self::Fishing => "fishing",
            Self::Hunting => "hunting",
            Self::Endurance => "endurance",
            Self::DiseaseResistance => "disease_resistance",
            Self::PoisonResistance => "poison_resistance",
            Self::Crafting => "crafting",
            Self::Engineering => "engineering",
            Self::Alchemy => "alchemy",
            Self::Research => "research",
            Self::Farming => "farming",
            Self::Cooking => "cooking",
            Self::Baking => "baking",
            Self::Healing => "healing",
            Self::AnimalHandling => "animal_handling",
            Self::Brewing => "brewing",
            Self::Trading => "trading",
            Self::Leadership => "leadership",
            Self::Persuasion => "persuasion",
            Self::Negotiation => "negotiation",
        }
    }

    /// Retourne la statistique principale qui influence ce skill
    pub fn primary_stat(&self) -> PrimaryStat {
        match self {
            Self::MeleeAttack | Self::Carrying | Self::Mining | Self::Lumberjacking | Self::Blacksmithing => {
                PrimaryStat::Strength
            }
            Self::RangedAttack | Self::Dodging | Self::Stealth | Self::Fishing | Self::Hunting => {
                PrimaryStat::Agility
            }
            Self::Endurance | Self::DiseaseResistance | Self::PoisonResistance => {
                PrimaryStat::Constitution
            }
            Self::Crafting | Self::Engineering | Self::Alchemy | Self::Research => {
                PrimaryStat::Intelligence
            }
            Self::Farming | Self::Cooking | Self::Baking | Self::Healing | Self::AnimalHandling | Self::Brewing => {
                PrimaryStat::Wisdom
            }
            Self::Trading | Self::Leadership | Self::Persuasion | Self::Negotiation => {
                PrimaryStat::Charisma
            }
        }
    }

    pub fn iter() -> impl Iterator<Item = SkillEnum> {
        [
            SkillEnum::MeleeAttack,
            SkillEnum::Carrying,
            SkillEnum::Mining,
            SkillEnum::Lumberjacking,
            SkillEnum::Blacksmithing,
            SkillEnum::RangedAttack,
            SkillEnum::Dodging,
            SkillEnum::Stealth,
            SkillEnum::Fishing,
            SkillEnum::Hunting,
            SkillEnum::Endurance,
            SkillEnum::DiseaseResistance,
            SkillEnum::PoisonResistance,
            SkillEnum::Crafting,
            SkillEnum::Engineering,
            SkillEnum::Alchemy,
            SkillEnum::Research,
            SkillEnum::Farming,
            SkillEnum::Cooking,
            SkillEnum::Baking,
            SkillEnum::Healing,
            SkillEnum::AnimalHandling,
            SkillEnum::Brewing,
            SkillEnum::Trading,
            SkillEnum::Leadership,
            SkillEnum::Persuasion,
            SkillEnum::Negotiation,
        ]
        .into_iter()
    }
}

// ============ PRIMARY STATS ============
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum PrimaryStat {
    Strength,
    Agility,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl PrimaryStat {
    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Agility => "Agility",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Strength => "strength",
            Self::Agility => "agility",
            Self::Constitution => "constitution",
            Self::Intelligence => "intelligence",
            Self::Wisdom => "wisdom",
            Self::Charisma => "charisma",
        }
    }
}

// ============ ITEM TYPES ============
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum ItemTypeEnum {
    Unknown = 0,
    Resource = 1,
    Consumable = 2,
    Equipment = 3,
    Tool = 4,
    Weapon = 5,
    Armor = 6,
    Accessory = 7,
}

impl ItemTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Resource),
            2 => Some(Self::Consumable),
            3 => Some(Self::Equipment),
            4 => Some(Self::Tool),
            5 => Some(Self::Weapon),
            6 => Some(Self::Armor),
            7 => Some(Self::Accessory),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Resource => "Resource",
            Self::Consumable => "Consumable",
            Self::Equipment => "Equipment",
            Self::Tool => "Tool",
            Self::Weapon => "Weapon",
            Self::Armor => "Armor",
            Self::Accessory => "Accessory",
        }
    }
}

// ============ EQUIPMENT SLOTS ============
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum EquipmentSlotEnum {
    Unknown = 0,
    Head = 1,
    Chest = 2,
    Legs = 3,
    Feet = 4,
    Hands = 5,
    MainHand = 6,
    OffHand = 7,
    Back = 8,
    Neck = 9,
    Ring1 = 10,
    Ring2 = 11,
}

impl EquipmentSlotEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Head),
            2 => Some(Self::Chest),
            3 => Some(Self::Legs),
            4 => Some(Self::Feet),
            5 => Some(Self::Hands),
            6 => Some(Self::MainHand),
            7 => Some(Self::OffHand),
            8 => Some(Self::Back),
            9 => Some(Self::Neck),
            10 => Some(Self::Ring1),
            11 => Some(Self::Ring2),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Head => "Head",
            Self::Chest => "Chest",
            Self::Legs => "Legs",
            Self::Feet => "Feet",
            Self::Hands => "Hands",
            Self::MainHand => "Main Hand",
            Self::OffHand => "Off Hand",
            Self::Back => "Back",
            Self::Neck => "Neck",
            Self::Ring1 => "Ring 1",
            Self::Ring2 => "Ring 2",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Head => "head",
            Self::Chest => "chest",
            Self::Legs => "legs",
            Self::Feet => "feet",
            Self::Hands => "hands",
            Self::MainHand => "main_hand",
            Self::OffHand => "off_hand",
            Self::Back => "back",
            Self::Neck => "neck",
            Self::Ring1 => "ring_1",
            Self::Ring2 => "ring_2",
        }
    }

    pub fn iter() -> impl Iterator<Item = EquipmentSlotEnum> {
        [
            EquipmentSlotEnum::Unknown,
            EquipmentSlotEnum::Head,
            EquipmentSlotEnum::Chest,
            EquipmentSlotEnum::Legs,
            EquipmentSlotEnum::Feet,
            EquipmentSlotEnum::Hands,
            EquipmentSlotEnum::MainHand,
            EquipmentSlotEnum::OffHand,
            EquipmentSlotEnum::Back,
            EquipmentSlotEnum::Neck,
            EquipmentSlotEnum::Ring1,
            EquipmentSlotEnum::Ring2,
        ]
        .into_iter()
    }
}
