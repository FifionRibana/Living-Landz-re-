// src/ui/frosted_glass/blur_capture.rs

use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    math::FloatOrd,
    prelude::*,
    render::render_resource::*,
    window::PrimaryWindow,
};

use crate::ui::frosted_glass::{
    BlurSettings, FrostedGlassMaterial, resources::BlurredSceneTexture,
};

/// Marker pour la caméra qui capture la scène pour le blur
#[derive(Component)]
pub struct BlurCaptureCamera;

/// Crée la texture de capture et la caméra secondaire
pub fn setup_blur_capture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut blur_texture: ResMut<BlurredSceneTexture>,
    window: Query<&Window, With<PrimaryWindow>>,
    blur_settings: Res<BlurSettings>,
) {
    let Ok(window) = window.single() else { return };

    let size = Extent3d {
        width: ((window.physical_width() as f32 / blur_settings.scale as f32) as u32).max(1),
        height: ((window.physical_height() as f32 / blur_settings.scale as f32) as u32).max(1),
        depth_or_array_layers: 1,
    };

    // Texture cible pour le rendu basse résolution
    let mut capture_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[128, 128, 128, 255],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    capture_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;

    let capture_handle = images.add(capture_image);
    blur_texture.handle = Some(capture_handle.clone());

    // Caméra de capture (render layer séparé pour exclure l'UI)
    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Image(bevy::camera::ImageRenderTarget {
                handle: capture_handle,
                scale_factor: FloatOrd(1. / (blur_settings.scale as f32)),
            }),
            order: -1, // Render avant la caméra principale
            ..default()
        },
        BlurCaptureCamera,
        RenderLayers::layer(0), // Ne capture que le layer 0 (game world)
    ));
}

/// Synchronise la taille et le border radius du Node avec le material
pub fn sync_material_size(
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    window: Query<&Window, With<PrimaryWindow>>,
    query: Query<(&MaterialNode<FrostedGlassMaterial>, &ComputedNode), Changed<ComputedNode>>,
) {
    let Ok(window) = window.single() else { return };
    let screen_size = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );

    for (mat_node, computed) in &query {
        if let Some(material) = materials.get_mut(&mat_node.0) {
            material.uniforms.size = computed.size();
            material.uniforms.screen_size = screen_size;

            let border = computed.border_radius();
            material.uniforms.border_radius = border.top_left;
        }
    }
}

/// Injecte la texture blurrée dans tous les materials
pub fn inject_scene_texture(
    blur_texture: Res<BlurredSceneTexture>,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window.single() else { return };
    let screen_size = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );

    for (_, material) in materials.iter_mut() {
        if material.background_image.is_none() {
            if let Some(ref blur_handle) = blur_texture.handle {
                if material.scene_texture.is_none() {
                    material.scene_texture = Some(blur_handle.clone())
                }
            }
        }

        if material.uniforms.screen_size == Vec2::ZERO {
            material.uniforms.screen_size = screen_size;
        }
    }
}
