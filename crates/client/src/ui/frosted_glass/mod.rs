// src/ui/frosted_glass/mod.rs

mod blur_capture;
mod blur_pipeline;
mod material;
mod plugin;
mod resources;

pub use blur_capture::*;
pub use material::{FrostedGlassConfig, FrostedGlassMaterial, FadeDirection};
pub use plugin::FrostedGlassPlugin;
pub use resources::BlurSettings;
pub mod prelude {
    pub use super::{FrostedGlassConfig, FrostedGlassMaterial, FrostedGlassPlugin};
}
