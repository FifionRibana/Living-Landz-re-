use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::BiomeTypeEnum;

// ============ BUILDING CATEGORIES ============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BuildingCategoryEnum {
    Unknown = 0,
    Natural = 1,
    SemiSpecialized = 2,
    Metal = 3,
    Earth = 4,
    Food = 5,
    Wood = 6,
    Textile = 7,
    FineArtisan = 8,
    AnimalBreeding = 9,
    ServiceCombined = 10,
    ServiceDedicated = 11,
    Defense = 12,
    Infrastructure = 13,
    Residential = 14,
}

impl BuildingCategoryEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Natural),
            2 => Some(Self::SemiSpecialized),
            3 => Some(Self::Metal),
            4 => Some(Self::Earth),
            5 => Some(Self::Food),
            6 => Some(Self::Wood),
            7 => Some(Self::Textile),
            8 => Some(Self::FineArtisan),
            9 => Some(Self::AnimalBreeding),
            10 => Some(Self::ServiceCombined),
            11 => Some(Self::ServiceDedicated),
            12 => Some(Self::Defense),
            13 => Some(Self::Infrastructure),
            14 => Some(Self::Residential),
            _ => None,
        }
    }
}

// ============ BUILDING SPECIFIC TYPE (for BuildingData dispatch) ============

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum BuildingSpecificTypeEnum {
    Unknown = 0,
    Tree = 1,
    ManufacturingWorkshop = 2,
    Agriculture = 3,
    AnimalBreeding = 4,
    Entertainment = 5,
    Cult = 6,
    Commerce = 7,
}

impl BuildingSpecificTypeEnum {
    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Tree),
            2 => Some(Self::ManufacturingWorkshop),
            3 => Some(Self::Agriculture),
            4 => Some(Self::AnimalBreeding),
            5 => Some(Self::Entertainment),
            6 => Some(Self::Cult),
            7 => Some(Self::Commerce),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Tree => "Tree",
            Self::ManufacturingWorkshop => "ManufacturingWorkshop",
            Self::Agriculture => "Agriculture",
            Self::AnimalBreeding => "AnimalBreeding",
            Self::Entertainment => "Entertainment",
            Self::Cult => "Cult",
            Self::Commerce => "Commerce",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Tree => "tree",
            Self::ManufacturingWorkshop => "manufacturing_workshop",
            Self::Agriculture => "agriculture",
            Self::AnimalBreeding => "animal_breeding",
            Self::Entertainment => "entertainment",
            Self::Cult => "cult",
            Self::Commerce => "commerce",
        }
    }

    pub fn iter() -> impl Iterator<Item = BuildingSpecificTypeEnum> {
        [
            Self::Unknown, Self::Tree, Self::ManufacturingWorkshop,
            Self::Agriculture, Self::AnimalBreeding, Self::Entertainment,
            Self::Cult, Self::Commerce,
        ]
        .into_iter()
    }
}

// ============ TREE TYPE ============

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum TreeTypeEnum {
    Cedar = 1,
    Larch = 2,
    Oak = 3,
}

impl TreeTypeEnum {
    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Cedar => "Cedar",
            Self::Larch => "Larch",
            Self::Oak => "Oak",
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Cedar => "cedar",
            Self::Larch => "larch",
            Self::Oak => "oak",
        }
    }

    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::Cedar),
            2 => Some(Self::Larch),
            3 => Some(Self::Oak),
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "cedar" => Some(Self::Cedar),
            "larch" => Some(Self::Larch),
            "oak" => Some(Self::Oak),
            _ => None,
        }
    }

    pub fn iter() -> impl Iterator<Item = TreeTypeEnum> {
        [Self::Cedar, Self::Larch, Self::Oak].into_iter()
    }

    pub fn from_biome(biome: BiomeTypeEnum) -> Vec<TreeTypeEnum> {
        match biome {
            BiomeTypeEnum::Grassland
            | BiomeTypeEnum::TropicalSeasonalForest
            | BiomeTypeEnum::TropicalRainForest
            | BiomeTypeEnum::TropicalDeciduousForest
            | BiomeTypeEnum::TemperateRainForest
            | BiomeTypeEnum::Wetland => vec![Self::Cedar, Self::Larch, Self::Oak],
            _ => vec![],
        }
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        match self {
            Self::Cedar => BuildingTypeEnum::Cedar,
            Self::Larch => BuildingTypeEnum::Larch,
            Self::Oak => BuildingTypeEnum::Oak,
        }
    }

    pub fn from_building_type(building_type: BuildingTypeEnum) -> Option<Self> {
        match building_type {
            BuildingTypeEnum::Cedar => Some(Self::Cedar),
            BuildingTypeEnum::Larch => Some(Self::Larch),
            BuildingTypeEnum::Oak => Some(Self::Oak),
            _ => None,
        }
    }
}

