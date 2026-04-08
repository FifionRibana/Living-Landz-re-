use std::collections::HashMap;

use bevy::prelude::*;

use crate::BuildingTypeEnum;

#[derive(Default, Resource)]
pub struct BuildingAtlas {
    pub sprites: HashMap<BuildingTypeEnum, Vec<String>>,
    pub handles: HashMap<String, Handle<Image>>,
}

impl BuildingAtlas {
    pub fn load(&mut self) {
        let building_types = [
            // Residential
            (BuildingTypeEnum::Campement, "base_camp", 1),
            (BuildingTypeEnum::ChaumierePalierI, "cottage_tier_i", 2),
            // Metal
            (BuildingTypeEnum::Forge, "forge", 1),
            (BuildingTypeEnum::Fonderie, "smelter", 2),
            // Earth
            (BuildingTypeEnum::Verrerie, "glassworks", 1),
            // Wood
            (BuildingTypeEnum::AtelierCharpentier, "carpenter_workshop", 1),
            // Food
            (BuildingTypeEnum::Ferme, "farm", 1),
            (BuildingTypeEnum::Cuisine, "kitchen", 1),
            (BuildingTypeEnum::Brasserie, "brewery", 2),
            (BuildingTypeEnum::Abattoir, "slaughterhouse", 1),
            (BuildingTypeEnum::Glaciere, "ice_house", 1),
            // Animal breeding
            (BuildingTypeEnum::Etable, "cowshed", 2),
            (BuildingTypeEnum::Porcherie, "pigsty", 1),
            (BuildingTypeEnum::Bergerie, "sheepfold", 1),
            (BuildingTypeEnum::Ecurie, "stable", 2),
            // Services
            (BuildingTypeEnum::Theatre, "theater", 1),
            (BuildingTypeEnum::LieuDeCulte, "place_of_worship", 1),
            (BuildingTypeEnum::PlaceMarche, "marketplace", 1),
        ];

        self.sprites
            .extend(building_types.iter().map(|(building_type, name, variations)| {
                let mut sprite_variations = Vec::new();

                for v in 1..=*variations {
                    sprite_variations.push(format!("{}_{:02}", name, v));
                }

                (*building_type, sprite_variations)
            }));
    }

    pub fn get_variations(&self, building_type: BuildingTypeEnum) -> Option<&[String]> {
        self.sprites.get(&building_type).map(|v| v.as_slice())
    }

    pub fn get_sprite(&self, building_type: BuildingTypeEnum, variant: usize) -> Option<&Handle<Image>> {
        let variations = self.get_variations(building_type)?;
        let sprite_name = variations.get(variant)?;
        self.handles.get(sprite_name)
    }
}
