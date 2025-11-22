use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct MoonAtlas {
    pub sprites: HashMap<String, String>,
    pub handles: HashMap<String, Handle<Image>>,
}

impl MoonAtlas {
    pub fn load(&mut self) {
        let phases = vec![
            "new_moon",
            "waxing_crescent",
            "first_quarter",
            "waxing_gibbous",
            "full_moon",
            "waning_gibbous",
            "last_quarter",
            "waning_crescent",
        ];

        for (index, phase) in phases.iter().enumerate() {
            self.sprites
                .insert(index.to_string(), format!("moons/{}", phase));
        }
    }

    pub fn get_handle(&self, phase_index: u32) -> Option<&Handle<Image>> {
        self.handles.get(&phase_index.to_string())
    }
}
