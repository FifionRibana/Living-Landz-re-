use std::collections::HashMap;

use bevy::prelude::*;

use crate::{TreeAge, TreeTypeEnum};

#[derive(Default, Resource)]
pub struct TreeAtlas {
    /// tree_type → list of variant names (used by server for generation)
    pub sprites: HashMap<TreeTypeEnum, Vec<String>>,
    /// variant name → image handle (individual sprites before packing)
    pub handles: HashMap<String, Handle<Image>>,

    /// variant name → atlas index (after packing)
    pub variants: HashMap<String, usize>,
    /// Packed atlas image handle
    pub atlas_image: Option<Handle<Image>>,
    /// Atlas layout handle
    pub atlas_layout: Option<Handle<TextureAtlasLayout>>,
    /// Atlas grid dimensions
    pub atlas_cols: u32,
    pub atlas_rows: u32,
    pub sprite_size: u32,
}

impl TreeAtlas {
    pub fn load(&mut self) {
        let tree_types = [
            (TreeTypeEnum::Cedar, "cedar", 3),
        ];

        self.sprites
            .extend(tree_types.iter().map(|(tree_type, name, variation)| {
                let mut variations = Vec::new();
                for age in TreeAge::iter() {
                    for v in 1..=*variation {
                        variations.push(format!("{}_{}_{:02}01", name, age.to_name(), v));
                    }
                }
                (*tree_type, variations)
            }));
    }

    pub fn get_variations(&self, tree_type: TreeTypeEnum) -> Option<&[String]> {
        self.sprites.get(&tree_type).map(|v| v.as_slice())
    }

    /// Get atlas UV rect for a variant (returns (u_min, v_min, u_max, v_max))
    pub fn get_atlas_uvs(&self, variant_name: &str) -> Option<[f32; 4]> {
        let idx = self.variants.get(variant_name)?;
        let col = (*idx as u32) % self.atlas_cols;
        let row = (*idx as u32) / self.atlas_cols;
        let atlas_w = (self.atlas_cols * self.sprite_size) as f32;
        let atlas_h = (self.atlas_rows * self.sprite_size) as f32;
        let u_min = (col * self.sprite_size) as f32 / atlas_w;
        let v_min = (row * self.sprite_size) as f32 / atlas_h;
        let u_max = ((col + 1) * self.sprite_size) as f32 / atlas_w;
        let v_max = ((row + 1) * self.sprite_size) as f32 / atlas_h;
        Some([u_min, v_min, u_max, v_max])
    }

    /// Fast lookup: (age_index, variation) → atlas_index
    pub fn get_atlas_index_fast(&self, age_idx: usize, variation: i32) -> Option<usize> {
        // Layout: 3 variations per age, ages in order
        // index = age_idx * 3 + (variation - 1)
        let idx = age_idx * 3 + (variation as usize - 1);
        if idx < self.variants.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// Fast UV lookup by index
    pub fn get_atlas_uvs_by_index(&self, idx: usize) -> Option<[f32; 4]> {
        if idx >= self.variants.len() { return None; }
        let col = (idx as u32) % self.atlas_cols;
        let row = (idx as u32) / self.atlas_cols;
        let atlas_w = (self.atlas_cols * self.sprite_size) as f32;
        let atlas_h = (self.atlas_rows * self.sprite_size) as f32;
        Some([
            (col * self.sprite_size) as f32 / atlas_w,
            (row * self.sprite_size) as f32 / atlas_h,
            ((col + 1) * self.sprite_size) as f32 / atlas_w,
            ((row + 1) * self.sprite_size) as f32 / atlas_h,
        ])
    }
}