// ============ MAIN BUILDING TYPE ENUM ============
// IDs follow the reference document spacing: 10 between types, 100 between categories.

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub enum BuildingTypeEnum {
    // 100-199 Semi-specialized workshops (hamlet)
    AtelierDuFeu = 100,
    AtelierDuBois = 110,
    AtelierTextile = 120,
    // 200-299 Metal chain
    Mine = 200,
    Carriere = 210,
    Charbonniere = 220,
    Fonderie = 230,
    Forge = 240,
    // 300-399 Earth chain
    SiteExtraction = 300,
    Briqueterie = 310,
    AtelierPotier = 320,
    Verrerie = 330,
    // 400-499 Food chain
    Ferme = 400,
    Moulin = 410,
    Cuisine = 420,
    Abattoir = 430,
    Brasserie = 440,
    QuaiPeche = 450,
    // 500-599 Wood chain
    AtelierCharpentier = 500,
    // 600-699 Textile chain
    Tannerie = 600,
    AtelierTissage = 610,
    // 700-799 Fine artisan
    AtelierArtisanatFin = 700,
    // 800-899 Animal breeding
    Poulailler = 800,
    Etable = 810,
    Bergerie = 820,
    Porcherie = 830,
    Ecurie = 840,
    Rucher = 850,
    // 900-999 Combined services (village)
    TempleHospice = 900,
    TaverneComptoir = 910,
    EntrepotComptoir = 920,
    // 1000-1099 Dedicated services
    PlaceMarche = 1000,
    LieuDeCulte = 1010,
    Hospice = 1020,
    BatimentAdmin = 1030,
    Taverne = 1040,
    BainsPublics = 1050,
    Theatre = 1060,
    Observatoire = 1070,
    Universite = 1080,
    // 1100-1199 Defense
    Palissade = 1100,
    TourDeGuet = 1110,
    MurPierre = 1120,
    Murailles = 1130,
    DoubleEnceinte = 1140,
    // 1200-1299 Infrastructure
    Entrepot = 1200,
    GrenierSilo = 1210,
    Glaciere = 1220,
    Puits = 1230,
    PontBois = 1240,
    PontPierre = 1250,
    QuaiPort = 1260,
    Aqueducs = 1270,
    // 1300-1399 Residential
    Campement = 1300,
    HuttePalierI = 1310,
    ChaumierePalierI = 1311,
    GrandeChaumiere = 1312,
    MaisonSimple = 1320,
    Maison = 1321,
    GrandeMaison = 1322,
    MaisonDeBourg = 1330,
    MaisonBourgeoise = 1331,
    HotelParticulier = 1332,
    Immeuble = 1340,
    GrandImmeuble = 1341,
    ComplexeUrbain = 1342,
    // 2000+ Natural
    Oak = 2000,
    Cedar = 2001,
    Larch = 2002,
}

