use crate::{ActionModeEnum, BuildingTypeEnum, ProfessionEnum};
use super::context::{UIActionContext, ActionEntry, ActionViewContext};

impl ActionModeEnum {
    /// Returns the list of actions available for this mode given the current context.
    /// This is the single source of truth used by both client (UI) and server (validation).
    pub fn available_actions(&self, ctx: &UIActionContext) -> Vec<ActionEntry> {
        match self {
            Self::RoadActionMode => road_actions(ctx),
            Self::BuildingActionMode => building_actions(ctx),
            Self::ProductionActionMode => production_actions(ctx),
            Self::TrainingActionMode => training_actions(ctx),
            Self::DiplomacyActionMode => diplomacy_actions(ctx),
        }
    }
}

// ─── Roads ──────────────────────────────────────────────────

fn road_actions(ctx: &UIActionContext) -> Vec<ActionEntry> {
    // Roads are available to all professions
    match ctx.view {
        ActionViewContext::Map => {
            vec![
                ActionEntry::new("plan_dirt_path", "Chemin de terre")
                    .with_description("Planifier un chemin de terre entre deux points")
                    .with_icon("ui/icons/road.png")
                    .with_duration(2),
                ActionEntry::new("plan_paved_road", "Route pavée")
                    .with_description("Planifier une route pavée")
                    .with_icon("ui/icons/road.png")
                    .with_cost("Pierre", 5)
                    .with_duration(4),
                ActionEntry::new("plan_highway", "Grande voie")
                    .with_description("Planifier une grande voie commerciale")
                    .with_icon("ui/icons/road.png")
                    .with_cost("Pierre", 10)
                    .with_cost("Bois", 5)
                    .with_duration(8),
            ]
        }
        ActionViewContext::Cell => {
            if !ctx.has_adjacent_road {
                return vec![]; // No road segment possible without adjacent road
            }
            vec![
                ActionEntry::new("build_road_segment", "Segment de route")
                    .with_description("Construire un segment de route vers une case adjacente")
                    .with_icon("ui/icons/road.png")
                    .with_cost("Pierre", 2)
                    .with_duration(1),
            ]
        }
    }
}

// ─── Buildings ──────────────────────────────────────────────

fn building_actions(ctx: &UIActionContext) -> Vec<ActionEntry> {
    if !ctx.is_cell_view() {
        return vec![];
    }

    // Filter: only professions that can build
    let can_build = ctx.has_any_profession(&[
        ProfessionEnum::Carpenter,
        ProfessionEnum::Mason,
        ProfessionEnum::Lumberjack,
        ProfessionEnum::Blacksmith,
    ]);

    if !can_build {
        return vec![];
    }

    if let Some(building) = ctx.building {
        // Existing building → show upgrades
        building_upgrades(building)
    } else {
        // Empty terrain → show constructible buildings
        constructible_buildings(ctx)
    }
}

fn building_upgrades(building: BuildingTypeEnum) -> Vec<ActionEntry> {
    match building {
        BuildingTypeEnum::Farm => vec![
            ActionEntry::new("upgrade_farm_irrigation", "Irrigation")
                .with_description("Ajouter un système d'irrigation")
                .with_icon("ui/icons/village.png")
                .with_cost("Bois", 10)
                .with_cost("Pierre", 5)
                .with_duration(6),
        ],
        BuildingTypeEnum::Blacksmith => vec![
            ActionEntry::new("upgrade_blacksmith_forge", "Forge améliorée")
                .with_description("Améliorer la forge pour des travaux plus complexes")
                .with_icon("ui/icons/village.png")
                .with_cost("Pierre", 15)
                .with_cost("Fer", 10)
                .with_duration(10),
        ],
        _ => vec![
            ActionEntry::new("upgrade_repair", "Réparations")
                .with_description("Réparer et entretenir le bâtiment")
                .with_icon("ui/icons/village.png")
                .with_cost("Bois", 5)
                .with_duration(3),
        ],
    }
}

