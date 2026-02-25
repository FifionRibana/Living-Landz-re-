// src/ui/frosted_glass/plugin.rs

use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSystems};

use crate::ui::frosted_glass::BlurSettings;
use crate::ui::frosted_glass::resources::BlurredSceneTexture;
use crate::ui::frosted_glass::{inject_scene_texture, setup_blur_capture, sync_material_size};

use super::blur_pipeline::{BlurPipeline, BlurTextures, prepare_blur_textures, run_blur_passes};
use super::material::FrostedGlassMaterial;

pub struct FrostedGlassPlugin; // {
/// Nombre de passes de downsample (2-4 recommandé)
// pub blur_iterations: u32,
/// Taille initiale du downsample (2 = demi-résolution)
// pub initial_downsample: u32,
// }

// impl Default for FrostedGlassPlugin {
//     fn default() -> Self {
//         Self {
//             blur_iterations: 3,
//             initial_downsample: 2,
//         }
//     }
// }

impl Plugin for FrostedGlassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<FrostedGlassMaterial>::default())
            .insert_resource(BlurSettings {
                iterations: 4, //self.blur_iterations,
                scale: 4,      //self.initial_downsample,
            })
            .init_resource::<BlurredSceneTexture>()
            // .add_systems(Startup, setup_blur_capture)
            .add_systems(
                PostUpdate,
                (sync_material_size, inject_scene_texture).chain(),
            );
    }

    // fn finish(&self, app: &mut App) {
    //     let render_app = app.sub_app_mut(RenderApp);

    //     render_app
    //         .init_resource::<BlurPipeline>()
    //         .init_resource::<BlurTextures>()
    //         .add_systems(
    //             Render,
    //             (
    //                 prepare_blur_textures.in_set(RenderSystems::PrepareResources),
    //                 run_blur_passes.in_set(RenderSystems::Render),
    //                 // .before(Node2d::MainOpaquePass),
    //             ),
    //         ).add_systems(Update, sync_frosted_glass_size);
    // }
}