impl BuildingTypeEnum {
    pub fn to_specific_type(&self) -> BuildingSpecificTypeEnum {
        match self {
            // Workshops → ManufacturingWorkshop
            Self::AtelierDuFeu | Self::AtelierDuBois | Self::AtelierTextile
            | Self::Fonderie | Self::Forge | Self::Verrerie
            | Self::AtelierCharpentier | Self::AtelierPotier
            | Self::Briqueterie | Self::AtelierArtisanatFin
            | Self::Tannerie | Self::AtelierTissage => BuildingSpecificTypeEnum::ManufacturingWorkshop,
            // Extraction → ManufacturingWorkshop (production buildings)
            Self::Mine | Self::Carriere | Self::Charbonniere
            | Self::SiteExtraction => BuildingSpecificTypeEnum::ManufacturingWorkshop,
            // Agriculture
            Self::Ferme | Self::Moulin => BuildingSpecificTypeEnum::Agriculture,
            // Animal breeding
            Self::Poulailler | Self::Etable | Self::Bergerie
            | Self::Porcherie | Self::Ecurie | Self::Rucher => BuildingSpecificTypeEnum::AnimalBreeding,
            // Food / Commerce
            Self::Cuisine | Self::Abattoir | Self::Brasserie
            | Self::QuaiPeche | Self::PlaceMarche | Self::Glaciere
            | Self::EntrepotComptoir => BuildingSpecificTypeEnum::Commerce,
            // Entertainment
            Self::Theatre | Self::BainsPublics | Self::Taverne
            | Self::TaverneComptoir | Self::Observatoire => BuildingSpecificTypeEnum::Entertainment,
            // Cult / Education
            Self::LieuDeCulte | Self::TempleHospice | Self::Hospice
            | Self::Universite | Self::BatimentAdmin => BuildingSpecificTypeEnum::Cult,
            // Trees
            Self::Oak | Self::Cedar | Self::Larch => BuildingSpecificTypeEnum::Tree,
            // Everything else (defense, infrastructure, residential)
            _ => BuildingSpecificTypeEnum::Unknown,
        }
    }

