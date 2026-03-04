mod camera_settings;
mod scene_render_target;

use bevy::camera::visibility::RenderLayers;
pub use camera_settings::CameraSettings;
pub use scene_render_target::SceneRenderTarget;

pub const GAME_LAYER: RenderLayers = RenderLayers::layer(0);
pub const DISPLAY_LAYER: RenderLayers = RenderLayers::layer(1);