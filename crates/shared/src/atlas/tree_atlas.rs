use std::collections::HashMap;

use bevy::prelude::*;

use crate::{TreeAge, TreeTypeEnum};

#[derive(Default, Resource)]
pub struct TreeAtlas {
    pub sprites: HashMap<TreeTypeEnum, Vec<String>>,
    pub handles: HashMap<String, Handle<Image>>,
}

impl TreeAtlas {
    pub fn load(&mut self) {
        let tree_types = [
            (TreeTypeEnum::Cedar, "cedar", 3, 6),
            // (TreeTypeEnum::Larch, "larch", 3, 6),
            // (TreeTypeEnum::Oak, "oak", 3, 6),
        ];

        self.sprites
            .extend(tree_types.iter().map(|(tree_type, name, variation, density)| {
                let mut variations = Vec::new();

                for age in TreeAge::iter() {
                    for v in 1..=*variation {
                        for d in 1..=*density {
                            variations.push(format!("{}_{}_{:02}{:02}", name, age.to_name(), v, d));
                        }
                    }
                    // let variations = (1..=*variation).(1..=*density).map(|i| format!("{}_{:02}", name, i)).collect();
                }

                (*tree_type, variations)
            }));
    }

    pub fn get_variations(&self, tree_type: TreeTypeEnum) -> Option<&[String]> {
        self.sprites.get(&tree_type).map(|v| v.as_slice())
    }
}
