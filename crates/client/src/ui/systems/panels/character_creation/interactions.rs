use bevy::prelude::*;
use bevy_ui_text_input::{TextInputBuffer, TextInputQueue, actions::TextInputAction, actions::TextInputEdit};

use crate::networking::client::NetworkClient;
use crate::state::resources::ConnectionStatus;
use crate::states::AppState;

use super::components::*;
use super::resources::CharacterCreationState;

// ─── Palette (hover colors) ─────────────────────────────────────────────────

const ARROW_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.10);
const ARROW_HOVER: Color = Color::srgba(0.79, 0.66, 0.30, 0.25);
const BTN_BG: Color = Color::srgb(0.23, 0.18, 0.12);
const BTN_HOVER: Color = Color::srgb(0.30, 0.24, 0.16);
const BTN_PRIMARY_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.20);
const BTN_PRIMARY_HOVER: Color = Color::srgba(0.79, 0.66, 0.30, 0.35);
const GENDER_ACTIVE_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.25);
const GENDER_INACTIVE_BG: Color = Color::srgba(0.79, 0.66, 0.30, 0.05);
const GENDER_ACTIVE_BORDER: Color = Color::srgb(0.79, 0.66, 0.30);
const GENDER_INACTIVE_BORDER: Color = Color::srgba(0.55, 0.45, 0.30, 0.3);
const GOLD: Color = Color::srgb(0.79, 0.66, 0.30);
const TEXT_DIM: Color = Color::srgb(0.42, 0.30, 0.19);

// ─── Arrow button clicks ────────────────────────────────────────────────────

pub fn handle_arrow_clicks(
    query: Query<(&Interaction, &LayerArrowButton), Changed<Interaction>>,
    mut creation_state: ResMut<CharacterCreationState>,
) {
    for (interaction, arrow) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(layer) = creation_state.layer_mut(arrow.category) {
            match arrow.direction {
                ArrowDirection::Prev => layer.prev(),
                ArrowDirection::Next => layer.next(),
            }
            info!(
                "Layer {:?} changed to {} / {}",
                arrow.category,
                layer.current + 1,
                layer.total
            );
        }
    }
}

pub fn update_arrow_hover(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<LayerArrowButton>)>,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(ARROW_HOVER),
            Interaction::None => BackgroundColor(ARROW_BG),
        };
    }
}

// ─── Gender buttons ─────────────────────────────────────────────────────────

pub fn handle_gender_click(
    query: Query<(&Interaction, &GenderButton), Changed<Interaction>>,
    mut creation_state: ResMut<CharacterCreationState>,
) {
    for (interaction, btn) in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Gender set to {:?}", btn.gender);
            creation_state.set_gender(btn.gender);
        }
    }
}

/// Update gender button visuals when gender changes.
pub fn update_gender_visuals(
    creation_state: Res<CharacterCreationState>,
    mut query: Query<(&GenderActiveIndicator, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut text_query: Query<&mut TextColor>,
) {
    if !creation_state.is_changed() {
        return;
    }

    for (indicator, mut bg, mut border, children) in query.iter_mut() {
        let is_active = indicator.gender == creation_state.gender;
        *bg = BackgroundColor(if is_active { GENDER_ACTIVE_BG } else { GENDER_INACTIVE_BG });
        *border = BorderColor::all(if is_active { GENDER_ACTIVE_BORDER } else { GENDER_INACTIVE_BORDER });

        // Update child text color
        for child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                text_color.0 = if is_active { GOLD } else { TEXT_DIM };
            }
        }
    }
}

// ─── Random all button ──────────────────────────────────────────────────────

pub fn handle_random_all_click(
    query: Query<&Interaction, (Changed<Interaction>, With<RandomAllButton>)>,
    mut creation_state: ResMut<CharacterCreationState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Randomizing all");
            creation_state.randomize_all();
        }
    }
}

pub fn update_random_all_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RandomAllButton>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(ARROW_HOVER),
            Interaction::None => BackgroundColor(ARROW_BG),
        };
    }
}

// ─── Random name button ─────────────────────────────────────────────────────

pub fn handle_random_name_click(
    query: Query<&Interaction, (Changed<Interaction>, With<RandomNameButton>)>,
    mut creation_state: ResMut<CharacterCreationState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Randomizing name");
            creation_state.randomize_name();
        }
    }
}

pub fn update_random_name_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RandomNameButton>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(ARROW_HOVER),
            Interaction::None => BackgroundColor(ARROW_BG),
        };
    }
}

// ─── Back button ────────────────────────────────────────────────────────────

pub fn handle_back_click(
    query: Query<&Interaction, (Changed<Interaction>, With<BackToLoginButton>)>,
    mut connection: ResMut<ConnectionStatus>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Back to login from character creation");
            connection.reset_auth();
            next_state.set(AppState::Login);
        }
    }
}

pub fn update_back_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToLoginButton>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BTN_HOVER),
            Interaction::None => BackgroundColor(BTN_BG),
        };
    }
}

// ─── Validate button ────────────────────────────────────────────────────────

pub fn handle_validate_click(
    query: Query<&Interaction, (Changed<Interaction>, With<ValidateButton>)>,
    creation_state: Res<CharacterCreationState>,
    mut network_client: Option<ResMut<NetworkClient>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let first_name = creation_state.first_name.trim();
        if first_name.is_empty() {
            warn!("Cannot create character: first name is empty");
            return;
        }

        info!(
            "Validating character: '{}' ({:?})",
            first_name, creation_state.gender
        );

        let selections: Vec<(String, usize)> = creation_state
            .layers
            .iter()
            .map(|l| (l.category.folder().to_string(), l.current))
            .collect();

        info!("Layer selections: {:?}", selections);

        if let Some(ref mut client) = network_client {
            let _ = client;
        }

        next_state.set(AppState::InGame);
    }
}

pub fn update_validate_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ValidateButton>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BTN_PRIMARY_HOVER),
            Interaction::None => BackgroundColor(BTN_PRIMARY_BG),
        };
    }
}

// ─── Sync name: resource ↔ TextInput ────────────────────────────────────────

/// Read TextInput → update resource (when user types manually).
/// Only runs when there's no pending programmatic push.
pub fn sync_name_input(
    query: Query<&TextInputBuffer, With<FirstNameInput>>,
    mut creation_state: ResMut<CharacterCreationState>,
) {
    // Don't sync while a programmatic push is pending (would overwrite it)
    if creation_state.pending_name_push.is_some() {
        return;
    }

    for buffer in query.iter() {
        let text = buffer.get_text().to_string();
        if text != creation_state.first_name {
            creation_state.first_name = text;
        }
    }
}

/// Push programmatic name changes → TextInput buffer.
/// Consumes `pending_name_push` to avoid feedback loops.
///
/// Uses `TextInputAction::SetContents`. If your version uses a different name
/// (e.g. `SetText`), update the variant below.
pub fn push_name_to_input(
    mut creation_state: ResMut<CharacterCreationState>,
    mut query: Query<&mut TextInputQueue, With<FirstNameInput>>,
) {
    let Some(new_name) = creation_state.pending_name_push.take() else {
        return;
    };

    for mut queue in query.iter_mut() {
        // queue.add(TextInputAction::Edit(TextInputEdit::SelectAll));
        // queue.add(TextInputAction::Edit(TextInputEdit::Delete));
        queue.add(TextInputAction::Edit(TextInputEdit::Paste(new_name.clone())));
    }
}
