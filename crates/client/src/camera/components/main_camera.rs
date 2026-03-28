use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use shared::grid::GridConfig;

use crate::{camera::resources::{DISPLAY_LAYER, GAME_LAYER, SceneRenderTarget}, state::resources::PlayerInfo};

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
            order: 0,
            ..default()
        },
        RenderTarget::Image(image_handle.clone().into()),
        MainCamera,
        GAME_LAYER,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
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


// TODO : Move to a correct place
pub fn center_camera_on_lord(
    player_info: Res<PlayerInfo>,
    grid_config: Res<GridConfig>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    if let Some(lord) = &player_info.lord {
        let hex = lord.current_cell.to_hex();
        let world_pos = grid_config.layout.hex_to_world_pos(hex);
        
        if let Ok(mut transform) = camera.single_mut() {
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
            info!(
                "Camera centered on lord at ({},{}) → world ({:.0},{:.0})",
                lord.current_cell.q, lord.current_cell.r,
                world_pos.x, world_pos.y
            );
        }
    }
}