use bincode::{Decode, Encode};

// ============================================================================
// ORGANIZATION TYPE ENUM
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
#[repr(i16)]
pub enum OrganizationType {
    // Territorial (1-20)
    Hamlet = 1,
    Village = 2,
    Town = 3,
    City = 4,
    Barony = 5,
    County = 6,
    Duchy = 7,
    Kingdom = 8,
    Empire = 9,

    // Religious (20-39)
    Chapel = 20,
    Church = 21,
    Abbey = 22,
    Diocese = 23,
    Archdiocese = 24,
    Temple = 25,
    Monastery = 26,

    // Commercial (40-59)
    Shop = 40,
    Workshop = 41,
    TradingPost = 42,
    Market = 43,
    MerchantGuild = 44,
    TradingCompany = 45,
    Bank = 46,

    // Social/Military (60-79)
    Militia = 60,
    MercenaryBand = 61,
    KnightOrder = 62,
    CraftersGuild = 63,
    ScholarsGuild = 64,
    ThievesGuild = 65,
    Army = 66,

    Unknown = 0,
}

impl OrganizationType {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Self {
        match id {
            1 => Self::Hamlet,
            2 => Self::Village,
            3 => Self::Town,
            4 => Self::City,
            5 => Self::Barony,
            6 => Self::County,
            7 => Self::Duchy,
            8 => Self::Kingdom,
            9 => Self::Empire,

            20 => Self::Chapel,
            21 => Self::Church,
            22 => Self::Abbey,
            23 => Self::Diocese,
            24 => Self::Archdiocese,
            25 => Self::Temple,
            26 => Self::Monastery,

            40 => Self::Shop,
            41 => Self::Workshop,
            42 => Self::TradingPost,
            43 => Self::Market,
            44 => Self::MerchantGuild,
            45 => Self::TradingCompany,
            46 => Self::Bank,

            60 => Self::Militia,
            61 => Self::MercenaryBand,
            62 => Self::KnightOrder,
            63 => Self::CraftersGuild,
            64 => Self::ScholarsGuild,
            65 => Self::ThievesGuild,
            66 => Self::Army,

            _ => Self::Unknown,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Hamlet => "Hamlet".to_string(),
            Self::Village => "Village".to_string(),
            Self::Town => "Town".to_string(),
            Self::City => "City".to_string(),
            Self::Barony => "Barony".to_string(),
            Self::County => "County".to_string(),
            Self::Duchy => "Duchy".to_string(),
            Self::Kingdom => "Kingdom".to_string(),
            Self::Empire => "Empire".to_string(),

            Self::Chapel => "Chapel".to_string(),
            Self::Church => "Church".to_string(),
            Self::Abbey => "Abbey".to_string(),
            Self::Diocese => "Diocese".to_string(),
            Self::Archdiocese => "Archdiocese".to_string(),
            Self::Temple => "Temple".to_string(),
            Self::Monastery => "Monastery".to_string(),

            Self::Shop => "Shop".to_string(),
            Self::Workshop => "Workshop".to_string(),
            Self::TradingPost => "Trading Post".to_string(),
            Self::Market => "Market".to_string(),
            Self::MerchantGuild => "Merchant Guild".to_string(),
            Self::TradingCompany => "Trading Company".to_string(),
            Self::Bank => "Bank".to_string(),

            Self::Militia => "Militia".to_string(),
            Self::MercenaryBand => "Mercenary Band".to_string(),
            Self::KnightOrder => "Knight Order".to_string(),
            Self::CraftersGuild => "Crafters Guild".to_string(),
            Self::ScholarsGuild => "Scholars Guild".to_string(),
            Self::ThievesGuild => "Thieves Guild".to_string(),
            Self::Army => "Army".to_string(),

            Self::Unknown => "Unknown".to_string(),
        }
    }

