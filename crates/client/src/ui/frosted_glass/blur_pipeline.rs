// src/ui/frosted_glass/blur_pipeline.rs

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, texture_storage_2d},
            *,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
    },
};

use crate::ui::frosted_glass::{BlurSettings, FrostedGlassMaterial};

#[derive(Resource)]
pub struct BlurPipeline {
    pub downsample_pipeline: CachedComputePipelineId,
    pub upsample_pipeline: CachedComputePipelineId,
    pub bind_group_layout: BindGroupLayout,
}

impl FromWorld for BlurPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = render_device.create_bind_group_layout(
            "blur_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Input texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // Sampler
                    sampler(SamplerBindingType::Filtering),
                    // Output texture (storage)
                    texture_storage_2d(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/kawase_blur.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();

        let downsample_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("kawase_downsample".into()),
                layout: vec![bind_group_layout.clone()],
                shader: shader.clone(),
                shader_defs: vec!["DOWNSAMPLE".into()],
                entry_point: Some("downsample".into()),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            });

        let upsample_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("kawase_upsample".into()),
            layout: vec![bind_group_layout.clone()],
            shader,
            shader_defs: vec!["UPSAMPLE".into()],
            entry_point: Some("upsample".into()),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        Self {
            downsample_pipeline,
            upsample_pipeline,
            bind_group_layout,
        }
    }
}

#[derive(Resource, Default)]
pub struct BlurTextures {
    pub chain: Vec<CachedTexture>,
    pub final_blur: Option<Handle<Image>>,
}

pub fn prepare_blur_textures(
    mut blur_textures: ResMut<BlurTextures>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    settings: Option<Res<BlurSettings>>,
    views: Query<&ViewTarget>,
) {
    let Some(settings) = settings else {
        return;
    };

    let Ok(view_target) = views.single() else {
        return;
    };

    let base_size = view_target.main_texture().size();
    let mut current_size = Extent3d {
        width: base_size.width / settings.scale,
        height: base_size.height / settings.scale,
        depth_or_array_layers: 1,
    };

    blur_textures.chain.clear();

    // Créer la chaîne de textures pour le blur
    for i in 0..settings.iterations {
        let texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("blur_texture"),
                size: current_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );
        blur_textures.chain.push(texture);

        current_size.width = (current_size.width / 2).max(1);
        current_size.height = (current_size.height / 2).max(1);
    }
}

pub fn run_blur_passes(
    blur_pipeline: Res<BlurPipeline>,
    blur_textures: Res<BlurTextures>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    views: Query<&ViewTarget>,
) {
    let Ok(view_target) = views.single() else {
        return;
    };

    let (Some(downsample_pipeline), Some(upsample_pipeline)) = (
        pipeline_cache.get_compute_pipeline(blur_pipeline.downsample_pipeline),
        pipeline_cache.get_compute_pipeline(blur_pipeline.upsample_pipeline),
    ) else {
        return;
    };

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("blur_encoder"),
    });

    // Downsample passes
    let mut input_view = view_target.main_texture_view();
    for texture in &blur_textures.chain {
        let output_view = &texture.default_view;

        // Créer bind group et dispatcher...
        // (code simplifié pour la lisibilité)

        input_view = output_view;
    }

    // Upsample passes (inverse)
    for texture in blur_textures.chain.iter().rev().skip(1) {
        // Upsample vers la texture précédente
    }

    render_queue.submit(std::iter::once(encoder.finish()));
}

// pub fn sync_frosted_glass_size(
//     mut materials: ResMut<Assets<FrostedGlassMaterial>>,
//     query: Query<(&MaterialNode<FrostedGlassMaterial>, &ComputedNode), Changed<ComputedNode>>,
// ) {
//     for (material_handle, computed) in &query {
//         if let Some(material) = materials.get_mut(&material_handle.0) {
//             material.size = computed.size();
//         }
//     }
// }