    pub fn to_id(self) -> i16 {
        self as i16
    }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            100 => Some(Self::AtelierDuFeu),
            110 => Some(Self::AtelierDuBois),
            120 => Some(Self::AtelierTextile),
            200 => Some(Self::Mine),
            210 => Some(Self::Carriere),
            220 => Some(Self::Charbonniere),
            230 => Some(Self::Fonderie),
            240 => Some(Self::Forge),
            300 => Some(Self::SiteExtraction),
            310 => Some(Self::Briqueterie),
            320 => Some(Self::AtelierPotier),
            330 => Some(Self::Verrerie),
            400 => Some(Self::Ferme),
            410 => Some(Self::Moulin),
            420 => Some(Self::Cuisine),
            430 => Some(Self::Abattoir),
            440 => Some(Self::Brasserie),
            450 => Some(Self::QuaiPeche),
            500 => Some(Self::AtelierCharpentier),
            600 => Some(Self::Tannerie),
            610 => Some(Self::AtelierTissage),
            700 => Some(Self::AtelierArtisanatFin),
            800 => Some(Self::Poulailler),
            810 => Some(Self::Etable),
            820 => Some(Self::Bergerie),
            830 => Some(Self::Porcherie),
            840 => Some(Self::Ecurie),
            850 => Some(Self::Rucher),
            900 => Some(Self::TempleHospice),
            910 => Some(Self::TaverneComptoir),
            920 => Some(Self::EntrepotComptoir),
            1000 => Some(Self::PlaceMarche),
            1010 => Some(Self::LieuDeCulte),
            1020 => Some(Self::Hospice),
            1030 => Some(Self::BatimentAdmin),
            1040 => Some(Self::Taverne),
            1050 => Some(Self::BainsPublics),
            1060 => Some(Self::Theatre),
            1070 => Some(Self::Observatoire),
            1080 => Some(Self::Universite),
            1100 => Some(Self::Palissade),
            1110 => Some(Self::TourDeGuet),
            1120 => Some(Self::MurPierre),
            1130 => Some(Self::Murailles),
            1140 => Some(Self::DoubleEnceinte),
            1200 => Some(Self::Entrepot),
            1210 => Some(Self::GrenierSilo),
            1220 => Some(Self::Glaciere),
            1230 => Some(Self::Puits),
            1240 => Some(Self::PontBois),
            1250 => Some(Self::PontPierre),
            1260 => Some(Self::QuaiPort),
            1270 => Some(Self::Aqueducs),
            1300 => Some(Self::Campement),
            1310 => Some(Self::HuttePalierI),
            1311 => Some(Self::ChaumierePalierI),
            1312 => Some(Self::GrandeChaumiere),
            1320 => Some(Self::MaisonSimple),
            1321 => Some(Self::Maison),
            1322 => Some(Self::GrandeMaison),
            1330 => Some(Self::MaisonDeBourg),
            1331 => Some(Self::MaisonBourgeoise),
            1332 => Some(Self::HotelParticulier),
            1340 => Some(Self::Immeuble),
            1341 => Some(Self::GrandImmeuble),
            1342 => Some(Self::ComplexeUrbain),
            2000 => Some(Self::Oak),
            2001 => Some(Self::Cedar),
            2002 => Some(Self::Larch),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::AtelierDuFeu => "fire_workshop",
            Self::AtelierDuBois => "wood_workshop",
            Self::AtelierTextile => "textile_workshop",
            Self::Mine => "mine",
            Self::Carriere => "quarry",
            Self::Charbonniere => "charcoal_kiln",
            Self::Fonderie => "smelter",
            Self::Forge => "forge",
            Self::SiteExtraction => "extraction_site",
            Self::Briqueterie => "brickworks",
            Self::AtelierPotier => "potter_workshop",
            Self::Verrerie => "glassworks",
            Self::Ferme => "farm",
            Self::Moulin => "mill",
            Self::Cuisine => "kitchen",
            Self::Abattoir => "slaughterhouse",
            Self::Brasserie => "brewery",
            Self::QuaiPeche => "fishing_wharf",
            Self::AtelierCharpentier => "carpenter_workshop",
            Self::Tannerie => "tannery",
            Self::AtelierTissage => "weaving_workshop",
            Self::AtelierArtisanatFin => "fine_craft_workshop",
            Self::Poulailler => "chicken_coop",
            Self::Etable => "cowshed",
            Self::Bergerie => "sheepfold",
            Self::Porcherie => "pigsty",
            Self::Ecurie => "stable",
            Self::Rucher => "apiary",
            Self::TempleHospice => "temple_hospice",
            Self::TaverneComptoir => "tavern_counter",
            Self::EntrepotComptoir => "warehouse_counter",
            Self::PlaceMarche => "marketplace",
            Self::LieuDeCulte => "place_of_worship",
            Self::Hospice => "hospice",
            Self::BatimentAdmin => "admin_building",
            Self::Taverne => "tavern",
            Self::BainsPublics => "public_baths",
            Self::Theatre => "theater",
            Self::Observatoire => "observatory",
            Self::Universite => "university",
            Self::Palissade => "palisade",
            Self::TourDeGuet => "watchtower",
            Self::MurPierre => "stone_wall",
            Self::Murailles => "ramparts",
            Self::DoubleEnceinte => "double_wall",
            Self::Entrepot => "warehouse",
            Self::GrenierSilo => "granary_silo",
            Self::Glaciere => "ice_house",
            Self::Puits => "well",
            Self::PontBois => "wooden_bridge",
            Self::PontPierre => "stone_bridge",
            Self::QuaiPort => "port",
            Self::Aqueducs => "aqueducts",
            Self::Campement => "base_camp",
            Self::HuttePalierI => "hut_tier_i",
            Self::ChaumierePalierI => "cottage_tier_i",
            Self::GrandeChaumiere => "large_cottage",
            Self::MaisonSimple => "simple_house",
            Self::Maison => "house",
            Self::GrandeMaison => "large_house",
            Self::MaisonDeBourg => "town_house",
            Self::MaisonBourgeoise => "bourgeois_house",
            Self::HotelParticulier => "mansion",
            Self::Immeuble => "tenement",
            Self::GrandImmeuble => "large_tenement",
            Self::ComplexeUrbain => "urban_complex",
            Self::Cedar => "cedar",
            Self::Larch => "larch",
            Self::Oak => "oak",
        }
    }

    // TODO: Move building enum data to database
    /// Marginal housing provided by this building (aggregated population capacity).
    /// Values from reference document column "Log" at tier C1.
    /// Named unit placement uses SlotConfiguration, not this value.
    pub fn housing_capacity(&self) -> u32 {
        match self {
            // Semi-specialized workshops
            Self::AtelierDuFeu | Self::AtelierDuBois | Self::AtelierTextile => 2,
            // Metal chain
            Self::Mine | Self::Carriere | Self::Charbonniere => 0,
            Self::Fonderie | Self::Forge => 2,
            // Earth chain
            Self::SiteExtraction => 0,
            Self::Briqueterie | Self::AtelierPotier | Self::Verrerie => 2,
            // Food chain
            Self::Ferme => 2,
            Self::Moulin | Self::Cuisine | Self::Brasserie => 2,
            Self::Abattoir => 1,
            Self::QuaiPeche => 0,
            // Wood chain
            Self::AtelierCharpentier => 2,
            // Textile chain
            Self::Tannerie => 1,
            Self::AtelierTissage => 2,
            // Fine artisan
            Self::AtelierArtisanatFin => 2,
            // Animal breeding
            Self::Poulailler | Self::Porcherie | Self::Rucher => 0,
            Self::Etable | Self::Bergerie => 1,
            Self::Ecurie => 2,
            // Combined services
            Self::TempleHospice => 2,
            Self::TaverneComptoir => 3,
            Self::EntrepotComptoir => 1,
            // Dedicated services
            Self::PlaceMarche => 0,
            Self::LieuDeCulte | Self::Hospice | Self::BainsPublics
            | Self::Theatre | Self::Observatoire => 2,
            Self::BatimentAdmin => 1,
            Self::Taverne => 4,
            Self::Universite => 3,
            // Defense
            Self::TourDeGuet => 1,
            Self::Palissade | Self::MurPierre | Self::Murailles
            | Self::DoubleEnceinte => 0,
            // Infrastructure
            Self::Entrepot | Self::GrenierSilo | Self::Glaciere
            | Self::Puits | Self::PontBois | Self::PontPierre
            | Self::QuaiPort | Self::Aqueducs => 0,
            // Residential
            Self::Campement => 4,
            Self::HuttePalierI => 10,
            Self::ChaumierePalierI => 18,
            Self::GrandeChaumiere => 28,
            Self::MaisonSimple => 42,
            Self::Maison => 62,
            Self::GrandeMaison => 88,
            Self::MaisonDeBourg => 125,
            Self::MaisonBourgeoise => 175,
            Self::HotelParticulier => 240,
            Self::Immeuble => 330,
            Self::GrandImmeuble => 435,
            Self::ComplexeUrbain => 555,
            // Nature
            Self::Oak | Self::Cedar | Self::Larch => 0,
        }
    }

    /// Number of concurrent production actions this building supports (tier C1).
    pub fn production_lines(&self) -> u32 {
        match self {
            Self::AtelierDuFeu | Self::AtelierDuBois | Self::AtelierTextile => 1,
            Self::Mine | Self::Carriere | Self::SiteExtraction => 1,
            Self::Charbonniere | Self::Fonderie | Self::Forge => 1,
            Self::Briqueterie | Self::AtelierPotier | Self::Verrerie => 1,
            Self::Ferme => 1,
            Self::Moulin | Self::Cuisine => 1,
            Self::Abattoir | Self::Brasserie | Self::QuaiPeche => 1,
            Self::AtelierCharpentier => 1,
            Self::Tannerie | Self::AtelierTissage => 1,
            Self::AtelierArtisanatFin => 1,
            Self::Poulailler | Self::Etable | Self::Bergerie
            | Self::Porcherie | Self::Ecurie | Self::Rucher => 1,
            Self::PlaceMarche => 1,
            Self::LieuDeCulte | Self::TempleHospice => 1,
            Self::TaverneComptoir | Self::Taverne => 1,
            Self::Oak | Self::Cedar | Self::Larch => 1,
            _ => 0,
        }
    }

    // TODO: Define trainable unit types per building — base unit should be Settler
    /// Relevant professions for this building (for immigrant spawning).
    pub fn relevant_professions(&self) -> &'static [crate::ProfessionEnum] {
        use crate::ProfessionEnum::*;
        match self {
            Self::AtelierDuFeu | Self::Fonderie | Self::Forge => &[Blacksmith],
            Self::AtelierDuBois | Self::AtelierCharpentier => &[Carpenter],
            Self::Verrerie | Self::AtelierPotier | Self::Briqueterie => &[Mason],
            Self::Ferme | Self::Moulin => &[Farmer],
            Self::Poulailler | Self::Etable | Self::Bergerie
            | Self::Porcherie | Self::Ecurie => &[Farmer],
            Self::Cuisine => &[Baker],
            Self::Brasserie => &[Brewer],
            Self::PlaceMarche => &[Merchant],
            Self::LieuDeCulte | Self::TempleHospice => &[Scholar],
            _ => &[Farmer],
        }
    }

    pub fn category(&self) -> BuildingCategoryEnum {
        match self {
            Self::AtelierDuFeu | Self::AtelierDuBois | Self::AtelierTextile => BuildingCategoryEnum::SemiSpecialized,
            Self::Mine | Self::Carriere | Self::Charbonniere
            | Self::Fonderie | Self::Forge => BuildingCategoryEnum::Metal,
            Self::SiteExtraction | Self::Briqueterie | Self::AtelierPotier
            | Self::Verrerie => BuildingCategoryEnum::Earth,
            Self::Ferme | Self::Moulin | Self::Cuisine | Self::Abattoir
            | Self::Brasserie | Self::QuaiPeche => BuildingCategoryEnum::Food,
            Self::AtelierCharpentier => BuildingCategoryEnum::Wood,
            Self::Tannerie | Self::AtelierTissage => BuildingCategoryEnum::Textile,
            Self::AtelierArtisanatFin => BuildingCategoryEnum::FineArtisan,
            Self::Poulailler | Self::Etable | Self::Bergerie
            | Self::Porcherie | Self::Ecurie | Self::Rucher => BuildingCategoryEnum::AnimalBreeding,
            Self::TempleHospice | Self::TaverneComptoir
            | Self::EntrepotComptoir => BuildingCategoryEnum::ServiceCombined,
            Self::PlaceMarche | Self::LieuDeCulte | Self::Hospice
            | Self::BatimentAdmin | Self::Taverne | Self::BainsPublics
            | Self::Theatre | Self::Observatoire | Self::Universite => BuildingCategoryEnum::ServiceDedicated,
            Self::Palissade | Self::TourDeGuet | Self::MurPierre
            | Self::Murailles | Self::DoubleEnceinte => BuildingCategoryEnum::Defense,
            Self::Entrepot | Self::GrenierSilo | Self::Glaciere
            | Self::Puits | Self::PontBois | Self::PontPierre
            | Self::QuaiPort | Self::Aqueducs => BuildingCategoryEnum::Infrastructure,
            Self::Campement | Self::HuttePalierI | Self::ChaumierePalierI
            | Self::GrandeChaumiere | Self::MaisonSimple | Self::Maison
            | Self::GrandeMaison | Self::MaisonDeBourg | Self::MaisonBourgeoise
            | Self::HotelParticulier | Self::Immeuble | Self::GrandImmeuble
            | Self::ComplexeUrbain => BuildingCategoryEnum::Residential,
            Self::Oak | Self::Cedar | Self::Larch => BuildingCategoryEnum::Natural,
        }
    }

    pub fn is_tree(&self) -> bool {
        matches!(self, Self::Oak | Self::Cedar | Self::Larch)
    }

    pub fn is_natural(&self) -> bool {
        self.category() == BuildingCategoryEnum::Natural
    }

    pub fn iter() -> impl Iterator<Item = BuildingTypeEnum> {
        [
            Self::AtelierDuFeu, Self::AtelierDuBois, Self::AtelierTextile,
            Self::Mine, Self::Carriere, Self::Charbonniere, Self::Fonderie, Self::Forge,
            Self::SiteExtraction, Self::Briqueterie, Self::AtelierPotier, Self::Verrerie,
            Self::Ferme, Self::Moulin, Self::Cuisine, Self::Abattoir, Self::Brasserie, Self::QuaiPeche,
            Self::AtelierCharpentier,
            Self::Tannerie, Self::AtelierTissage,
            Self::AtelierArtisanatFin,
            Self::Poulailler, Self::Etable, Self::Bergerie, Self::Porcherie, Self::Ecurie, Self::Rucher,
            Self::TempleHospice, Self::TaverneComptoir, Self::EntrepotComptoir,
            Self::PlaceMarche, Self::LieuDeCulte, Self::Hospice, Self::BatimentAdmin,
            Self::Taverne, Self::BainsPublics, Self::Theatre, Self::Observatoire, Self::Universite,
            Self::Palissade, Self::TourDeGuet, Self::MurPierre, Self::Murailles, Self::DoubleEnceinte,
            Self::Entrepot, Self::GrenierSilo, Self::Glaciere, Self::Puits,
            Self::PontBois, Self::PontPierre, Self::QuaiPort, Self::Aqueducs,
            Self::Campement, Self::HuttePalierI, Self::ChaumierePalierI, Self::GrandeChaumiere,
            Self::MaisonSimple, Self::Maison, Self::GrandeMaison,
            Self::MaisonDeBourg, Self::MaisonBourgeoise, Self::HotelParticulier,
            Self::Immeuble, Self::GrandImmeuble, Self::ComplexeUrbain,
            Self::Oak, Self::Cedar, Self::Larch,
        ]
        .into_iter()
    }

    pub fn to_tree_type(&self) -> Option<TreeTypeEnum> {
        TreeTypeEnum::from_building_type(*self)
    }
}