    pub fn category(&self) -> OrganizationCategory {
        match self {
            Self::Hamlet | Self::Village | Self::Town | Self::City |
            Self::Barony | Self::County | Self::Duchy | Self::Kingdom | Self::Empire => {
                OrganizationCategory::Territorial
            }
            Self::Chapel | Self::Church | Self::Abbey | Self::Diocese |
            Self::Archdiocese | Self::Temple | Self::Monastery => {
                OrganizationCategory::Religious
            }
            Self::Shop | Self::Workshop | Self::TradingPost | Self::Market |
            Self::MerchantGuild | Self::TradingCompany | Self::Bank => {
                OrganizationCategory::Commercial
            }
            Self::Militia | Self::MercenaryBand | Self::KnightOrder |
            Self::CraftersGuild | Self::ScholarsGuild | Self::ThievesGuild | Self::Army => {
                OrganizationCategory::Social
            }
            Self::Unknown => OrganizationCategory::Unknown,
        }
    }
}

// ============================================================================
// ORGANIZATION CATEGORY
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum OrganizationCategory {
    Territorial,
    Religious,
    Commercial,
    Social,
    Unknown,
}

// ============================================================================
// ROLE TYPE ENUM
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
#[repr(i16)]
pub enum RoleType {
    // Territorial leaders (1-19)
    Emperor = 1,
    King = 2,
    Duke = 3,
    Count = 4,
    Baron = 5,
    Mayor = 6,
    VillageElder = 7,
    Headman = 8,

    // Territorial officers (20-39)
    Chancellor = 20,
    Marshal = 21,
    Steward = 22,
    Treasurer = 23,
    DeputyMayor = 24,
    Sheriff = 25,
    TaxCollector = 26,

    // Religious leaders (40-59)
    Pope = 40,
    Archbishop = 41,
    Bishop = 42,
    Abbot = 43,
    Priest = 44,
    Cardinal = 45,
    Prior = 46,
    Chaplain = 47,

    // Commercial leaders (60-79)
    GuildMaster = 60,
    TradeDirector = 61,
    Shopkeeper = 62,
    WorkshopMaster = 63,
    TradeTreasurer = 64,
    MarketMaster = 65,
    Banker = 66,

    // Military/Social leaders (80-99)
    GrandMaster = 80,
    Commander = 81,
    Captain = 82,
    Lieutenant = 83,
    Sergeant = 84,
    MasterCraftsman = 85,
    ScholarMaster = 86,
    GuildmasterThief = 87,

    Unknown = 0,
}

impl RoleType {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Self {
        match id {
            1 => Self::Emperor,
            2 => Self::King,
            3 => Self::Duke,
            4 => Self::Count,
            5 => Self::Baron,
            6 => Self::Mayor,
            7 => Self::VillageElder,
            8 => Self::Headman,

            20 => Self::Chancellor,
            21 => Self::Marshal,
            22 => Self::Steward,
            23 => Self::Treasurer,
            24 => Self::DeputyMayor,
            25 => Self::Sheriff,
            26 => Self::TaxCollector,

            40 => Self::Pope,
            41 => Self::Archbishop,
            42 => Self::Bishop,
            43 => Self::Abbot,
            44 => Self::Priest,
            45 => Self::Cardinal,
            46 => Self::Prior,
            47 => Self::Chaplain,

            60 => Self::GuildMaster,
            61 => Self::TradeDirector,
            62 => Self::Shopkeeper,
            63 => Self::WorkshopMaster,
            64 => Self::TradeTreasurer,
            65 => Self::MarketMaster,
            66 => Self::Banker,

            80 => Self::GrandMaster,
            81 => Self::Commander,
            82 => Self::Captain,
            83 => Self::Lieutenant,
            84 => Self::Sergeant,
            85 => Self::MasterCraftsman,
            86 => Self::ScholarMaster,
            87 => Self::GuildmasterThief,

            _ => Self::Unknown,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Emperor => "Emperor".to_string(),
            Self::King => "King".to_string(),
            Self::Duke => "Duke".to_string(),
            Self::Count => "Count".to_string(),
            Self::Baron => "Baron".to_string(),
            Self::Mayor => "Mayor".to_string(),
            Self::VillageElder => "Village Elder".to_string(),
            Self::Headman => "Headman".to_string(),

            Self::Chancellor => "Chancellor".to_string(),
            Self::Marshal => "Marshal".to_string(),
            Self::Steward => "Steward".to_string(),
            Self::Treasurer => "Treasurer".to_string(),
            Self::DeputyMayor => "Deputy Mayor".to_string(),
            Self::Sheriff => "Sheriff".to_string(),
            Self::TaxCollector => "Tax Collector".to_string(),

            Self::Pope => "Pope".to_string(),
            Self::Archbishop => "Archbishop".to_string(),
            Self::Bishop => "Bishop".to_string(),
            Self::Abbot => "Abbot".to_string(),
            Self::Priest => "Priest".to_string(),
            Self::Cardinal => "Cardinal".to_string(),
            Self::Prior => "Prior".to_string(),
            Self::Chaplain => "Chaplain".to_string(),

            Self::GuildMaster => "Guild Master".to_string(),
            Self::TradeDirector => "Trade Director".to_string(),
            Self::Shopkeeper => "Shopkeeper".to_string(),
            Self::WorkshopMaster => "Workshop Master".to_string(),
            Self::TradeTreasurer => "Trade Treasurer".to_string(),
            Self::MarketMaster => "Market Master".to_string(),
            Self::Banker => "Banker".to_string(),

            Self::GrandMaster => "Grand Master".to_string(),
            Self::Commander => "Commander".to_string(),
            Self::Captain => "Captain".to_string(),
            Self::Lieutenant => "Lieutenant".to_string(),
            Self::Sergeant => "Sergeant".to_string(),
            Self::MasterCraftsman => "Master Craftsman".to_string(),
            Self::ScholarMaster => "Scholar Master".to_string(),
            Self::GuildmasterThief => "Guildmaster Thief".to_string(),

            Self::Unknown => "Unknown".to_string(),
        }
    }

