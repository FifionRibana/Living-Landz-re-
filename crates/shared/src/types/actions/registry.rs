use crate::{ActionModeEnum, BuildingTypeEnum, ProfessionEnum};
use super::context::{UIActionContext, ActionEntry, ActionViewContext, GameDataRef};

impl ActionModeEnum {
    /// Returns the list of actions available for this mode given the current context.
    /// This is the single source of truth used by both client (UI) and server (validation).
    pub fn available_actions(
        &self,
        ctx: &UIActionContext,
        game_data: Option<&GameDataRef>,
    ) -> Vec<ActionEntry> {
        match self {
            Self::RoadActionMode => road_actions(ctx),
            Self::BuildingActionMode => building_actions(ctx, game_data),
            Self::ProductionActionMode => production_actions(ctx, game_data),
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

fn building_actions(ctx: &UIActionContext, game_data: Option<&GameDataRef>) -> Vec<ActionEntry> {
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
        constructible_buildings(ctx, game_data)
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

fn constructible_buildings(
    _ctx: &UIActionContext,
    game_data: Option<&GameDataRef>,
) -> Vec<ActionEntry> {
    let Some(gd) = game_data else {
        return vec![];
    };

    let mut entries = Vec::new();

    let buildings = [
        (BuildingTypeEnum::Blacksmith, "blacksmith"),
        (BuildingTypeEnum::BlastFurnace, "blast_furnace"),
        (BuildingTypeEnum::Bloomery, "bloomery"),
        (BuildingTypeEnum::CarpenterShop, "carpenter_shop"),
        (BuildingTypeEnum::GlassFactory, "glass_factory"),
        (BuildingTypeEnum::Farm, "farm"),
        (BuildingTypeEnum::Cowshed, "cowshed"),
        (BuildingTypeEnum::Piggery, "piggery"),
        (BuildingTypeEnum::Sheepfold, "sheepfold"),
        (BuildingTypeEnum::Stable, "stable"),
        (BuildingTypeEnum::Theater, "theater"),
        (BuildingTypeEnum::Temple, "temple"),
        (BuildingTypeEnum::Bakehouse, "bakehouse"),
        (BuildingTypeEnum::Brewery, "brewery"),
        (BuildingTypeEnum::Distillery, "distillery"),
        (BuildingTypeEnum::Slaughterhouse, "slaughterhouse"),
        (BuildingTypeEnum::IceHouse, "ice_house"),
        (BuildingTypeEnum::Market, "market"),
    ];

    for (bt_enum, slug) in buildings {
        let bt_id = bt_enum.to_id() as i32;
        // Use translated name from DB; fallback to slug
        let name = gd.item_names
            .get(&(-(bt_id))) // building names won't be in item_names
            .cloned()
            .unwrap_or_else(|| bt_enum.to_name_lowercase().to_string());

        let mut entry = ActionEntry::new(
            &format!("build_{}", slug),
            &name,
        )
        .with_description(&format!("Construire un(e) {}", name))
        .with_icon("ui/icons/village.png")
        .with_duration(10);

        // Check if player has enough resources
        let costs = gd.building_costs(bt_id);
        let mut can_build = true;
        for cost in &costs {
            let item_name = gd.item_name(cost.item_id);
            entry = entry.with_cost(&item_name, cost.quantity as u32);

            let have = gd.inventory.get(&cost.item_id).copied().unwrap_or(0);
            if have < cost.quantity {
                can_build = false;
            }
        }
        entry.executable = can_build;

        entries.push(entry);
    }

    entries
}

// ─── Production ─────────────────────────────────────────────

fn production_actions(
    ctx: &UIActionContext,
    game_data: Option<&GameDataRef>,
) -> Vec<ActionEntry> {
    if !ctx.is_cell_view() {
        return vec![];
    }

    let Some(building) = ctx.building else {
        return vec![];
    };

    let Some(gd) = game_data else {
        return vec![];
    };

    let bt_id = building.to_id();

    gd.recipes_for_building(bt_id)
        .into_iter()
        .map(|recipe| ActionEntry::from_recipe_net(recipe, gd))
        .collect()
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
