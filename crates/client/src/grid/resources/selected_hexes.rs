use std::collections::HashSet;

use bevy::prelude::*;
use hexx::Hex;

#[derive(Resource, Default)]
pub struct SelectedHexes {
    pub ids: HashSet<Hex>,
}

impl SelectedHexes {
    pub fn add(&mut self, hex: Hex) {
        if !self.ids.contains(&hex) {
            self.ids.insert(hex);
        }
    }

    pub fn remove(&mut self, hex: Hex) {
        self.ids.remove(&hex);
    }

    pub fn toggle(&mut self, hex: Hex) {
        if self.ids.contains(&hex) {
            self.remove(hex);
        } else {
            self.add(hex);
        }
    }

    pub fn clear(&mut self) {
        self.ids.clear();
    }

    pub fn selection_count(&self) -> usize {
        self.ids.len()
    }

    pub fn is_selected(&self, hex: Hex) -> bool {
        self.ids.contains(&hex)
    }
}