use std::collections::HashMap;

use bevy::prelude::*;

use crate::TreeType;

#[derive(Default, Resource)]
pub struct TreeAtlas {
    pub sprites: HashMap<TreeType, Vec<String>>,
    pub handles: HashMap<String, Handle<Image>>,
}

impl TreeAtlas {
    pub fn load(&mut self) {
        let tree_types = [
            (TreeType::Cedar, "cedar", 20),
            (TreeType::Larch, "larch", 24),
            (TreeType::Oak, "oak", 20),
        ];

        self.sprites
            .extend(tree_types.iter().map(|(tree_type, name, count)| {
                let variations = (1..=*count).map(|i| format!("{}_{:02}", name, i)).collect();

                (*tree_type, variations)
            }));
    }

    pub fn get_variations(&self, tree_type: TreeType) -> Option<&[String]> {
        self.sprites.get(&tree_type).map(|v| v.as_slice())
    }
}
