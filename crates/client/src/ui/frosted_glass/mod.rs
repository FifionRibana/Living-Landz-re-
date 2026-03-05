// src/ui/frosted_glass/mod.rs

mod blur_capture;
mod blur_pipeline;
mod material;
mod wipe_material;
mod plugin;
mod resources;

pub use blur_capture::*;
pub use material::{FrostedGlassConfig, FrostedGlassMaterial, FadeDirection};
pub use plugin::FrostedGlassPlugin;
pub use resources::BlurSettings;
pub use wipe_material::{WipeMaterial, WipeUniforms};
pub mod prelude {
    pub use super::{FrostedGlassConfig, FrostedGlassMaterial, FrostedGlassPlugin};
    pub use super::{WipeMaterial,WipeUniforms};
}
