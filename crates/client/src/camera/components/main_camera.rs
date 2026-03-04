use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

use crate::camera::resources::{DISPLAY_LAYER, GAME_LAYER, SceneRenderTarget};

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
) {
    let window = windows.single().unwrap();
    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);
    commands.insert_resource(SceneRenderTarget(image_handle.clone()));

    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Image(image_handle.clone().into()),
            order: 0,
            ..default()
        },
        MainCamera,
        GAME_LAYER,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        DISPLAY_LAYER,
    ));

    commands.spawn((
        Sprite {
            image: image_handle,
            custom_size: Some(Vec2::new(
                window.resolution.width(),
                window.resolution.height(),
            )),
            ..default()
        },
        DISPLAY_LAYER,
    ));
}
