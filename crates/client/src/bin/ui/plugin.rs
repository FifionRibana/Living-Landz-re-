use bevy::prelude::*;

use super::ui;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, ui::setup_debug_ui)
            .add_systems(Update, ui::update_debug_ui);
    }
}