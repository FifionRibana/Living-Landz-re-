use crate::{BuildingTypeEnum, ProfessionEnum};

/// A single resource quantity (input or output).
#[derive(Debug, Clone)]
pub struct ResourceAmount {
    pub name: &'static str,
    pub quantity: u32,
}

/// A production recipe definition.
#[derive(Debug, Clone)]
pub struct RecipeDefinition {
    /// Unique string ID (matches action_id prefix "produce_X")
    pub id: &'static str,
    /// Display name (French)
    pub name: &'static str,
    /// Description
    pub description: &'static str,
    /// Icon asset path
    pub icon: &'static str,
    /// Which building this recipe requires
    pub building: BuildingTypeEnum,
    /// Which profession can execute it
    pub profession: ProfessionEnum,
    /// Input resources consumed
    pub inputs: &'static [(&'static str, u32)],
    /// Output resources produced
    pub outputs: &'static [(&'static str, u32)],
    /// Duration in game ticks
    pub duration_ticks: u32,
}

impl RecipeDefinition {
    pub fn inputs_vec(&self) -> Vec<ResourceAmount> {
        self.inputs
            .iter()
            .map(|(name, qty)| ResourceAmount {
                name,
                quantity: *qty,
            })
            .collect()
    }

    pub fn outputs_vec(&self) -> Vec<ResourceAmount> {
        self.outputs
            .iter()
            .map(|(name, qty)| ResourceAmount {
                name,
                quantity: *qty,
            })
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════
//  RECIPE REGISTRY — all production recipes in the game
// ═══════════════════════════════════════════════════════════════

pub static RECIPES: &[RecipeDefinition] = &[
    // ─── Blacksmith ─────────────────────────────────────────
    RecipeDefinition {
        id: "iron_sword",
        name: "Épée en fer",
        description: "Forger une épée en fer",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Blacksmith,
        profession: ProfessionEnum::Blacksmith,
        inputs: &[("Fer", 3), ("Bois", 1)],
        outputs: &[("Épée en fer", 1)],
        duration_ticks: 4,
    },
    RecipeDefinition {
        id: "iron_tools",
        name: "Outils en fer",
        description: "Fabriquer des outils en fer",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Blacksmith,
        profession: ProfessionEnum::Blacksmith,
        inputs: &[("Fer", 2)],
        outputs: &[("Outils", 1)],
        duration_ticks: 3,
    },
    RecipeDefinition {
        id: "horseshoes",
        name: "Fers à cheval",
        description: "Forger des fers à cheval",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Blacksmith,
        profession: ProfessionEnum::Blacksmith,
        inputs: &[("Fer", 1)],
        outputs: &[("Fers à cheval", 2)],
        duration_ticks: 2,
    },
    RecipeDefinition {
        id: "nails",
        name: "Clous",
        description: "Fabriquer un lot de clous",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Blacksmith,
        profession: ProfessionEnum::Blacksmith,
        inputs: &[("Fer", 1)],
        outputs: &[("Clous", 20)],
        duration_ticks: 2,
    },
    // ─── Carpenter Shop ─────────────────────────────────────
    RecipeDefinition {
        id: "planks",
        name: "Planches",
        description: "Scier des planches",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::CarpenterShop,
        profession: ProfessionEnum::Carpenter,
        inputs: &[("Bois", 2)],
        outputs: &[("Planches", 4)],
        duration_ticks: 2,
    },
    RecipeDefinition {
        id: "furniture",
        name: "Meubles",
        description: "Fabriquer des meubles",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::CarpenterShop,
        profession: ProfessionEnum::Carpenter,
        inputs: &[("Bois", 5), ("Clous", 10)],
        outputs: &[("Meubles", 1)],
        duration_ticks: 5,
    },
    RecipeDefinition {
        id: "barrel",
        name: "Tonneau",
        description: "Assembler un tonneau",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::CarpenterShop,
        profession: ProfessionEnum::Carpenter,
        inputs: &[("Bois", 4), ("Fer", 1)],
        outputs: &[("Tonneau", 1)],
        duration_ticks: 3,
    },
    RecipeDefinition {
        id: "wooden_shield",
        name: "Bouclier en bois",
        description: "Fabriquer un bouclier en bois",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::CarpenterShop,
        profession: ProfessionEnum::Carpenter,
        inputs: &[("Bois", 3), ("Cuir", 1)],
        outputs: &[("Bouclier", 1)],
        duration_ticks: 4,
    },
    // ─── Farm ───────────────────────────────────────────────
    RecipeDefinition {
        id: "wheat",
        name: "Blé",
        description: "Cultiver du blé",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Farm,
        profession: ProfessionEnum::Farmer,
        inputs: &[],
        outputs: &[("Blé", 6)],
        duration_ticks: 6,
    },
    RecipeDefinition {
        id: "vegetables",
        name: "Légumes",
        description: "Cultiver des légumes",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Farm,
        profession: ProfessionEnum::Farmer,
        inputs: &[],
        outputs: &[("Légumes", 4)],
        duration_ticks: 4,
    },
    RecipeDefinition {
        id: "flax",
        name: "Lin",
        description: "Cultiver du lin",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Farm,
        profession: ProfessionEnum::Farmer,
        inputs: &[],
        outputs: &[("Lin", 3)],
        duration_ticks: 5,
    },
    RecipeDefinition {
        id: "hay",
        name: "Foin",
        description: "Récolter du foin",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Farm,
        profession: ProfessionEnum::Farmer,
        inputs: &[],
        outputs: &[("Foin", 8)],
        duration_ticks: 3,
    },
    // ─── Bakehouse ──────────────────────────────────────────
    RecipeDefinition {
        id: "bread",
        name: "Pain",
        description: "Cuire du pain",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Bakehouse,
        profession: ProfessionEnum::Baker,
        inputs: &[("Blé", 2)],
        outputs: &[("Pain", 3)],
        duration_ticks: 2,
    },
    RecipeDefinition {
        id: "pastry",
        name: "Pâtisserie",
        description: "Préparer des pâtisseries",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Bakehouse,
        profession: ProfessionEnum::Baker,
        inputs: &[("Blé", 3), ("Beurre", 1)],
        outputs: &[("Pâtisserie", 2)],
        duration_ticks: 3,
    },
    // ─── Brewery ────────────────────────────────────────────
    RecipeDefinition {
        id: "beer",
        name: "Bière",
        description: "Brasser de la bière",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Brewery,
        profession: ProfessionEnum::Brewer,
        inputs: &[("Blé", 3)],
        outputs: &[("Bière", 2)],
        duration_ticks: 6,
    },
    RecipeDefinition {
        id: "mead",
        name: "Hydromel",
        description: "Brasser de l'hydromel",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Brewery,
        profession: ProfessionEnum::Brewer,
        inputs: &[("Miel", 2)],
        outputs: &[("Hydromel", 2)],
        duration_ticks: 8,
    },
    // ─── Market ─────────────────────────────────────────────
    RecipeDefinition {
        id: "trade_buy",
        name: "Acheter",
        description: "Acheter des marchandises au marché",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Market,
        profession: ProfessionEnum::Merchant,
        inputs: &[],
        outputs: &[],
        duration_ticks: 1,
    },
    RecipeDefinition {
        id: "trade_sell",
        name: "Vendre",
        description: "Vendre des marchandises au marché",
        icon: "ui/icons/cog.png",
        building: BuildingTypeEnum::Market,
        profession: ProfessionEnum::Merchant,
        inputs: &[],
        outputs: &[],
        duration_ticks: 1,
    },
];

/// Lookup a recipe by ID.
pub fn get_recipe(id: &str) -> Option<&'static RecipeDefinition> {
    RECIPES.iter().find(|r| r.id == id)
}

/// Get all recipes for a given building type.
pub fn recipes_for_building(building: BuildingTypeEnum) -> Vec<&'static RecipeDefinition> {
    RECIPES.iter().filter(|r| r.building == building).collect()
}

/// Get all recipes a given profession can execute.
pub fn recipes_for_profession(profession: ProfessionEnum) -> Vec<&'static RecipeDefinition> {
    RECIPES
        .iter()
        .filter(|r| r.profession == profession)
        .collect()
}
