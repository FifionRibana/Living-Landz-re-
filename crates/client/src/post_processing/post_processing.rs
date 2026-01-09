//! Post-processing "Registre Seigneurial" pour Bevy 0.17.x
//!
//! Effets combinés: vignettage, grain de papier, teinte sépia

use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner, RenderGraphExt},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor,
            ShaderStages, ShaderType, TextureFormat, TextureSampleType, VertexState,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        RenderApp,
    },
};

/// Shader WGSL embarqué
const SHADER_SOURCE: &str = r#"
struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct PostProcessSettings {
    vignette_intensity: f32,
    vignette_radius: f32,
    grain_intensity: f32,
    grain_speed: f32,
    tint_intensity: f32,
    tint_color: vec3<f32>,
    time: f32,
    _padding: f32,
}

@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOutput {
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);
    
    var output: FullscreenVertexOutput;
    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>((x + 1.0) * 0.5, 1.0 - (y + 1.0) * 0.5);
    return output;
}

fn random(st: vec2<f32>) -> f32 {
    return fract(sin(dot(st, vec2<f32>(12.9898, 78.233))) * 43758.5453123);
}

fn animated_noise(uv: vec2<f32>, time: f32, speed: f32) -> f32 {
    let t = time * speed;
    let noise1 = random(uv + vec2<f32>(t, 0.0));
    let noise2 = random(uv + vec2<f32>(0.0, t));
    return (noise1 + noise2) * 0.5;
}

fn vignette(uv: vec2<f32>, intensity: f32, radius: f32) -> f32 {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);
    let vignette_factor = smoothstep(radius, radius + 0.5, dist);
    return 1.0 - (vignette_factor * intensity);
}

fn apply_tint(color: vec3<f32>, tint: vec3<f32>, intensity: f32) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    let tinted = luminance * tint;
    return mix(color, tinted, intensity);
}

fn adjust_saturation(color: vec3<f32>, saturation: f32) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    let gray = vec3<f32>(luminance);
    return mix(gray, color, saturation);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    var color = textureSample(screen_texture, screen_sampler, uv).rgb;
    
    color = adjust_saturation(color, 0.85);
    color = apply_tint(color, settings.tint_color, settings.tint_intensity);
    
    let grain = animated_noise(uv * 512.0, settings.time, settings.grain_speed);
    let grain_offset = (grain - 0.5) * 2.0 * settings.grain_intensity;
    color = color + vec3<f32>(grain_offset);
    
    let vignette_factor = vignette(uv, settings.vignette_intensity, settings.vignette_radius);
    color = color * vignette_factor;
    
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
    
    return vec4<f32>(color, 1.0);
}
"#;

/// Plugin principal pour le post-processing médiéval
pub struct MedievalPostProcessPlugin;

/// Handle du shader stocké comme ressource
#[derive(Resource, Clone)]
struct MedievalShaderHandle(Handle<Shader>);

impl Plugin for MedievalPostProcessPlugin {
    fn build(&self, app: &mut App) {
        // Charger le shader dans l'app principale
        let shader_handle = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .add(Shader::from_wgsl(SHADER_SOURCE, file!()));
        
        app.insert_resource(MedievalShaderHandle(shader_handle.clone()));
        
        app.add_plugins((
            ExtractComponentPlugin::<MedievalPostProcessSettings>::default(),
            UniformComponentPlugin::<MedievalPostProcessSettings>::default(),
        ));

        // Système pour mettre à jour le temps
        app.add_systems(Update, update_time);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        
        // Passer le handle au render app
        render_app.insert_resource(MedievalShaderHandle(shader_handle));

        render_app
            // Ajouter le node pour les caméras 2D
            .add_render_graph_node::<ViewNodeRunner<MedievalPostProcessNode>>(
                Core2d,
                MedievalPostProcessLabel,
            )
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::Tonemapping,
                    MedievalPostProcessLabel,
                    Node2d::EndMainPassPostProcessing,
                ),
            )
            // Ajouter le node pour les caméras 3D également
            .add_render_graph_node::<ViewNodeRunner<MedievalPostProcessNode>>(
                Core3d,
                MedievalPostProcessLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    MedievalPostProcessLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<MedievalPostProcessPipeline>();
    }
}

/// Composant de configuration pour le post-processing
/// Ajouter ce composant à une caméra pour activer les effets
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct MedievalPostProcessSettings {
    /// Intensité du vignettage (0.0 - 1.0, recommandé: 0.3 - 0.5)
    pub vignette_intensity: f32,

    /// Rayon du vignettage - distance depuis le centre (0.0 - 1.0, recommandé: 0.4 - 0.6)
    pub vignette_radius: f32,

    /// Intensité du grain de papier (0.0 - 1.0, recommandé: 0.03 - 0.08)
    pub grain_intensity: f32,

    /// Vitesse d'animation du grain (recommandé: 0.5 - 2.0)
    pub grain_speed: f32,

    /// Intensité de la teinte sépia (0.0 - 1.0, recommandé: 0.1 - 0.25)
    pub tint_intensity: f32,

    /// Couleur de teinte RGB normalisée (sépia: 0.76, 0.65, 0.47)
    pub tint_color: Vec3,

    /// Temps écoulé (mis à jour automatiquement)
    pub time: f32,

    /// Padding pour alignement GPU
    pub _padding: f32,
}