// ============ SUB-TYPE ENUMS (for BuildingSpecific data dispatch) ============
// These map 1:1 with the old specific enums. They use the SAME IDs as BuildingTypeEnum
// so that to_building_type() is trivial.

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum ManufacturingWorkshopTypeEnum {
    AtelierDuFeu = 100,
    AtelierDuBois = 110,
    AtelierTextile = 120,
    Mine = 200,
    Carriere = 210,
    Charbonniere = 220,
    Fonderie = 230,
    Forge = 240,
    SiteExtraction = 300,
    Briqueterie = 310,
    AtelierPotier = 320,
    Verrerie = 330,
    AtelierCharpentier = 500,
    Tannerie = 600,
    AtelierTissage = 610,
    AtelierArtisanatFin = 700,
}

impl ManufacturingWorkshopTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }

    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            100 => Some(Self::AtelierDuFeu),
            110 => Some(Self::AtelierDuBois),
            120 => Some(Self::AtelierTextile),
            200 => Some(Self::Mine),
            210 => Some(Self::Carriere),
            220 => Some(Self::Charbonniere),
            230 => Some(Self::Fonderie),
            240 => Some(Self::Forge),
            300 => Some(Self::SiteExtraction),
            310 => Some(Self::Briqueterie),
            320 => Some(Self::AtelierPotier),
            330 => Some(Self::Verrerie),
            500 => Some(Self::AtelierCharpentier),
            600 => Some(Self::Tannerie),
            610 => Some(Self::AtelierTissage),
            700 => Some(Self::AtelierArtisanatFin),
            _ => None,
        }
    }

    pub fn to_name_lowercase(&self) -> &'static str {
        BuildingTypeEnum::from_id(self.to_id()).unwrap().to_name_lowercase()
    }

    pub fn to_building_type(&self) -> BuildingTypeEnum {
        BuildingTypeEnum::from_id(self.to_id()).unwrap()
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::AtelierDuFeu, Self::AtelierDuBois, Self::AtelierTextile,
            Self::Mine, Self::Carriere, Self::Charbonniere, Self::Fonderie, Self::Forge,
            Self::SiteExtraction, Self::Briqueterie, Self::AtelierPotier, Self::Verrerie,
            Self::AtelierCharpentier, Self::Tannerie, Self::AtelierTissage, Self::AtelierArtisanatFin,
        ].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum AgricultureTypeEnum {
    Ferme = 400,
    Moulin = 410,
}