fn constructible_buildings(ctx: &UIActionContext) -> Vec<ActionEntry> {
    // Filter buildings by terrain type compatibility
    let mut entries = Vec::new();

    let buildings = [
        ("blacksmith", "Forge", &[("Bois", 10u32), ("Pierre", 15), ("Fer", 5)] as &[_], 12u32),
        ("carpenter_shop", "Menuiserie", &[("Bois", 15), ("Pierre", 5)], 8),
        ("farm", "Ferme", &[("Bois", 18), ("Pierre", 8)], 10),
        ("bakehouse", "Boulangerie", &[("Bois", 15), ("Pierre", 20), ("Argile", 10)], 10),
        ("brewery", "Brasserie", &[("Bois", 20), ("Pierre", 15), ("Cuivre", 5)], 12),
        ("market", "Marché", &[("Bois", 35), ("Pierre", 20), ("Tissu", 10)], 15),
        ("cowshed", "Étable", &[("Bois", 20), ("Pierre", 10), ("Paille", 15)], 8),
        ("sheepfold", "Bergerie", &[("Bois", 18), ("Pierre", 10), ("Paille", 12)], 8),
        ("stable", "Écurie", &[("Bois", 25), ("Pierre", 15), ("Paille", 20)], 10),
        ("temple", "Temple", &[("Pierre", 50), ("Bois", 30), ("Or", 10)], 20),
        ("theater", "Théâtre", &[("Bois", 40), ("Pierre", 30), ("Tissu", 20)], 18),
    ];

    for (id, name, costs, duration) in buildings {
        let mut entry = ActionEntry::new(
            &format!("build_{}", id),
            name,
        )
        .with_description(&format!("Construire un(e) {}", name.to_lowercase()))
        .with_icon("ui/icons/village.png")
        .with_duration(duration);

        for (resource, qty) in costs {
            entry = entry.with_cost(resource, *qty);
        }

        entries.push(entry);
    }

    entries
}

// ─── Production ─────────────────────────────────────────────

fn production_actions(ctx: &UIActionContext) -> Vec<ActionEntry> {
    if !ctx.is_cell_view() {
        return vec![];
    }

    let Some(building) = ctx.building else {
        return vec![];
    };

    // Get recipes for this building, filtered by available professions
    let all_recipes = building_recipes(building);
    all_recipes
        .into_iter()
        .filter(|entry| {
            entry
                .required_profession
                .as_ref()
                .map(|prof| ctx.has_profession(prof))
                .unwrap_or(true) // No requirement = anyone can do it
        })
        .collect()
}

fn building_recipes(building: BuildingTypeEnum) -> Vec<ActionEntry> {
    match building {
        BuildingTypeEnum::Blacksmith => vec![
            ActionEntry::new("produce_iron_sword", "Épée en fer")
                .with_description("Forger une épée en fer")
                .with_icon("ui/icons/cog.png")
                .with_cost("Fer", 3)
                .with_cost("Bois", 1)
                .with_profession(ProfessionEnum::Blacksmith)
                .with_duration(4),
            ActionEntry::new("produce_iron_tools", "Outils en fer")
                .with_description("Fabriquer des outils en fer")
                .with_icon("ui/icons/cog.png")
                .with_cost("Fer", 2)
                .with_profession(ProfessionEnum::Blacksmith)
                .with_duration(3),
            ActionEntry::new("produce_horseshoes", "Fers à cheval")
                .with_description("Forger des fers à cheval")
                .with_icon("ui/icons/cog.png")
                .with_cost("Fer", 1)
                .with_profession(ProfessionEnum::Blacksmith)
                .with_duration(2),
        ],
        BuildingTypeEnum::CarpenterShop => vec![
            ActionEntry::new("produce_planks", "Planches")
                .with_description("Scier des planches")
                .with_icon("ui/icons/cog.png")
                .with_cost("Bois", 2)
                .with_profession(ProfessionEnum::Carpenter)
                .with_duration(2),
            ActionEntry::new("produce_furniture", "Meubles")
                .with_description("Fabriquer des meubles")
                .with_icon("ui/icons/cog.png")
                .with_cost("Bois", 5)
                .with_profession(ProfessionEnum::Carpenter)
                .with_duration(5),
            ActionEntry::new("produce_barrel", "Tonneau")
                .with_description("Assembler un tonneau")
                .with_icon("ui/icons/cog.png")
                .with_cost("Bois", 4)
                .with_profession(ProfessionEnum::Carpenter)
                .with_duration(3),
        ],
        BuildingTypeEnum::Farm => vec![
            ActionEntry::new("produce_wheat", "Blé")
                .with_description("Cultiver du blé")
                .with_icon("ui/icons/cog.png")
                .with_profession(ProfessionEnum::Farmer)
                .with_duration(6),
            ActionEntry::new("produce_vegetables", "Légumes")
                .with_description("Cultiver des légumes")
                .with_icon("ui/icons/cog.png")
                .with_profession(ProfessionEnum::Farmer)
                .with_duration(4),
            ActionEntry::new("produce_flax", "Lin")
                .with_description("Cultiver du lin")
                .with_icon("ui/icons/cog.png")
                .with_profession(ProfessionEnum::Farmer)
                .with_duration(5),
        ],
        BuildingTypeEnum::Bakehouse => vec![
            ActionEntry::new("produce_bread", "Pain")
                .with_description("Cuire du pain")
                .with_icon("ui/icons/cog.png")
                .with_cost("Blé", 2)
                .with_profession(ProfessionEnum::Baker)
                .with_duration(2),
            ActionEntry::new("produce_pastry", "Pâtisserie")
                .with_description("Préparer des pâtisseries")
                .with_icon("ui/icons/cog.png")
                .with_cost("Blé", 3)
                .with_cost("Beurre", 1)
                .with_profession(ProfessionEnum::Baker)
                .with_duration(3),
        ],
        BuildingTypeEnum::Brewery => vec![
            ActionEntry::new("produce_beer", "Bière")
                .with_description("Brasser de la bière")
                .with_icon("ui/icons/cog.png")
                .with_cost("Blé", 3)
                .with_profession(ProfessionEnum::Brewer)
                .with_duration(6),
            ActionEntry::new("produce_mead", "Hydromel")
                .with_description("Brasser de l'hydromel")
                .with_icon("ui/icons/cog.png")
                .with_cost("Miel", 2)
                .with_profession(ProfessionEnum::Brewer)
                .with_duration(8),
        ],
        BuildingTypeEnum::Market => vec![
            ActionEntry::new("trade_buy", "Acheter")
                .with_description("Acheter des marchandises au marché")
                .with_icon("ui/icons/cog.png")
                .with_profession(ProfessionEnum::Merchant)
                .with_duration(1),
            ActionEntry::new("trade_sell", "Vendre")
                .with_description("Vendre des marchandises au marché")
                .with_icon("ui/icons/cog.png")
                .with_profession(ProfessionEnum::Merchant)
                .with_duration(1),
        ],
        _ => vec![],
    }
}