    pub fn authority_level(&self) -> i16 {
        match self {
            Self::Emperor | Self::Pope => 1,
            Self::King => 5,
            Self::Archbishop => 8,
            Self::Duke => 10,
            Self::Cardinal | Self::GrandMaster => 10,
            Self::Chancellor => 12,
            Self::Marshal => 13,
            Self::Steward => 14,
            Self::Count => 15,
            Self::GuildMaster => 15,
            Self::Treasurer => 16,
            Self::TradeDirector => 17,
            Self::Bishop => 18,
            Self::Commander => 19,
            Self::Baron => 20,
            Self::Banker => 20,
            Self::Abbot => 22,
            Self::GuildmasterThief => 23,
            Self::Prior => 24,
            Self::TradeTreasurer => 25,
            Self::Mayor => 25,
            Self::DeputyMayor => 26,
            Self::MasterCraftsman => 27,
            Self::Sheriff => 28,
            Self::ScholarMaster => 29,
            Self::MarketMaster => 30,
            Self::Priest => 32,
            Self::Captain => 33,
            Self::WorkshopMaster => 35,
            Self::VillageElder => 30,
            Self::Headman => 35,
            Self::Chaplain => 38,
            Self::TaxCollector => 40,
            Self::Lieutenant => 42,
            Self::Shopkeeper => 45,
            Self::Sergeant => 50,
            Self::Unknown => 100,
        }
    }
}

// ============================================================================
// MEMBERSHIP STATUS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum MembershipStatus {
    Active,
    Suspended,
    Honorary,
}

impl MembershipStatus {
    pub fn to_string(&self) -> String {
        match self {
            Self::Active => "active".to_string(),
            Self::Suspended => "suspended".to_string(),
            Self::Honorary => "honorary".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "suspended" => Self::Suspended,
            "honorary" => Self::Honorary,
            _ => Self::Active,
        }
    }
}

// ============================================================================
// DIPLOMATIC RELATION TYPE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum DiplomaticRelationType {
    Allied,
    Neutral,
    Hostile,
    AtWar,
    TradeAgreement,
    NonAggression,
}

impl DiplomaticRelationType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Allied => "allied".to_string(),
            Self::Neutral => "neutral".to_string(),
            Self::Hostile => "hostile".to_string(),
            Self::AtWar => "at_war".to_string(),
            Self::TradeAgreement => "trade_agreement".to_string(),
            Self::NonAggression => "non_aggression".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "allied" => Self::Allied,
            "neutral" => Self::Neutral,
            "hostile" => Self::Hostile,
            "at_war" => Self::AtWar,
            "trade_agreement" => Self::TradeAgreement,
            "non_aggression" => Self::NonAggression,
            _ => Self::Neutral,
        }
    }
}
