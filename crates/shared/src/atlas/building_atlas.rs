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
            // ManufacturingWorkshops
            (BuildingTypeEnum::Blacksmith, "blacksmith", 1),
            (BuildingTypeEnum::BlastFurnace, "blast_furnace", 1),
            (BuildingTypeEnum::Bloomery, "bloomery", 1),
            (BuildingTypeEnum::CarpenterShop, "carpenter_shop", 1),
            (BuildingTypeEnum::GlassFactory, "glass_factory", 1),
            // Agriculture
            (BuildingTypeEnum::Farm, "farm", 1),
            // AnimalBreeding
            (BuildingTypeEnum::Cowshed, "cowshed", 2),
            (BuildingTypeEnum::Piggery, "piggery", 1),
            (BuildingTypeEnum::Sheepfold, "sheepfold", 1),
            (BuildingTypeEnum::Stable, "stable", 2),
            // Entertainment
            (BuildingTypeEnum::Theater, "theater", 1),
            // Cult
            (BuildingTypeEnum::Temple, "temple", 1),
            // Commerce
            (BuildingTypeEnum::Bakehouse, "bakehouse", 1),
            (BuildingTypeEnum::Brewery, "brewery", 1),
            (BuildingTypeEnum::Distillery, "distillery", 1),
            (BuildingTypeEnum::Slaughterhouse, "slaughterhouse", 1),
            (BuildingTypeEnum::IceHouse, "ice_house", 1),
            (BuildingTypeEnum::Market, "market", 1),
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