// ─── Training ───────────────────────────────────────────────

fn training_actions(ctx: &UIActionContext) -> Vec<ActionEntry> {
    if !ctx.is_cell_view() {
        return vec![];
    }

    // For each selected profession, show what they can train into
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for prof in &ctx.selected_professions {
        for target in trainable_professions(prof) {
            if seen.insert(target) {
                entries.push(
                    ActionEntry::new(
                        &format!("train_{}", target.to_name_lowercase()),
                        &format!("Former : {}", target.to_name_fr()),
                    )
                    .with_description(&format!(
                        "Former cette unité au métier de {}",
                        target.to_name_fr()
                    ))
                    .with_icon("ui/icons/laurels-trophy.png")
                    .with_duration(10),
                );
            }
        }
    }

    entries
}

/// Which professions can a given profession train into?
fn trainable_professions(from: &ProfessionEnum) -> Vec<ProfessionEnum> {
    use ProfessionEnum::*;
    match from {
        Unknown => vec![
            Farmer, Lumberjack, Miner, Fisherman, Hunter,
        ],
        Farmer => vec![Baker, Cook, Brewer],
        Lumberjack => vec![Carpenter],
        Miner => vec![Blacksmith, Mason],
        Fisherman => vec![Cook],
        Hunter => vec![Warrior],
        Baker => vec![Cook],
        Cook => vec![Baker, Brewer],
        Carpenter => vec![Mason],
        Mason => vec![Carpenter],
        Blacksmith => vec![],
        Merchant => vec![Scholar],
        Scholar => vec![Merchant, Healer],
        Healer => vec![Scholar],
        Warrior => vec![Hunter],
        Brewer => vec![Cook],
    }
}

// ─── Diplomacy ──────────────────────────────────────────────

fn diplomacy_actions(ctx: &UIActionContext) -> Vec<ActionEntry> {
    let has_diplomat = ctx.has_any_profession(&[
        ProfessionEnum::Merchant,
        ProfessionEnum::Scholar,
    ]);

    if !has_diplomat {
        return vec![];
    }

    vec![
        ActionEntry::new("send_envoy", "Envoyer un émissaire")
            .with_description("Envoyer un émissaire diplomatique")
            .with_icon("ui/icons/bookmarklet.png")
            .with_profession(ProfessionEnum::Merchant)
            .with_duration(5),
        ActionEntry::new("propose_trade", "Proposer un échange")
            .with_description("Proposer un accord commercial")
            .with_icon("ui/icons/bookmarklet.png")
            .with_profession(ProfessionEnum::Merchant)
            .with_duration(3),
        ActionEntry::new("research", "Recherche")
            .with_description("Mener des recherches")
            .with_icon("ui/icons/bookmarklet.png")
            .with_profession(ProfessionEnum::Scholar)
            .with_duration(8),
    ]
}
