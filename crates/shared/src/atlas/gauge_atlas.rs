use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct GaugeAtlas {
    pub sprites: HashMap<String, String>,
    pub handles: HashMap<String, Handle<Image>>,
}

impl GaugeAtlas {
    pub fn load(&mut self) {
        for position in 0..=4 {
            for light in 0..=3 {
                self.sprites.insert(
                    format!("{}_{}", position, light),
                    format!("gauge_elements_{}_{}", position, light),
                );
            }
        }
    }

    pub fn get_variations(&self, position: u32, light: u32) -> Option<&String> {
        self.sprites.get(&format!("{}_{}", position, light))
    }

    pub fn get_handles(&self, position: u32, light: u32) -> Option<&Handle<Image>> {
        self.handles.get(&format!("{}_{}", position, light))
    }
}
