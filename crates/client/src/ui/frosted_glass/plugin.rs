// src/ui/frosted_glass/plugin.rs

use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSystems};

use crate::ui::frosted_glass::{BlurSettings, WipeMaterial};
use crate::ui::frosted_glass::resources::BlurredSceneTexture;
use crate::ui::frosted_glass::{inject_scene_texture, setup_blur_capture, sync_material_size};

use super::blur_pipeline::{BlurPipeline, BlurTextures, prepare_blur_textures, run_blur_passes};
use super::material::FrostedGlassMaterial;

pub struct FrostedGlassPlugin;

// TODO: Re-enable blur pipeline (setup_blur_capture, prepare_blur_textures, run_blur_passes) when the frosted glass effect is finalized

impl Plugin for FrostedGlassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<FrostedGlassMaterial>::default())
            .insert_resource(BlurSettings {
                iterations: 4,
                scale: 4,
            })
            .init_resource::<BlurredSceneTexture>()
            .add_systems(
                PostUpdate,
                (sync_material_size, inject_scene_texture).chain(),
            )
            .add_plugins(UiMaterialPlugin::<WipeMaterial>::default());
    }
}