impl AgricultureTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }
    pub fn from_id(id: i16) -> Option<Self> {
        match id { 400 => Some(Self::Ferme), 410 => Some(Self::Moulin), _ => None }
    }
    pub fn to_name_lowercase(&self) -> &'static str {
        BuildingTypeEnum::from_id(self.to_id()).unwrap().to_name_lowercase()
    }
    pub fn to_building_type(&self) -> BuildingTypeEnum {
        BuildingTypeEnum::from_id(self.to_id()).unwrap()
    }
    pub fn iter() -> impl Iterator<Item = Self> { [Self::Ferme, Self::Moulin].into_iter() }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum AnimalBreedingTypeEnum {
    Poulailler = 800,
    Etable = 810,
    Bergerie = 820,
    Porcherie = 830,
    Ecurie = 840,
    Rucher = 850,
}

impl AnimalBreedingTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            800 => Some(Self::Poulailler), 810 => Some(Self::Etable),
            820 => Some(Self::Bergerie), 830 => Some(Self::Porcherie),
            840 => Some(Self::Ecurie), 850 => Some(Self::Rucher),
            _ => None,
        }
    }
    pub fn to_name_lowercase(&self) -> &'static str {
        BuildingTypeEnum::from_id(self.to_id()).unwrap().to_name_lowercase()
    }
    pub fn to_building_type(&self) -> BuildingTypeEnum {
        BuildingTypeEnum::from_id(self.to_id()).unwrap()
    }
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Poulailler, Self::Etable, Self::Bergerie, Self::Porcherie, Self::Ecurie, Self::Rucher].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum EntertainmentTypeEnum {
    Theatre = 1060,
    TaverneComptoir = 910,
    Taverne = 1040,
    BainsPublics = 1050,
    Observatoire = 1070,
}