impl MedievalPostProcessSettings {
    /// Preset par défaut - effet subtil
    pub fn subtle() -> Self {
        Self {
            vignette_intensity: 0.25,
            vignette_radius: 0.5,
            grain_intensity: 0.03,
            grain_speed: 1.0,
            tint_intensity: 0.12,
            tint_color: Vec3::new(0.76, 0.65, 0.47),
            time: 0.0,
            _padding: 0.0,
        }
    }

    /// Preset modéré - bon équilibre
    pub fn moderate() -> Self {
        Self {
            vignette_intensity: 0.4,
            vignette_radius: 0.45,
            grain_intensity: 0.00,
            grain_speed: 1.0,
            tint_intensity: 0.18,
            tint_color: Vec3::new(0.76, 0.65, 0.47),
            time: 0.0,
            _padding: 0.0,
        }
    }

    /// Preset prononcé - effet très visible
    pub fn pronounced() -> Self {
        Self {
            vignette_intensity: 0.55,
            vignette_radius: 0.4,
            grain_intensity: 0.08,
            grain_speed: 1.5,
            tint_intensity: 0.25,
            tint_color: Vec3::new(0.72, 0.60, 0.42),
            time: 0.0,
            _padding: 0.0,
        }
    }

    /// Preset pour carte ancienne
    pub fn old_map() -> Self {
        Self {
            vignette_intensity: 0.35,
            vignette_radius: 0.5,
            grain_intensity: 0.06,
            grain_speed: 0.5,
            tint_intensity: 0.22,
            tint_color: Vec3::new(0.80, 0.68, 0.45),
            time: 0.0,
            _padding: 0.0,
        }
    }

    /// Désactiver tous les effets
    pub fn none() -> Self {
        Self {
            vignette_intensity: 0.0,
            vignette_radius: 1.0,
            grain_intensity: 0.0,
            grain_speed: 0.0,
            tint_intensity: 0.0,
            tint_color: Vec3::ONE,
            time: 0.0,
            _padding: 0.0,
        }
    }

    pub fn with_vignette(mut self, intensity: f32, radius: f32) -> Self {
        self.vignette_intensity = intensity;
        self.vignette_radius = radius;
        self
    }

    pub fn with_grain(mut self, intensity: f32, speed: f32) -> Self {
        self.grain_intensity = intensity;
        self.grain_speed = speed;
        self
    }

    pub fn with_tint(mut self, intensity: f32, color: Vec3) -> Self {
        self.tint_intensity = intensity;
        self.tint_color = color;
        self
    }

    pub fn with_sepia(mut self, intensity: f32) -> Self {
        self.tint_intensity = intensity;
        self.tint_color = Vec3::new(0.76, 0.65, 0.47);
        self
    }
}

impl Default for MedievalPostProcessSettings {
    fn default() -> Self {
        Self::moderate()
    }
}

/// Couleurs de teinte prédéfinies
pub mod tint_colors {
    use bevy::prelude::Vec3;

    pub const SEPIA: Vec3 = Vec3::new(0.76, 0.65, 0.47);
    pub const PARCHMENT: Vec3 = Vec3::new(0.80, 0.70, 0.48);
    pub const IRON_GALL: Vec3 = Vec3::new(0.65, 0.55, 0.42);
    pub const VELLUM: Vec3 = Vec3::new(0.75, 0.70, 0.58);
    pub const HEMP_PAPER: Vec3 = Vec3::new(0.70, 0.65, 0.55);
}

fn update_time(time: Res<Time>, mut query: Query<&mut MedievalPostProcessSettings>) {
    for mut settings in &mut query {
        settings.time = time.elapsed_secs();
    }
}

// ============================================================================
// Render Graph Implementation
// ============================================================================

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct MedievalPostProcessLabel;

#[derive(Default)]
struct MedievalPostProcessNode;

impl ViewNode for MedievalPostProcessNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static DynamicUniformIndex<MedievalPostProcessSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<MedievalPostProcessPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<MedievalPostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "medieval_post_process_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("medieval_post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct MedievalPostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for MedievalPostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "medieval_post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<MedievalPostProcessSettings>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Récupérer le handle du shader depuis la ressource
        let shader = world.resource::<MedievalShaderHandle>().0.clone();

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("medieval_post_process_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: VertexState {
                        shader: shader.clone(),
                        shader_defs: vec![],
                        entry_point: Some("vertex".into()),
                        buffers: vec![],
                    },
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: Some("fragment".into()),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::Rgba8UnormSrgb,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}