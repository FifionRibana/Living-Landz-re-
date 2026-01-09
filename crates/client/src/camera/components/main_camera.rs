use bevy::prelude::*;

use crate::post_processing::MedievalPostProcessSettings;

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
        // MedievalPostProcessSettings::moderate(),
    ));
}