impl EntertainmentTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1060 => Some(Self::Theatre), 910 => Some(Self::TaverneComptoir),
            1040 => Some(Self::Taverne), 1050 => Some(Self::BainsPublics),
            1070 => Some(Self::Observatoire),
            _ => None,
        }
    }
    pub fn to_name_lowercase(&self) -> &'static str {
        BuildingTypeEnum::from_id(self.to_id()).unwrap().to_name_lowercase()
    }
    pub fn to_building_type(&self) -> BuildingTypeEnum {
        BuildingTypeEnum::from_id(self.to_id()).unwrap()
    }
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Theatre, Self::TaverneComptoir, Self::Taverne, Self::BainsPublics, Self::Observatoire].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum CultTypeEnum {
    LieuDeCulte = 1010,
    TempleHospice = 900,
    Hospice = 1020,
    Universite = 1080,
    BatimentAdmin = 1030,
}

impl CultTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1010 => Some(Self::LieuDeCulte), 900 => Some(Self::TempleHospice),
            1020 => Some(Self::Hospice), 1080 => Some(Self::Universite),
            1030 => Some(Self::BatimentAdmin),
            _ => None,
        }
    }
    pub fn to_name_lowercase(&self) -> &'static str {
        BuildingTypeEnum::from_id(self.to_id()).unwrap().to_name_lowercase()
    }
    pub fn to_building_type(&self) -> BuildingTypeEnum {
        BuildingTypeEnum::from_id(self.to_id()).unwrap()
    }
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::LieuDeCulte, Self::TempleHospice, Self::Hospice, Self::Universite, Self::BatimentAdmin].into_iter()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum CommerceTypeEnum {
    Cuisine = 420,
    Abattoir = 430,
    Brasserie = 440,
    QuaiPeche = 450,
    PlaceMarche = 1000,
    Glaciere = 1220,
    EntrepotComptoir = 920,
}

