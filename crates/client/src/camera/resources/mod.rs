mod camera_settings;
mod scene_render_target;

use bevy::camera::visibility::RenderLayers;
pub use camera_settings::CameraSettings;
pub use scene_render_target::{SceneRenderTarget, CellSceneRenderTarget};

pub const GAME_LAYER: RenderLayers = RenderLayers::layer(0);
pub const DISPLAY_LAYER: RenderLayers = RenderLayers::layer(1);
pub const CELL_SCENE_LAYER: RenderLayers = RenderLayers::layer(2);