/// Authentication panels module

pub mod components;
pub mod systems;

pub use components::*;
pub use systems::*;

use bevy::prelude::*;
use crate::states::AuthScreen;

pub struct AuthPlugin;

impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app
            // Spawn login panel when entering AuthScreen::Login
            .add_systems(OnEnter(AuthScreen::Login), systems::setup_login_panel)
            // Spawn register panel when entering AuthScreen::Register
            .add_systems(OnEnter(AuthScreen::Register), systems::setup_register_panel)
            // Login interaction systems — only run in AuthScreen::Login
            .add_systems(
                Update,
                (
                    systems::handle_login_button_click,
                    systems::handle_to_register_button_click,
                    systems::handle_login_button_hover,
                    systems::handle_test_character_creation_click,
                    systems::handle_test_coat_of_arms_click,
                    systems::handle_test_button_hover,
                )
                    .run_if(in_state(AuthScreen::Login)),
            )
            // Register interaction systems — only run in AuthScreen::Register
            .add_systems(
                Update,
                (
                    systems::handle_register_button_click,
                    systems::handle_back_button_click,
                    systems::handle_register_button_hover,
                )
                    .run_if(in_state(AuthScreen::Register)),
            );
    }
}