impl CommerceTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            420 => Some(Self::Cuisine), 430 => Some(Self::Abattoir),
            440 => Some(Self::Brasserie), 450 => Some(Self::QuaiPeche),
            1000 => Some(Self::PlaceMarche), 1220 => Some(Self::Glaciere),
            920 => Some(Self::EntrepotComptoir),
            _ => None,
        }
    }
    pub fn to_name_lowercase(&self) -> &'static str {
        BuildingTypeEnum::from_id(self.to_id()).unwrap().to_name_lowercase()
    }
    pub fn to_building_type(&self) -> BuildingTypeEnum {
        BuildingTypeEnum::from_id(self.to_id()).unwrap()
    }
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Cuisine, Self::Abattoir, Self::Brasserie, Self::QuaiPeche,
         Self::PlaceMarche, Self::Glaciere, Self::EntrepotComptoir].into_iter()
    }
}

// Legacy compat — kept for DwellingsTypeEnum and UrbanismTypeEnum since they're
// used in some data models but not part of the new building type system.

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum DwellingsTypeEnum {
    House = 1,
}

impl DwellingsTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }
    pub fn from_id(id: i16) -> Option<Self> { match id { 1 => Some(Self::House), _ => None } }
    pub fn to_name_lowercase(&self) -> &'static str { "house" }
    pub fn iter() -> impl Iterator<Item = Self> { [Self::House].into_iter() }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub enum UrbanismTypeEnum {
    Path = 1,
    Road = 2,
    PavedRoad = 3,
    Avenue = 4,
}

impl UrbanismTypeEnum {
    pub fn to_id(self) -> i16 { self as i16 }
    pub fn from_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::Path), 2 => Some(Self::Road),
            3 => Some(Self::PavedRoad), 4 => Some(Self::Avenue),
            _ => None,
        }
    }
    pub fn to_name_lowercase(&self) -> &'static str {
        match self {
            Self::Path => "path", Self::Road => "road",
            Self::PavedRoad => "paved_road", Self::Avenue => "avenue",
        }
    }
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Path, Self::Road, Self::PavedRoad, Self::Avenue].into_iter()
    }
}
