use crate::camera::resources::{CELL_SCENE_LAYER, CellSceneRenderTarget, DISPLAY_LAYER};
use crate::states::GameView;
use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

#[derive(Component)]
pub struct CellSceneCamera;

#[derive(Component)]
pub struct CellSceneDisplay;

pub fn setup_cell_scene_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    mut cell_render_target: ResMut<CellSceneRenderTarget>,
) {
    let window = windows.single().unwrap();
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);
    cell_render_target.handle = Some(image_handle.clone());

    // Camera rendering cell scene to texture
    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Image(image_handle.clone().into()),
            order: 1,
            is_active: false,
            ..default()
        },
        CellSceneCamera,
        CELL_SCENE_LAYER,
    ));

    // Display sprite on DISPLAY_LAYER (shows the cell scene, hidden by default)
    commands.spawn((
        Sprite {
            image: image_handle,
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        Visibility::Hidden,
        CellSceneDisplay,
        DISPLAY_LAYER,
    ));
}

pub fn toggle_cell_scene_camera(
    game_view: Res<State<GameView>>,
    mut camera_query: Query<&mut Camera, With<CellSceneCamera>>,
    mut display_query: Query<&mut Visibility, With<CellSceneDisplay>>,
) {
    let in_cell = *game_view.get() == GameView::Cell;

    for mut camera in camera_query.iter_mut() {
        camera.is_active = in_cell;
    }
    for mut vis in display_query.iter_mut() {
        *vis = if in_cell {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
