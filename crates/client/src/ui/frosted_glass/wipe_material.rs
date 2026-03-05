use bevy::{prelude::*, render::render_resource::*, shader::ShaderRef};

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct WipeMaterial {
    #[uniform(0)]
    pub uniforms: WipeUniforms,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

#[derive(ShaderType, Clone, Debug)]
pub struct WipeUniforms {
    pub size: Vec2,
    pub screen_size: Vec2,
    pub edge_fade: f32,
    pub visibility: f32,
}

impl UiMaterial for WipeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wipe_image.wgsl".into()
    }
}
